# ViewThatFits

ViewThatFits is a SwiftUI-style adaptive container that picks the first child that fits the available space, falling back to the last child. It adapts to horizontal, vertical or both axes (Light/Dark `#1d1d1d` / `#ececec` via system scheme, inner Text uses SF Pro).

## Constructor

```rust
pub fn new() -> Self
pub fn horizontal() -> Self
pub fn vertical() -> Self
```

Creates an adaptive container. `new()` uses `Both` axes, `horizontal()` and `vertical()` constrain to one axis.

## Builder Methods

| Method | Signature | Description |
|---|---|---|
| `axis` | `axis(self, a: ViewThatFitsAxis) -> Self` | Set fitting axis (`Horizontal`, `Vertical`, `Both`) |
| `child` | `child(self, w: impl Widget + 'static) -> Self` | Add child widget (in order, first that fits is shown) |
| `children` | `children(self, v: Vec<Box<dyn Widget>>) -> Self` | Replace children |
| `to_view` | `to_view(self) -> View` | Wrap in a `View` with frame `300×80` |

## ViewThatFitsAxis

```rust
pub enum ViewThatFitsAxis { Horizontal, Vertical, Both }
```

- `Horizontal` — fits if `natural_width <= available.width`
- `Vertical` — fits if `natural_height <= available.height`
- `Both` — fits only if both dimensions fit

## Behavior

- Measurement uses `Widget::to_gtk().measure()` for natural size at render time (`ViewContent::render` receives `frame: Rect` with available size). When `frame` is zero (pure `Widget` path), the first child is shown; true fitting happens when wrapped in a `View` with a frame (as in the demo).
- If no child fits, the last child (fallback) is shown even if it overflows, matching SwiftUI `ViewThatFits` fallback semantics.
- The container itself is transparent — children sit directly on the app background. No extra card or border is added by the element.
- Axes `ViewThatFits::vertical()` is used for "ViewThatFits Vertical" in the reference (adapts to available vertical space).

## Usage / Example

```rust
use tontooui::prelude::*;
use tontooui::{ViewThatFits, ViewThatFitsAxis};
use uikit::view::View;

// Available width 300 → first child (300×20) fits; width 220 → second (200×20) fits
let wide = View::new(
    ViewThatFits::new()
        .child(Text::new("Available width 300"))
        .child(Text::new("Available width 200"))
        .child(Text::new("Fallback"))
).with_frame(0.0, 0.0, 300.0, 24.0);

let narrow = View::new(
    ViewThatFits::new()
        .child(Text::new("Available width 300"))
        .child(Text::new("Available width 200"))
        .child(Text::new("Fallback"))
).with_frame(0.0, 0.0, 220.0, 24.0);

// Vertical variant
let vertical = View::new(
    ViewThatFits::vertical()
        .child(Text::new("Size w300 h30"))
        .child(Text::new("Size w200 h20"))
        .child(Text::new("Fallback"))
).with_frame(0.0, 0.0, 320.0, 50.0);
```

See `examples/view_that_fits.rs` for the 1:1 demo (2 variants: `ViewThatFits Vertical` and `ViewThatFits`) directly on background, pure TontooUI API, Ampeln visible.

## Cross References

- [MAIN.md](MAIN.md) -- library overview
- [Divider.md](Divider.md) -- separator used between fitted views
