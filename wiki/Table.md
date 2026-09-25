# Table

Basic table in `src/elements/tables/table.rs`: fixed header with
click-to-sort, scrollable row pills, optional Ctrl/Shift multi-select
and inline cell editing. The body always fills the placed rect:
columns stretch by weight, leftover height draws empty outline pills,
and overflow scrolls (rows through the integrated `Scrollbar`, wide
columns through a slim horizontal bar).

## Geometry

| Token | Value |
|---|---|
| `TABLE_HEADER_H` | 30 px fixed header |
| `TABLE_ROW_H` / `TABLE_ROW_GAP` | 34 px pills with 6 px gaps |
| `TABLE_RADIUS` | 8 px pill radius |
| `TABLE_FONT_SIZE` / `TABLE_HEADER_SIZE` | 13 px cells / 12 px semibold titles |
| `TABLE_PAD_X` / `TABLE_COL_GAP` | 12 px text inset / 16 px column gap |
| `TABLE_BAR_W` / `TABLE_HBAR_H` | 10 px overlay bars |
| `TABLE_SELECTED_ALPHA` | 0.35 accent fill for selected rows |
| `TABLE_ROW_FILL_DARK` / `TABLE_ROW_FILL_LIGHT` | white 10 alpha / black 10 alpha |
| `TABLE_DIVIDER_DARK` / `TABLE_DIVIDER_LIGHT` | white 36 alpha / black 31 alpha |

## Columns

```rust
pub fn new(title: impl Into<String>) -> TableColumn
pub fn weight(self, weight: f32) -> Self
pub fn min_width(self, px: f32) -> Self
pub fn sortable(self, sortable: bool) -> Self
pub fn editable(self, editable: bool) -> Self
pub fn sort_by(self, cmp: impl Fn(&str, &str) -> Ordering + 'static) -> Self
```

- `weight` (default `1.0`) shares the table width; `min_width`
  (default `80.0`) forces horizontal scrolling past the table width.
- `sortable` (default `true`) allows header-click sorting;
  `editable` (default `false`) allows inline editing.
- `sort_by` replaces the smart sort for that column (numbers
  numeric, otherwise case-insensitive text). Returning
  `Ordering::Equal` keeps insertion order (stable sort).

## Element

```rust
pub fn new(columns: Vec<TableColumn>, rows: Vec<Vec<String>>) -> Self
pub fn selectable(self, selectable: bool) -> Self
pub fn edit_on_double_click(self, enabled: bool) -> Self
pub fn on_sort(self, callback: impl FnMut(usize, bool) + 'static) -> Self
pub fn on_select(self, callback: impl FnMut(Vec<usize>) + 'static) -> Self
pub fn on_edit_request(self, callback: impl FnMut(usize, usize) + 'static) -> Self
pub fn on_edit_commit(self, callback: impl FnMut(usize, usize, String) + 'static) -> Self
pub fn set_rows(&mut self, rows: Vec<Vec<String>>)
pub fn set_cell(&mut self, row: usize, col: usize, value: impl Into<String>)
pub fn cell_value(&self, row: usize, col: usize) -> &str
pub fn selected_rows(&self) -> Vec<usize>
pub fn select_rows(&mut self, rows: &[usize])
pub fn clear_selection(&mut self)
pub fn sort_by_column(&mut self, col: usize, ascending: bool) -> bool
pub fn clear_sort(&mut self)
pub fn sort_state(&self) -> Option<(usize, bool)>
pub fn begin_edit(&mut self, row: usize, col: usize) -> bool
pub fn commit_edit(&mut self) -> bool
pub fn cancel_edit(&mut self) -> bool
pub fn is_editing(&self) -> bool
pub fn type_text(&mut self, content: &str)
pub fn key(&mut self, key: Key) -> bool
pub fn set_modifiers(&mut self, ctrl: bool, shift: bool)
pub fn cell_at(&self, x: f64, y: f64) -> Option<TableHit>
pub fn is_cell_editable(&self, row: usize, col: usize) -> bool
pub fn set_theme(&mut self, accent: Color, dark: bool)
pub fn set_focused(&mut self, focused: bool)
```

- Row indices are display positions (post-sort). Selection tracks
  content across sorts; `set_rows` clears selection and reapplies the
  sort silently. Ragged rows pad with `""` or truncate.
- Header click sorts ascending first, then toggles; the sorted
  column shows an accent chevron. `sort_by_column` fires `on_sort`,
  `clear_sort` restores insertion order.
- Selection needs `selectable(true)`: click selects one row,
  Ctrl-click toggles, Shift-click takes the range from the anchor.
  Clicking empty filler clears. `on_select` fires with the sorted
  selected positions.
- Editing needs an editable column: double-click (when
  `edit_on_double_click`, default `true`) or manual `begin_edit`
  opens an inline `BasicTextField` over the cell. Enter commits
  (`on_edit_commit`), Escape cancels, clicking outside commits.

## Hit Testing

```rust
pub enum TableHit {
    Header(usize),
    Cell(usize, usize),
}
```

`cell_at` maps a point to a header or cell for context menus:
right-click, map with `cell_at`, open the menu, call `begin_edit`
from the action (see `examples/table.rs`).

## Shell Wiring

The app forwards `mouse_down`, `mouse_up`, `set_hover`,
`mouse_wheel`, `text`, `key` and `set_modifiers` (Ctrl/Shift from
`App::set_modifiers`, see [Renderer.md](Renderer.md)) into the table.
`flex` is `1.0` so stacks hand the table the remaining space.

## Usage / Example

Run `cargo run --example table`: sortable Name/Age/City/Email table
(City uses a custom length comparator), Ctrl/Shift multi-select with
the count in the titlebar, double-click or right-click `Edit Cell`
for inline editing, vertical bar plus horizontal bar on narrow
windows.

```rust
let mut table = BasicTable::new(
    vec![
        TableColumn::new("Name").weight(1.4).editable(true),
        TableColumn::new("Age").weight(0.6).min_width(60.0),
    ],
    vec![vec!["Alice".into(), "28".into()]],
).selectable(true);
table.set_theme(accent, true);
```

## Cross References

- [Layout.md](Layout.md) – stacks hosting tables, `View` trait, `flex`
- [Renderer.md](Renderer.md) – `App::set_modifiers`, wheel and key forwarding
- [Scrollbar.md](Scrollbar.md) – integrated vertical bar behavior
- [ScrollView.md](ScrollView.md) – clipping and bar sync pattern
- [Menu.md](Menu.md) – context menu wiring for manual edit actions
- [Textfield.md](Textfield.md) – inline editor field behavior
- [Theme.md](Theme.md) – accent selection tint and mode grays
