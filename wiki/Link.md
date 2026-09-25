# Link

Link category in `src/elements/link/`: `BasicLink` in `basic.rs` is
a blue label with an optional leading SF icon (like the reference
rows: "Visit Apple", code icon plus "Swift.org"), `StyledLink` in
`styled.rs` is a pill button opening a URL (filled dark or
bordered), and `LinkWithImage` in `image.rs` pairs an async URL
image with a link below (like the reference logo plus "Download
App"). Clicking opens the URL in its default handler.

## Geometry

| Token | Value |
|---|---|
| `LINK_BLUE` | `#0A84FF` link blue (both modes) |
| `LINK_BLUE_PRESSED` | Link blue at 140 alpha while held (basic preset) |
| `LINK_ICON_GAP` | 8 px icon-label gap |
| `LINK_ICON_SIZE` | 20 px leading icon box |
| `LINK_PILL_RADIUS` | 16 px styled pill radius |
| `LINK_PILL_PAD_X` / `LINK_PILL_PAD_Y` | 28 px / 14 px styled pill padding |
| `LINK_PILL_BORDER` | 2.5 px bordered pill stroke |
| `LINK_PILL_BG` | `#1E2024` filled pill background |
| `LINK_IMAGE_GAP` | 20 px image-link gap |

## open_url

```rust
pub fn open_url(url: &str) -> bool
pub fn is_openable(url: &str) -> bool
```

- Opens `url` in its default handler as a detached process (never
  blocking the UI): browsers for `http`/`https`, the mail app for
  `mailto` (`start` on Windows, `open` on macOS, `xdg-open`
  elsewhere on Unix). `is_openable` is the pure scheme gate;
  anything else reports false without side effects.

## BasicLink

```rust
pub fn new(label: impl Into<String>, url: impl Into<String>) -> Self
pub fn icon(self, name: impl Into<String>) -> Self
pub fn color(self, color: Color) -> Self
pub fn set_color(&mut self, color: Option<Color>)
pub fn color_value(&self) -> Option<Color>
pub fn opener(self, opener: impl FnMut(&str) + 'static) -> Self
pub fn set_opener(&mut self, opener: Option<Box<dyn FnMut(&str)>>)
pub fn set_label(&mut self, label: impl Into<String>)
pub fn set_url(&mut self, url: impl Into<String>)
pub fn label_value(&self) -> &str
pub fn url_value(&self) -> &str
pub fn set_focused(&mut self, focused: bool)
pub fn open(&mut self)
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Click (down plus up inside) opens the URL: through `opener` when
  set (tests, in-app routing), otherwise the system handler.
  Outside releases open nothing. `open` triggers the same path
  manually.
- `.color()` swaps blue for any color (icon and pressed dimming
  follow); `None` back to blue. Hover underlines the label; the
  press dims it. Unfocused windows desaturate like the palette.

## StyledLink

```rust
pub enum LinkStyle { Filled, Border }
pub fn new(label: impl Into<String>, url: impl Into<String>) -> Self
pub fn style(self, style: LinkStyle) -> Self
pub fn color(self, color: Color) -> Self
pub fn opener(self, opener: impl FnMut(&str) + 'static) -> Self
pub fn set_label(&mut self, label: impl Into<String>)
pub fn set_url(&mut self, url: impl Into<String>)
pub fn set_style(&mut self, style: LinkStyle)
pub fn set_color(&mut self, color: Color)
pub fn label_value(&self) -> &str
pub fn url_value(&self) -> &str
pub fn style_value(&self) -> LinkStyle
pub fn set_focused(&mut self, focused: bool)
pub fn open(&mut self)
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- `Filled` is the dark pill with a colored label, `Border` the
  transparent pill with a colored border and label. Same open path
  as `BasicLink`; hover fills (border) or lightens (filled)
  slightly.

## LinkWithImage

```rust
pub fn new(image_url: impl Into<String>, label: impl Into<String>, link_url: impl Into<String>, width: f32, height: f32) -> Self
pub fn tint(self, color: Color) -> Self
pub fn set_tint(&mut self, tint: Option<Color>)
pub fn tint_value(&self) -> Option<Color>
pub fn spacing(self, px: f32) -> Self
pub fn opener(self, opener: impl FnMut(&str) + 'static) -> Self
pub fn image_mut(&mut self) -> &mut UrlImage
pub fn link_mut(&mut self) -> &mut BasicLink
pub fn set_image_fit(&mut self, fit: ImageFit)
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Async URL image (spinner, `Error {code}`) centered over a
  `BasicLink`, both centered in the card. The image stays
  normal/colorful as downloaded unless `.tint()` pins a fixed
  color (RGB replaced, alpha kept), e.g. a blue glyph. Presses and
  hovers forward to both parts.

## Usage / Example

```rust
use tontooui::elements::{BasicLink, View, VStack};

let stack = VStack::new()
  .spacing(16.0)
  .child(BasicLink::new("Visit Apple", "https://apple.com"))
  .child(BasicLink::new("Swift.org", "https://swift.org")
    .icon("chevron.left.forwardslash.chevron.right"));

let pill = StyledLink::new("Styled Link", "https://example.com");
let card = LinkWithImage::new(
    "https://example.com/logo.png",
    "Download App",
    "https://example.com/download",
    320.0,
    200.0,
)
.tint(Color::from_rgb8(0x0a, 0x84, 0xff));
```

Wire input per frame (see `examples/link.rs`):

```rust
link.mouse_down(x, y);
link.mouse_up(x, y);
link.set_hover(x as f32, y as f32);
link.set_focused(focused);
```

## Cross References

- [Images.md](Images.md) – `SFSymbolImage` used for the icon
- [Text.md](Text.md) – blue label text
- [Button.md](Button.md) – press tracking pattern
