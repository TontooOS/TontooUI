# Renderer

The renderer owns the window, the wgpu/Vello GPU state and the main loop. It
renders every frame with Vello into an intermediate texture and blits the
result to the winit surface. There is no UIKit layer and no GTK dependency.

## Modules

| Module | Path | Description |
|---|---|---|
| `window` | `src/renderer/window.rs` | winit event loop, surface management, `View` trait, `run` |
| `text` | `src/renderer/text.rs` | Parley font system and scene text drawing |
| `frame` | `src/renderer/frame.rs` | Window frame: shadows, rounded body, edge, outline |

## Window

```rust
pub const BACKGROUND: Color;
```

Window background color. Currently `#1d1d1d` (dark mode base).

```rust
pub const WINDOW_CORNER_RADIUS: f32;
```

Standard window corner radius in logical px. Currently `20.0`, following
the macOS 27 Golden Gate direction (one fixed radius, tighter than Tahoe).
Windows are undecorated (`with_decorations(false)`); apps draw their own
chrome, including rounded corners, traffic lights and title bars.

> **Note:** Window shadows are compositor-side on Wayland. winit exposes a
> shadow switch only on Windows (`with_undecorated_shadow`); on Linux there
> is no client-side API, so TontooCompositor decides whether a window gets a
> shadow. TontooUI draws no shadows itself.

```rust
pub fn run(title: &str, width: u32, height: u32, app: impl App + 'static) -> Result<(), Box<dyn Error>>
```

Opens a window with `title` and logical size `width` x `height` and runs
`app` until the window closes.

- Returns `Err` when the event loop cannot start (e.g. no display server).
- When no compatible GPU exists, an error is printed and the loop exits.
- The loop uses continuous redraw (`ControlFlow::Poll`) so animations and
  cursor blink work without extra timers.

## App

```rust
pub trait App {
    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, viewport: Viewport, time_secs: f64);
    fn mouse_down(&mut self, _x: f64, _y: f64) {}
    fn mouse_move(&mut self, _x: f64, _y: f64) {}
    fn set_focused(&mut self, _focused: bool) {}
    fn text(&mut self, _text: &str) {}
    fn key(&mut self, _key: Key) {}
    fn drag_region(&self) -> Option<(f32, f32, f32, f32)> {
        None
    }
    fn poll_window_command(&mut self) -> Option<WindowCommand> {
        None
    }
}
```

```rust
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
```

An `App` owns a tree of element `View`s (see [Layout.md](Layout.md)) and
forwards events into it. All coordinates are logical pixels; place content
inside `viewport` (window size minus the 24 px frame margin). `time_secs`
is seconds since the window opened (use it for blink and animation
phases).

```rust
pub enum Key {
    Backspace,
    Left,
    Right,
    Enter,
    Escape,
}
```

```rust
pub enum WindowCommand {
    Close,
    Minimize,
    ToggleMaximize,
}
```

Window operations requested by content (e.g. traffic lights). Return one
from `poll_window_command`; the shell consumes it once per frame and calls
`exit`, `set_minimized(true)` or toggles `set_maximized`. Cursor moves
arrive via `mouse_move` (logical px) and focus changes via `set_focused`.

Non-printable keys forwarded to the view. Printable input arrives via
`text()` as already-decoded strings (including key repeat).

## Frame Pipeline

Each `RedrawRequested` event runs these steps:

1. `scene.reset()` clears the previous frame.
2. `frame::draw_behind()` records shadows and the rounded body. The window
   is transparent and the surface is cleared transparent, so the corners
   stay see-through. Content drawn by views can still paint over the
   cutout; per-window clipping is not implemented yet.
3. `view.draw()` records GPU commands into the scene.
4. `frame::draw_frame()` records inner highlight, edge and outline above
   the content so bars and fields never cover the frame.
5. `renderer.render_to_texture()` renders the scene into the surface target
   texture with `AaConfig::Msaa8` and a transparent base color.
4. `TextureBlitter` copies the target texture to the acquired surface texture.
5. The surface texture is presented.

Surface errors are handled per frame:

| Condition | Behavior |
|---|---|
| `Success` / `Suboptimal` | Present normally |
| `Timeout` / `Occluded` / `Validation` | Skip the frame silently |
| `Outdated` / `Lost` | Reconfigure the surface and redraw |

`Resized` events with non-zero dimensions call
`RenderContext::resize_surface`. Zero-size events are ignored.

## FontSystem

```rust
pub struct FontSystem {
    pub scale: f32,
}
```

Wraps a Parley `FontContext` (system fonts, so SF Pro resolves on TontooOS)
and a `LayoutContext`. `scale` is the window scale factor set by the loop;
layouts are built in physical pixels so glyphs stay crisp.

```rust
pub fn layout_text(&mut self, content: &str, size: f32, color: Color, max_width: Option<f32>) -> Layout<SolidBrush>
```

Lays out `content` at logical `size` px. `max_width` is in logical px;
`None` disables wrapping. Default style is the system UI font with 1.25
relative line height.

```rust
pub fn layout_size(layout: &Layout<SolidBrush>) -> (f32, f32)
```

Physical width/height of a finished layout. Divide by `FontSystem::scale`
for logical units.

```rust
pub fn draw_layout(scene: &mut Scene, layout: &Layout<SolidBrush>, x: f32, y: f32, scale: f32)
```

Draws a finished layout at logical position (`x`, `y`). Iterates glyph runs
and records them with `Scene::draw_glyphs`. Only glyph runs are drawn;
inline boxes are skipped.

## Frame

```rust
pub const MARGIN: f32;
pub const EDGE: Color;
pub const INNER_TOP: Color;
pub const OUTER: Color;
```

Recreates the old UIKit window style, drawn by the shell before every
`view.draw()` so all windows look the same:

| Layer | Value |
|---|---|
| Margin | `24.0` logical px to the screen edge (`MARGIN`) |
| Body | `RoundedRect` with `BACKGROUND` and `WINDOW_CORNER_RADIUS` |
| Shadows | `0 2px 4px` black 15%, `0 4px 12px` black 12%, `0 7px 16px` black 8% (gaussian blur via `draw_blurred_rounded_rect`; shrunk from the UIKit spec so the reach fits the 24 px margin without clipping) |
| Inner | 1 px inner ring with a top-to-transparent white gradient (`INNER_TOP`) |
| Edge | 1 px stroke in `EDGE` (white 14%) |
| Outline | Outer 1 px ring in `OUTER` (black 55%) |

```rust
pub fn content_rect(width: f32, height: f32) -> (f32, f32, f32, f32)
```

Logical `(x, y, width, height)` inside the frame for the given logical
window size. The shell converts it to the `Viewport` passed to views.

## Usage / Example

```rust
use tontooui::elements::Text;
use tontooui::renderer::FontSystem;
use tontooui::renderer::window::{App, Viewport, run};
use vello::Scene;
use vello::peniko::Color;

struct Hello {
    label: Text,
}

impl App for Hello {
    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, viewport: Viewport, _t: f64) {
        self.label.set_position(viewport.x + 8.0, viewport.y + 12.0);
        self.label.draw(scene, fonts);
    }
}

fn main() {
    run("Hello", 800, 600, Hello {
        label: Text::new("Hello, TontooUI!").size(28.0).color(Color::WHITE),
    }).unwrap();
}
```

## Cross References

- [Text.md](Text.md) – static text element drawn through `FontSystem`
- [TextInput.md](TextInput.md) – editable text element with keyboard input
- [Titlebar.md](Titlebar.md) – custom decoration bar with drag region
- [Layout.md](Layout.md) – VStack, HStack, ZStack, Spacer and the Element trait
