# Form

Form category in `src/elements/forms/form.rs`: settings-style
`Form` with titled `FormSection` groups and `FormRow` label/control
rows. Each section draws its group in the `GroupBox` body
(`GROUP_BG_DARK` / `GROUP_BG_LIGHT`, `GROUP_RADIUS`), one title
above, one footnote below, dividers between rows. Row kinds cover
the four reference blocks: `Basic Form` (text rows), `Form
Sections` (titled groups), `Form with Picker` (dropdown rows) and
`Form Button Row` (centered app-built buttons).

## Geometry

| Token | Value |
|---|---|
| `FORM_ROW_H` | 46 px row height |
| `FORM_PAD` | 16 px group inset |
| `FORM_LABEL_GAP` | 16 px label-to-control gap |
| `FORM_LABEL_SIZE` / `FORM_TITLE_SIZE` / `FORM_NOTE_SIZE` | 15 px labels / 17 px semibold titles / 13 px dim footnotes |
| `FORM_TITLE_H` / `FORM_TITLE_GAP` | 26 px title block / 8 px gap |
| `FORM_NOTE_H` / `FORM_NOTE_GAP` | 20 px footnote block / 8 px gap |
| `FORM_SECTION_GAP` | 28 px between sections |
| `FORM_BUTTON_GAP` | 12 px between buttons |
| `FORM_ICON_SIZE` / `FORM_ICON_GAP` | 18 px SF Symbol box / 8 px gap |
| `FORM_MIN_W` | 320 px intrinsic minimum width |

Controls in a section align to the widest label (plus icon), so
inputs, switches and dropdowns share one right column.

## FormRow

```rust
pub fn text(label: impl Into<String>, value: impl Into<String>) -> Self
pub fn secure(label: impl Into<String>, value: impl Into<String>) -> Self
pub fn toggle(label: impl Into<String>, on: bool) -> Self
pub fn picker(label: impl Into<String>, options: Vec<String>, selected: usize) -> Self
pub fn buttons(buttons: Vec<Button>) -> Self
pub fn icon(self, name: impl Into<String>) -> Self
pub fn placeholder(self, placeholder: impl Into<String>) -> Self
pub fn on_change(self, callback: impl FnMut(&str) + 'static) -> Self
pub fn on_toggle(self, callback: impl FnMut(bool) + 'static) -> Self
pub fn on_pick(self, callback: impl FnMut(usize) + 'static) -> Self
pub fn text_value(&self) -> &str
pub fn set_text(&mut self, value: impl Into<String>)
pub fn is_on(&self) -> bool
pub fn set_on(&mut self, on: bool)
pub fn selected_index(&self) -> usize
pub fn option_text(&self) -> &str
pub fn select(&mut self, index: usize) -> bool
```

- Text and secure rows edit borderless (no fill, ring or border,
  see [Textfield.md](Textfield.md)) with the label on the left.
- Toggle rows use the switch style without a label of their own.
- Picker rows wrap a `Menu` dropdown: the button shows the selected
  option with a checkmark on it. Menu `on_action` indices land in
  the row through shared state and apply on the next `mouse_up` or
  draw (one frame at most); programmatic `select` updates button,
  checkmark and fires `on_pick`.
- Button rows center app-built `Button`s (styles and press
  callbacks stay on the buttons).

## FormSection

```rust
pub fn new() -> Self
pub fn titled(title: impl Into<String>) -> Self
pub fn title(self, title: impl Into<String>) -> Self
pub fn row(self, row: FormRow) -> Self
pub fn footnote(self, note: impl Into<String>) -> Self
pub fn row_mut(&mut self, index: usize) -> Option<&mut FormRow>
```

## Form

```rust
pub fn new() -> Self
pub fn section(self, section: FormSection) -> Self
pub fn section_mut(&mut self, index: usize) -> Option<&mut FormSection>
pub fn text_value(&self, section: usize, row: usize) -> &str
pub fn set_text(&mut self, section: usize, row: usize, value: impl Into<String>)
pub fn is_on(&self, section: usize, row: usize) -> bool
pub fn set_on(&mut self, section: usize, row: usize, on: bool)
pub fn selected_index(&self, section: usize, row: usize) -> usize
pub fn select(&mut self, section: usize, row: usize, index: usize) -> bool
pub fn content_height(&self) -> f32
pub fn wants_backdrop(&self) -> bool
pub fn wants_text_cursor(&self) -> bool
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_glass(&mut self, mode: ThemeMode, amount: GlassAmount)
pub fn set_focused(&mut self, focused: bool)
pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32)
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64)
pub fn mouse_move(&mut self, x: f64, y: f64)
pub fn mouse_wheel(&mut self, dx: f64, dy: f64)
pub fn type_text(&mut self, content: &str)
pub fn key(&mut self, key: Key) -> bool
```

- `content_height` needs no fonts (fixed rows): size windows and
  scroll containers with it. `measure` reports the minimum width
  with the full height, so a `ScrollView` pages long forms.
- The app forwards presses, moves, wheel (open picker panels),
  `text`, `key`, `set_hover` and the viewport (picker clamping)
  every frame, plus theme, glass and focus.
- `wants_backdrop` is true while a picker panel is open or a
  switch knob is held; return it from `App::wants_backdrop`.
  `wants_text_cursor` is true while an input hovers (I-beam).

## Usage / Example

Run `cargo run --example form`: basic text form, titled sections
with toggles and a protocol picker, profile plus schedule pickers,
and an agreement toggle with footnote over centered Cancel/Save,
all in a scroll view.

```rust
let mut form = Form::new().section(
    FormSection::titled("Connection")
        .row(FormRow::text("Host", "tontoo.os"))
        .row(FormRow::toggle("Use SSH Key", false)),
);
form.set_theme(accent, true);
```

## Cross References

- [Groupbox.md](Groupbox.md) – group body fill and radius
- [Textfield.md](Textfield.md) – borderless inputs, shortcuts, I-beam
- [Toggle.md](Toggle.md) – switch rows
- [Menu.md](Menu.md) – picker dropdowns, viewport clamping
- [Button.md](Button.md) – button rows
- [Images.md](Images.md) – leading row icons
- [ScrollView.md](ScrollView.md) – paging long forms
- [Layout.md](Layout.md) – `View` protocol
- [Theme.md](Theme.md) – accent, mode and glass stage
