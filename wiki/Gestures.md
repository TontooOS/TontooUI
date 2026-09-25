# Gestures

Gestures category in `src/elements/gestures/`: `GestureArea<V>` in
`area.rs` wraps any child element and tracks tap, double tap, long
press, drag, magnify (mouse wheel) and hover inside its placed
rect, reporting them through callbacks back to the app. The child
fills the area; with `draggable` it follows the drag offset and
with `zoomable` it scales around the center. Display-only
otherwise (the wrapper draws just its child).

## Geometry

| Token | Value |
|---|---|
| `GESTURE_LONG_PRESS_SECONDS` | 0.6 s hold time for a long press (mirrors context menus) |
| `GESTURE_DOUBLE_TAP_SECONDS` | 0.4 s window for the second tap of a double tap |
| `GESTURE_MOVE_SLOP` | 10 px wander allowance: moving further cancels taps and long press, starts a drag |
| `GESTURE_MAGNIFY_STEP` | 0.005 zoom factor per wheel notch (logical px delta scaled by this) |
| `GESTURE_MAGNIFY_MIN` / `GESTURE_MAGNIFY_MAX` | 0.25 / 4.0 zoom clamp |

## GestureArea

```rust
pub fn new(child: V) -> Self
pub fn on_tap(self, callback: impl FnMut() + 'static) -> Self
pub fn on_double_tap(self, callback: impl FnMut() + 'static) -> Self
pub fn on_long_press(self, callback: impl FnMut() + 'static) -> Self
pub fn on_drag(self, callback: impl FnMut(f32, f32) + 'static) -> Self
pub fn on_magnify(self, callback: impl FnMut(f32) + 'static) -> Self
pub fn on_hover(self, callback: impl FnMut(bool) + 'static) -> Self
pub fn draggable(self, enabled: bool) -> Self
pub fn zoomable(self, enabled: bool) -> Self
pub fn child_mut(&mut self) -> &mut V
pub fn drag_offset(&self) -> (f32, f32)
pub fn scale_value(&self) -> f32
pub fn is_hovered(&self) -> bool
pub fn reset(&mut self)
pub fn rect(&self) -> (f32, f32, f32, f32)
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64)
pub fn mouse_move(&mut self, x: f64, y: f64)
pub fn mouse_wheel(&mut self, dx: f64, dy: f64)
```

- Presses outside the placed rect are ignored. Tap is a quick
  release inside without wandering and without a long press having
  fired. Double tap is a second tap within the window and slop: each
  release still fires `on_tap`, the second additionally fires
  `on_double_tap`. Long press fires once while holding past the hold
  time (polled every draw, so no release is needed); wandering past
  the slop cancels it and starts a drag instead, which also
  suppresses the tap.
- Drag reports the total `(dx, dy)` offset from the press start in
  logical px on every move past the slop. Magnify needs a hover
  first (`mouse_move` tracks it) and reports the clamped scale,
  starting at 1.0.
- Hover fires `on_hover(true)` when the pointer enters the placed
  rect and `on_hover(false)` when it leaves, edge triggered (no
  repeats while resting). `is_hovered` reads the live state, e.g.
  for status lines. Nested areas track through `View::set_hover`
  as well as `mouse_move`.
- `draggable` shifts the child's placed origin by the drag offset;
  `zoomable` scales its placed size around the area center.
  Rect-filling children scale truly; baked glyphs keep their size
  (same rule as `Animated`).
- `measure` stays untransformed so stacks never jitter; `reset`
  returns offset and scale to rest. No network, no timers beyond
  `Instant`: all state transitions are unit-tested without sleeps.

## Usage / Example

```rust
use tontooui::elements::{GestureArea, Rectangle, View, VStack};
use vello::peniko::Color;

let pad = GestureArea::new(Rectangle::new(220.0, 110.0).fill(Color::from_rgb8(0x00, 0x7a, 0xff)))
  .on_tap(|| println!("tap"))
  .on_long_press(|| println!("long"))
  .on_drag(|dx, dy| println!("drag {dx} {dy}"))
  .on_magnify(|s| println!("zoom {s}"))
  .on_hover(|inside| println!("hover {inside}"))
  .draggable(true)
  .zoomable(true);
```

Wire input per frame from the app (see `examples/gestures.rs`):

```rust
pad.mouse_down(x, y);
pad.mouse_up(x, y);
pad.mouse_move(x, y);
pad.mouse_wheel(dx, dy);
```

## Cross References

- [Layout.md](Layout.md) – stacks forward presses to nested children
- [Animation.md](Animation.md) – `Animated` shares the scale-box rule
- [Images.md](Images.md) – zoomable raster children
- [Shapes.md](Shapes.md) – draggable pads
