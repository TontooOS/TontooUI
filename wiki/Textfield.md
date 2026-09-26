# Textfield

Textfield category in `src/elements/textfield/`: `BasicTextField`
in `basic.rs` is the slim single-line field (like the top reference
row), `LargeTextField` in `large.rs` the roomier single-line field
(like the bottom row), `SecureField` in `secure.rs` masks secrets
behind bullets, and `TextEditor` in `editor.rs` is the large
multi-line editor. All share the `FieldCore` editing core in
`mod.rs`. Click inside to focus (the caret lands at the click);
ESC or a click outside deselects. While selected the field shows an
accent focus ring in the user's accent color (wired through
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
| `LARGE_EDITOR_FONT_SIZE` / `LARGE_EDITOR_PAD` / `LARGE_EDITOR_RADIUS` | 15 px / 14 px / 12 px large editor metrics |
| `LARGE_EDITOR_WRAP_W` / `LARGE_EDITOR_MIN_H` | 320 px intrinsic wrap width / 220 px minimum height |
| `LARGE_EDITOR_BG_DARK` / `LARGE_EDITOR_BG_LIGHT` | `#141416` / `#F2F2F5` large editor fill |
| `SEARCH_FONT_SIZE` / `SEARCH_ICON_SIZE` / `SEARCH_PAD_X` / `SEARCH_GAP` | 14 px / 16 px / 14 px / 8 px search metrics |

## FieldCore

Shared editing core behind all variants: text plus caret (always
a char boundary, UTF-8 safe), placeholder, selection and accent.
`masked` echoes bullets per char (secure fields), `multiline` keeps
`\n` (editor, single line filters it). Layouts cache per content,
color, size, wrap and scale per the crisp text rules.
`insert` skips control chars and newlines; `backspace` deletes the
highlight, else the char before the caret; `track_caret` scrolls
long text so the caret stays visible.

## Selection

The highlight is a fixed `anchor` plus the moving `caret`, visible
while they differ. Click places the caret (resolved in `draw`,
where fonts are available); double-click highlights the word;
dragging extends from the press anchor (the app forwards
`set_hover` and `mouse_up` for this). Shift+Left/Right/Up/Down
extend per direction (see [Renderer.md](Renderer.md) for the shell
mapping). The highlight paints as an accent wash under the text
(per wrapped line in editors). Every variant exposes it:

```rust
pub fn select_all(&mut self)
pub fn selected_text(&self) -> String
pub fn selection_range(&self) -> Option<(usize, usize)>
```

## Shortcuts and Undo

Ctrl shortcuts arrive as `Key` intents from the shell (Ctrl+A/C/X/
V/Z/Y plus Ctrl+Shift+Z for redo, see [Renderer.md](Renderer.md))
and work in every variant through the shared `handle_key`:

| Shortcut | Action |
|---|---|
| Ctrl+A | Highlight everything |
| Ctrl+C / Ctrl+X | Copy / cut the highlight to the clipboard |
| Ctrl+V | Paste clipboard text at the caret (replaces the highlight) |
| Ctrl+Z / Ctrl+Y | Undo / redo (100 steps, text plus caret) |

Copy and paste use the system clipboard (`arboard`) with an
in-process fallback where no display server answers, so shortcuts
keep working headless. Typing or pasting over a highlight replaces
it in a single undo step. Programmatic `set_text` clears the undo
stacks. Direct stack access per variant:

```rust
pub fn undo(&mut self) -> bool
pub fn redo(&mut self) -> bool
pub fn copy_selection(&mut self) -> bool
pub fn cut_selection(&mut self) -> bool
pub fn paste_clipboard(&mut self)
```

## Text Cursor

Hovering a field shows the I-beam pointer. The variant tracks the
hover through `set_hover` and reports it:

```rust
pub fn wants_text_cursor(&self) -> bool
```

The app returns `CursorKind::Text` from `App::cursor` then (see
[Renderer.md](Renderer.md)); the shell sets the winit cursor after
every move. Tables forward this while their inline editor runs.

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
pub fn borderless(self, borderless: bool) -> Self
pub fn set_borderless(&mut self, borderless: bool)
pub fn align_right(self, align_right: bool) -> Self
pub fn set_align_right(&mut self, align_right: bool)
```

- `borderless` paints text, caret and highlight only (no fill,
  ring or border) for inputs embedded in form rows (see
  [Form.md](Form.md)).
- `align_right` hugs short content to the box end (form rows);
  long content scrolls like left-aligned, clicks map accordingly.
- `type_text` inserts at the caret while selected (the app forwards
  its `text` here); programmatic `set_text` moves the caret to the
  end without firing `on_change`, which fires with the full text on
  every user edit.
- `key` handles Backspace, Left/Right (Shift extends), the Ctrl
  shortcuts and ESC while selected and reports whether it consumed
  the key (the app forwards its `key` here). `mouse_down` focuses
  inside (caret lands at the click) and deselects anywhere else, so
  the app forwards every press here; the app also forwards
  `set_hover` (highlight drag plus I-beam tracking) and `mouse_up`
  (ends the drag).
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
  parley line ranges, Shift extends), vertical caret tracking with
  clipping. Intrinsic measure wraps at `EDITOR_WRAP_W` with an
  `EDITOR_MIN_H` floor; place generously (the demo uses a fixed
  tall box).
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

See `examples/textfield.rs` for the full demo (all six fields
with a live value readout and a pending-key queue for the editors).

## SearchField

```rust
pub fn new(placeholder: impl Into<String>) -> Self
pub fn set_theme(&mut self, mode: ThemeMode, accent: Color, glass: GlassAmount)
pub fn set_focused(&mut self, focused: bool)
pub fn on_change(self, callback: impl FnMut(&str) + 'static) -> Self
pub fn set_text(&mut self, text: impl Into<String>)
pub fn set_placeholder(&mut self, placeholder: impl Into<String>)
pub fn text_value(&self) -> &str
pub fn is_selected(&self) -> bool
pub fn type_text(&mut self, content: &str)
pub fn key(&mut self, key: Key) -> bool
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Toolbar-like clear (`Lens`) glass capsule with a magnifier icon and a
  single-line input on top. Same editing contract as the basic
  field, no accent ring (the glass carries the look). The frost
  needs the shell blur pass: the app opts in with `wants_backdrop`
  while visible, and the capsule skips the capture pass so the blur
  stays clean.

## LargeTextEditor

Same API as `TextEditor` (`new`, `set_theme`, `on_change`,
`set_text`, `type_text`, `key(fonts, key)`, `mouse_down`,
`is_selected`, `rect`), roomier in every direction with a
near-black inset fill. Both editors share the multiline caret
geometry and draw in `mod.rs`.

## Cross References

- [Text.md](Text.md) – field text and placeholder rendering
- [Button.md](Button.md) – accent wiring pattern
- [Theme.md](Theme.md) – accent and unfocused desaturation
- [Layout.md](Layout.md) – `View` protocol
