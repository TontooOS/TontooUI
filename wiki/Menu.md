# Menu

Menu is a SwiftUI-style menu with Liquid Glass (transparent, frosted) and Light/Dark support. It opens on click (Menu) or secondary gesture (ContextMenu). Supports items, dividers, sections and nested submenus, with SF Pro typography and adaptive colors (Dark `#1d1d1d` / Light `#ececec`).

## Constructor

```rust
pub fn new(label: impl Into<String>) -> Self
```

Creates a menu with a trigger button label.

```rust
pub struct ContextMenu { child: Box<dyn Widget>, entries: Vec<MenuEntry>, preview: Option<Box<dyn Widget>> }
pub fn new(child: impl Widget + 'static) -> Self
```

Wraps a child and shows the menu on right-click (secondary gesture).

## Types

```rust
pub enum MenuRole { Default, Destructive }
pub struct MenuItem { label: String, description: Option<String>, icon: Option<String>, trailing_icon: Option<String>, role: MenuRole }
pub enum MenuEntry { Item(MenuItem), Divider, Section { title: Option<String>, items: Vec<MenuEntry> }, Submenu { title: String, items: Vec<MenuEntry> }, TagDots { title: String, colors: Vec<(u8, u8, u8)> } }
```

## Builder Methods — Menu

| Method | Signature | Description |
|---|---|---|
| `item` | `item(self, label: impl Into<String>) -> Self` | Add plain item |
| `item_with_desc` | `item_with_desc(self, label: impl Into<String>, desc: impl Into<String>) -> Self` | Item with secondary description |
| `item_with_icon` | `item_with_icon(self, label: impl Into<String>, icon: impl Into<String>) -> Self` | Item with SF Symbol icon |
| `item_full` | `item_full(self, item: MenuItem) -> Self` | Add fully configured `MenuItem` (role, handler) |
| `divider` | `divider(self) -> Self` | Horizontal divider (8% opacity) |
| `section` | `section(self, title: impl Into<String>, f: FnOnce(Vec<MenuEntry>)->Vec<MenuEntry>) -> Self` | Section with header |
| `section_items` | `section_items(self, title: Option<String>, items: Vec<MenuEntry>) -> Self` | Section with explicit items |
| `submenu` | `submenu(self, title: impl Into<String>, items: Vec<MenuEntry>) -> Self` | Nested submenu |
| `entry` | `entry(self, e: MenuEntry) -> Self` | Push raw entry |

## Builder Methods — MenuItem

| Method | Signature | Description |
|---|---|---|
| `new` | `new(label: impl Into<String>) -> Self` | Create item |
| `description` | `description(self, d: impl Into<String>) -> Self` | Secondary text |
| `icon` | `icon(self, name: impl Into<String>) -> Self` | SF Symbol (CoreIcon `assets/icons/name.png`) |
| `trailing_icon` | `trailing_icon(self, name: impl Into<String>) -> Self` | SF Symbol at the trailing (right) edge, after the label |
| `destructive` | `destructive(self) -> Self` | Red role `#ff3b30` |
| `on_activate` | `on_activate(self, f: Fn() + Send + Sync + 'static) -> Self` | Click handler, auto-closes popover |

## Builder Methods — ContextMenu

| Method | Signature | Description |
|---|---|---|
| `item` | `item(self, label: impl Into<String>) -> Self` | Add item |
| `entry` | `entry(self, e: MenuEntry) -> Self` | Raw entry |
| `divider` | `divider(self) -> Self` | Divider |
| `preview` | `preview(self, w: impl Widget + 'static) -> Self` | Custom preview widget above menu (shows "Custom Preview" pill) |
| `entries` | `entries(self, v: Vec<MenuEntry>) -> Self` | Replace entries |

## Behavior

- Glass: `rgba(38,38,40,0.84)` + `1px solid rgba(255,255,255,0.12)` Dark, `rgba(248,248,250,0.92)` + `rgba(0,0,0,0.08)` Light, `12px` radius, `0 12px 40px rgba(0,0,0,0.35)` shadow, SF Pro. Outer `popover` is fully transparent (`background: transparent; border: none;` via `STYLE_PROVIDER_PRIORITY_USER`) — only inner box has glass, no extra white rim.
- Popover is a separate `GdkSurface` popup window (own window in background) — made transparent via display-level CSS provider, `has_arrow(false)`, `autohide(true)`.
- `ContextMenu` uses `GestureClick button 3` (right-click) with `set_pointing_to` at cursor; preview variant prepends pill + separator.
- Icons are recolored via CoreIcon alpha-mask tint (zinc-400/500) to theme gray.
- `TagDots` renders a non-interactive tag row (title plus colored CSS dots, e.g. Finder Tags) with no hover action.
- No manual Light/Dark toggle — follows `resolve_scheme` / `ColorScheme::detect_system`.

## Usage / Example

```rust
use tontooui::prelude::*;
use tontooui::{Menu, MenuItem, MenuEntry, ContextMenu};

// Menu with divider, section and nested submenu
let menu = Menu::new("Menu")
    .entry(MenuEntry::Item(MenuItem::new("Button 3").description("Description 3")))
    .entry(MenuEntry::Item(MenuItem::new("Button 2").icon("pencil")))
    .entry(MenuEntry::Divider)
    .section_items(Some("foo".into()), vec![
        MenuEntry::Item(MenuItem::new("Button 1")),
    ])
    .submenu("Other", vec![MenuEntry::Item(MenuItem::new("Button 1"))]);

// Context menu with custom preview
let ctx = ContextMenu::new(Text::new("Right-click me"))
    .preview(Text::new("Preview"))
    .item("Button 1")
    .divider();
```

See `examples/menus.rs` for the 1:1 demo (8 variants: Menu Action Button, Menu, Context Menu Preview, Context Menu, Fixed Order, Nested, Divider, Section) directly on background, pure TontooUI API, Ampeln visible.

## Cross References

- [MAIN.md](MAIN.md) -- library overview
- [Divider.md](Divider.md) -- separator line used as menu divider
- [Button.md](Button.md) -- trigger button styling
