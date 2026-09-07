# LabeledContent

SwiftUI-style LabeledContent category for TontooUI, recreating labeled informational views. The category contains three `initializer` elements. Rendering is directly on the window background (#1d1d1d dark / #ececec light) with no extra card, using `SF Pro Display`.

## CustomLabeledContent

```rust
pub struct CustomLabeledContent { /* ... */ }
impl CustomLabeledContent {
    pub fn new() -> Self;
    pub fn title(self, v: impl Into<String>) -> Self;
    pub fn subtitle(self, v: impl Into<String>) -> Self;
    pub fn value(self, v: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a standard labeled element, with a view that conveys the value of the content.

- Preview: dark pill `160x34` with star icon, `Foo` plus `Sub` on the left and blue `bar` on the right.
- Size `180x110`.

## FormattedLabeledContent

```rust
pub struct FormattedLabeledContent { /* ... */ }
impl FormattedLabeledContent {
    pub fn new() -> Self;
    pub fn title(self, v: impl Into<String>) -> Self;
    pub fn amount(self, v: f64) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a labeled informational view from a formatted value.

- Preview: dark pill `160x30` with `Amount` left and formatted `42.00` right.
- Size `180x110`.

## LabeledContent

```rust
pub struct LabeledContent { /* ... */ }
impl LabeledContent {
    pub fn new() -> Self;
    pub fn title(self, v: impl Into<String>) -> Self;
    pub fn value(self, v: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a labeled informational view.

- Preview: dark pill `160x30` with `Foo` left and dimmed `Bar` right.
- Size `180x110`.

## Usage / Example

Run the gallery demo:

```bash
cargo run --example labeled_contents
```

Minimal usage:

```rust
use tontooui::prelude::*;

let custom = CustomLabeledContent::new().title("Foo").subtitle("Sub").value("bar");
let formatted = FormattedLabeledContent::new().title("Amount").amount(42.0);
let plain = LabeledContent::new().title("Foo").value("Bar");

let root = VStack::new()
    .spacing(8.0)
    .child(custom.to_view())
    .child(plain.to_view());
```

Category folder layout:

```
src/elements/labeled_contents/
  mod.rs                       // category root
  custom_labeled_content.rs    // CustomLabeledContent
  formatted_labeled_content.rs // FormattedLabeledContent
  labeled_content.rs           // LabeledContent
```

## Cross References

- [Label.md](Label.md) -- labels pair title plus icon; labeled content pairs label plus value
- [Text.md](Text.md) -- values render with text styles
