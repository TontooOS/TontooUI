# ScrollView

ScrollView is a SwiftUI-style scrollable container. It wraps content in a `GtkScrolledWindow` with Light/Dark adaptive background (`#1d1d1d` Dark / `#ececec` Light) and an optional hard edge effect (hard cutoff dividing line). Own category/folder `src/elements/scroll_views/`.

## Constructor

```rust
pub fn new() -> Self
```

Creates a vertical scroll view (`vertical: true`, `horizontal: false`, `Soft` edge).

## Builder Methods

| Method | Signature | Description |
|---|---|---|
| `content` | `content(self, w: impl Widget + 'static) -> Self` | Scrollable widget (e.g. `VStack` with rows) |
| `view_content` | `view_content(self, v: View) -> Self` | Scrollable `View` (alternative to `content`) |
| `horizontal` | `horizontal(self, h: bool) -> Self` | Enable horizontal scrolling |
| `vertical` | `vertical(self, v: bool) -> Self` | Enable vertical scrolling |
| `edge_effect` | `edge_effect(self, e: ScrollEdgeEffect) -> Self` | `Soft` or `Hard` |
| `hard_edge` | `hard_edge(self) -> Self` | Shorthand for `Hard` (hard cutoff + 1px dividing line) |
| `color_scheme` | `color_scheme(self, c: ColorScheme) -> Self` | Force Dark/Light |
| `to_view` | `to_view(self) -> View` | Wrap in a `View` (360×220) |

## ScrollEdgeEffect

```rust
pub enum ScrollEdgeEffect { Soft, Hard }
```

- `Soft` — no edge line, standard scroll.
- `Hard` — `HardScrollEdgeEffect` from the screenshot: a `1px` dividing line (`rgba(255,255,255,0.10)` Dark / `rgba(0,0,0,0.10)` Light) at the top edge of the scroll content with hard cutoff (no fade).

## Behavior

- Uses `GtkScrolledWindow` with `Automatic`/`Never` policies per `horizontal`/`vertical`.
- Background follows the app background (`#1d1d1d` Dark / `#ececec` Light) via `scrolledwindow` and `viewport` CSS. The hard edge line is a `GtkSeparator` appended above the content.
- The element sits directly on the app background — no card wrapper — and follows the system scheme.
- Content is any `Widget` (pure TontooUI API: `VStack` with `Text`/`Divider` etc., SF Pro for text).

## Usage / Example

```rust
use tontooui::prelude::*;
use tontooui::ScrollView;

let content = VStack::new().spacing(6.0)
    .child(Text::new("Item 5"))
    .child(Divider::horizontal().length(200.0))
    .child(Text::new("Item 6"));

let scroll = tontooui::ScrollView::new()
    .content(content)
    .hard_edge()
    .vertical(true);
```

See `examples/scroll_views.rs` for the 1:1 demo (`HardScrollEdgeEffect` with Items 5–9) directly on background, pure TontooUI API, Ampeln visible. Also used via `ScrollView::new().content(view)` in `Divider` sections.

## Cross References

- [MAIN.md](MAIN.md) -- library overview
- [Divider.md](Divider.md) -- separator used inside ScrollView content
- [List.md](List.md) -- list often placed inside a ScrollView
