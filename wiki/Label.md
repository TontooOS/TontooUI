# Label

Label category in `src/elements/label/`: `BasicLabel` in `basic.rs`
is an SF icon plus title (like the reference rows), `ImageLabel` in
`image.rs` pairs an async URL image with a title, `StyledLabel` in
`styled.rs` renders title, body and colored status styles, and
`IconLabel` in `icon.rs` is a bare icon with no text. Display-only
throughout (no mouse handling).

## Geometry

| Token | Value |
|---|---|
| `LABEL_ICON_SIZE` | 24 px leading SF icon box |
| `LABEL_GAP` | 12 px icon/image-text gap |
| `LABEL_IMAGE_SIZE` | 44 px async image box |
| `LABEL_DOT` | 16 px status dot diameter |
| `LABEL_ICON_GRAY` | White 220 alpha default icon tint |

## BasicLabel

```rust
pub fn new(icon: impl Into<String>, title: impl Into<String>) -> Self
pub fn icon_color(self, color: Color) -> Self
pub fn title(self, title: impl Into<String>) -> Self
pub fn set_theme(&mut self, mode: ThemeMode)
pub fn set_focused(&mut self, focused: bool)
pub fn set_title(&mut self, title: impl Into<String>)
pub fn set_icon(&mut self, name: impl Into<String>)
pub fn set_icon_color(&mut self, color: Option<Color>)
pub fn icon_value(&self) -> &str
pub fn title_value(&self) -> &str
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Composed from `SFSymbolImage` and `BasicText` (title style) in a
  centered row. Without `.icon_color()` the glyph follows the theme
  text color. All setters rebuild on change only.

## ImageLabel

```rust
pub fn new(url: impl Into<String>, title: impl Into<String>) -> Self
pub fn tint(self, color: Color) -> Self
pub fn set_tint(&mut self, tint: Option<Color>)
pub fn tint_value(&self) -> Option<Color>
pub fn set_image_url(&mut self, url: impl Into<String>)
pub fn retry(&mut self)
pub fn set_theme(&mut self, mode: ThemeMode)
pub fn set_focused(&mut self, focused: bool)
pub fn set_title(&mut self, title: impl Into<String>)
pub fn url_value(&self) -> &str
pub fn title_value(&self) -> &str
pub fn image_mut(&mut self) -> &mut UrlImage
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Async URL image (`UrlImage`: spinner while loading, `Error
  {code}` on failure, `retry`) plus title style text. The image
  stays colorful as downloaded unless `.tint()` pins a fixed color.
  Theme and title updates happen in place, so downloads keep
  running across per-frame calls.

## StyledLabel

```rust
pub enum LabelStyle { Title, Body, Status(Color) }
pub fn new(text: impl Into<String>, style: LabelStyle) -> Self
pub fn title(text: impl Into<String>) -> Self
pub fn body(text: impl Into<String>) -> Self
pub fn status(text: impl Into<String>, color: Color) -> Self
pub fn set_theme(&mut self, mode: ThemeMode)
pub fn set_focused(&mut self, focused: bool)
pub fn set_text(&mut self, text: impl Into<String>)
pub fn set_style(&mut self, style: LabelStyle)
pub fn text_value(&self) -> &str
pub fn style_value(&self) -> LabelStyle
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Title is semibold theme text, body regular dim text, status a
  colored dot plus colored text (like the green "Downloaded" row).
  Layouts cache per scale per the crisp text rules.

## IconLabel

```rust
pub fn new(icon: impl Into<String>) -> Self
pub fn size(self, px: f32) -> Self
pub fn icon_color(self, color: Color) -> Self
pub fn set_theme(&mut self, mode: ThemeMode)
pub fn set_focused(&mut self, focused: bool)
pub fn set_icon(&mut self, name: impl Into<String>)
pub fn icon_value(&self) -> &str
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Thin wrapper over `SFSymbolImage`, so glyphs always match the
  icon rows. Setters update in place (no running state to lose).

## Usage / Example

```rust
use tontooui::elements::{BasicLabel, IconLabel, ImageLabel, StyledLabel, View, VStack};
use vello::peniko::Color;

let green = Color::from_rgb8(0x34, 0xc7, 0x59);
let stack = VStack::new()
  .spacing(14.0)
  .child(BasicLabel::new("star", "Star"))
  .child(ImageLabel::new("https://example.com/a.png", "Custom Title"))
  .child(StyledLabel::status("Downloaded", green))
  .child(IconLabel::new("heart"));
```

See `examples/label.rs` for the full demo (reference rows, tinted
image label, status label, tinted icons).

## Cross References

- [Images.md](Images.md) – `SFSymbolImage` and async `UrlImage`
- [Text.md](Text.md) – title and body text styles
- [Link.md](Link.md) – tappable counterpart with the same icon slot
