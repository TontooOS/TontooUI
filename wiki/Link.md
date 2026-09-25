# Link

Link category in `src/elements/link/`: `BasicLink` in `basic.rs` is
a blue label with an optional leading SF icon (like the reference
rows: "Visit Apple", code icon plus "Swift.org"). Clicking opens the
URL in the default browser; hover underlines.

## Geometry

| Token | Value |
|---|---|
| `LINK_BLUE` | `#0A84FF` link blue (both modes) |
| `LINK_BLUE_PRESSED` | Link blue at 140 alpha while held |
| `LINK_ICON_GAP` | 8 px icon-label gap |
| `LINK_ICON_SIZE` | 20 px leading icon box |

## open_url

```rust
pub fn open_url(url: &str) -> bool
```

- Opens `url` in the default browser as a detached process (never
  blocking the UI): `start` on Windows, `open` on macOS,
  `xdg-open` elsewhere on Unix. Only `http`/`https` URLs launch;
  anything else reports false without side effects.

## BasicLink

```rust
pub fn new(label: impl Into<String>, url: impl Into<String>) -> Self
pub fn icon(self, name: impl Into<String>) -> Self
pub fn opener(self, opener: impl FnMut(&str) + 'static) -> Self
pub fn set_label(&mut self, label: impl Into<String>)
pub fn set_url(&mut self, url: impl Into<String>)
pub fn label_value(&self) -> &str
pub fn url_value(&self) -> &str
pub fn set_focused(&mut self, focused: bool)
pub fn open(&mut self)
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Click (down plus up inside) opens the URL: through `opener` when
  set (tests, in-app routing), otherwise `open_url`. Outside
  releases open nothing. `open` triggers the same path manually.
- Hover underlines the label; the press dims it to
  `LINK_BLUE_PRESSED`. Unfocused windows desaturate the blue like
  the palette. The icon keeps the manual link tint.

## Usage / Example

```rust
use tontooui::elements::{BasicLink, View, VStack};

let stack = VStack::new()
  .spacing(16.0)
  .child(BasicLink::new("Visit Apple", "https://apple.com"))
  .child(BasicLink::new("Swift.org", "https://swift.org")
    .icon("chevron.left.forwardslash.chevron.right"));
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
