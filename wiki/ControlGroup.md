# ControlGroup

SwiftUI-style ControlGroup category for TontooUI, recreating the `ControlGroup` family from the macOS 26 interface. The category contains one initializer (`ControlGroup`) and four styles (`PaletteControlGroupStyle`, `NavigationControlGroupStyle`, `MenuControlGroupStyle`, `CompactMenuControlGroupStyle`). All elements render directly on the window background (#1d1d1d dark / #ececec light) with no extra card, using `SF Pro Display`.

## ControlGroupStyle

```rust
pub enum ControlGroupStyle {
    Automatic,
    Palette,
    Navigation,
    Menu,
    CompactMenu,
}
```

Mirrors `SwiftUI.ControlGroupStyle`. `Automatic` is the default (`ControlGroup` without an explicit style). Each variant maps to one of the four style cards.

| Variant | Label |
|---|---|
| `Automatic` | `automatic` |
| `Palette` | `palette` |
| `Navigation` | `navigation` |
| `Menu` | `menu` |
| `CompactMenu` | `compactMenu` |

## ControlGroup

```rust
pub struct ControlGroup { /* ... */ }

impl ControlGroup {
    pub fn new() -> Self;
    pub fn child(self, label: impl Into<String>) -> Self;
    pub fn children(self, labels: Vec<String>) -> Self;
    pub fn style(self, style: ControlGroupStyle) -> Self;
    pub fn control_group_style(self, style: ControlGroupStyle) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a new ControlGroup with the specified children.

- Default children are `["Increase", "Decrease"]`.
- `child` / `children` replace or extend the child list.
- `style` / `control_group_style` select the presentation (alias).
- `to_view` size varies by style: `220×36` automatic, `180×74` palette, `180×64` navigation, `190×54` menu, `160×52` compact.

Rendering directly on window (no extra card):

- **Automatic:** single horizontal pill (`190×28`, `999px` radius, `2px` padding, `1px` hairline) with `Increase | Decrease` and a vertical separator. Background `#2c2c2e` dark / `#ffffff` light.
- **Palette:** vertical palette `140×44` (`12px` radius) with rows `− Decrease` / `+ Increase` (`10px` SF Pro) and a centered search row `⌕ Foo` (`9px`). Directly on window, frosted `rgba(44,44,46,0.6)` dark.
- **Navigation:** `3×` rows of `+ Increase  − Decrease` (`9px` dim) stacked vertically.
- **Menu:** horizontal pill `150×28` with `Increase` `Decrease` plus centered `⌕ Foo` below.
- **CompactMenu:** compact pill `100×28` with `+` `−` (`11px` Semibold) plus `⌕ Foo` below.

`is_interactive() == true`.

## PaletteControlGroupStyle

```rust
pub struct PaletteControlGroupStyle { /* ... */ }

impl PaletteControlGroupStyle {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Style — a control group style that presents its content as a palette. `ViewContent` renders a `ControlGroup::new().style(Palette)` directly on window (`180×74`).

## NavigationControlGroupStyle

```rust
pub struct NavigationControlGroupStyle { /* ... */ }

impl NavigationControlGroupStyle {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Style — the navigation control group style. Renders `ControlGroup::new().style(Navigation)` directly on window (`180×64`).

## MenuControlGroupStyle

```rust
pub struct MenuControlGroupStyle { /* ... */ }

impl MenuControlGroupStyle {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Style — a control group style that presents its content as a menu when the user interacts. Renders `ControlGroup::new().style(Menu)` directly on window (`190×54`).

## CompactMenuControlGroupStyle

```rust
pub struct CompactMenuControlGroupStyle { /* ... */ }

impl CompactMenuControlGroupStyle {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Style — a control group style that presents its content as a compact menu when the user interacts. Renders `ControlGroup::new().style(CompactMenu)` directly on window (`160×52`).

## Usage / Example

Run the gallery demo (recreates the 5-card screenshot):

```bash
cargo run --example control_groups
```

Minimal usage:

```rust
use tontooui::prelude::*;

// Initializer
let group = ControlGroup::new()
    .child("Increase")
    .child("Decrease");

// Style via enum
let palette = ControlGroup::new().style(ControlGroupStyle::Palette);

// Style via dedicated type (preview widgets)
let nav_preview = NavigationControlGroupStyle::new().to_view();
let menu_preview = MenuControlGroupStyle::new().to_view();

let root = VStack::new()
    .spacing(8.0)
    .child(group.to_view())
    .child(palette.to_view());
```

Category folder layout:

```
src/elements/control_groups/
  mod.rs              // category root
  control_group.rs    // ControlGroup + ControlGroupStyle (initializer)
  palette.rs          // PaletteControlGroupStyle
  navigation.rs       // NavigationControlGroupStyle
  menu.rs             // MenuControlGroupStyle
  compact_menu.rs     // CompactMenuControlGroupStyle
```

## Cross References

- [Button.md](Button.md) -- `ControlGroup` groups `Button` children
- [Toolbar.md](Toolbar.md) -- toolbar also groups controls
- [Menu.md](Menu.md) -- menu presentation style
