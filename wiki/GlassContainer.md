# GlassContainer

A container that keeps its content but replaces the background with
live-composited liquid glass. The content stays fully functional (typing,
clicks) while the glass is rendered behind it from a backdrop image.

## GlassMaterial

```rust
pub struct GlassMaterial {
    pub tint_r: f32,      // 0..=255
    pub tint_g: f32,      // 0..=255
    pub tint_b: f32,      // 0..=255
    pub tint_a: f32,      // 0..=100
    pub sigma: f32,       // 0..=25, backdrop gaussian blur
    pub refraction: f32,  // 0..=100, lens magnification
    pub depth: f32,       // 0..=100, thickness (bend + edge darkening)
    pub dispersion: f32,  // 0..=100, chromatic aberration
    pub saturation: f32,  // 0..=200, 100 is neutral
    pub brightness: f32,  // 20..=180, 100 is neutral
    pub contrast: f32,    // 0..=200, 100 is neutral
    pub specular: f32,    // 0..=100, sheen intensity
    pub spec_angle: f32,  // 0..=360, sheen light angle
    pub rim: f32,         // 0..=100, rim light intensity
    pub grain: f32,       // 0..=100, frost grain amount
    pub dim: f32,         // 0..=80, backdrop dim
    pub blur_on: bool,
    pub refraction_on: bool,
    pub dispersion_on: bool,
    pub grain_on: bool,
    pub specular_on: bool,
    pub rim_on: bool,
    pub tint_on: bool,
    pub dim_on: bool,
}
```

`GlassMaterial::default()` is the frosted menu-bar look. Each `*_on`
flag gates a whole stage; a slider at its neutral value is equivalent.

## render_glass

```rust
pub fn render_glass(mat: &GlassMaterial, backdrop: &RgbImage, w: u32, h: u32, radius: f32) -> (RgbImage, f32);
```

Composites `w`x`h` liquid glass over `backdrop` behind a rounded rect
with corner `radius` (capsule when `radius >= h / 2`). The backdrop is
center-cropped when larger and cover-scaled when smaller. Returns the
image plus the blur stage time in milliseconds for perf HUDs.

- Returns an empty image when `w` or `h` is `0`; never panics.
- Pixels outside the mask are the pure backdrop, bit-identical.
- `radius` above `h / 2` is clamped.

## GlassContainer

```rust
pub struct GlassContainer { /* ... */ }
impl GlassContainer {
    pub fn new(content: impl Widget + 'static) -> Self;
    pub fn material(self, m: GlassMaterial) -> Self;
    pub fn tint(self, c: Color, alpha: f32) -> Self;
    pub fn sigma(self, s: f32) -> Self;
    pub fn refraction(self, r: f32) -> Self;
    pub fn behind(self, b: GlassBehind) -> Self;
    pub fn behind_widget(self, w: impl Widget + 'static) -> Self;
    pub fn behind_image(self, img: RgbImage) -> Self;
    pub fn behind_file(self, path: impl Into<String>) -> Self;
    pub fn behind_color(self, c: Color) -> Self;
    pub fn size(self, w: f32, h: f32) -> Self;
    pub fn radius(self, r: f32) -> Self;
    pub fn padding(self, p: f32) -> Self;
    pub fn to_view(self) -> View;
}
```

Defaults: `320x64`, capsule radius, `8px` padding, lying directly on the
app background (`#1d1d1d` dark / `#ececec` light). Content floats above
the glass and must paint transparently where the glass shows through
(see `TextInput::transparent()`).

### GlassBehind

The element behind is always shown live under the glass. The frost
composites from its pixels wherever they are known.

| Variant | Live display | Frost pixels |
|---|---|---|
| `Widget(Box<dyn Widget>)` | Yes | Dark fill fallback (GTK4 cannot rasterize arbitrary widgets) |
| `Image(RgbImage)` | Yes | The image |
| `File(String)` | Yes | Loaded file, dark fill when missing |
| `Color(Color)` | Yes | The color |

> **Note:** true backdrop blur over arbitrary live UI belongs in the
> compositor (TontooCompositor), not in the toolkit — GTK4 offers no API
> to snapshot a widget into pixels.

## TextInput::transparent

```rust
pub fn transparent(self) -> Self;
```

Paints no background or border (hover stays borderless, focus keeps the
accent ring), so the field can sit on glass. Text, placeholder and caret
colors are unchanged.

## Usage / Example

Apply glass to a text input — the input stays live, its background
becomes glass:

```rust
use tontooui::prelude::*;

let field = GlassContainer::new(
    TextInput::new("Search...").transparent().width(320.0),
)
.behind_file("examples/assets/glass_bg.jpg")
.size(380.0, 64.0)
.radius(20.0);
```

Run the demo:

```bash
cargo run --example glass_textfield
```

Tune every material value live (16 sliders + 8 toggles + frame graph):

```bash
cargo run --example glass_playground
```

## Cross References

- [View.md](View.md) -- `GlassEffect` palette preview and core `glassEffect` modifier
- [Material.md](Material.md) -- frosted material presets used below glass
- [TextInput.md](TextInput.md) -- text input element
