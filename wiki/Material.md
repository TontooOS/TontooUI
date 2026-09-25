# Material

Material category in `src/elements/material/`: `Material<V>` in
`material.rs` wraps any child view over a plain translucent rounded
fill in one of five thicknesses (ultra thin, thin, regular, thick,
ultra thick). Deliberately plain transparency — no zoom, no backdrop
sampling, nothing glass-complex. Presses forward to the child, so
bars and buttons stay interactive inside.

## Geometry

| Token | Value |
|---|---|
| `MATERIAL_RADIUS` | 20 px corner radius (clamped to half the smaller side) |

| Kind | Overlay alpha |
|---|---|
| `UltraThin` | 0.18 white (dark) / black (light) |
| `Thin` | 0.30 white (dark) / black (light) |
| `Regular` | 0.45 white (dark) / black (light) |
| `Thick` | 0.60 white (dark) / black (light) |
| `UltraThick` | 0.75 white (dark) / black (light) |

## MaterialKind

```rust
pub enum MaterialKind { UltraThin, Thin, Regular, Thick, UltraThick }
pub const ALL_MATERIALS: [MaterialKind; 5]
pub fn alpha(&self) -> f32
pub fn name(&self) -> &'static str
```

## Material

```rust
pub fn new(child: V, kind: MaterialKind) -> Self
pub fn ultra_thin(child: V) -> Self
pub fn thin(child: V) -> Self
pub fn regular(child: V) -> Self
pub fn thick(child: V) -> Self
pub fn ultra_thick(child: V) -> Self
pub fn kind(self, kind: MaterialKind) -> Self
pub fn radius(self, px: f32) -> Self
pub fn child_mut(&mut self) -> &mut V
pub fn set_theme(&mut self, dark: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn set_kind(&mut self, kind: MaterialKind)
pub fn set_radius(&mut self, px: f32)
pub fn kind_value(&self) -> MaterialKind
pub fn overlay(&self) -> Color
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- `measure` is the child size; the fill paints the full placed
  rect. Unfocused windows desaturate the veil like the palette.
- Content stays interactive through the `View` protocol
  (`mouse_down`, `mouse_up`, `set_hover` forward to the child).

## Usage / Example

```rust
use tontooui::elements::{BasicText, Material, View, VStack};

let bar = Material::regular(
    BasicText::new("The standard material blur effect."),
);
```

See `examples/material.rs` for the full demo (all five veils over
a blue-to-purple backdrop, like the reference rows).

## Cross References

- [Glass.md](Glass.md) – full liquid glass for complex frosted panels
- [Layout.md](Layout.md) – stacks size and place bars via `View`
- [Shapes.md](Shapes.md) – rounded geometry
