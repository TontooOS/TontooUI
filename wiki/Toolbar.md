# Toolbar

Toolbar category in `src/elements/toolbar/`: `BasicToolbar` in
`basic.rs` is a small capsule toolbar in the clear (`Lens`) glass
finish holding icon buttons with optional vertical dividers. Icons
come from CoreIcon (`COREICON_ASSETS_DIR` override or the system
resources on TontooOS); a missing icon draws an empty cell that still
fires its action.

> **Note:** the toolbar never uses the `Frosted` finish. The body is
> always `GlassType::Lens` (clear minified center, blur only on the
> rim).

## Tokens

| Token | Value |
|---|---|
| `TOOLBAR_HEIGHT` | `36.0` |
| `TOOLBAR_RADIUS` | `18.0`, capsule |
| `TOOLBAR_ICON_SIZE` | `18.0` box, aspect kept |
| `TOOLBAR_PAD_X` | `8.0` |
| `TOOLBAR_GAP` | `4.0` between cells |
| `TOOLBAR_HIT` | `28.0` square cell |
| `TOOLBAR_DIVIDER_W` | `9.0` divider cell |
| `TOOLBAR_DIVIDER_H` | `20.0` divider line |

```rust
pub const TOOLBAR_HEIGHT: f32;    // 36.0, kept small on purpose
pub const TOOLBAR_RADIUS: f32;    // 18.0
pub const TOOLBAR_ICON_SIZE: f32; // 18.0
pub const TOOLBAR_PAD_X: f32;     // 8.0
pub const TOOLBAR_GAP: f32;       // 4.0
pub const TOOLBAR_HIT: f32;       // 28.0
pub const TOOLBAR_DIVIDER_W: f32; // 9.0
pub const TOOLBAR_DIVIDER_H: f32; // 20.0
```

## Items

```rust
pub enum ToolbarItem {
    Icon(String),
    Divider,
}
```

`Icon` is an SF Symbol button, `Divider` is a thin vertical line
between icons. Dividers are display-only: they never hover, never
press and never fire `on_action` (which reports the item index, so
icons after a divider keep their position index).

```rust
pub fn from_items(items: Vec<ToolbarItem>) -> Self
pub fn items(self, items: Vec<ToolbarItem>) -> Self
pub fn item(self, item: ToolbarItem) -> Self
pub fn divider(self) -> Self
pub fn set_items(&mut self, items: Vec<ToolbarItem>)
```

The `from_icons` / `icons` / `icon` / `set_icons` helpers build
icon-only toolbars.

## Placement

```rust
pub enum ToolbarPlacement {
    Leading,
    Center,
    Trailing,
}
```

Placement aligns the icon cells inside the toolbar rect. `Leading`
starts at the leading edge, `Center` centers them, `Trailing` ends at
the trailing edge.

```rust
pub fn placement(self, placement: ToolbarPlacement) -> Self
pub fn set_placement(&mut self, placement: ToolbarPlacement)
```

## States

Hover tints the icon cell; pressing deepens the same tint. Dark mode
lightens (white overlay), light mode darkens (black overlay). Each
icon is a normal button action firing `on_action` with its index.

```rust
pub fn on_action(self, callback: impl FnMut(usize) + 'static) -> Self
pub fn press(&mut self, x: f32, y: f32) -> Option<usize>
pub fn mouse_down(&mut self, x: f64, y: f64)
pub fn mouse_up(&mut self, x: f64, y: f64)
pub fn mouse_move(&mut self, x: f32, y: f32)
```

- `press` fires the callback immediately and returns the hit index.
- `mouse_down` arms the cell, `mouse_up` fires only when released on
  the same cell.
- `Returns None when` no cell was hit or the toolbar is disabled.

## Theme

```rust
pub fn set_theme(&mut self, mode: ThemeMode, amount: GlassAmount)
pub fn set_focused(&mut self, focused: bool)
```

`set_theme` forwards the glass amount to the `Lens` body and picks
the icon color plus the hover/press tint direction. The finish stays
`Lens` and never switches to `Frosted`. Unfocused windows desaturate
the icon color like the rest of the palette.

## Usage / Example

```rust
use tontooui::elements::{BasicToolbar, ToolbarItem, ToolbarPlacement, View};

let mut bar = BasicToolbar::from_items(vec![
    ToolbarItem::icon("chevron.left"),
    ToolbarItem::divider(),
    ToolbarItem::icon("chevron.right"),
])
.placement(ToolbarPlacement::Leading)
.on_action(|index| println!("toolbar tap {index}"));
bar.set_theme(tontooui::theme::ThemeMode::Dark, tontooui::theme::GlassAmount::Glass);
```

Apps using glass must return `true` from `App::wants_backdrop` so the
shell runs the blur pass (see `examples/toolbar.rs`).

## Cross References

- [Glass.md](Glass.md) – `Lens` body, backdrop pass
- [Button.md](Button.md) – hover/press tint reference
- [Layout.md](Layout.md) – `View` protocol (`measure`, `place`, `draw`)
