# BarSwitcher

Single-select toolbar in `src/elements/barswitcher.rs`: the
`BasicToolbar` capsule in the clear (`Lens`) glass finish, but each
cell is a selectable option holding an SF Symbol icon, a text label,
or both. The selected cell keeps the gray hover-style highlight;
clicking another cell moves the selection there and fires `on_select`
with the new index so the app can swap views.

> **Note:** like the toolbar the finish stays `GlassType::Lens` and
> never switches to `Frosted`.

## Tokens

| Token | Value |
|---|---|
| `BARSWITCHER_FONT_SIZE` | `13.0` label size |
| `BARSWITCHER_ICON_TEXT_GAP` | `6.0` gap between icon and label |
| `BARSWITCHER_TEXT_PAD_X` | `10.0` side padding of labeled cells |
| `BARSWITCHER_MIN_W` | `44.0` minimum width of labeled cells |
| `BARSWITCHER_SELECTED_WEIGHT` | `600.0` semibold selected label |
| `BARSWITCHER_WEIGHT` | `400.0` regular label |

```rust
pub const BARSWITCHER_FONT_SIZE: f32;      // 13.0
pub const BARSWITCHER_ICON_TEXT_GAP: f32;  // 6.0
pub const BARSWITCHER_TEXT_PAD_X: f32;     // 10.0
pub const BARSWITCHER_MIN_W: f32;          // 44.0
```

Geometry (height, capsule radius, icon box, padding, gaps,
icon-only cell) is shared with the toolbar (`TOOLBAR_HEIGHT`,
`TOOLBAR_RADIUS`, `TOOLBAR_ICON_SIZE`, `TOOLBAR_PAD_X`,
`TOOLBAR_GAP`, `TOOLBAR_HIT`; see [Toolbar.md](Toolbar.md)).

## Items

```rust
pub struct BarSwitcherItem {
    pub icon: Option<String>,
    pub label: Option<String>,
}
```

```rust
pub fn icon(name: impl Into<String>) -> Self
pub fn text(label: impl Into<String>) -> Self
pub fn both(name: impl Into<String>, label: impl Into<String>) -> Self
```

`icon` is an icon-only cell, `text` a label-only cell, `both` draws
the icon left and the label right, centered as one group. Fully
empty items take no space and never hit.

```rust
pub fn from_items(items: Vec<BarSwitcherItem>) -> Self
pub fn items(self, items: Vec<BarSwitcherItem>) -> Self
pub fn item(self, item: BarSwitcherItem) -> Self
pub fn set_items(&mut self, items: Vec<BarSwitcherItem>)
```

## Selection

```rust
pub fn selected(self, index: usize) -> Self
pub fn select(&mut self, index: usize)
pub fn set_selected(&mut self, index: usize)
pub fn selected_index(&self) -> usize
pub fn selected_item(&self) -> Option<&BarSwitcherItem>
pub fn len(&self) -> usize
pub fn is_empty(&self) -> bool
```

- `selected` sets the initial selection without firing `on_select`.
- `select` / `set_selected` apply instantly (no animation) and fire
  `on_select` only when the selection changed. Out-of-range indices
  clamp to the last item; empty switchers stay at `0`.
- Clicking the already-selected cell fires nothing.

```rust
pub fn on_select(self, callback: impl FnMut(usize) + 'static) -> Self
pub fn set_on_select(&mut self, callback: impl FnMut(usize) + 'static)
```

## States

The selected cell keeps the gray hover-style highlight (white
overlay in dark mode, black overlay in light mode); hovering other
cells shows the same tint and pressing deepens it, exactly like
`BasicToolbar`. The selected label draws semibold, the rest regular.

```rust
pub fn press(&mut self, x: f32, y: f32) -> Option<usize>
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64)
pub fn mouse_move(&mut self, x: f32, y: f32)
pub fn placement(self, placement: ToolbarPlacement) -> Self
pub fn set_placement(&mut self, placement: ToolbarPlacement)
```

- `press` selects the hit cell immediately (firing on change) and
  returns its index.
- `mouse_down` arms the cell, `mouse_up` selects only when released
  on the same cell.
- `Returns None when` no cell was hit or the switcher is disabled.
- `placement` aligns the cells inside the rect (`Leading`, `Center`,
  `Trailing`; default is `Center`).

## Theme

```rust
pub fn set_theme(&mut self, mode: ThemeMode, amount: GlassAmount)
pub fn set_focused(&mut self, focused: bool)
pub fn disabled(self, disabled: bool) -> Self
pub fn set_disabled(&mut self, disabled: bool)
```

`set_theme` forwards the glass amount to the `Lens` body and picks
the icon/label color plus the tint direction. Unfocused windows
desaturate the color; disabled switchers dim to 40% opacity, drop
the tint and never select.

## Usage / Example

```rust
use tontooui::elements::{BarSwitcher, BarSwitcherItem, View};

let mut switcher = BarSwitcher::from_items(vec![
    BarSwitcherItem::both("square.grid.2x2", "Grid"),
    BarSwitcherItem::icon("list.bullet"),
    BarSwitcherItem::text("Settings"),
])
.selected(0)
.on_select(|index| println!("show view {index}"));
switcher.set_theme(tontooui::theme::ThemeMode::Dark, tontooui::theme::GlassAmount::Glass);
```

Poll `selected_index()` / `selected_item()` per frame to swap views
(see `examples/barswitcher.rs`). Apps using glass must return `true`
from `App::wants_backdrop` so the shell runs the blur pass.

## Cross References

- [Toolbar.md](Toolbar.md) – capsule look, `Lens` body, tint reference
- [Picker.md](Picker.md) – `SegmentedPicker` select-event reference
- [Glass.md](Glass.md) – `Lens` body, backdrop pass
- [Layout.md](Layout.md) – `View` protocol (`measure`, `place`, `draw`)
