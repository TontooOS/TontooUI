# Sheet

SwiftUI-style Sheet category for TontooUI, recreating sheet presentation modifiers. The category contains twelve `modifier` elements that cover placement, dismissal, sizing, backgrounds, drag indicator, detents, and presentation initializers. Rendering is directly on the window background (#1d1d1d dark / #ececec light) with no extra card, using `SF Pro Display`.

## Enums

```rust
pub enum SheetPlacementKind { Sheet, Popover, FullScreen }
pub enum SheetBackgroundInteractionKind { Disabled, Enabled }
pub enum SheetDragIndicator { Visible, Hidden, Automatic }
pub enum SheetDetent { Medium, Large, Fraction(f32), Height(f32) }
```

| Type | Values |
|---|---|
| `SheetPlacementKind` | `Sheet` `Popover` `FullScreen` |
| `SheetBackgroundInteractionKind` | `Disabled` `Enabled` |
| `SheetDragIndicator` | `Visible` `Hidden` `Automatic` |
| `SheetDetent` | `Medium` `Large` `Fraction` `Height` |

## SheetPlacement

```rust
pub struct SheetPlacement { /* ... */ }
impl SheetPlacement {
    pub fn new() -> Self;
    pub fn placement(self, v: SheetPlacementKind) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — sets the placement of a presentation within the presenting view.

- Preview: phone `180x110` with a small sheet docked top-left for `Sheet`, centered pill for `Popover`, near-full for `FullScreen`.
- Size `180x110`.

## DisableSheetDismissSwipe

```rust
pub struct DisableSheetDismissSwipe { /* ... */ }
impl DisableSheetDismissSwipe {
    pub fn new() -> Self;
    pub fn disabled(self, v: bool) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — conditionally prevents interactive dismissal of presentations like popover.

- Preview: bottom sheet `180x62` with `swipe locked` hint.
- Size `180x110`.

## PageScreenSheetSize

```rust
pub struct PageScreenSheetSize { /* ... */ }
impl PageScreenSheetSize {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — on devices smaller than a page of paper, such as iPhone or Apple Watch, page sizing fills the screen.

- Preview: centered page card `150x88` with `Bar`.
- Size `180x110`.

## FittedSheetSizing

```rust
pub struct FittedSheetSizing { /* ... */ }
impl FittedSheetSizing {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — sets the sizing of the containing presentation to fitted content size.

- Preview: tiny centered pill `44x34` with `Bar`.
- Size `180x110`.

## SheetCornerRadius

```rust
pub struct SheetCornerRadius { /* ... */ }
impl SheetCornerRadius {
    pub fn new() -> Self;
    pub fn radius(self, v: f32) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — requests that the presentation have a specific corner radius.

- Preview: bottom sheet `180x72` with `28px` top radius and `Bar`.
- Size `180x110`.

## PrioritizeSheetContentScrolling

```rust
pub struct PrioritizeSheetContentScrolling { /* ... */ }
impl PrioritizeSheetContentScrolling {
    pub fn new() -> Self;
    pub fn prioritize(self, v: bool) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — configure the behavior of swipe gestures on a presentation.

- Preview: sheet `150x92` with scrollable list `Bar 9` through `Bar 14`.
- Size `180x110`.

## SheetBackgroundInteraction

```rust
pub struct SheetBackgroundInteraction { /* ... */ }
impl SheetBackgroundInteraction {
    pub fn new() -> Self;
    pub fn interaction(self, v: SheetBackgroundInteractionKind) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — controls whether people can interact with the view behind a presentation.

- Preview: blue behind-text `You can still tap on me with the sheet open` plus bottom sheet `160x56`.
- Size `180x110`.

## SheetBackground

```rust
pub struct SheetBackground { /* ... */ }
impl SheetBackground {
    pub fn new() -> Self;
    pub fn color(self, c: Color) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — sets the presentation background of the enclosing sheet using a shape style.

- Preview: blue `#0A84FF` sheet `180x78` with `Bar`.
- Size `180x110`.

## SheetDragIndicatorVisibility

```rust
pub struct SheetDragIndicatorVisibility { /* ... */ }
impl SheetDragIndicatorVisibility {
    pub fn new() -> Self;
    pub fn visibility(self, v: SheetDragIndicator) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — sets the visibility of the drag indicator on top of a sheet.

- Preview: sheet `160x84` with `40x5` white handle plus `Bar`; handle hidden for `Hidden`.
- Size `180x110`.

## SheetSize

```rust
pub struct SheetSize { /* ... */ }
impl SheetSize {
    pub fn new() -> Self;
    pub fn detents(self, v: Vec<SheetDetent>) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — sets the available detents for the enclosing sheet.

- Preview: half-height sheet `170x56` with handle and `Bar` (medium detent hint).
- Size `180x110`.

## ItemSheet

```rust
pub struct ItemSheet { /* ... */ }
impl ItemSheet {
    pub fn new() -> Self;
    pub fn item(self, v: Option<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — presents a sheet using the given item as a data source for the sheet content.

- Preview: UUID label on phone plus bottom sheet when `item` is `Some`; no sheet when `None`.
- Size `180x110`.

## BooleanSheet

```rust
pub struct BooleanSheet { /* ... */ }
impl BooleanSheet {
    pub fn new() -> Self;
    pub fn presented(self, v: bool) -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — presents a sheet when a binding to a Boolean value that you provide is true.

- Preview: bottom sheet `160x84` with `Bar` when `presented` is true; phone only when false.
- Size `180x110`.

## Usage / Example

Run the gallery demo (recreates the 12-card screenshot):

```bash
cargo run --example sheets
```

Minimal usage:

```rust
use tontooui::prelude::*;

let sheet = BooleanSheet::new().presented(true);
let item = ItemSheet::new().item(Some("id-123".to_string()));
let detents = SheetSize::new().detents(vec![SheetDetent::Medium, SheetDetent::Large]);
let radius = SheetCornerRadius::new().radius(28.0);

let root = VStack::new()
    .spacing(8.0)
    .child(sheet.to_view())
    .child(radius.to_view());
```

Category folder layout:

```
src/elements/sheets/
  mod.rs                    // category root
  sheet_placement.rs        // SheetPlacement
  disable_dismiss.rs        // DisableSheetDismissSwipe
  page_size.rs              // PageScreenSheetSize
  fitted_sizing.rs          // FittedSheetSizing
  corner_radius.rs          // SheetCornerRadius
  prioritize_scrolling.rs   // PrioritizeSheetContentScrolling
  background_interaction.rs // SheetBackgroundInteraction
  background.rs             // SheetBackground
  drag_indicator.rs         // SheetDragIndicatorVisibility
  size.rs                   // SheetSize
  item_sheet.rs             // ItemSheet
  boolean_sheet.rs          // BooleanSheet
```

## Cross References

- [View.md](View.md) -- `ManageSubscriptionsSheet` is a related sheet-style overlay
- [ScrollView.md](ScrollView.md) -- `PrioritizeSheetContentScrolling` uses scrollable content semantics
- [Material.md](Material.md) -- `SheetBackground` can use material backgrounds
