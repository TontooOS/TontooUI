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
pub enum ListRow { Item { text: String, badge: Option<String> }, Section(String) }
pub fn item(text: impl Into<String>) -> Self
pub fn section(title: impl Into<String>) -> Self
pub fn badge(self, badge: impl Into<String>) -> Self
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
pub fn rect(&self) -> (f32, f32, f32, f32)
pub fn content_height(&self) -> f32
```

- `measure` returns the fill width and `content_height`: rows times
  row height, one hairline per inner gap, plus `LIST_SECTION_GAP`
  above every section title except a leading one. An empty list is
  0 px tall; hidden dividers take no space.
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
```

Wire the live theme per frame (see `examples/list.rs`):

```rust
list.set_theme(palette.divider, dark);
list.set_focused(focused);
```

## Cross References

- [Divider.md](Divider.md) – shared fill extent and theme divider colors
- [Layout.md](Layout.md) – stacks hand the list the full parent size
- [Theme.md](Theme.md) – theme divider color and unfocused desaturation
