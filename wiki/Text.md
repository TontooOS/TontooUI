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

## Formatting

`FormattedText` in `formatted.rs` renders inline spans (bold, italic,
code, underline, strikethrough, colors, links) from markdown
(`FormattedText::markdown`) or explicit spans
(`FormattedText::spans`). `line_limit` truncates with an ellipsis,
links fire `on_link` on click.

```rust
pub fn markdown(source: impl Into<String>) -> Self
pub fn spans(spans: Vec<Span>) -> Self
pub fn line_limit(self, lines: usize) -> Self
pub fn accent(self, accent: Color) -> Self
pub fn on_link(self, callback: impl FnMut(&str) + 'static) -> Self
pub fn link_at(&self, x: f32, y: f32) -> Option<String>
```

Markdown subset, single paragraph: `**bold**`, `*italic*`,
`_italic_`, `***bold italic***`, `~~strikethrough~~`, `` `code` ``
(monospace), `[label](url)`, `\` escapes the next character.
Unmatched markers stay literal; `_` needs word flanking so
`foo_bar` keeps its underscores.

Explicit spans cover what markdown cannot (colored decorations):

```rust
pub struct Span {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
    pub color: Option<Color>,
    pub underline: bool,
    pub underline_color: Option<Color>,
    pub strikethrough: bool,
    pub strikethrough_color: Option<Color>,
    pub link: Option<String>,
}
```

```rust
pub fn new(text: impl Into<String>) -> Self
pub fn bold(self) -> Self
pub fn italic(self) -> Self
pub fn code(self) -> Self
pub fn color(self, color: Color) -> Self
pub fn underline(self) -> Self
pub fn underline_color(self, color: Color) -> Self
pub fn strikethrough(self) -> Self
pub fn strikethrough_color(self, color: Color) -> Self
pub fn link(self, url: impl Into<String>) -> Self
pub fn parse_markdown(source: &str) -> Vec<Span>
```

- Links render in the accent color (default theme blue, `accent()`
  overrides), underlined, and hit-test exactly on their glyphs
  (`Cluster::from_point_exact`): padding never counts as a link.
- `line_limit(n)` keeps the longest char-prefix plus "…" fitting `n`
  lines (binary search over one re-layout per probe, on dirty only).
- Decorations paint from Parley run metrics
  (`underline_offset`/`underline_size`, `strikethrough_offset`/
  `strikethrough_size`); decoration colors fall back to the span
  color, then the base foreground.
- Base `foreground`/`style`/`alignment`/`width` builders mirror
  `BasicText`. Gradient foregrounds paint default runs; explicit
  colors keep theirs.

```rust
use tontooui::elements::{FormattedText, Span, View};

let md = FormattedText::markdown("**Bold** and *italic* together");
let colored = FormattedText::spans(vec![
    Span::new("Underlined Red").underline_color(Color::from_rgb8(0xff, 0x3b, 0x30)),
    Span::new(" and "),
    Span::new("Green Strike").strikethrough_color(Color::from_rgb8(0x34, 0xc7, 0x59)),
]);
let link = FormattedText::markdown("[Link](https://example.com)")
    .on_link(|url| println!("open {url}"));
let short = FormattedText::markdown("Long text…").width(300.0).line_limit(1);
```

## Labeled Text

`LabeledText` in `labeled.rs` is text with an SF Symbol (SwiftUI
`Label`): an icon from CoreIcon beside a real `BasicText`, so style,
foreground (including gradients), alignment and wrapping behave
identically. A missing icon draws the text alone.

```rust
pub fn new(text: impl Into<String>, icon: impl Into<String>) -> Self
pub fn style(self, style: TextStyle) -> Self
pub fn foreground(self, foreground: TextForeground) -> Self
pub fn foreground_color(self, color: Color) -> Self
pub fn alignment(self, alignment: TextAlignment) -> Self
pub fn width(self, px: f32) -> Self
pub fn icon_size(self, px: f32) -> Self
pub fn gap(self, px: f32) -> Self
pub fn icon_color(self, color: Color) -> Self
```

| Token | Value |
|---|---|
| `LABELED_GAP` | `6.0` icon-to-text gap |

- The icon box defaults to the text size (`icon_size()` overrides,
  aspect kept) and centers on the first line.
- The icon inherits the resolved text color unless `icon_color`
  overrides it: leave it off for tinted text (blue handset row), set
  it for a contrasting icon (red heart row).

```rust
use tontooui::elements::{LabeledText, TextStyle, View};

let starred = LabeledText::new("Starred", "star").style(TextStyle::Title2);
let custom = LabeledText::new("Custom Label", "heart")
    .icon_color(Color::from_rgb8(0xff, 0x3b, 0x30));
let handset = LabeledText::new("Handset", "phone")
    .foreground_color(Color::from_rgb8(0x00, 0x7a, 0xff));
```

## Cross References

- [Renderer.md](Renderer.md) – `FontSystem` layout and backdrop
  pipeline (unchanged base API)
- [Layout.md](Layout.md) – `View` protocol (`measure`, `place`, `draw`)
- [Theme.md](Theme.md) – palette behind `Primary`/`Secondary`
