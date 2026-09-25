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

See `examples/colors.rs` for the full demo (all thirteen swatches,
the four labeled reference bars, plus a pick button with a swatch
and the frosted picker popup).

## ColorPicker

```rust
pub struct Hsv { pub h: f32, pub s: f32, pub v: f32 }
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> Hsv
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32)
pub fn color_to_hsva(color: Color) -> (Hsv, f32)
pub fn hsva_to_color(hsv: Hsv, alpha: f32) -> Color
pub fn new() -> Self
pub fn on_change(self, callback: impl FnMut(Color) + 'static) -> Self
pub fn color(self, color: Color) -> Self
pub fn set_color(&mut self, color: Color)
pub fn selected(&self) -> Color
pub fn hsv_value(&self) -> Hsv
pub fn set_theme(&mut self, mode: ThemeMode, glass: GlassAmount)
pub fn set_focused(&mut self, focused: bool)
pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32)
pub fn show(&mut self)
pub fn dismiss(&mut self)
pub fn is_visible(&self) -> bool
pub fn panel_width() -> f32
pub fn panel_height() -> f32
pub fn card_rect(&self) -> (f32, f32, f32, f32)
pub fn wheel_rect(&self) -> (f32, f32, f32)
pub fn brightness_rect(&self) -> (f32, f32, f32, f32)
pub fn opacity_rect(&self) -> (f32, f32, f32, f32)
pub fn point_to_hs(&self, x: f32, y: f32) -> (f32, f32)
pub fn hs_point(&self) -> (f32, f32)
pub fn rect(&self) -> (f32, f32, f32, f32)
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_move(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64)
```

| Token | Value |
|---|---|
| `PICKER_PAD` | 16 px panel padding |
| `PICKER_WHEEL` | 232 px wheel diameter |
| `PICKER_GAP` | 16 px wheel-to-brightness gap |
| `PICKER_BAR_H` | 28 px slider height |
| `PICKER_LABEL_GAP` / `PICKER_ROW_GAP` | 8 px / 6 px label gaps |
| `PICKER_RADIUS` | 20 px panel corner radius |
| `PICKER_LABEL_SIZE` | 13 px label size |
| `PICKER_PILL_W` | 64 px percent pill width |
| `PICKER_LABEL_GRAY` | White 160 alpha label on frost |
| `PICKER_PILL` | `#1E1E20` percent pill fill |
| `PICKER_CHECK_A` / `PICKER_CHECK_B` | `#C0C0C0` / `#808080` checker squares |
| `PICKER_CHECK` | 10 px checker size |

- Menu-like frosted popup (`GlassType::Frosted`): hue/saturation
  wheel with a crosshair (baked 256 px texture, white center glow),
  brightness slider (full color into black, ring knob), opacity
  label plus checker transparency slider (transparent left, opaque
  right) with a percent pill. Triggered with `show` by a button or
  the app; reports every change through `on_change`, reads back via
  `selected`; outside clicks dismiss keeping the selection.
- Drags track across moves with clamping; the brightness bar runs
  bright left into black right, the opacity bar transparent left
  into opaque right. Knob centers travel inset by the knob radius
  and the crosshair clamps inside the disc, so rings never leave
  their bars.
  Skips itself in the backdrop capture pass, so the frost samples
  only what sits behind it — the app opts in with
  `wants_backdrop` while visible (see the demo, like the date
  picker).

## Cross References

- [Shapes.md](Shapes.md) – `ShapeFill` paints solids and gradients into shapes
- [Theme.md](Theme.md) – theme accents overlap the system palette
- [Renderer.md](Renderer.md) – display scale for spanning brushes
