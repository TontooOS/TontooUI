# Shapes

Shapes category in `src/elements/shapes/`: `Rectangle` in
`rectangle.rs`, `Circle` in `circle.rs`, `RoundedRectangle` in
`rounded.rs`, `Capsule` in `capsule.rs` and `CustomShape` in
`custom.rs`. Every shape paints a solid color, a linear gradient or a
radial gradient when filled, and draws the outline only (transparent
fill, `stroke` border) when unfilled, like the reference rows: filled
blue, red outline and blue-to-purple gradient rectangles; solid green,
orange outline and radial-highlight circles; solid teal, purple
outline and yellow-to-orange gradient capsules; solid blue triangle
and solid purple hexagon customs. Display-only (no mouse handling).
Shared paint types live in `mod.rs`.

## ShapeFill

```rust
pub enum ShapeFill { Solid(Color), LinearGradient { colors: Vec<Color>, angle_deg: f32 }, RadialGradient { colors: Vec<Color> } }
pub fn solid(color: Color) -> Self
pub fn linear(colors: Vec<Color>, angle_deg: f32) -> Self
pub fn radial(colors: Vec<Color>) -> Self
```

- `Solid` is one flat color. `LinearGradient` blends across the shape
  bounds at `angle_deg` (0 = left to right, 90 = top to bottom, like
  the reference gradients). `RadialGradient` blends from the shape
  center to its edge (like the blue ball reference row).
- Stops spread evenly from 0.0 to 1.0; a single color behaves like a
  solid and empty input falls back to `SHAPE_DEFAULT_FILL` so the
  brush is never empty.
- Unfocused windows desaturate every stop like the rest of the palette.

## Geometry

| Token | Value |
|---|---|
| `SHAPE_DEFAULT_FILL` | `#007AFF` system blue fill default |
| `SHAPE_DEFAULT_STROKE` | `#FF3B30` system red outline default |
| `SHAPE_STROKE_W` | 4 px default outline width |
| `SHAPE_CORNER_RADIUS` | 16 px default `RoundedRectangle` radius |
| `SHAPE_SHADOW_DY` / `SHAPE_SHADOW` | 2 px offset, black 64 alpha drop shadow |

## Rectangle

