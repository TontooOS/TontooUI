# Shapes

SwiftUI-style Shapes category for TontooUI, recreating shape views. The category contains seven `modifier` elements that cover circles, ellipses, capsules, rectangles, and container-relative shapes. Rendering is directly on the window background (#1d1d1d dark / #ececec light) with no extra card, using `SF Pro Display`.

## Circle

```rust
pub struct Circle { /* ... */ }
impl Circle {
    pub fn new() -> Self;
    pub fn fill(self, c: Color) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — a circle shape centered in its frame.

- Preview: blue `#0A84FF` circle `72x72` with `Foo` on a `140x110` phone card.
- Size `180x110`.

## Ellipse

```rust
pub struct Ellipse { /* ... */ }
impl Ellipse {
    pub fn new() -> Self;
    pub fn fill(self, c: Color) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — an elliptical shape filling its frame.

- Preview: blue ellipse `104x60` with `Foo`.
- Size `180x110`.

## Capsule

```rust
pub struct Capsule { /* ... */ }
impl Capsule {
    pub fn new() -> Self;
    pub fn fill(self, c: Color) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — a capsule (stadium) shape filling its frame.

- Preview: blue capsule `112x44` (`999px` radius) with `Foo`.
- Size `180x110`.

## RectangleShape

```rust
pub struct RectangleShape { /* ... */ }
impl RectangleShape {
    pub fn new() -> Self;
    pub fn fill(self, c: Color) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — a rectangular shape filling its frame.

- Preview: blue rectangle `96x76` with `Foo`.
- Size `180x110`.

## RoundedRectangle

```rust
pub struct RoundedRectangle { /* ... */ }
impl RoundedRectangle {
    pub fn new() -> Self;
    pub fn corner_radius(self, v: f32) -> Self;
    pub fn fill(self, c: Color) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — a rectangle with rounded corners.

- Preview: blue rounded rect `100x76` (`16px` radius) with `Foo`.
- Size `180x110`.

## UnevenRoundedRectangle

```rust
pub struct UnevenRoundedRectangle { /* ... */ }
impl UnevenRoundedRectangle {
    pub fn new() -> Self;
    pub fn corners(self, tl: f32, tr: f32, bl: f32, br: f32) -> Self;
    pub fn fill(self, c: Color) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — a rectangle with uneven corner radii.

- Preview: blue rect `100x76` with corners `24/8/8/24` and `Foo`.
- Size `180x110`.

## ContainerRelativeShape

```rust
pub struct ContainerRelativeShape { /* ... */ }
impl ContainerRelativeShape {
    pub fn new() -> Self;
    pub fn inset(self, v: f32) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — a shape that is replaced by an inset version of the current container shape.

- Preview: nested shapes on phone — blue `84x88` (`22px`) with `Foo`, red `56x56` (`14px`) with `Bar`, green `36x28` (`8px`) with `Foo`.
- Size `180x110`.

## Usage / Example

Run the gallery demo:

```bash
cargo run --example shapes
```

Minimal usage:

```rust
use tontooui::prelude::*;

let circle = Circle::new();
let rounded = RoundedRectangle::new().corner_radius(16.0);
let uneven = UnevenRoundedRectangle::new().corners(24.0, 8.0, 8.0, 24.0);
let container = ContainerRelativeShape::new().inset(8.0);

let root = VStack::new()
    .spacing(8.0)
    .child(circle.to_view())
    .child(container.to_view());
```

Category folder layout:

```
src/elements/shapes/
  mod.rs                         // category root
  circle.rs                      // Circle
  ellipse.rs                     // Ellipse
  capsule.rs                     // Capsule
  rectangle.rs                   // RectangleShape
  rounded_rectangle.rs           // RoundedRectangle
  uneven_rounded_rectangle.rs    // UnevenRoundedRectangle
  container_relative_shape.rs    // ContainerRelativeShape
```

## Cross References

- [View.md](View.md) -- `GlassEffect` can overlay shapes with frosted glass
- [Color.md](Color.md) -- `fill` uses `Color` system palettes
- [Sheet.md](Sheet.md) -- `SheetCornerRadius` uses matching corner radius semantics
