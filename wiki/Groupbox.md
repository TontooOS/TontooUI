# Groupbox

Groupbox category in `src/elements/groupbox/`: `BasicGroupBox` in
`basic.rs` is a rounded fill slightly lighter than the app
background with simple centered text (like the reference rows).
Display-only (no mouse handling). More variants plug in beside it.

## Geometry

| Token | Value |
|---|---|
| `GROUP_BG_DARK` | `#272D30` fill, a step lighter than the `#1B2022` background |
| `GROUP_BG_LIGHT` | `#F2F2F5` fill: on white a light gray shade reads as raised |
| `GROUP_RADIUS` | 12 px corner radius |
| `GROUP_PAD` | 12 px inner padding |

## BasicGroupBox

```rust
pub fn new(content: impl Into<String>) -> Self
pub fn set_theme(&mut self, mode: ThemeMode)
pub fn set_focused(&mut self, focused: bool)
pub fn set_text(&mut self, content: impl Into<String>)
pub fn text_value(&self) -> &str
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- `measure` is the centered body text plus padding on every side.
  The fill paints the full placed rect, so directly placed boxes
  span their width (stacks cap children at intrinsic size and
  position them per alignment).
- Unfocused windows desaturate the fill like the palette.

## Usage / Example

```rust
use tontooui::elements::{BasicGroupBox, View, VStack};
use tontooui::theme::ThemeMode;

let stack = VStack::new()
  .spacing(20.0)
  .child(BasicGroupBox::new("This is content inside a GroupBox."));
```

Wire the live theme per frame (see `examples/groupbox.rs`):

```rust
group.set_theme(mode);
group.set_focused(focused);
```

## Cross References

- [Text.md](Text.md) – centered body text inside the box
- [Theme.md](Theme.md) – background step and unfocused desaturation
- [Layout.md](Layout.md) – stacks size and place boxes via `View`
