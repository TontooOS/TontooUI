# Scrollbar

Overlay scrollbar in `src/elements/scrollbar.rs` (no subcategory): a
side bar for scrollable content. The thumb stays hidden until needed:
it fades in while scrolling, hovering or dragging and fades out after
`SCROLLBAR_HIDE_DELAY` idle seconds. Hovering widens the thumb and
reveals the full track from top to bottom, so the whole travel range
is visible while aiming. Dragging it follows the mouse directly and
clicking the track jumps one page toward the click.

## Geometry

| Token | Value |
|---|---|
| `SCROLLBAR_W` / `SCROLLBAR_W_HOVER` | 6 px idle thumb / 10 px hover and drag thumb |
| `SCROLLBAR_MIN_THUMB` | 24 px minimum thumb length |
| `SCROLLBAR_MIN_TRACK` | 44 px minimum track height (stack measure) |
| `SCROLLBAR_ALPHA` | 180 thumb base alpha (multiplied by the fade) |
| `SCROLLBAR_FADE_SECONDS` | 0.25 s fade in and out |
| `SCROLLBAR_HIDE_DELAY` | 1.0 s idle before fading out |
| `SCROLLBAR_PAGE_ANIM_SECONDS` | 0.25 s track-click page jump |
| `SCROLLBAR_HOVER_LIGHTEN` | 0.15 hover lighten toward white |
| `SCROLLBAR_PRESS_DARKEN` | 0.12 press darken toward black |
| `SCROLLBAR_WIDEN_SPEED` | 30 px per second widen speed |
| `SCROLLBAR_HIT_W` | 14 px generous thumb hit width |
| `SCROLLBAR_GRAY` | `#8E8E93` default thumb gray |
| `SCROLLBAR_TRACK_DARK` / `SCROLLBAR_TRACK_LIGHT` | Translucent full-height track, shown while aiming |
| `SCROLLBAR_DEFAULT_ACCENT` | `#007AFF` theme accent that maps to gray |

## Scrollbar

```rust
pub fn new() -> Self
pub fn content(self, total: f32, visible: f32) -> Self
pub fn accent(self, color: Color) -> Self
pub fn disabled(self, disabled: bool) -> Self
pub fn on_scroll(self, callback: impl FnMut(f32) + 'static) -> Self
pub fn set_content(&mut self, total: f32, visible: f32)
pub fn set_rect(&mut self, x: f32, y: f32, w: f32, h: f32)
pub fn offset(&self) -> f32
pub fn max_offset(&self) -> f32
pub fn scrollable(&self) -> bool
pub fn is_visible(&self) -> bool
pub fn opacity(&self) -> f32
pub fn set_offset(&mut self, offset: f32)
pub fn scroll_by(&mut self, delta: f32)
pub fn scroll_to(&mut self, offset: f32)
pub fn flash(&mut self)
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_move(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64)
pub fn mouse_wheel(&mut self, dx: f64, dy: f64)
```

- The offset model is logical px like the date popup lists: `total`
  is the content length, `visible` the viewport length, `0.0` the top
  and `max_offset()` (`total - visible`, at least `0.0`) the bottom.
  Without overflow (`total <= visible`) the bar never shows and all
  input is ignored.
- The thumb length is proportional (`track * visible / total`,
  at least `SCROLLBAR_MIN_THUMB`) and floats right-aligned in the
  placed rect. The bar spans the placed height, so prefer manual
  `set_rect` (or a `Frame`) for full-height side placement;
  `measure` returns the hover width by `SCROLLBAR_MIN_TRACK`.
- `mouse_wheel` scrolls immediately (`dy` in logical px, down
  positive, like the date popup) and flashes the bar.
  `set_offset` and `scroll_by` are immediate and fire `on_scroll`
  when the offset changed; `set_offset` to the same value flashes
  without firing. `scroll_to` tweens to the offset with a
  `CubicOut` tween while the offset follows the thumb, so the
  content glides with it.
- Pressing the thumb drags it with a stable grab offset (no jump);
  releasing ends the drag anywhere. Clicking the track above or
  below the thumb jumps one page (`visible`) toward the click.
- Hovering or dragging reveals the full-height track behind the
  thumb (subtle mode-gray fill), so the travel range reads from top
  to bottom while aiming; it fades out with the same speed when the
  pointer leaves.
- The fade targets opacity `1.0` while dragging, hovering or within
  `SCROLLBAR_HIDE_DELAY` of the last activity, else `0.0`, at
  `SCROLLBAR_FADE_SECONDS` speed. `flash` restarts the delay (e.g.
  after the content changed). `set_content` flashes and reclamps
  the offset into range.
- Color: the thumb is `SCROLLBAR_GRAY` while the theme accent is the
  default (`Multicolor` and `Blue` both resolve to
  `SCROLLBAR_DEFAULT_ACCENT`) and follows any other theme accent. A
  manual `accent` wins over the system accent. Hover lightens the
  thumb by `SCROLLBAR_HOVER_LIGHTEN`, pressing darkens it by
  `SCROLLBAR_PRESS_DARKEN`. Unfocused windows desaturate it like the
  rest of the palette.

```rust
let mut bar = Scrollbar::new()
    .content(1320.0, 280.0)
    .on_scroll(|offset| println!("offset: {offset}"));
bar.set_theme(accent, true);
bar.set_rect(x, y, 10.0, 280.0);
```

## Usage / Example

Run `cargo run --example scrollbar`: 60 text lines in a clipped
column with the scrollbar at the right edge. Scroll with the wheel,
hover to widen the bar, drag the thumb or click the track to page.
The titlebar shows the live offset; the bar fades out when idle.

## Cross References

- [Picker.md](Picker.md) – date popup year list with the same offset model
- [Slider.md](Slider.md) – drag, track-click tween and `on_change` callback
- [Layout.md](Layout.md) – stacks hosting the bar, `View` trait, `Frame`
- [Renderer.md](Renderer.md) – frame loop, `App` shell, mouse forwarding
- [Animation.md](Animation.md) – tween driver used by the page jump
- [Theme.md](Theme.md) – accent, mode and Multicolor default
