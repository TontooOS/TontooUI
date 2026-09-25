# ScrollView

Vertical scroll container in `src/elements/scrollview.rs`: wraps one
child `View` and keeps the window at its size. The child keeps its
intrinsic height, but only the placed rect is visible: content outside
is clipped and reached through an integrated `Scrollbar` (wheel, thumb
drag, track page jump) instead of growing the window.

## Geometry

| Token | Value |
|---|---|
| `SCROLLVIEW_BAR_W` | 10 px overlay bar, matches `SCROLLBAR_W_HOVER` |

## ScrollView

```rust
pub fn new(child: impl View + 'static) -> Self
pub fn child_mut<T: View + 'static>(&mut self) -> Option<&mut T>
pub fn total(&self) -> f32
pub fn visible(&self) -> f32
pub fn offset(&self) -> f32
pub fn max_offset(&self) -> f32
pub fn scrollable(&self) -> bool
pub fn set_offset(&mut self, offset: f32)
pub fn scroll_by(&mut self, delta: f32)
pub fn scroll_to(&mut self, offset: f32)
pub fn flash(&mut self)
pub fn accent(self, color: Color) -> Self
pub fn disabled(self, disabled: bool) -> Self
pub fn on_scroll(self, callback: impl FnMut(f32) + 'static) -> Self
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_move(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64)
pub fn mouse_wheel(&mut self, dx: f64, dy: f64)
```

- Place with a bounded height (directly with the viewport or as a
  stack child); `flex` is `1.0`, so stacks hand it the remaining space
  instead of its full content height and surrounding windows keep
  their size. `measure` reports the intrinsic content size.
- The bar overlays the right edge (`SCROLLVIEW_BAR_W`), stays hidden
  until needed and fades out after `SCROLLBAR_HIDE_DELAY` idle seconds
  (see [Scrollbar.md](Scrollbar.md)). Without overflow
  (`total <= visible`) all bar input is ignored.
- `draw` re-syncs every frame: wheel and thumb drags change the offset
  without a new `place` call, and grown children (new rows, longer
  text) are picked up. `set_content` flashes the bar, so it only runs
  when the `(total, visible)` model changed; every-frame calls would
  keep the bar awake forever.
- Presses on the bar drive the bar and never reach the child
  underneath; presses inside the viewport forward with raw coordinates
  (the child was placed shifted by the offset, so scrolled-away content
  never hits). Presses outside the viewport are dropped.
- `mouse_wheel` scrolls in logical px (right/down positive, like the
  shell). The child re-places on the next draw, so no fonts are needed.
- The `View` trait carries no `mouse_down` with press tracking for
  some children (e.g. `FormattedText` link arming is inherent-only):
  forward `mouse_down` to the concrete child as well when links must
  arm. `mouse_up` already arrives through the trait.
- `child_mut::<T>()` reaches the wrapped child for state updates
  (theming rows, retitling), so nesting stays transparent.
- Returns `None` from `child_mut` for a wrong type.

```rust
let mut scroll = ScrollView::new(
    VStack::new().spacing(4.0).child(BasicText::new("Row 1")),
);
scroll.place(fonts, x, y, width, 400.0);
scroll.draw(scene, fonts, images);
```

## Usage / Example

Run `cargo run --example scrollview`: 60 text rows in a fixed 420 px
window with the scrollbar at the right edge. Adding rows never grows
the window; scroll with the wheel, drag the thumb or click the track.
The titlebar shows the live offset. `cargo run --example text` shows
the same pattern for a mixed text stack with clickable links.

## Cross References

- [Scrollbar.md](Scrollbar.md) – overlay bar used inside, fade and drag model
- [Layout.md](Layout.md) – stacks hosting the view, `View` trait, `flex`
- [Renderer.md](Renderer.md) – frame loop, `App` shell, screen-size clamp
- [Text.md](Text.md) – scrollable text rows, link press handling
