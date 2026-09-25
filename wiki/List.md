# List

List category in `src/elements/list/`: `BasicList` in `basic.rs` is
a static text list with full-bleed hairlines between rows, like the
reference (Row 1..Row 10, dividers between rows, no divider after
the last row). Items can carry a right-aligned badge (`Inbox` with
`5`) and dim section titles (`Grouped`) group the items below them.
Display-only: no selection, no hover, no mouse handling. The list
spans the whole parent width (intrinsic width is the divider fill
extent) with no edge gap. Row text follows the theme (white/black),
badges and section titles use the dim theme text, and dividers
follow the theme divider color unless the dev sets manual colors.

## Geometry

| Token | Value |
|---|---|
| `LIST_ROW_H` | 28 px row height |
| `LIST_FONT_SIZE` | 13 px row text |
| `LIST_PAD_X` | 12 px horizontal text inset |
| `LIST_DIVIDER_H` | 1 px hairline between rows |
| `LIST_SECTION_GAP` | 20 px extra space above a section (none when leading) |

## ListRow

```rust
pub enum ListRow { Item { text: String, badge: Option<String>, style: ListRowStyle }, Section { title: String, style: ListRowStyle } }
pub fn item(text: impl Into<String>) -> Self
pub fn section(title: impl Into<String>) -> Self
pub fn badge(self, badge: impl Into<String>) -> Self
pub fn style(self, style: ListRowStyle) -> Self
pub fn text_color(self, color: Color) -> Self
pub fn background(self, color: Color) -> Self
pub fn divider(self, show: bool) -> Self
pub fn no_divider(self) -> Self
pub fn set_style(&mut self, style: ListRowStyle)
pub fn row_style(&self) -> ListRowStyle
pub fn text(&self) -> &str
pub fn badge_text(&self) -> Option<&str>
pub fn is_section(&self) -> bool
```

- `item` is a plain row; `badge` adds the right-aligned dim value
  (`Inbox` with `5`, `Trash` with `100`). Badgeless items (`Sent`)
  render text only.
- `section` is a dim title row that groups the items below it
  (`Grouped`). `badge` on a section is a no-op and `badge_text`
  returns `None` there.
- `text_color`, `background` and `divider`/`no_divider` are
  shortcuts for `style` with a single field set; `set_style`
  replaces the whole style in place.

## ListRowStyle

```rust
pub struct ListRowStyle { pub text: Option<Color>, pub background: Option<Color>, pub divider: Option<bool> }
pub fn new() -> Self
pub fn text_color(self, color: Color) -> Self
pub fn background(self, color: Color) -> Self
pub fn divider(self, show: bool) -> Self
```

- Every field is opt-in: a default style renders exactly like an
  unstyled row. `text` wins over the list text/dim colors (item
  text, section title and badge alike).
- `background` is a full-bleed fill behind the row text (`Custom
  Background`); `None` is transparent.
- `divider` overrides the list `show_dividers` for this row only
  (`Some(false)` hides its hairline, `Some(true)` forces one);
  never draws after the last row.

## BasicList

```rust
pub fn new(rows: Vec<String>) -> Self
pub fn from_slice(rows: &[&str]) -> Self
pub fn from_rows(rows: Vec<ListRow>) -> Self
pub fn row_height(self, px: f32) -> Self
pub fn font_size(self, px: f32) -> Self
pub fn padding(self, px: f32) -> Self
pub fn show_dividers(self, show: bool) -> Self
pub fn text_color(self, color: Color) -> Self
pub fn dim_color(self, color: Color) -> Self
pub fn divider_color(self, color: Color) -> Self
pub fn clear_manual_colors(self) -> Self
pub fn set_theme(&mut self, divider: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn set_rows(&mut self, rows: Vec<String>)
pub fn set_rows_slice(&mut self, rows: &[&str])
pub fn set_list_rows(&mut self, rows: Vec<ListRow>)
pub fn push_row(&mut self, row: impl Into<String>)
pub fn push_item(&mut self, text: impl Into<String>)
pub fn push_section(&mut self, title: impl Into<String>)
pub fn clear(&mut self)
pub fn rows(&self) -> &[ListRow]
pub fn row_count(&self) -> usize
pub fn set_row_height(&mut self, px: f32)
pub fn set_row_style(&mut self, index: usize, style: ListRowStyle) -> bool
pub fn row_divider(&self, index: usize) -> bool
pub fn rect(&self) -> (f32, f32, f32, f32)
pub fn content_height(&self) -> f32
```

- `measure` returns the fill width and `content_height`: rows times
  row height, one hairline per shown divider, plus
  `LIST_SECTION_GAP` above every section title except a leading
  one. An empty list is 0 px tall; hidden dividers take no space.
  `row_divider` reports whether row `index` draws its hairline
  (false past the last row); `set_row_style` restyles one row by
  index and returns false when out of bounds.
- `row_height` clamps to >= 0 (zero-height rows draw nothing),
  `font_size` to >= 1 and `padding` to >= 0.
- `text_color`, `dim_color` (badges, section titles) and
  `divider_color` win over `set_theme` until `clear_manual_colors`
  restores theme following.
- Row text is left-aligned at `LIST_PAD_X` and vertically centered
  in its row; badges are right-aligned at the same inset in dim
  text. Dividers span the full placed width; the last row has no
  trailing divider.
