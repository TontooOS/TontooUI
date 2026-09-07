# Label

SwiftUI-style Label category for TontooUI, recreating label initializers and styles. The category contains three `initializer` elements and one `style` element. Rendering is directly on the window background (#1d1d1d dark / #ececec light) with no extra card, using `SF Pro Display`.

## CustomLabel

```rust
pub struct CustomLabel { /* ... */ }
impl CustomLabel {
    pub fn new() -> Self;
    pub fn title(self, v: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a label with a custom title and icon.

- Preview: custom ring icon plus `Foo Bar` centered on a `180x110` phone card.
- Size `180x110`.

## ImageLabel

```rust
pub struct ImageLabel { /* ... */ }
impl ImageLabel {
    pub fn new() -> Self;
    pub fn title_key(self, v: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a label with an icon image and a title generated from a localized string.

- Preview: two image boxes with hats plus `Foo` on phone.
- Size `180x110`.

## SystemImageLabel

```rust
pub struct SystemImageLabel { /* ... */ }
impl SystemImageLabel {
    pub fn new() -> Self;
    pub fn system_name(self, v: impl Into<String>) -> Self;
    pub fn title_key(self, v: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a label with a system icon image and a title generated from a localized string.

- Preview: system glyph `square.grid.2x2` plus `Foo`, dimmed on phone.
- Size `180x110`.

## LabelStyles

```rust
pub enum LabelStyleKind { TitleAndIcon, TitleOnly, IconOnly }
pub struct LabelStyles { /* ... */ }
impl LabelStyles {
    pub fn new() -> Self;
    pub fn style(self, v: LabelStyleKind) -> Self;
    pub fn to_view(self) -> View;
}
```

Style — sets the style for labels within this view.

| Type | Values |
|---|---|
| `LabelStyleKind` | `TitleAndIcon` `TitleOnly` `IconOnly` |

- Preview: three `Foo` rows with icon/title visibility following the style.
- Size `180x110`.

## Usage / Example

Run the gallery demo:

```bash
cargo run --example labels
```

Minimal usage:

```rust
use tontooui::prelude::*;

let custom = CustomLabel::new().title("Foo Bar");
let image = ImageLabel::new().title_key("welcome");
let system = SystemImageLabel::new().system_name("star").title_key("Foo");
let styles = LabelStyles::new().style(LabelStyleKind::TitleAndIcon);

let root = VStack::new()
    .spacing(8.0)
    .child(custom.to_view())
    .child(styles.to_view());
```

Category folder layout:

```
src/elements/labels/
  mod.rs                // category root
  custom_label.rs       // CustomLabel
  image_label.rs        // ImageLabel
  system_image_label.rs // SystemImageLabel
  label_styles.rs       // LabelStyles
```

## Cross References

- [Text.md](Text.md) -- `TextFormat` covers formatted text; `Label` adds icon plus title
- [Button.md](Button.md) -- buttons can host label content
- [Shapes.md](Shapes.md) -- label icons can reuse shape glyphs
