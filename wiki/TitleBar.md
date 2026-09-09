# TitleBar

macOS-style decoration bar with traffic lights, title and app content. A
standalone element so TontooUI apps can build windows with a top bar: the
traffic lights stay reserved on the left, the app fills the rest through
custom content (replacing the title zone), and the title text can be
hidden. This element is unrelated to `Sidebar`: the sidebar keeps its own
fixed layout and accepts no title-bar content.

```rust
use tontooui::prelude::*;

let bar = TitleBar::new()
    .title("Finder")
    .content(HStack::new()
        .spacing(8.0)
        .child(Button::new("Share")))
    .without_maximize();

let view = View::new(bar);
```

## Layout

From left to right the bar contains:

1. Traffic lights (close + minimize + maximize, always reserved)
2. Custom content via `content` (fills the rest, lays out its own
   alignment) **or** the centered title text
3. Nothing else: no balancing spacer is needed with custom content

| Method | Description |
|---|---|
| `title(text)` | Centered title text (empty = no title) |
| `show_title(bool)` | Show or hide the title text (default `true`) |
| `without_title()` | Hide the title text |
| `content(widget)` | Custom content filling the bar after the reserved lights; replaces the title zone |
| `show_maximize(bool)` | Show or hide the green maximize button (default `true`) |
| `without_maximize()` | Hide the green button, keep close and minimize |
| `minimize_enabled(bool)` | Enable the middle button (default `true`); disabled stays gray and ignores clicks |
| `height(f32)` | Explicit bar height in px (`0.0` = auto, default `31.0`) |
| `has_content()` | Whether custom content is set |
| `bar_height()` | Effective bar height in px |

- `content` accepts any UIKit widget, including other TontooUI elements
  (they implement the UIKit `Widget` trait).
- `new` never fails, all builder methods return `Self`.
- `render` builds a UIKit `TrafficLights` bar with the same settings, so
  the element matches the App window bar exactly.

## Sidebar Boundary

`Sidebar` has no title-bar API on purpose: no `content`, no `title` and
no light toggles. Sidebar traffic lights, search and item list stay fixed
so sidebar windows keep one consistent look. Apps needing a custom top
bar use `TitleBar`, not `Sidebar`.

## Usage / Example

In-content bar above app content (with `no_window_bar` on the App):

```rust
use tontooui::prelude::*;

app.set_root(
    VStack::new()
        .spacing(0.0)
        .child(TitleBar::new()
            .title("Demo")
            .content(HStack::new()
                .spacing(8.0)
                .child(Button::new("Refresh"))))
        .child(content)
);
```

Borderless windows combine it with the UIKit frame switch (same `App`,
re-exported through the TontooUI prelude):

```rust
app.no_window_frame();
```

## Cross References

- [Sidebar.md](Sidebar.md) -- sidebar with traffic lights and item list (no title-bar API)
- [Toolbar.md](Toolbar.md) -- glass capsule toolbar elements
- [Button.md](Button.md) -- SwiftUI-style button element
- [MAIN.md](MAIN.md) -- wiki entry point
