# StoreView

SwiftUI-style StoreView category for TontooUI, recreating App Store collection views and buttons. The category contains four `initializer` elements and two `modifier` elements. Rendering is directly on the window background (#1d1d1d dark / #ececec light) with no extra card, using `SF Pro Display`.

## IconPhaseStoreView

```rust
pub enum StoreIconPhase { Icon, Placeholder, Unavailable }
pub struct IconPhaseStoreView { /* ... */ }
impl IconPhaseStoreView {
    pub fn new() -> Self;
    pub fn phase(self, v: StoreIconPhase) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a view to load a collection of products from the App Store using icon phases.

| Type | Values |
|---|---|
| `StoreIconPhase` | `Icon` `Placeholder` `Unavailable` |

- Preview: `VIP Kitty Pass` row plus `Kitty Hat` row with `? Unavailable` hint and prices.
- Size `180x110`.

## PlaceholderIconStoreView

```rust
pub struct PlaceholderIconStoreView { /* ... */ }
impl PlaceholderIconStoreView {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a view to load a collection of products from the App Store using placeholder icons.

- Preview: three skeleton bars on phone.
- Size `180x110`.

## CustomIconStoreView

```rust
pub struct CustomIconStoreView { /* ... */ }
impl CustomIconStoreView {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a view to load a collection of products from the App Store using custom icons.

- Preview: dropdown header plus two product rows with star icon and prices.
- Size `180x110`.

## StoreViewElement

```rust
pub struct StoreViewElement { /* ... */ }
impl StoreViewElement {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a view to load and merchandise a collection of products from the App Store.

- Preview: two merchandised product rows with prices.
- Size `180x110`.

## StoreCancellationButton

```rust
pub struct StoreCancellationButton { /* ... */ }
impl StoreCancellationButton {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — a type of button that people use to dismiss the current store presentation.

- Preview: product listing with an `X` dismiss circle top-right.
- Size `180x110`.

## RestorePurchasesButton

```rust
pub struct RestorePurchasesButton { /* ... */ }
impl RestorePurchasesButton {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Modifier — a type of button that people use to restore purchases.

- Preview: blue `Restore Missing Purchases` pill `150x30`.
- Size `180x110`.

## Usage / Example

Run the gallery demo:

```bash
cargo run --example store_views
```

Minimal usage:

```rust
use tontooui::prelude::*;

let store = StoreViewElement::new();
let cancel = StoreCancellationButton::new();
let restore = RestorePurchasesButton::new();

let root = VStack::new()
    .spacing(8.0)
    .child(store.to_view())
    .child(restore.to_view());
```

Category folder layout:

```
src/elements/store_views/
  mod.rs                // category root
  icon_phase.rs         // IconPhaseStoreView + StoreIconPhase
  placeholder_icon.rs   // PlaceholderIconStoreView
  custom_icon.rs        // CustomIconStoreView
  store_view.rs         // StoreViewElement
  cancellation_button.rs // StoreCancellationButton
  restore_button.rs     // RestorePurchasesButton
```

## Data and Callbacks

Listings render from `StoreProduct` vectors with gallery defaults; rows are real transparent buttons reporting their index. Selection is controlled state (checkmark follows `selected`). There is no store backend — the hosting app connects its own kit inside the callbacks.

| Element | Data builders | Callbacks |
|---|---|---|
| `IconPhaseStoreView` | `phase`, `products`, `product`, `clear_products`, `selected` | `on_select(index)` |
| `PlaceholderIconStoreView` | `rows` (loading state) | none |
| `CustomIconStoreView` | `products`, `product`, `clear_products`, `selected` | `on_select(index)` |
| `StoreViewElement` | `products`, `product`, `clear_products`, `selected` | `on_select(index)` |
| `StoreCancellationButton` | `products`, `product`, `clear_products` | `on_cancel` |
| `RestorePurchasesButton` | none | `on_restore` |

```rust
use tontooui::prelude::*;

let store = StoreViewElement::new()
    .clear_products()
    .product("VIP Kitty Pass", "Full premium", "$0.99/month")
    .selected(0)
    .on_select(|i| println!("picked row {}", i));
let restore = RestorePurchasesButton::new().on_restore(|| println!("restore"));
```

## Cross References

- [ProductView.md](ProductView.md) -- single-product views; store views list collections of them
- [AsyncImage.md](AsyncImage.md) -- product icons load like async images
