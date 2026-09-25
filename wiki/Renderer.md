# Renderer

The renderer owns the window, the wgpu/Vello GPU state and the main loop. It
renders every frame with Vello into an intermediate texture and blits the
result to the winit surface. There is no UIKit layer and no GTK dependency.

## Modules

| Module | Path | Description |
|---|---|---|
| `window` | `src/renderer/window.rs` | winit event loop, surface management, `App` trait, `run` |
| `text` | `src/renderer/text.rs` | Parley font system and scene text drawing |
| `frame` | `src/renderer/frame.rs` | Window frame: shadows, rounded body, edge, outline |
| `backdrop` | `src/renderer/backdrop.rs` | Offscreen capture + separable gaussian blur for glass |
| `images` | `src/renderer/images.rs` | SF Symbol cache, per-frame `ImageLoader`, backdrop access |

## Window

```rust
pub const BACKGROUND: Color;
```

Window background color. Currently `#1B2022` (dark mode base).

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
pub const SCREEN_MARGIN: f32;
pub const MIN_WINDOW: u32;
```

No window may start bigger than the screen: the requested size is
clamped to the primary monitor minus `SCREEN_MARGIN` (currently
`48.0` logical px, reserving room for taskbars and docks; winit only
reports the full monitor size) and at least `MIN_WINDOW` (currently
`320` logical px per dimension). Without a monitor (e.g. headless)
the request is kept. The same bound is installed as the max inner
size, so windows can never be resized beyond the screen either.
Overflowing content must scroll inside (see
[ScrollView.md](ScrollView.md)) instead of growing the window.

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
    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
        viewport: Viewport,
        time_secs: f64,
    );
    fn mouse_down(&mut self, _x: f64, _y: f64) {}
    fn mouse_move(&mut self, _x: f64, _y: f64) {}
    fn mouse_wheel(&mut self, _dx: f64, _dy: f64) {}
    fn set_focused(&mut self, _focused: bool) {}
    fn text(&mut self, _text: &str) {}
    fn key(&mut self, _key: Key) {}
    fn drag_region(&self) -> Option<(f32, f32, f32, f32)> {
        None
    }
    fn poll_window_command(&mut self) -> Option<WindowCommand> {
        None
    }
    fn background(&self) -> Color {
        BACKGROUND
    }
    fn transparent_body(&self) -> bool {
        false
    }
    fn wants_backdrop(&self) -> bool {
        false
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
`background` is read every frame for the window body (see
[Theme.md](Theme.md)); the default is the dark base color.

```rust
pub fn transparent_body(&self) -> bool
pub fn wants_backdrop(&self) -> bool
```

- `transparent_body` skips the window background fill so only frame lines,
  bars and glass show over the desktop.
- `wants_backdrop` enables the two-pass backdrop blur (see Backdrop Blur).
  Return true only while backdrop glass is on screen (e.g. a slider knob is
  held) so idle frames stay single-pass.

Non-printable keys forwarded to the view. Printable input arrives via
`text()` as already-decoded strings (including key repeat). Wheel
scrolling arrives via `mouse_wheel` in logical px (right/down
positive, line steps normalized to 20 px). Right-button presses
arrive via `context_click` (context menus); touch contacts arrive
via `touch` with a `TouchPhase`:

```rust
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}
```

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

When `App::wants_backdrop()` is true the shell inserts a capture pass before
step 1 of the final draw (see Backdrop Blur); frame lines are recorded only
on the final pass.

Surface errors are handled per frame:

| Condition | Behavior |
|---|---|
| `Success` / `Suboptimal` | Present normally |
| `Timeout` / `Occluded` / `Validation` | Skip the frame silently |
| `Outdated` / `Lost` | Reconfigure the surface and redraw |

`Resized` events with non-zero dimensions call
`RenderContext::resize_surface`. Zero-size events are ignored.

## Backdrop Blur

When `App::wants_backdrop()` returns true, the shell runs two Vello passes
per frame so glass can sample a blurred copy of the in-app content behind
it (window body, tracks, labels, bars):

1. Capture: `scene.reset()`, `draw_behind()`, `App::draw()` with
   `ImageLoader::set_capture_pass(true)` (glass bodies skip themselves),
   no frame lines. Rendered into the offscreen `content` texture.
2. Blur: two compute dispatches (horizontal then vertical) gaussian-blur
   `content` into `output` (sigma `BACKDROP_SIGMA`, default 8 physical px,
   clamp-to-edge). The sharp `content` texture is registered alongside the
   blurred `output` so views can sample both.
3. Final: `scene.reset()`, `draw_behind()`, `App::draw()` with
   `ImageLoader::set_backdrop(Some(image))` plus
   `set_backdrop_sharp(Some(sharp))`, frame lines, render to the
   surface target, blit.

Desktop pixels behind a transparent window still belong to the compositor;
this pass blurs only what the app itself draws.

```rust
pub const BACKDROP_SIGMA: f32;
pub struct BackdropBlur { .. }
pub fn fill_backdrop(scene: &mut Scene, images: &ImageLoader<'_>, shape: &impl Shape)
pub fn fill_backdrop_lens(scene: &mut Scene, images: &ImageLoader<'_>, shape: &impl Shape, center: Point, zoom: f64)
pub fn stroke_backdrop_edge(scene: &mut Scene, images: &ImageLoader<'_>, ring: &RoundedRect, width: f64)
pub fn fill_lens_glass(scene: &mut Scene, images: &ImageLoader<'_>, rect: &Rect, radius: f64, zoom: f64, edge_width: f64)
```

- `BackdropBlur` owns the three offscreen targets and the compute pipeline;
  the shell creates one per window and resizes it with `ensure_size`.
- `fill_backdrop` paints `shape` with the blurred capture when
  `ImageLoader::backdrop()` is `Some`; no-op otherwise (single-pass frames).
  Scene coordinates are physical px and the texture is full-window physical
  size, so identity maps image pixel (0, 0) to scene (0, 0).
- `fill_backdrop_lens` paints `shape` with the sharp capture zoomed by
  `zoom` around `center` (both physical px) via the brush transform
  (below 1.0 minifies, above 1.0 magnifies); no-op
  when `backdrop_sharp()` is `None`.
- `stroke_backdrop_edge` strokes `ring` with the blurred capture; callers
  inset the ring by half the band and stroke at full band width so the frost
  sits inside the body outline.

## FontSystem

```rust
pub struct FontSystem {
    pub scale: f32,
}
```

Wraps a Parley `FontContext` (system fonts, so SF Pro resolves on TontooOS)
and a `LayoutContext`. `scale` is the window scale factor set by the loop;
it is passed to Parley as the display scale so glyph positions quantize to
physical pixel boundaries, and layouts are built in physical pixels so
glyphs stay crisp.

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

Draws a finished layout at logical position (`x`, `y`). The origin is
snapped to physical pixels first (a fractional offset would push the
quantized glyphs off-grid and blur the text, esp. at fractional window
scales like 125%/150%), then glyph runs are recorded with
`Scene::draw_glyphs` and hinting enabled. Only glyph runs are drawn;
inline boxes are skipped.

## Images

```rust
pub fn get(&mut self, name: &str, tint: Color, target_px: u32) -> Option<(ImageData, u32, u32)>
pub fn backdrop(&self) -> Option<&ImageData>
pub fn backdrop_sharp(&self) -> Option<&ImageData>
pub fn is_capture_pass(&self) -> bool
```

Per-frame SF Symbol access for views. Resolves CoreIcon artwork by name
(`COREICON_ASSETS_DIR` override or system resources on TontooOS),
recolors the black glyph to `tint` and uploads once; later frames hit the
shell-owned cache. Assets are 1024 px: they downscale on the CPU with
Lanczos3 to `target_px` (pass ~2x the display size) because GPU
minification without mipmaps turns them to mush. Returns the upload plus
natural size; callers scale with the draw transform preserving aspect.
Missing or undecodable assets return `None` so callers skip the icon.

Backdrop access during the two-pass frame:

| Method | Returns |
|---|---|
| `backdrop()` | Blurred capture for glass edge fills, or `None` on the capture pass / single-pass frames |
| `backdrop_sharp()` | Sharp capture for the zoomed lens center, or `None` on the capture pass / single-pass frames |
| `is_capture_pass()` | True while recording the pre-blur capture; glass bodies must skip drawing |

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
use tontooui::elements::Titlebar;
use tontooui::renderer::FontSystem;
use tontooui::renderer::window::{App, Viewport, run};
use vello::Scene;

struct Hello {
    bar: Titlebar,
}

impl App for Hello {
    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, viewport: Viewport, _t: f64) {
        self.bar.set_rect(viewport.x, viewport.y, viewport.width);
        self.bar.draw(scene, fonts);
    }
}

fn main() {
    run("Hello", 800, 600, Hello {
        bar: Titlebar::new("Hello"),
    }).unwrap();
}
```

## Cross References

- [Titlebar.md](Titlebar.md) – custom decoration bar with drag region
- [Layout.md](Layout.md) – VStack, HStack, ZStack, Spacer and the View trait
- [Theme.md](Theme.md) – live dark/light plus accent with fade animation
- [Glass.md](Glass.md) – liquid glass container plus transparent body
- [Button.md](Button.md) – standard button with CoreIcon SF Symbols
- [Slider.md](Slider.md) – slider with steps, labels, ticks and glass track
