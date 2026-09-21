# Renderer

The renderer owns the window, the wgpu/Vello GPU state and the main loop. It
renders every frame with Vello into an intermediate texture and blits the
result to the winit surface. There is no UIKit layer and no GTK dependency.

## Modules

| Module | Path | Description |
|---|---|---|
| `window` | `src/renderer/window.rs` | winit event loop, surface management, `View` trait, `run` |
| `text` | `src/renderer/text.rs` | Parley font system and scene text drawing |

## Window

```rust
pub const BACKGROUND: Color;
```

Window background color. Currently `#1d1d1d` (dark mode base).

```rust
pub fn run(title: &str, width: u32, height: u32, view: impl View + 'static) -> Result<(), Box<dyn Error>>
```

Opens a window with `title` and logical size `width` x `height` and runs
`view` until the window closes.

- Returns `Err` when the event loop cannot start (e.g. no display server).
- When no compatible GPU exists, an error is printed and the loop exits.
- The loop uses continuous redraw (`ControlFlow::Poll`) so animations and
  cursor blink work without extra timers.

## View

```rust
pub trait View {
    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, width: f32, height: f32, time_secs: f64);
    fn mouse_down(&mut self, _x: f64, _y: f64) {}
    fn text(&mut self, _text: &str) {}
    fn key(&mut self, _key: Key) {}
}
```

Content hosted in a window. All coordinates are logical pixels; `width` and
`height` are the current logical window size and `time_secs` is seconds since
the window opened (use it for blink and animation phases).

```rust
pub enum Key {
    Backspace,
    Left,
    Right,
    Enter,
    Escape,
}
```

Non-printable keys forwarded to the view. Printable input arrives via
`text()` as already-decoded strings (including key repeat).

## Frame Pipeline

Each `RedrawRequested` event runs these steps:

1. `scene.reset()` clears the previous frame.
2. `view.draw()` records GPU commands into the scene.
3. `renderer.render_to_texture()` renders the scene into the surface target
   texture with `AaConfig::Msaa8` and `BACKGROUND` as base color.
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

## Usage / Example

```rust
use tontooui::elements::Text;
use tontooui::renderer::FontSystem;
use tontooui::renderer::window::{View, run};
use vello::Scene;
use vello::peniko::Color;

struct Hello {
    label: Text,
}

impl View for Hello {
    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, _w: f32, _h: f32, _t: f64) {
        self.label.draw(scene, fonts);
    }
}

fn main() {
    run("Hello", 800, 600, Hello {
        label: Text::new("Hello, TontooUI!").size(28.0).color(Color::WHITE).at(32.0, 36.0),
    }).unwrap();
}
```

## Cross References

- [Text.md](Text.md) – static text element drawn through `FontSystem`
- [TextInput.md](TextInput.md) – editable text element with keyboard input
