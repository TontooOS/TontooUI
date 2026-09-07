# Color

SwiftUI-style Color category for TontooUI, recreating the Color system from the macOS 26 interface. The category contains 10 elements that mirror the SwiftUI `Color` modifiers and UIKit system palettes: opacity/style, gradient/modifier, hierarchical variants/modifier, and seven type palettes (separator, content background, text, fill, label, semantic, standard). All elements are theme-aware where applicable (light `#ececec` / dark `#1d1d1d`) and use `SF Pro Display` for labels.

Category badge mapping follows the reference screenshots: `style` for opacity, `modifier` for gradient/variants, `type` for the remaining seven.

## ColorOpacity

```rust
pub struct ColorOpacity { /* ... */ }

impl ColorOpacity {
    pub fn new(base: Color) -> Self;
    pub fn base(self, color: Color) -> Self;
    pub fn opacities(self, values: Vec<f32>) -> Self;
    pub fn swatch_size(self, size: f32) -> Self;
    pub fn color_with_opacity(&self, opacity: f32) -> Color;
    pub fn to_view(self) -> View;
}
```

Style — the SwiftUI `Color.opacity(_:)` modifier. The preview renders five swatches (opacities `1.0, 0.75, 0.50, 0.25, 0.12`) directly on the window background (#1d1d1d dark / #ececec light), no extra card — matching the "Color Opacity — The Color opacity modifier" card. `color_with_opacity` multiplies `base.a` by the requested opacity and clamps to `0.0..=1.0`. Default base is `Color(0,122,255)` (system blue).

Behavior notes:

- Values outside `0.0..=1.0` are clamped.
- The widget itself is non-interactive (`is_interactive() == false`).

## ColorGradient

```rust
pub struct ColorGradient { /* ... */ }

impl ColorGradient {
    pub fn new(colors: Vec<Color>) -> Self;
    pub fn linear(colors: Vec<Color>) -> Self;
    pub fn colors(self, colors: Vec<Color>) -> Self;
    pub fn swatch_size(self, s: f32) -> Self;
    pub fn css_gradient(&self) -> String;
    pub fn to_view(self) -> View;
}
```

Modifier — the SwiftUI `Color.gradient` modifier. The preview interpolates the supplied `colors` into eight discrete swatches (4×2 grid) plus a horizontal gradient bar, directly on the window background — no extra card. `css_gradient()` returns `linear-gradient(to right, …)` or `transparent` for an empty list. `linear` is an alias for `new`.

Behavior notes:

- Empty `colors` yields a transparent placeholder.
- Single-color input returns the color's CSS directly.
- Interpolation is linear in sRGB.

## HierarchicalVariant / ColorVariants

```rust
pub enum HierarchicalVariant {
    Primary,
    Secondary,
    Tertiary,
    Quaternary,
}

impl HierarchicalVariant {
    pub fn opacity(self) -> f32;
    pub fn label(self) -> &'static str;
}

pub struct ColorVariants { /* ... */ }

impl ColorVariants {
    pub fn new() -> Self;
    pub fn bases(self, colors: Vec<Color>) -> Self;
    pub fn color_for(&self, base: Color, level: HierarchicalVariant) -> Color;
    pub fn to_view(self) -> View;
}
```

Modifier — `HierarchicalShapeStyle`, a shape style that maps to one of the numbered content styles. `HierarchicalVariant` maps to opacities `1.0, 0.6, 0.30, 0.15`. `ColorVariants` renders three base colors (default blue, red, yellow) each at four levels (12 swatches, 3 rows × 4 columns). `color_for` multiplies `base.a` by the level opacity.

| Variant | Opacity | Label |
|---|---|---|
| `Primary` | `1.0` | `Primary` |
| `Secondary` | `0.6` | `Secondary` |
| `Tertiary` | `0.30` | `Tertiary` |
| `Quaternary` | `0.15` | `Quaternary` |

## UIKitSeparatorColor / UIKitSeparatorColors

```rust
pub enum UIKitSeparatorColor {
    Separator,
    OpaqueSeparator,
}

impl UIKitSeparatorColor {
    pub fn color(self, is_dark: bool) -> Color;
    pub fn label(self) -> &'static str;
}

pub struct UIKitSeparatorColors { /* ... */ }

impl UIKitSeparatorColors {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Type — the UIKit separator colors also used in other components such as `List` separators.

| Variant | Light | Dark |
|---|---|---|
| `Separator` | `rgba(60,60,67,0.29)` | `rgba(84,84,88,0.65)` |
| `OpaqueSeparator` | `#C6C6C8` | `#38383A` |

`UIKitSeparatorColors` palette shows four swatches (separator/opaque × light/dark hint) plus a demo `gtk::Separator` line, directly on the window background — no extra card.

## UIKitBackgroundColor / UIKitContentBackgroundColors

```rust
pub enum UIKitBackgroundColor {
    SystemBackground,
    SecondarySystemBackground,
    TertiarySystemBackground,
    SystemGroupedBackground,
    SecondarySystemGroupedBackground,
    TertiarySystemGroupedBackground,
}

impl UIKitBackgroundColor {
    pub fn color(self, is_dark: bool) -> Color;
    pub fn label(self) -> &'static str;
}

pub struct UIKitContentBackgroundColors { /* ... */ }

impl UIKitContentBackgroundColors {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Type — the UIKit content background colors also used in `List` and `Form`.

| Variant | Light | Dark |
|---|---|---|
| `SystemBackground` | `#FFFFFF` | `#1d1d1d` |
| `SecondarySystemBackground` | `#F2F2F7` | `#2C2C2E` |
| `TertiarySystemBackground` | `#FFFFFF` | `#3A3A3C` |
| `SystemGroupedBackground` | `#F2F2F7` | `#000000` |
| `SecondarySystemGroupedBackground` | `#FFFFFF` | `#1d1d1d` |
| `TertiarySystemGroupedBackground` | `#F2F2F7` | `#2C2C2E` |

Palette shows four representative swatches plus a `content` chip, directly on the window background — no extra card.

## UIKitTextColor / UIKitTextColors

```rust
pub enum UIKitTextColor {
    PlaceholderText,
    Link,
}

impl UIKitTextColor {
    pub fn color(self, is_dark: bool) -> Color;
    pub fn label(self) -> &'static str;
}

pub struct UIKitTextColors { /* ... */ }

impl UIKitTextColors {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Type — the UIKit text colors also used in `TextField` placeholders.

| Variant | Light | Dark |
|---|---|---|
| `PlaceholderText` | `rgba(60,60,67,0.30)` | `rgba(235,235,245,0.30)` |
| `Link` | `#0A84FF` | `#0A84FF` |

Palette renders two lines of placeholder text.

## UIKitFillColor / UIKitFillColors

```rust
pub enum UIKitFillColor {
    SystemFill,
    SecondarySystemFill,
    TertiarySystemFill,
    QuaternarySystemFill,
}

impl UIKitFillColor {
    pub fn color(self, is_dark: bool) -> Color;
    pub fn label(self) -> &'static str;
}

pub struct UIKitFillColors { /* ... */ }

impl UIKitFillColors {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Type — UIKit fill colors.

| Variant | Light | Dark |
|---|---|---|
| `SystemFill` | `rgba(120,120,128,0.20)` | `rgba(120,120,128,0.36)` |
| `SecondarySystemFill` | `rgba(120,120,128,0.16)` | `rgba(120,120,128,0.32)` |
| `TertiarySystemFill` | `rgba(118,118,128,0.12)` | `rgba(118,118,128,0.24)` |
| `QuaternarySystemFill` | `rgba(116,116,128,0.08)` | `rgba(116,116,128,0.18)` |

Palette shows four swatches with decreasing alpha, directly on the window background — no extra card.

## UIKitLabelColor / UIKitLabelColors

```rust
pub enum UIKitLabelColor {
    Label,
    SecondaryLabel,
    TertiaryLabel,
    QuaternaryLabel,
}

impl UIKitLabelColor {
    pub fn color(self, is_dark: bool) -> Color;
    pub fn label(self) -> &'static str;
}

pub struct UIKitLabelColors { /* ... */ }

impl UIKitLabelColors {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Type — UIKit label colors.

| Variant | Light | Dark |
|---|---|---|
| `Label` | `#000000` | `#FFFFFF` |
| `SecondaryLabel` | `rgba(60,60,67,0.60)` | `rgba(235,235,245,0.60)` |
| `TertiaryLabel` | `rgba(60,60,67,0.30)` | `rgba(235,235,245,0.30)` |
| `QuaternaryLabel` | `rgba(60,60,67,0.18)` | `rgba(235,235,245,0.18)` |

Palette renders two rows: "Label · Secondary Label" and "Tertiary Label · Quaternary Label" with adaptive alpha.

## SemanticColor / SemanticColors

```rust
pub enum SemanticColor {
    Primary,
    Secondary,
}

impl SemanticColor {
    pub fn color(self, is_dark: bool) -> Color;
    pub fn label(self) -> &'static str;
}

pub struct SemanticColors { /* ... */ }

impl SemanticColors {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Type — semantic colors that adapt to the current scheme.

| Variant | Light | Dark |
|---|---|---|
| `Primary` | `#000000` | `#FFFFFF` |
| `Secondary` | `rgba(60,60,67,0.60)` | `rgba(235,235,245,0.60)` |

Palette shows two lines: "Primary Text Color" / "Secondary Text Color".

## StandardColor / StandardColors

```rust
pub enum StandardColor {
    Red,
    Orange,
    Yellow,
    Green,
    Mint,
    Teal,
    Cyan,
    Blue,
    Indigo,
    Purple,
    Pink,
    Brown,
    Gray,
    White,
    Black,
    Clear,
}

impl StandardColor {
    pub fn color(self) -> Color;
    pub fn label(self) -> &'static str;
}

pub struct StandardColors { /* ... */ }

impl StandardColors {
    pub fn new() -> Self;
    pub fn columns(self, n: usize) -> Self;
    pub fn swatch_size(self, s: f32) -> Self;
    pub fn to_view(self) -> View;
}
```

Type — the SwiftUI standard colors. `grid12()` is the 12-color grid shown in the screenshot. Color values:

| Variant | Hex |
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
| `White` | `#FFFFFF` |
| `Black` | `#000000` |
| `Clear` | `transparent` |

`StandardColors` renders a 4×3 grid (12 swatches) directly on the window background — no extra card. `columns` and `swatch_size` customize the grid.

## Usage / Example

Run the full gallery demo (recreates the 10-card screenshot grid):

```bash
cargo run --example colors
```

Minimal usage:

```rust
use tontooui::prelude::*;

// Modifiers
let faded = ColorOpacity::new(Color::from_rgb(10, 132, 255)).opacities(vec![1.0, 0.5, 0.2]);
let grad = ColorGradient::linear(vec![Color::from_rgb(10,132,255), Color::from_rgb(0,40,100)]);
let level = HierarchicalVariant::Secondary;
let variant_color = ColorVariants::new().color_for(Color::from_rgb(255,59,48), level);

// UIKit system colors
let label = UIKitLabelColor::SecondaryLabel.color(true); // dark
let fill = UIKitFillColor::SystemFill.color(false);       // light
let sep = UIKitSeparatorColor::Separator.color(true);
let bg = UIKitBackgroundColor::SystemBackground.color(false);
let ph = UIKitTextColor::PlaceholderText.color(true);

// Semantic & standard
let primary = SemanticColor::Primary.color(false);
let red = StandardColor::Red.color();

// Widgets (paste into a VStack/HStack)
let opacity_widget = ColorOpacity::new(Color::from_rgb(0,122,255)).to_view();
let standard_palette = StandardColors::new().to_view();
```

## Cross References

- [Button.md](Button.md) -- button tint uses `StandardColor` / `Color` values
- [Divider.md](Divider.md) -- divider color aligns with `UIKitSeparatorColor`
- [List.md](List.md) -- list backgrounds map to `UIKitBackgroundColor`
- [Sidebar.md](Sidebar.md) -- sidebar uses system background colors
