# Text

Text category in `src/elements/text/`: `BasicText` in `text.rs` is a
label with a `TextStyle` size (`style.rs`) and a `TextForeground`
color or gradient (`foreground.rs`). The underlying `FontSystem` API
in `src/renderer/text.rs` (`layout_text`, `layout_text_weighted`,
`draw_layout`) is unchanged: solid colors reuse `draw_layout`, only
gradients take a custom draw loop painting the same glyph runs with
a horizontal gradient brush.

## BasicText

```rust
pub fn new(content: impl Into<String>) -> Self
pub fn style(self, style: TextStyle) -> Self
pub fn foreground(self, foreground: TextForeground) -> Self
pub fn foreground_color(self, color: Color) -> Self
pub fn foreground_gradient(self, colors: Vec<Color>) -> Self
pub fn alignment(self, alignment: TextAlignment) -> Self
pub fn width(self, px: f32) -> Self
```

- Single line by default; `width` fixes the box so longer content
  wraps.
- `set_text`, `set_style`, `set_foreground`, `set_alignment`,
  `set_width(Option<f32>)`, `set_theme(ThemeMode)`,
  `set_focused(bool)` update live and mark the layout dirty.
- No mouse handling: display-only.

```rust
pub enum TextAlignment {
    Leading,
    Center,
    Trailing,
}
```

`Center`/`Trailing` center each wrapped line via Parley (`Layout::align`)
and shift unbounded text inside the placed rect.

## Style

```rust
pub enum TextStyle {
    LargeTitle,
    Title,
    Title2,
    Title3,
    Headline,
    Subheadline,
    Body,
    Callout,
    Footnote,
    Caption,
    Caption2,
}
```

| Style | Size | Weight |
|---|---|---|
| `LargeTitle` | `34.0` | `400.0` |
| `Title` | `28.0` | `400.0` |
| `Title2` | `22.0` | `400.0` |
| `Title3` | `20.0` | `400.0` |
| `Headline` | `17.0` | `600.0` |
| `Subheadline` | `15.0` | `400.0` |
| `Body` | `17.0` | `400.0` |
| `Callout` | `16.0` | `400.0` |
| `Footnote` | `13.0` | `400.0` |
| `Caption` | `12.0` | `400.0` |
| `Caption2` | `11.0` | `400.0` |

Sizes follow the SwiftUI type scale; only `Headline` is semibold.

```rust
pub fn size(self) -> f32
pub fn weight(self) -> f32
```

## Foreground

```rust
pub enum TextForeground {
    Primary,
    Secondary,
    Tertiary,
    Color(Color),
    Gradient(Vec<Color>),
}
```

- `Primary`/`Secondary` resolve to the theme palette (`text`,
  `text_dim`); `Tertiary` is white at 40% alpha in dark mode and
  black at 30% in light mode (`TEXT_TERTIARY_DARK`,
  `TEXT_TERTIARY_LIGHT`).
- `Color` is fixed in both modes.
- `Gradient` spreads its colors evenly as a horizontal gradient
  across the text bounds. One color behaves like `Color`, an empty
  list falls back to `Primary`.
- Unfocused windows desaturate every variant like the rest of the
  palette.

```rust
pub fn resolve(&self, mode: ThemeMode, focused: bool) -> ResolvedForeground
```

## Usage / Example

```rust
use tontooui::elements::{BasicText, TextAlignment, TextForeground, TextStyle, View};
use vello::peniko::Color;

let title = BasicText::new("Large Title").style(TextStyle::LargeTitle);
let wrapped = BasicText::new("Long paragraph...")
    .width(200.0)
    .alignment(TextAlignment::Center);
let rainbow = BasicText::new("Gradient Text").foreground_gradient(vec![
    Color::from_rgb8(0xff, 0x3b, 0x30),
    Color::from_rgb8(0xff, 0xcc, 0x00),
    Color::from_rgb8(0x34, 0xc7, 0x59),
    Color::from_rgb8(0x00, 0x7a, 0xff),
    Color::from_rgb8(0xaf, 0x52, 0xde),
]);
```

See `examples/text.rs` for the full style and foreground catalog.

## Cross References

- [Renderer.md](Renderer.md) – `FontSystem` layout and backdrop
  pipeline (unchanged base API)
- [Layout.md](Layout.md) – `View` protocol (`measure`, `place`, `draw`)
- [Theme.md](Theme.md) – palette behind `Primary`/`Secondary`