- Unfocused windows desaturate text, badges, sections and dividers
  like the rest of the palette. No mouse methods: forward nothing
  (see `examples/list.rs`, titlebar only).

## Usage / Example

```rust
use tontooui::elements::{BasicList, ListRow, View, VStack};

let stack = VStack::new().spacing(0.0).child(
  BasicList::from_slice(&["Row 1", "Row 2", "Row 3"]),
);

let grouped = BasicList::from_rows(vec![
  ListRow::item("Item 1"),
  ListRow::item("Item 2"),
  ListRow::section("Grouped"),
  ListRow::item("Item 3"),
  ListRow::item("Item 4"),
]);

let mailbox = BasicList::from_rows(vec![
  ListRow::item("Inbox").badge("5"),
  ListRow::item("Drafts").badge("12"),
  ListRow::item("Sent"),
  ListRow::item("Trash").badge("100"),
]);

let styled = BasicList::from_rows(vec![
  ListRow::item("Default Row"),
  ListRow::item("Custom Background").background(Color::from_rgba8(
    0x00, 0x7a, 0xff, 38,
  )),
  ListRow::item("Tinted Item")
    .text_color(Color::from_rgb8(0x64, 0xd2, 0xff))
    .no_divider(),
]);
```

Wire the live theme per frame (see `examples/list.rs`):

```rust
list.set_theme(palette.divider, dark);
list.set_focused(focused);
```

## DisclosureGroup

`DisclosureGroup` in `disclosure.rs` is an expandable group: a
header row (chevron plus title) with an indented `BasicList` below
it, like the reference (Fruits open with Apple/Banana/Cherry/Date,
Vegetables and Grains closed). Only a press plus release on the
chevron box toggles the group; clicks on the title or children do
nothing.

| Token | Value |
|---|---|
| `DISCLOSURE_ANIM_SECONDS` | 0.25 s expand/collapse tween (`CubicOut`) |
| `DISCLOSURE_INDENT` | 20 px default children indent |
| `DISCLOSURE_CHEV_W` / `DISCLOSURE_CHEV_H` | 7 px / 10 px chevron glyph |
| `DISCLOSURE_CHEV_STROKE` | 1.8 px chevron stroke |
| `DISCLOSURE_CHEV_PAD` / `DISCLOSURE_CHEV_GAP` | 6 px pad before / 8 px gap after the chevron |
| `DISCLOSURE_HIT_W` | 28 px generous chevron hit width |

```rust
pub fn new(title: impl Into<String>, children: Vec<ListRow>) -> Self
pub fn from_slice(title: impl Into<String>, children: &[&str]) -> Self
pub fn open(self, open: bool) -> Self
pub fn indent(self, px: f32) -> Self
pub fn title_color(self, color: Color) -> Self
pub fn chevron_color(self, color: Color) -> Self
pub fn disabled(self, disabled: bool) -> Self
pub fn on_toggle(self, callback: impl FnMut(bool) + 'static) -> Self
pub fn set_theme(&mut self, divider: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
pub fn set_title(&mut self, title: impl Into<String>)
pub fn title(&self) -> &str
pub fn set_children(&mut self, children: Vec<ListRow>)
pub fn children(&self) -> &[ListRow]
pub fn push_child(&mut self, row: ListRow)
pub fn list_mut(&mut self) -> &mut BasicList
pub fn is_open(&self) -> bool
pub fn progress(&self) -> f32
pub fn set_open(&mut self, open: bool)
pub fn toggle(&mut self)
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_move(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64)
```

- Toggling animates: the children height tweens between 0 and full
  while the chevron morphs from `>` to `v`, and rows below glide
  down because `measure` follows the animated progress. Children
  draw into a clip layer, so partially revealed rows never spill
  outside the animated bounds. A full-bleed hairline sits below the
  header and another closes the group after the children.
- `toggle` flips with animation and fires `on_toggle` with the new
  state; programmatic `set_open` animates without firing.
  `progress` reads the animated state (0.0 closed, 1.0 open).
- Child rows keep the full `BasicList` feature set (badges,
  sections, row styles) via `list_mut`; `set_theme` and
  `set_focused` forward divider, mode and focus to the children.
- Apps forward mouse events to the group (see
  `examples/list.rs`); `View::mouse_up` forwards to the same
  toggle path. Disabled groups ignore all input.

## Usage / Example

```rust
use tontooui::elements::{DisclosureGroup, View, VStack};

let stack = VStack::new().spacing(0.0)
  .child(
    DisclosureGroup::from_slice("Fruits", &["Apple", "Banana"])
      .open(true),
  )
  .child(DisclosureGroup::from_slice("Vegetables", &["Carrot"]));
```

Wire theme and input per frame (see `examples/list.rs`):

```rust
group.set_theme(palette.divider, dark);
group.set_focused(focused);
// in mouse handlers:
group.mouse_down(x, y);
group.mouse_up(x, y);
```

## Cross References

- [Divider.md](Divider.md) – shared fill extent and theme divider colors
- [Layout.md](Layout.md) – stacks hand the list the full parent size
- [Theme.md](Theme.md) – theme divider color and unfocused desaturation
- [Animation.md](Animation.md) – tween driver behind the group reveal
