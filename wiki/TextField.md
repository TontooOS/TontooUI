# TextField

SwiftUI-style TextField category for TontooUI, recreating text field styles. The category contains one `modifier` element. Rendering is directly on the window background (#1d1d1d dark / #ececec light) with no extra card, using `SF Pro Display`. The base input control is `TextInput`; this category holds the field styles.

## CapsuleTextField

```rust
pub struct CapsuleTextField { /* ... */ }
impl CapsuleTextField {
    pub fn new() -> Self;
    pub fn placeholder(self, v: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — gives your text field a capsule shape. This only has a visible effect on macOS (Liquid Glass dark mode preview); on other platforms the field falls back to a rounded style.

- Preview: frosted capsule pill `150x32` (`999px` radius) with placeholder plus caret on a navy `180x110` phone card.
- Size `180x110`.

## Usage / Example

Run the gallery demo:

```bash
cargo run --example text_fields
```

Minimal usage:

```rust
use tontooui::prelude::*;

let field = CapsuleTextField::new().placeholder("Search");

let root = VStack::new()
    .spacing(8.0)
    .child(field.to_view());
```

Category folder layout:

```
src/elements/text_fields/
  mod.rs                // category root
  capsule_text_field.rs // CapsuleTextField
```

## Cross References

- [TextInput.md](TextInput.md) -- the base single-line input control the style applies to
- [Text.md](Text.md) -- formatted text rendering inside fields
- [Label.md](Label.md) -- placeholder text pairs with label styles
