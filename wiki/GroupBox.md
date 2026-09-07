# GroupBox

GroupBox is a SwiftUI-style inset grouped container. It renders a rounded rect that sits on the window background — lighter than the window in dark mode (`#2C2C2E` on `#1d1d1d`) and darker in light mode (`#E6E6E8` on `#ececec`) — with `16px` radius and hairline separators between rows. Own category/folder `src/elements/group_boxes/`.

## Constructor

```rust
pub fn new() -> Self
pub fn with_label(label: impl Into<String>) -> Self
```

Creates an empty GroupBox. `with_label` adds a label above the box (SF Pro 11px semibold).

## Builder Methods

| Method | Signature | Description |
|---|---|---|
| `label` | `label(self, l: impl Into<String>) -> Self` | Label above the box (e.g. "Hello World") |
| `background` | `background(self, c: Color) -> Self` | Custom background (overrides adaptive `#2C2C2E`/`#E6E6E8`) |
| `child` | `child(self, w: impl Widget + 'static) -> Self` | Append row widget (Toggle, Text, etc.) |
| `color_scheme` | `color_scheme(self, c: ColorScheme) -> Self` | Force Dark/Light |
| `width` | `width(self, w: f32) -> Self` | Fixed container width |
| `to_view` | `to_view(self) -> View` | Wrap in a `View` |

## Behavior

- Background is adaptive via `resolve_scheme`: Dark `#2C2C2E`, Light `#E6E6E8`. Custom `background` overrides.
- Separators between rows are `1px` `rgba(255,255,255,0.08)` Dark / `rgba(0,0,0,0.08)` Light, with `6px` vertical margins.
- No border — only rounded rect with inner padding `6px 14px`.
- Label is `SF Pro Display` 11px semibold (`#ececec` Dark / `#1d1d1d` Light) with 4px bottom margin.
- When `children` is empty, the box shows the lorem placeholder from the screenshot (GroupBox preview).
- The element is placed directly on the app background — no card wrapper — and follows the system scheme live.

## Usage / Example

```rust
use tontooui::prelude::*;

// With label (as in screenshot)
let with_label = GroupBox::with_label("Hello World")
    .child(Text::new("Lorem ipsum dolor sit amet..."))
    .width(260.0);

// Simple
let simple = GroupBox::new()
    .child(Text::new("Lorem ipsum..."))
    .width(260.0);

// Custom background
let custom = GroupBox::new()
    .background(Color::from_hex("#0A84FF").unwrap())
    .child(Text::new("Lorem ipsum...").color(Color::WHITE))
    .width(260.0);
```

See `examples/group_boxes.rs` for the 1:1 demo (3 variants: `GroupBox With Label`, `GroupBox`, `GroupBox Background`) directly on background, pure TontooUI API, Ampeln visible.

## Cross References

- [MAIN.md](MAIN.md) -- library overview
- [Divider.md](Divider.md) -- separator used between GroupBox rows
