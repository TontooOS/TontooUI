# Toolbar

SwiftUI-style toolbar elements for TontooUI, recreating the iOS 26 / macOS 26
Liquid Glass toolbar from the interface dumps. The family lives in
`src/elements/toolbars/`: a [`Toolbar`](#toolbar) bar container,
[`ToolbarItem`](#toolbaritem), [`ToolbarSpacer`](#toolbarspacer) and the
shared enums `ToolbarItemPlacement` / `ToolbarSpacerSizing`. Consecutive
items with a shared background merge into one glass capsule; spacers separate
or expand between groups. Theme-aware (dark / light).

## ToolbarItemPlacement

```rust
pub enum ToolbarItemPlacement {
    Automatic,
    Principal,
    PrimaryAction,
    AccessoryBar,
}
```

Mirrors `SwiftUI.ToolbarItemPlacement.Role` (automatic, principal,
primaryAction, accessoryBar). `Principal` places the item in the leading
title area of the bar.

## ToolbarSpacerSizing

```rust
pub enum ToolbarSpacerSizing {
    Fixed,
    Flexible,
}
```

Mirrors `SwiftUI.SpacerSizing.Kind` (flexible, fixed). `Fixed` takes up the
standard gap between item groups, `Flexible` expands to fill the remaining
bar width.

## Toolbar

```rust
pub struct Toolbar { /* ... */ }

impl Toolbar {
    pub fn new() -> Self;
    pub fn item(self, item: ToolbarItem) -> Self;
    pub fn spacer(self, spacer: ToolbarSpacer) -> Self;
    pub fn color_scheme(self, c: ColorScheme) -> Self;
    pub fn width(self, width: f32) -> Self;
    pub fn to_view(self) -> View;
}
```

Renders a horizontal bar. Behavior notes:

- Consecutive items sharing the background are wrapped into one glass
  capsule (`.tb-group` CSS class: translucent background, hairline border,
  full rounding).
- A spacer seals the current capsule; `ToolbarSpacer::fixed()` inserts the
  standard 6 px group gap, `ToolbarSpacer::flexible()` inserts an expanding
  gap.
- Items placed with `ToolbarItemPlacement::Principal` render in a leading
  title area before the capsule groups.

## ToolbarItem

```rust
pub struct ToolbarItem { /* ... */ }

impl ToolbarItem {
    pub fn new(icon: impl Into<String>) -> Self;
    pub fn label(self, label: impl Into<String>) -> Self;
    pub fn content(self, widget: impl Widget + 'static) -> Self;
    pub fn placement(self, placement: ToolbarItemPlacement) -> Self;
    pub fn shared_background(self, visible: bool) -> Self;
    pub fn hidden(self, hidden: bool) -> Self;
    pub fn color_scheme(self, c: ColorScheme) -> Self;
    pub fn on_click(self, handler: impl Fn() + Send + Sync + 'static) -> Self;
}
```

A flat glyph button inside the bar. Behavior notes:

- `new(icon)` loads an SF Symbol PNG from the CoreIcon assets (feature
  `coreicon`), recolored to the scheme glyph color (near-white dark /
  near-black light) and cached in the temp dir.
- `shared_background(false)` renders the bare glyph without the glass
  capsule, mirroring `sharedBackgroundVisibility(.hidden)`.
- `hidden(true)` skips rendering entirely, mirroring
  `toolbarItemHidden(_:)` (`SwiftUI.ToolbarItemHiddenModifier`).
- `content(...)` replaces the icon/label with custom widget content, used
  with `ToolbarItemPlacement::Principal` for title-area content.
- Hover/press states tint the item background instead of the glass.

## ToolbarSpacer

```rust
pub struct ToolbarSpacer { /* ... */ }

impl ToolbarSpacer {
    pub fn fixed() -> Self;
    pub fn flexible() -> Self;
    pub fn sizing(&self) -> ToolbarSpacerSizing;
}
```

Mirrors `SwiftUI.ToolbarSpacer { sizing: SpacerSizing }`. A fixed spacer
closes the current glass capsule and inserts the standard group gap; a
flexible spacer expands (`hexpand`) to push following groups to the far end.

## Usage / Example

Run the demo (recreates the SwiftUI toolbar example cards):

```bash
cargo run --example toolbars
```

Minimal usage:

```rust
use tontooui::prelude::*;

let bar = Toolbar::new()
    .item(ToolbarItem::new("chevron.up").on_click(|| println!("up")))
    .item(ToolbarItem::new("chevron.down").on_click(|| println!("down")))
    .spacer(ToolbarSpacer::fixed())
    .item(ToolbarItem::new("ellipsis").on_click(|| println!("menu")));
```

## Cross References

- [Button.md](Button.md) -- SwiftUI-style button element (shares the icon
  pipeline and glass styling approach)
- [Sidebar.md](Sidebar.md) -- sidebar with traffic lights and item list
