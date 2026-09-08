# ConcentricRectangle

SwiftUI-style ConcentricRectangle category for TontooUI, recreating concentric rectangle shapes. The category contains two `initializer` elements. Rendering is directly on the window background (#1d1d1d dark / #ececec light) with no extra card, using `SF Pro Display`.

## UniformConcentricRectangle

```rust
pub struct UniformConcentricRectangle { /* ... */ }
impl UniformConcentricRectangle {
    pub fn new() -> Self;
    pub fn corner_radius(self, v: f32) -> Self;
    pub fn inset(self, v: f32) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — create a rectangle with the same corner style set on four corners.

- Preview: translucent gray outer `100x78` (`20px`), red inner inset by `8px`, blue base capsule `120x16`.
- Size `180x110`.

## ConcentricRectangle

```rust
pub struct ConcentricRectangle { /* ... */ }
impl ConcentricRectangle {
    pub fn new() -> Self;
    pub fn corners(self, tl: f32, tr: f32, bl: f32, br: f32) -> Self;
    pub fn inset(self, v: f32) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — a concentric rectangle whose corner radii are defined from the same circle.

- Preview: gray outer `100x78` with per-corner radii, red inner with each radius minus the inset, blue base capsule.
- Size `180x110`.

## Usage / Example

Run the gallery demo:

```bash
cargo run --example concentric_rectangles
```

Minimal usage:

```rust
use tontooui::prelude::*;

let uniform = UniformConcentricRectangle::new().corner_radius(20.0).inset(8.0);
let concentric = ConcentricRectangle::new().corners(22.0, 22.0, 22.0, 22.0).inset(8.0);

let root = VStack::new()
    .spacing(8.0)
    .child(uniform.to_view())
    .child(concentric.to_view());
```

Category folder layout:

```
src/elements/concentric_rectangles/
  mod.rs                            // category root
  uniform_concentric_rectangle.rs   // UniformConcentricRectangle
  concentric_rectangle.rs           // ConcentricRectangle
```

## Cross References

- [Shapes.md](Shapes.md) -- `RoundedRectangle` and `UnevenRoundedRectangle` share corner radius semantics
- [Color.md](Color.md) -- red and blue fills use the standard palette
