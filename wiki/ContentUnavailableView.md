# ContentUnavailableView

ContentUnavailableView is a recreation of the iOS SwiftUI empty states
(`ContentUnavailableView`, `ContentUnavailableView.search(text:)` and
`ContentUnavailableView.search`). It shows a centered SF Symbol icon, a bold
title and a thin hint message below, optionally with a call-to-action button
(e.g. "Switch Account"). The query is rendered statically in the title; there
is no search field. The element follows the system color scheme automatically
(Dark `#1d1d1d` / Light `#ececec`, SF Pro).

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
| `title` | `title(self, prefix: impl Into<String>) -> Self` | Title prefix before the quoted query; when query is empty and prefix is not "No Results for", the title renders as plain prefix (e.g. "No Mail") |
| `message` | `message(self, text: impl Into<String>) -> Self` | Hint message below the title |
| `icon` | `icon(self, name: impl Into<String>) -> Self` | SF Symbol name (e.g. "tray", "magnifyingglass"); default "magnifyingglass" |
| `button` | `button(self, label: impl Into<String>) -> Self` | Optional pill button below the message (e.g. "Switch Account") |
| `frame` | `frame(self, w: f32, h: f32) -> Self` | Element size |
| `width` | `width(self, w: f32) -> Self` | Element width |
| `height` | `height(self, h: f32) -> Self` | Element height |
| `color_scheme` | `color_scheme(self, c: ColorScheme) -> Self` | Force dark/light (default: system detection) |

## Behavior

- The title renders as `No Results for "foo"`. When the query is empty and
  the prefix is "No Results for", the quoted slot shows a space, matching the
  reference SwiftUI fallback; for plain titles like "No Mail" an empty query
  renders just the prefix.
- Icons are the real SF Symbols from CoreIcon (default `magnifyingglass`,
  `tray`/`envelope` for mail) recolored to the theme gray (alpha-mask tint).
- Colors follow the reference design: light mode uses zinc tones
  (`#18181b` title, `#a1a1aa` secondary), dark mode uses `#ececec` title,
  `#71717a` secondary. Background is transparent — the view sits directly on
  the app background (`#1d1d1d` dark / `#ececec` light, SF Pro).
- The button (if set) renders as a blue pill (`#0A84FF`, white SF Pro 12px
  semibold, 999px radius) below the message.
- Without the `coreicon` feature the element renders without icons.

## Usage / Example

```rust
use tontooui::prelude::*;

// Search with text: "No Results for "foo""
let search_text = ContentUnavailableView::new()
    .query("foo")
    .icon("magnifyingglass")
    .width(320.0).height(170.0);

// Plain search: "No Results"
let search = ContentUnavailableView::new()
    .title("No Results")
    .query("")
    .icon("magnifyingglass")
    .width(220.0).height(170.0);

// Placeholder with button: "No Mail"
let unavailable = ContentUnavailableView::new()
    .title("No Mail")
    .message("New mails you receive will appear here.")
    .icon("tray")
    .button("Switch Account")
    .width(220.0).height(170.0);
```

See `examples/content_unavailable.rs` for the 1:1 demo (3 variants directly on the
background, pure TontooUI API, Ampeln visible).

## Cross References

- [MAIN.md](MAIN.md) -- library overview
- [Sidebar.md](Sidebar.md) -- another element using CoreIcon SF Symbols