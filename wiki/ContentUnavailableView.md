# ContentUnavailableView

ContentUnavailableView is a recreation of the iOS SwiftUI empty state ("No
Results for ..."). It shows a centered magnifier icon, a bold title and a thin
hint message below. The query is rendered statically in the title; there is no
search field. There is no light/dark toggle: the element follows the given or
detected color scheme automatically.

## Constructor

```rust
pub fn new() -> Self
```

Creates the empty state with default English strings (`No Results for`,
`Check the spelling or try a new search.`).

## Builder Methods

| Method | Signature | Description |
|---|---|---|
| `query` | `query(self, text: impl Into<String>) -> Self` | Query text shown quoted in the title |
| `title` | `title(self, prefix: impl Into<String>) -> Self` | Title prefix before the quoted query |
| `message` | `message(self, text: impl Into<String>) -> Self` | Hint message below the title |
| `frame` | `frame(self, w: f32, h: f32) -> Self` | Element size |
| `width` | `width(self, w: f32) -> Self` | Element width |
| `height` | `height(self, h: f32) -> Self` | Element height |
| `color_scheme` | `color_scheme(self, c: ColorScheme) -> Self` | Force dark/light (default: system detection) |

## Behavior

- The title renders as `No Results for "foo"`. When the query is empty the
  quoted slot shows a space, matching the reference SwiftUI fallback.
- Icons are the real SF Symbols from CoreIcon: `magnifyingglass` is recolored
  to the theme gray (alpha-mask tint over the shipped asset).
- Colors follow the reference design: light mode uses zinc tones
  (`#18181b` title, `#a1a1aa` secondary), dark mode uses `#ececec` title,
  `#71717a` secondary.
- The element is not interactive (no search field, no callbacks).
- Without the `coreicon` feature the element renders without icons.

## Usage / Example

```rust
use tontooui::prelude::*;

let view = ContentUnavailableView::new()
    .query("foo")
    .width(320.0)
    .height(250.0);
```

## Cross References

- [MAIN.md](MAIN.md) -- library overview
- [Sidebar.md](Sidebar.md) -- another element using CoreIcon SF Symbols