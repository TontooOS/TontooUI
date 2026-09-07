# Link

SwiftUI-style Link category for TontooUI, recreating the `Link` family from the macOS 26 interface. The category contains five elements: `HelpLink`, `TextFieldLink`, `CustomPreviewShareLink`, `ShareLink`, and `Link`. All elements are rendered directly on the window background (#1d1d1d dark / #ececec light) with no extra card, using `SF Pro Display`.

## HelpLink

```rust
pub struct HelpLink { /* ... */ }

impl HelpLink {
    pub fn new() -> Self;
    pub fn label(self, label: impl Into<String>) -> Self;
    pub fn on_activate(self, handler: impl Fn() + Send + Sync + 'static) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — a button with a standard appearance that opens app-specific help. Mirrors `SwiftUI.HelpLink`.

- Renders as a circular `?` button (`36×36`, `rgba(255,255,255,0.12)` dark / `rgba(0,0,0,0.06)` light) directly on window.
- `label` defaults to `"Help"`; a non-default label is shown as secondary text beside the button.
- `on_activate` is invoked on click.
- `is_interactive() == true`.

## TextFieldLink

```rust
pub struct TextFieldLink { /* ... */ }

impl TextFieldLink {
    pub fn new(title: impl Into<String>) -> Self;
    pub fn prompt(self, prompt: impl Into<String>) -> Self;
    pub fn on_submit(self, handler: impl Fn(String) + Send + Sync + 'static) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — a control that requests text input from the user when pressed. Mirrors `SwiftUI.TextFieldLink`.

- Renders as a pill button `160×36` (`#3a3a3c`, `18px` radius, white `SF Pro 13px Medium`) directly on window, as in the screenshot "Set Text".
- `prompt` defaults to `"Enter text"`.
- `on_submit` receives the prompt string on click (in a real app would present a `TextField`).

## CustomPreviewShareLink

```rust
pub struct CustomPreviewShareLink { /* ... */ }

impl CustomPreviewShareLink {
    pub fn new(share_label: impl Into<String>, title: impl Into<String>) -> Self;
    pub fn subtitle(self, subtitle: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates an instance, with a custom label, that presents the share interface. Mirrors `ShareLink` with `SharePreview`.

- Preview shows a blue `↗` + `share_label` (e.g. "Share Cats") on top, and a `200×56` frosted card below with a three-cat image placeholder and `title` ("Derpy Cats"), directly on window (no extra card beyond the frosted preview `rgba(44,44,46,0.55)` + hairline border).
- Size `220×120`.

## ShareLink

```rust
pub struct ShareLink { /* ... */ }

impl ShareLink {
    pub fn new(items: Vec<String>) -> Self;
    pub fn item(item: impl Into<String>) -> Self;
    pub fn items(self, items: Vec<String>) -> Self;
    pub fn preview(self, title: impl Into<String>, subtitle: impl Into<String>) -> Self;
    pub fn on_share(self, handler: impl Fn(String) + Send + Sync + 'static) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — a view that controls a sharing presentation. Mirrors `SwiftUI.ShareLink`.

- Default items `["Share ...", "Explore SwiftUI", "Foo"]` rendered as a row of blue `↗` + label `10px` directly on window.
- `preview` defaults to `"Visual Library for SwiftUI Compon..."` / `"explore.swiftui.com"`; rendered as a `240×36` frosted row with a `28×28` blue `◈` icon, `10px Semibold` title and `8px` subtitle.
- `on_share` attaches a `GestureClick` per item.

## Link

```rust
pub struct Link { /* ... */ }

impl Link {
    pub fn new(title: impl Into<String>, destination: impl Into<String>) -> Self;
    pub fn destination(self, url: impl Into<String>) -> Self;
    pub fn on_activate(self, handler: impl Fn(String) + Send + Sync + 'static) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — a control for navigating to a URL. Mirrors `SwiftUI.Link`.

- Renders as blue `#0A84FF` `SF Pro 11px` text (`"Explore SwiftUI"` by default) with transparent `GtkButton` (`has_frame=false`), directly on window. Hover underlines via CSS.
- `on_activate` receives the destination; if absent, falls back to `xdg-open` on Linux and `println!`.
- Size derived from title length (`len×7+20 × 24`).

## Usage / Example

Run the gallery demo (recreates the 5-card screenshot):

```bash
cargo run --example links
```

Minimal usage:

```rust
use tontooui::prelude::*;

let help = HelpLink::new().on_activate(|| println!("help"));
let field = TextFieldLink::new("Set Text").on_submit(|text| println!("{}", text));
let custom = CustomPreviewShareLink::new("Share Cats", "Derpy Cats");
let share = ShareLink::new(vec!["Share ...".into(), "Explore SwiftUI".into()]);
let link = Link::new("Explore SwiftUI", "https://explore.swiftui.com")
    .on_activate(|url| println!("open {}", url));

let root = VStack::new()
    .spacing(8.0)
    .child(help.to_view())
    .child(field.to_view())
    .child(link.to_view());
```

Category folder layout:

```
src/elements/links/
  mod.rs                         // category root
  help_link.rs                   // HelpLink
  text_field_link.rs             // TextFieldLink
  custom_preview_share_link.rs   // CustomPreviewShareLink
  share_link.rs                  // ShareLink
  link.rs                        // Link
```

## Cross References

- [Button.md](Button.md) -- buttons and system roles
- [Text.md](Text.md) -- text rendering (Link uses blue text)
- [Material.md](Material.md) -- frosted ShareLink preview
