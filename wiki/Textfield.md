# Textfield

Textfield category in `src/elements/textfield/`: `BasicTextField`
in `basic.rs` is the slim single-line field (like the top reference
row), `LargeTextField` in `large.rs` the roomier single-line field
(like the bottom row), `SecureField` in `secure.rs` masks secrets
behind bullets, and `TextEditor` in `editor.rs` is the large
multi-line editor. All share the `FieldCore` editing core in
`mod.rs`. Click inside to select (caret to end); ESC or a click
outside deselects. While selected the field shows an accent focus
ring in the user's accent color (wired through
`set_theme(accent, dark)` like a normal button), a blinking caret,
placeholder, typing, Backspace and caret keys.

## Geometry

| Token | Value |
|---|---|
| `TEXTFIELD_BG_DARK` / `TEXTFIELD_BG_LIGHT` | `#2C2C2E` / `#FFFFFF` field fill |
| `TEXTFIELD_BORDER_DARK` / `TEXTFIELD_BORDER_LIGHT` | White 36 alpha / black 60 alpha idle border |
| `TEXTFIELD_ACCENT` | `#007AFF` default accent |
| `TEXTFIELD_RING_W` | 2 px accent ring while selected |
| `TEXTFIELD_PLACEHOLDER_DARK` / `TEXTFIELD_PLACEHOLDER_LIGHT` | `#9A9A9E` / `#6E6E72` placeholder gray |
| `TEXTFIELD_CARET_W` | 2 px caret width |
| `TEXTFIELD_BLINK_SECONDS` | 1.06 s caret blink period |
| `TEXTFIELD_FONT_SIZE` / `TEXTFIELD_PAD_X` / `TEXTFIELD_PAD_Y` / `TEXTFIELD_RADIUS` | 13 px / 8 px / 6 px / 8 px basic metrics |
| `LARGE_FIELD_FONT_SIZE` / `LARGE_FIELD_PAD_X` / `LARGE_FIELD_PAD_Y` / `LARGE_FIELD_RADIUS` | 15 px / 12 px / 10 px / 10 px large metrics |
| `EDITOR_FONT_SIZE` / `EDITOR_PAD` / `EDITOR_RADIUS` | 14 px / 12 px / 10 px editor metrics |
| `EDITOR_WRAP_W` / `EDITOR_MIN_H` | 240 px intrinsic wrap width / 120 px minimum height |

## FieldCore

Shared editing core behind all variants: text plus caret (always
a char boundary, UTF-8 safe), placeholder, selection and accent.
`masked` echoes bullets per char (secure fields), `multiline` keeps
`\n` (editor, single line filters it). Layouts cache per content,
color, size, wrap and scale per the crisp text rules.
`insert` skips control chars and newlines; `backspace` deletes the
char before the caret; `track_caret` scrolls long text so the caret
stays visible.

## BasicTextField / LargeTextField

```rust
pub fn new(placeholder: impl Into<String>) -> Self
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn on_change(self, callback: impl FnMut(&str) + 'static) -> Self
pub fn set_text(&mut self, text: impl Into<String>)
pub fn set_placeholder(&mut self, placeholder: impl Into<String>)
pub fn text_value(&self) -> &str
pub fn placeholder_value(&self) -> &str
pub fn is_selected(&self) -> bool
pub fn type_text(&mut self, content: &str)
pub fn key(&mut self, key: Key) -> bool
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- `type_text` inserts at the caret while selected (the app forwards
  its `text` here); programmatic `set_text` moves the caret to the
  end without firing `on_change`, which fires with the full text on
  every user edit.
- `key` handles Backspace, Left/Right and ESC while selected and
  reports whether it consumed the key (the app forwards its `key`
  here). `mouse_down` selects inside (caret to end) and deselects
  anywhere else, so the app forwards every press here.
- Draw paints the fill, the accent ring while selected (subtle
  border otherwise), and the text clipped to the padded box with
  caret tracking plus a blinking accent caret. Unfocused windows
  desaturate like the palette.

## SecureField

```rust
pub fn new(placeholder: impl Into<String>) -> Self
```

- Same slim API as `BasicTextField` (`set_theme`, `on_change`,
  `set_text`, `type_text`, `key`, `mouse_down`, `is_selected`,
  `rect`): every char echoes as a bullet, so shoulders see nothing.
  `text_value` always returns the real text. Caret math counts chars,
  not bytes.

## TextEditor

```rust
pub fn new(placeholder: impl Into<String>) -> Self
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn on_change(self, callback: impl FnMut(&str) + 'static) -> Self
pub fn set_text(&mut self, text: impl Into<String>)
pub fn set_placeholder(&mut self, placeholder: impl Into<String>)
pub fn text_value(&self) -> &str
pub fn is_selected(&self) -> bool
pub fn type_text(&mut self, content: &str)
pub fn key(&mut self, fonts: &mut FontSystem, key: Key) -> bool
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Large multi-line editor: wrapped text, Enter breaks the line,
  Up/Down/Left/Right move the caret (Up/Down keep the column via
  parley line ranges), vertical caret tracking with clipping.
  Intrinsic measure wraps at `EDITOR_WRAP_W` with an `EDITOR_MIN_H`
  floor; place generously (the demo uses a fixed tall box).
- `key` takes fonts for the multiline caret geometry (unlike the
  single-line fields); buffer keys to `draw` when the app has no
  fonts in its `key` handler (see the demo's `pending` queue).

## Usage / Example

```rust
use tontooui::elements::{BasicTextField, LargeTextField};
use tontooui::renderer::window::Key;

let mut field = BasicTextField::new("Enter text here");
field.set_theme(accent, true);

// Input: every press reaches the fields, text and keys the selected one.
fn mouse_down(&mut self, x: f64, y: f64) {
    field.mouse_down(x, y);
}
fn text(&mut self, text: &str) {
    field.type_text(text);
}
fn key(&mut self, key: Key) {
    field.key(key);
}
```

See `examples/textfield.rs` for the full demo (all four fields
with a live value readout and a pending-key queue for the editor).

## Cross References

- [Text.md](Text.md) – field text and placeholder rendering
- [Button.md](Button.md) – accent wiring pattern
- [Theme.md](Theme.md) – accent and unfocused desaturation
- [Layout.md](Layout.md) – `View` protocol
