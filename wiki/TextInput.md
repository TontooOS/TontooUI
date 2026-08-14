# TextInput

TextInput is a single-line text input field with a SwiftUI-style builder API. It wraps GTK4's `Entry` widget with TontooOS styling (dark theme, SF Pro font, accent color on focus).

## Constructor

```rust
pub fn new(placeholder: impl Into<String>) -> Self
```

Creates a new text input with the given placeholder text.

## Builder Methods

| Method | Signature | Description |
|---|---|---|
| `text` | `text(self, text: impl Into<String>) -> Self` | Set initial text value |
| `password` | `password(self) -> Self` | Hide input text (show dots) |
| `disabled` | `disabled(self) -> Self` | Disable the input |
| `accent_color` | `accent_color(self, color: Color) -> Self` | Set focus border color |
| `frame` | `frame(self, width: f32, height: f32) -> Self` | Set size |
| `on_change` | `on_change(self, handler: impl Fn(String) + Send + Sync + 'static) -> Self` | Callback on text change |
| `on_submit` | `on_submit(self, handler: impl Fn(String) + Send + Sync + 'static) -> Self` | Callback on Enter press |
| `at` | `at(self, x: f32, y: f32) -> Self` | Absolute positioning |

## Accessor Methods

| Method | Return | Description |
|---|---|---|
| `text_value()` | `&str` | Current text value |
| `placeholder()` | `&str` | Placeholder text |

## ViewContent Implementation

TextInput implements `ViewContent`, so it can be wrapped in a `View`:

```rust
let view = View::new(input).with_frame(16.0, 16.0, 300.0, 44.0);
```

## Usage / Example

```rust
use tontooui::prelude::*;

let input = TextInput::new("Search...")
    .text("Hello")
    .password()
    .accent_color(Color::from_hex("#FF6B2B").unwrap())
    .frame(300.0, 44.0)
    .on_change(|text| println!("Changed: {}", text))
    .on_submit(|text| println!("Submitted: {}", text));
```

## Cross References

- [MAIN.md](MAIN.md) -- library overview
- [Slider.md](Slider.md) -- another interactive element
