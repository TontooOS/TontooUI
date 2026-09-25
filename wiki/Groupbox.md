# Groupbox

Groupbox category in `src/elements/groupbox/`: `BasicGroupBox` in
`basic.rs` is a rounded fill slightly lighter than the app
background with simple centered text (like the reference rows), and
`StyledGroupBox` in `styled.rs` holds rows with a leading element
and a label in the same box (like the reference settings rows).
Display-only except for the leading elements, which stay
interactive. More variants plug in beside them.

## Geometry

| Token | Value |
|---|---|
| `GROUP_BG_DARK` | `#272D30` fill, a step lighter than the `#1B2022` background |
| `GROUP_BG_LIGHT` | `#F2F2F5` fill: on white a light gray shade reads as raised |
| `GROUP_RADIUS` | 12 px corner radius |
| `GROUP_PAD` | 12 px inner padding |
| `GROUP_ROW_GAP` | 12 px leading-label gap |
| `GROUP_ROW_SPACING` | 14 px row spacing |
| `GROUP_SYMBOL_SIZE` | 24 px leading SF icon box |

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

## StyledGroupBox

```rust
pub fn new() -> Self
pub fn row(self, leading: impl View + 'static, label: impl Into<String>) -> Self
pub fn toggle_row(self, label: impl Into<String>, on: bool) -> Self
pub fn check_row(self, label: impl Into<String>, checked: bool) -> Self
pub fn symbol_row(self, symbol: impl Into<String>, label: impl Into<String>) -> Self
pub fn row_len(&self) -> usize
pub fn row_leading_mut<T: View + 'static>(&mut self, index: usize) -> Option<&mut T>
pub fn row_label(&self, index: usize) -> Option<&str>
pub fn set_row_label(&mut self, index: usize, label: impl Into<String>)
pub fn set_theme(&mut self, mode: ThemeMode)
pub fn set_focused(&mut self, focused: bool)
pub fn set_hover(&mut self, x: f32, y: f32)
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Rows pair any leading view (toggles, SF icons, toolbars) with a
  label; `check_row` matches the reference checkbox rows.
  Presses, releases and hovers forward to the leadings through the
  `View` protocol, so checkboxes flip without extra wiring.
- The box fill and the labels follow `set_theme`; leading elements
  keep their own theme and focus — wire them through
  `row_leading_mut` (see the demo for the toggle pattern).

## View set_hover

```rust
fn set_hover(&mut self, _x: f32, _y: f32) {}
```

- New default on the `View` protocol (mirrors `mouse_down`/`mouse_up`).
  `Button` and `Toggle` override it with their hit tracking; stacks
  (`VStack`, `HStack`, `ZStack`) and wrappers (`Padding`,
  `Background`, `Frame`) forward it to their children.

## Cross References

- [Text.md](Text.md) – centered body text inside the box
- [Theme.md](Theme.md) – background step and unfocused desaturation
- [Layout.md](Layout.md) – stacks size and place boxes via `View`
