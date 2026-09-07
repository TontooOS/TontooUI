# Divider

Divider is a SwiftUI-style separator line. It renders a thin horizontal or vertical line with Light/Dark adaptive colors (Dark `#3a3a3d` / Light `#d1d1d6`), SF Pro context via surrounding Text, and customizable thickness.

## Constructor

```rust
pub fn new() -> Self
pub fn horizontal() -> Self
pub fn vertical() -> Self
```

Creates a divider. `new()` / `horizontal()` is horizontal, `vertical()` is vertical.

## Builder Methods

| Method | Signature | Description |
|---|---|---|
| `thickness` | `thickness(self, t: f32) -> Self` | Line thickness in px (min 0.5, default 1.0) |
| `color` | `color(self, c: Color) -> Self` | Override color (default adaptive) |
| `length` | `length(self, l: f32) -> Self` | Fixed length (width for horizontal, height for vertical) |
| `frame` | `frame(self, w: f32, h: f32) -> Self` | Set length via frame (w for horizontal, h for vertical) |
| `to_view` | `to_view(self) -> View` | Wrap in a `View` with frame |

## Behavior

- Colors are resolved via `resolve_scheme(None)`: Dark `#3a3a3d` (explicit `#3a3a3d`), Light `#d1d1d6`. An explicit `color` overrides the adaptive default.
- Thickness controls `min-height` (horizontal) or `min-width` (vertical) via CSS `separator { background-color: hex; }` on `gtk::Separator`.
- Length: if set, `width_request` (horizontal) or `height_request` (vertical) is set; otherwise the divider expands to the available width/height from its parent (`frame` or parent layout).
- The element is not interactive and sits directly on the background (`#1d1d1d` Dark / `#ececec` Light) — no card wrapper.
- Follows the same `Light/Dark` logic as `Menu` dividers (`rgba(255,255,255,0.08)` Dark) but with solid hex for standalone use.

## Usage / Example

```rust
use tontooui::prelude::*;

// Horizontal full-width divider
let h = Divider::horizontal().length(180.0);

// Vertical divider between texts
let row = HStack::new().spacing(12.0)
    .child(Text::new("Left"))
    .child(Divider::vertical().length(60.0))
    .child(Text::new("Right"));

// Thick tinted divider
let thick = Divider::horizontal().thickness(2.0).length(180.0).color(Color::from_hex("#0A84FF").unwrap());
```

See `examples/dividers.rs` for the demo (4 variants: Horizontal, Vertical, Thick, Section) directly on background, pure TontooUI API, Ampeln visible. Also used inside `Menu` as `MenuEntry::Divider`.

## Cross References

- [MAIN.md](MAIN.md) -- library overview
- [Menu.md](Menu.md) -- `Divider` as `MenuEntry::Divider`
- [ViewThatFits.md](ViewThatFits.md) -- adaptive container often separated by dividers