```rust
pub fn new(width: f32, height: f32) -> Self
pub fn fill(self, color: Color) -> Self
pub fn linear_gradient(self, colors: Vec<Color>, angle_deg: f32) -> Self
pub fn radial_gradient(self, colors: Vec<Color>) -> Self
pub fn filled(self, filled: bool) -> Self
pub fn stroke(self, color: Color) -> Self
pub fn stroke_width(self, px: f32) -> Self
pub fn shadow(self, shadow: bool) -> Self
pub fn set_fill(&mut self, fill: ShapeFill)
pub fn set_filled(&mut self, filled: bool)
pub fn set_stroke(&mut self, color: Color)
pub fn set_stroke_width(&mut self, px: f32)
pub fn set_shadow(&mut self, shadow: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn fill_value(&self) -> &ShapeFill
pub fn is_filled(&self) -> bool
pub fn stroke_color(&self) -> Color
pub fn stroke_width_value(&self) -> f32
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- `measure` returns the intrinsic size; `place` stores the drawn rect.
- `filled(true)` paints the interior with `ShapeFill` (solid, linear
  or radial). `filled(false)` draws the outline only: transparent
  fill with the `stroke` border at `stroke_width` (minimum 1 px so the
  outline never vanishes). Filled shapes stay borderless like the
  reference rows.
- Sizes and `stroke_width` clamp to >= 0; an empty rect draws nothing.
- `shadow` draws a 2 px offset translucent copy behind the shape.
  Shapes are flat display elements, so the default is off.

## Circle

```rust
pub fn new(diameter: f32) -> Self
pub fn fill(self, color: Color) -> Self
pub fn linear_gradient(self, colors: Vec<Color>, angle_deg: f32) -> Self
pub fn radial_gradient(self, colors: Vec<Color>) -> Self
pub fn filled(self, filled: bool) -> Self
pub fn stroke(self, color: Color) -> Self
pub fn stroke_width(self, px: f32) -> Self
pub fn shadow(self, shadow: bool) -> Self
pub fn default_stroke() -> Color
pub fn set_fill(&mut self, fill: ShapeFill)
pub fn set_filled(&mut self, filled: bool)
pub fn set_stroke(&mut self, color: Color)
pub fn set_stroke_width(&mut self, px: f32)
pub fn set_shadow(&mut self, shadow: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn fill_value(&self) -> &ShapeFill
pub fn is_filled(&self) -> bool
pub fn stroke_color(&self) -> Color
pub fn stroke_width_value(&self) -> f32
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Same fill/outline/shadow contract as `Rectangle`, drawn as an
  ellipse over the placed rect.
- Defaults match the reference rows: green `#34C759` fill, orange
  `#FF9500` outline.

## RoundedRectangle

```rust
pub fn new(width: f32, height: f32, corner_radius: f32) -> Self
pub fn fill(self, color: Color) -> Self
pub fn linear_gradient(self, colors: Vec<Color>, angle_deg: f32) -> Self
pub fn radial_gradient(self, colors: Vec<Color>) -> Self
pub fn filled(self, filled: bool) -> Self
pub fn stroke(self, color: Color) -> Self
pub fn stroke_width(self, px: f32) -> Self
pub fn corner_radius(self, px: f32) -> Self
pub fn shadow(self, shadow: bool) -> Self
pub fn set_fill(&mut self, fill: ShapeFill)
pub fn set_filled(&mut self, filled: bool)
pub fn set_stroke(&mut self, color: Color)
pub fn set_stroke_width(&mut self, px: f32)
pub fn set_corner_radius(&mut self, px: f32)
pub fn set_shadow(&mut self, shadow: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn fill_value(&self) -> &ShapeFill
pub fn is_filled(&self) -> bool
pub fn stroke_color(&self) -> Color
pub fn stroke_width_value(&self) -> f32
pub fn corner_radius_value(&self) -> f32
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Same fill/outline/shadow contract as `Rectangle`, drawn as a
  `RoundedRect`. The radius clamps to half the smaller placed side so
  huge radii degrade to a capsule instead of breaking.

## Capsule

```rust
pub fn new(width: f32, height: f32) -> Self
pub fn fill(self, color: Color) -> Self
pub fn linear_gradient(self, colors: Vec<Color>, angle_deg: f32) -> Self
pub fn radial_gradient(self, colors: Vec<Color>) -> Self
pub fn filled(self, filled: bool) -> Self
pub fn stroke(self, color: Color) -> Self
pub fn stroke_width(self, px: f32) -> Self
pub fn shadow(self, shadow: bool) -> Self
pub fn default_stroke() -> Color
pub fn set_fill(&mut self, fill: ShapeFill)
pub fn set_filled(&mut self, filled: bool)
pub fn set_stroke(&mut self, color: Color)
pub fn set_stroke_width(&mut self, px: f32)
pub fn set_shadow(&mut self, shadow: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn fill_value(&self) -> &ShapeFill
pub fn is_filled(&self) -> bool
pub fn stroke_color(&self) -> Color
pub fn stroke_width_value(&self) -> f32
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Same fill/outline/shadow contract as `Rectangle`. The radius is
  always half the smaller placed side, so the ends stay fully round
  at any size.
- Defaults match the reference rows: teal fill, indigo `#5856D6` outline.

## CustomShape

```rust
pub fn points(points: Vec<(f32, f32)>, width: f32, height: f32) -> Self
pub fn triangle(width: f32, height: f32) -> Self
pub fn diamond(width: f32, height: f32) -> Self
pub fn polygon(sides: usize, width: f32, height: f32) -> Self
pub fn pentagon(width: f32, height: f32) -> Self
pub fn hexagon(width: f32, height: f32) -> Self
pub fn star(width: f32, height: f32) -> Self
pub fn fill(self, color: Color) -> Self
pub fn linear_gradient(self, colors: Vec<Color>, angle_deg: f32) -> Self
pub fn radial_gradient(self, colors: Vec<Color>) -> Self
pub fn filled(self, filled: bool) -> Self
pub fn stroke(self, color: Color) -> Self
pub fn stroke_width(self, px: f32) -> Self
pub fn shadow(self, shadow: bool) -> Self
pub fn set_fill(&mut self, fill: ShapeFill)
pub fn set_filled(&mut self, filled: bool)
pub fn set_stroke(&mut self, color: Color)
pub fn set_stroke_width(&mut self, px: f32)
pub fn set_shadow(&mut self, shadow: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn fill_value(&self) -> &ShapeFill
pub fn is_filled(&self) -> bool
pub fn stroke_color(&self) -> Color
pub fn stroke_width_value(&self) -> f32
pub fn points_value(&self) -> &[(f32, f32)]
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Points are normalized (0.0 to 1.0 across the shape bounds) and
  scaled to the placed rect on draw; out-of-range points clamp.
- `polygon` clamps to at least 3 sides and starts at the top, so the
  hexagon gets the reference top tip. Fewer than 3 points draw nothing.
- Same fill/outline/shadow contract as `Rectangle`.

## Usage / Example

```rust
use tontooui::elements::{Capsule, Circle, CustomShape, Rectangle, View, VStack};
use vello::peniko::Color;

let stack = VStack::new()
  .spacing(28.0)
  .child(Rectangle::new(240.0, 120.0).fill(Color::from_rgb8(0x0b, 0x5c, 0xe6)))
  .child(Rectangle::new(240.0, 120.0).stroke(Color::from_rgb8(0xee, 0x2c, 0x2c)).filled(false))
  .child(Rectangle::new(240.0, 120.0).linear_gradient(vec![
    Color::from_rgb8(0x1e, 0x6f, 0xf2),
    Color::from_rgb8(0xaf, 0x52, 0xde),
  ], 0.0))
  .child(Circle::new(120.0).radial_gradient(vec![
    Color::WHITE,
    Color::from_rgb8(0x2e, 0x7c, 0xf6),
  ]))
  .child(Capsule::new(240.0, 90.0).linear_gradient(vec![
    Color::from_rgb8(0xff, 0xcc, 0x00),
    Color::from_rgb8(0xff, 0x7a, 0x00),
  ], 0.0))
  .child(CustomShape::hexagon(120.0, 110.0).fill(Color::from_rgb8(0xaf, 0x52, 0xde)));
```

Wire focus per frame (see `examples/shapes.rs`):

```rust
shape.set_focused(focused);
```

## Cross References

- [Layout.md](Layout.md) – stacks size and place shapes via `View`
- [Theme.md](Theme.md) – unfocused desaturation of fills and outlines
- [Divider.md](Divider.md) – another display-only category
