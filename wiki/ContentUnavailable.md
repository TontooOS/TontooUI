# ContentUnavailable

Unavailable category in `src/elements/unavailable/`: `ContentUnavailable`
in `content.rs` is an empty-state placeholder with a large SF Symbol
on top, a semibold title, a gray message and an optional prominent
refresh button (like the reference rows: tray icon, "No Data",
"There is no data to display yet. Pull down to refresh.", blue
Refresh). Composed from the stock views (symbol, texts, button in a
centered stack); more placeholder variants plug in the same way.

## Geometry

| Token | Value |
|---|---|
| `UNAVAILABLE_ICON_SIZE` | 64 px SF icon box |
| `UNAVAILABLE_ICON_GRAY` | `#8E8E93` default icon tint (both modes) |
| `UNAVAILABLE_ICON_GAP` | 16 px icon-title gap |
| `UNAVAILABLE_TITLE_GAP` | 8 px title-message gap |
| `UNAVAILABLE_BUTTON_GAP` | 16 px message-button gap |
| `UNAVAILABLE_WRAP_WIDTH` | 320 px message wrap width |

## ContentUnavailable

```rust
pub fn new(icon: impl Into<String>, title: impl Into<String>, message: impl Into<String>) -> Self
pub fn icon(self, name: impl Into<String>) -> Self
pub fn icon_color(self, color: Color) -> Self
pub fn title(self, title: impl Into<String>) -> Self
pub fn message(self, message: impl Into<String>) -> Self
pub fn refresh(self, enabled: bool) -> Self
pub fn refresh_label(self, label: impl Into<String>) -> Self
pub fn accent(self, color: Color) -> Self
pub fn set_theme(&mut self, mode: ThemeMode, accent: Color)
pub fn set_focused(&mut self, focused: bool)
pub fn set_title(&mut self, title: impl Into<String>)
pub fn set_message(&mut self, message: impl Into<String>)
pub fn set_refresh(&mut self, enabled: bool)
pub fn is_refreshing(&self) -> bool
pub fn take_refreshed(&mut self) -> bool
pub fn finish_refresh(&mut self)
pub fn set_hover(&mut self, x: f32, y: f32)
pub fn icon_value(&self) -> &str
pub fn refresh_visible(&self) -> bool
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- The icon resolves through CoreIcon like everywhere else; without
  `.icon_color()` it stays `UNAVAILABLE_ICON_GRAY`. The title uses
  the semibold headline style, the message the secondary (dim)
  foreground with centered wrapping.
- The refresh button is on by default ("Refresh"); `refresh(false)`
  removes it, `refresh_label` renames it (implies on). Its accent
  defaults to system blue.
- Presses reach the button through the `View` protocol (stacks
  forward); hover needs the explicit `set_hover` forward. The app
  polls `take_refreshed` per frame (true once per click, marks the
  refresh in flight) and calls `finish_refresh` when the reload
  lands. `set_text`-style live updates go through `set_title` and
  `set_message`.

## Usage / Example

```rust
use tontooui::elements::{ContentUnavailable, View, VStack};

let mut empty = ContentUnavailable::new(
    "tray",
    "No Data",
    "There is no data to display yet. Pull down to refresh.",
);

// Per frame:
empty.set_theme(mode, accent);
empty.set_focused(focused);
if empty.take_refreshed() {
    // ...reload...
    empty.finish_refresh();
}
empty.draw(scene, fonts, images);
```

See `examples/unavailable.rs` for the full demo (refresh variant
with a counter, button-less variant and search variant).

## SearchEmpty

```rust
pub fn new(icon: impl Into<String>, title: impl Into<String>, message: impl Into<String>) -> Self
pub fn icon(self, name: impl Into<String>) -> Self
pub fn icon_color(self, color: Color) -> Self
pub fn title(self, title: impl Into<String>) -> Self
pub fn message(self, message: impl Into<String>) -> Self
pub fn set_theme(&mut self, mode: ThemeMode)
pub fn set_focused(&mut self, focused: bool)
pub fn set_title(&mut self, title: impl Into<String>)
pub fn set_message(&mut self, message: impl Into<String>)
pub fn icon_value(&self) -> &str
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Search empty state: icon, title and message with no refresh
  button (like the reference rows: magnifier, "No Results", "Check
  the spelling or try a new search."). Thin wrapper over the
  `ContentUnavailable` composition, so layout always matches.
- `set_theme` takes only the mode (no button, so no accent); all
  setters rebuild on change only, like the sibling.

## Cross References

- [Images.md](Images.md) – `SFSymbolImage` used for the icon
- [Text.md](Text.md) – headline title and secondary message
- [Button.md](Button.md) – prominent refresh button
- [Layout.md](Layout.md) – centered stack composition
