# Colors

Colors category in `src/elements/colors/`: `SystemColor` in
`system.rs` names the thirteen system colors (no hex or RGB needed)
and `GradientPaint` in `gradients.rs` builds the general linear,
vertical, radial and angular (conic sweep) gradients, like the
reference bars. Paint-only helpers (no views): devs fill shapes,
text and bars with them.

## System Colors

| Name | Value |
|---|---|
| `Red` | `#FF3B30` |
| `Orange` | `#FF9500` |
| `Yellow` | `#FFCC00` |
| `Green` | `#34C759` |
| `Mint` | `#00C7BE` |
| `Teal` | `#30B0C7` |
| `Cyan` | `#32ADE6` |
| `Blue` | `#007AFF` |
| `Indigo` | `#5856D6` |
| `Purple` | `#AF52DE` |
| `Pink` | `#FF2D55` |
| `Brown` | `#A2845E` |
| `Gray` | `#8E8E93` |

```rust
pub enum SystemColor { Red, Orange, Yellow, Green, Mint, Teal, Cyan, Blue, Indigo, Purple, Pink, Brown, Gray }
pub const ALL_SYSTEM_COLORS: [SystemColor; 13]
pub fn name(&self) -> &'static str
pub fn from_str(raw: &str) -> Option<Self>
pub fn color(&self) -> Color
```

- Values are the Apple system colors, matching the theme accents
  where they overlap. `from_str` parses display names
  case-insensitively (`None` for unknown names); `Color: From<SystemColor>`
  converts implicitly.

## GradientPaint

```rust
pub enum GradientPaint {
    Linear { colors: Vec<Color>, angle_deg: f32 },
    Vertical { colors: Vec<Color> },
    Radial { colors: Vec<Color> },
    Angular { colors: Vec<Color> },
}
pub fn linear(colors: Vec<Color>, angle_deg: f32) -> Self
pub fn linear_system(colors: Vec<SystemColor>, angle_deg: f32) -> Self
pub fn vertical(colors: Vec<Color>) -> Self
pub fn vertical_system(colors: Vec<SystemColor>) -> Self
pub fn radial(colors: Vec<Color>) -> Self
pub fn radial_system(colors: Vec<SystemColor>) -> Self
pub fn angular(colors: Vec<Color>) -> Self
pub fn angular_system(colors: Vec<SystemColor>) -> Self
pub fn preset_linear() -> Self
pub fn preset_vertical() -> Self
pub fn preset_radial() -> Self
pub fn preset_angular() -> Self
pub fn colors(&self) -> Vec<Color>
pub fn brush(&self, x: f32, y: f32, width: f32, height: f32, scale: f32) -> Brush
```

- `Linear` blends at `angle_deg` (0 = left to right);
  `Vertical` blends top to bottom; `Radial` glows from the center
  out to the edge; `Angular` sweeps conically around the center.
- The presets match the reference bars: blue to purple horizontal,
  red-orange-yellow vertical, bright center glow into purple, and
  the conic rainbow wheel (which closes its loop).
- Stops spread evenly from 0.0 to 1.0; empty input falls back to
  blue so the brush is never empty. `brush` spans the paint over a
  logical rect at the display scale (`fonts.scale`).

## Usage / Example

```rust
use tontooui::elements::{GradientPaint, SystemColor};
use vello::peniko::Color;

// No hex needed:
let teal: Color = SystemColor::Teal.color();
let tint: Color = SystemColor::from_str("pink").unwrap_or(SystemColor::Gray).into();

// Reference bars:
let linear = GradientPaint::preset_linear();
let sunset = GradientPaint::vertical_system(vec![
    SystemColor::Red,
    SystemColor::Orange,
    SystemColor::Yellow,
]);
let brush = linear.brush(x, y, width, height, fonts.scale);
```

See `examples/colors.rs` for the full demo (all thirteen swatches
plus the four labeled reference bars).

## Cross References

- [Shapes.md](Shapes.md) – `ShapeFill` paints solids and gradients into shapes
- [Theme.md](Theme.md) – theme accents overlap the system palette
- [Renderer.md](Renderer.md) – display scale for spanning brushes
