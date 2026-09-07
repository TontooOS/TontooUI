# Material

SwiftUI-style Material category for TontooUI, recreating the frosted glass `Material` from the macOS 26 interface. The category currently contains the single type element `Materials` that shows all thicknesses. Rendering is directly on the window background (#1d1d1d dark / #ececec light) with no extra card — matching the `Color` and `Text` galleries — and uses `SF Pro Display` for labels.

## Material

```rust
pub enum Material {
    UltraThin,
    Thin,
    Regular,
    Thick,
    UltraThick,
    Bar,
}

impl Material {
    pub fn label(self) -> &'static str;
    pub fn alpha(self, is_dark: bool) -> f32;
    pub fn color(self, is_dark: bool) -> Color;
    pub fn border_css(self, is_dark: bool) -> String;
}
```

Mirrors `SwiftUI.Material` (`ultraThinMaterial` … `ultraThickMaterial` + `bar`).

| Variant | Label | Alpha (dark) | Alpha (light) |
|---|---|---|---|
| `UltraThin` | `ultraThinMaterial` | `0.08` | `0.05` |
| `Thin` | `thinMaterial` | `0.14` | `0.09` |
| `Regular` | `regularMaterial` | `0.22` | `0.14` |
| `Thick` | `thickMaterial` | `0.36` | `0.22` |
| `UltraThick` | `ultraThickMaterial` | `0.54` | `0.32` |
| `Bar` | `bar` | `0.72` | `0.58` |

`color()` returns a white frosted `Color` (`rgba(255,255,255, alpha)`) for both schemes — the translucency shows the TontooOS window background through. `border_css()` returns a hairline (`rgba(255,255,255,0.14)` dark / `rgba(0,0,0,0.08)` light) that ensures the swatch remains visible directly on window without a card.

`palette()` returns the 5 main thicknesses shown in the reference card; `all()` includes `Bar`.

## Materials

```rust
pub struct Materials { /* ... */ }

impl Materials {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Type — "All materials". The widget renders two rows directly on the window:

- Row 1: 5 swatches `44×44` `ultraThin` … `ultraThick`, `10px` radius, with a subtle blue-cyan tint on the first three to match the screenshot's top bar (`44,180,255` blended at 12%). Background is the material `color()` with `border_css()` and a soft `box-shadow: 0 1px 8px rgba(0,0,0,0.12)` to hint blur.
- Row 2: 3 muted swatches `44×28` `thin`/`regular`/`thick` with slight desaturation, mimicking the bottom row where materials are shown over a photographic background — but rendered directly on window (no image, no card) so the window background shows through.

Size is `260×96`. `is_interactive() == false`, `SF Pro` context via surrounding UI.

## Usage / Example

Run the gallery demo (recreates the single-card screenshot):

```bash
cargo run --example materials
```

Minimal usage:

```rust
use tontooui::prelude::*;

// Single material value
let is_dark = true;
let regular = Material::Regular.color(is_dark);
let border = Material::Regular.border_css(is_dark);

// Palette widget directly on window
let palette = Materials::new().to_view();
let root = VStack::new()
    .spacing(8.0)
    .child(palette)
    .child(Text::new(Material::Thin.label()).font_size(10.0));
```

Category folder layout:

```
src/elements/materials/
  mod.rs       // category root
  material.rs  // Material + Materials (type)
```

## Cross References

- [Color.md](Color.md) -- color palettes also render directly on window
- [Text.md](Text.md) -- text formatting initializer
- [GroupBox.md](GroupBox.md) -- inset grouped container with custom background (alternative to material)
