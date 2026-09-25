# Divider

Divider category in `src/elements/dividers/`: `HorizontalDivider` in
`horizontal.rs` is a full-width hairline and `VerticalDivider` in
`vertical.rs` is the full-height counterpart. Both are full bleed:
the intrinsic size on the fill axis is `DIVIDER_FILL`, so stacks hand
them the whole parent width (horizontal) or height (vertical) with no
edge gap. Display-only (no mouse handling). The line follows the
theme divider color unless the dev sets a manual color. Named looks
come from `DividerStyle` in `mod.rs`.

## Geometry

| Token | Value |
|---|---|
| `DIVIDER_THIN` / `DIVIDER_THICK` | 1 px hairline / 5 px heavy line |
| `DIVIDER_FILL` | 32768 px intrinsic fill extent (full bleed, no edge gap) |
| `DIVIDER_DARK` / `DIVIDER_LIGHT` | White 36 alpha / black 31 alpha theme default |
| `DIVIDER_RED` / `DIVIDER_BLUE` | `#FF3B30` / `#007AFF` preset line colors |
| `DIVIDER_PADDED_INSET` | 24 px symmetric inset of the padded style |

## DividerStyle

```rust
pub enum DividerStyle { Default, Red, Thick, BlueThick, Padded }
pub fn thickness(&self) -> f32
pub fn inset(&self) -> f32
pub fn color(&self) -> Option<Color>
```

- `Default` is the thin theme line, full bleed. `Red` is the same
  geometry in `DIVIDER_RED`. `Thick` is the theme color at
  `DIVIDER_THICK`. `BlueThick` is thick in `DIVIDER_BLUE`.
  `Padded` is the thin theme line with `DIVIDER_PADDED_INSET` on
  both ends.
- `color` returns `None` for theme-following presets (the line
  tracks `set_theme`) and `Some` for `Red` and `BlueThick` (manual
  color wins over the theme).

## HorizontalDivider

```rust
pub fn new() -> Self
pub fn styled(style: DividerStyle) -> Self
pub fn apply_style(self, style: DividerStyle) -> Self
pub fn thickness(self, px: f32) -> Self
pub fn color(self, color: Color) -> Self
pub fn inset(self, px: f32) -> Self
pub fn rounded(self, rounded: bool) -> Self
pub fn set_theme(&mut self, divider: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn set_thickness(&mut self, px: f32)
pub fn set_inset(&mut self, px: f32)
pub fn thickness_value(&self) -> f32
pub fn inset_value(&self) -> f32
pub fn line_color(&self) -> Color
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- `measure` returns (`DIVIDER_FILL`, thickness): in a `VStack` the
  line spans the full stack width at exactly its thickness. `inset`
  only shortens the drawn line ends, never the placed rect.
- The line stays vertically centered when the parent hands it extra
  height. `thickness`, `inset` and setters clamp to >= 0; a zero
  thickness or empty rect draws nothing.
- `rounded` draws round ends (visible on thick lines); the default
  is square, like the reference rows.
- `color` wins over `set_theme` until cleared with `apply_style`
  and a theme-following preset. Unfocused windows desaturate the
  line like the rest of the palette.

## VerticalDivider

```rust
pub fn new() -> Self
pub fn styled(style: DividerStyle) -> Self
pub fn apply_style(self, style: DividerStyle) -> Self
pub fn thickness(self, px: f32) -> Self
pub fn color(self, color: Color) -> Self
pub fn inset(self, px: f32) -> Self
pub fn rounded(self, rounded: bool) -> Self
pub fn set_theme(&mut self, divider: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn set_thickness(&mut self, px: f32)
pub fn set_inset(&mut self, px: f32)
pub fn thickness_value(&self) -> f32
pub fn inset_value(&self) -> f32
pub fn line_color(&self) -> Color
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Mirrors `HorizontalDivider` along the y axis: `measure` returns
  (thickness, `DIVIDER_FILL`), so in an `HStack` the line spans the
  full row height. `inset` is the symmetric vertical (top/bottom)
  inset of the padded look.

## Usage / Example

```rust
use tontooui::elements::{DividerStyle, HorizontalDivider, View, VStack};

let stack = VStack::new()
  .spacing(24.0)
  .child(HorizontalDivider::styled(DividerStyle::Default))
  .child(HorizontalDivider::styled(DividerStyle::Red))
  .child(HorizontalDivider::styled(DividerStyle::Thick))
  .child(HorizontalDivider::styled(DividerStyle::BlueThick))
  .child(HorizontalDivider::styled(DividerStyle::Padded));
```

Wire the live theme per frame (see `examples/divider.rs`):

```rust
line.set_theme(palette.divider, dark);
line.set_focused(focused);
```

## Cross References

- [Layout.md](Layout.md) – stacks hand dividers the full parent size
- [Theme.md](Theme.md) – theme divider color and unfocused desaturation
- [Titlebar.md](Titlebar.md) – divider between titlebar and content
