# SubscriptionStoreView

SwiftUI-style SubscriptionStoreView category for TontooUI, recreating subscription store views. The category contains six `initializer` elements. Rendering is directly on the window background (#1d1d1d dark / #ececec light) with no extra card, using `SF Pro Display`.

## CustomGroupSubscriptionStoreView

```rust
pub struct CustomGroupSubscriptionStoreView { /* ... */ }
impl CustomGroupSubscriptionStoreView {
    pub fn new() -> Self;
    pub fn group_id(self, v: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a SubscriptionStoreView with custom grouping.

- Preview: Monthly/Yearly tabs plus selected `VIP Kitty Pass` card and blue `Subscribe` button.
- Size `180x110`.

## CustomHeaderSubscriptionStoreView

```rust
pub struct CustomHeaderSubscriptionStoreView { /* ... */ }
impl CustomHeaderSubscriptionStoreView {
    pub fn new() -> Self;
    pub fn group_id(self, v: impl Into<String>) -> Self;
    pub fn header(self, v: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a view that loads all the subscriptions in a subscription group with a custom header.

- Preview: custom header plus one option card.
- Size `180x110`.

## UpgradeOnlySubscriptionStoreView

```rust
pub struct UpgradeOnlySubscriptionStoreView { /* ... */ }
impl UpgradeOnlySubscriptionStoreView {
    pub fn new() -> Self;
    pub fn group_id(self, v: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a view that loads all subscriptions from a subscription group, upgrade options only.

- Preview: app icon grid plus `Kitty Pass` hero title.
- Size `180x110`.

## GroupSubscriptionStoreView

```rust
pub struct GroupSubscriptionStoreView { /* ... */ }
impl GroupSubscriptionStoreView {
    pub fn new() -> Self;
    pub fn group_id(self, v: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a view that loads all subscriptions in a subscription group.

- Preview: selected monthly card plus unselected yearly card.
- Size `180x110`.

## SingleSubscriptionStoreView

```rust
pub struct SingleSubscriptionStoreView { /* ... */ }
impl SingleSubscriptionStoreView {
    pub fn new() -> Self;
    pub fn product_id(self, v: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a view that loads subscriptions based on a collection of products (single).

- Preview: app icon grid plus `Kitty Pass` hero title.
- Size `180x110`.

## SubscriptionStoreViewElement

```rust
pub struct SubscriptionStoreViewElement { /* ... */ }
impl SubscriptionStoreViewElement {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a view that loads subscriptions based on a collection of products.

- Preview: two stacked option cards with selection dots plus renewal note.
- Size `180x110`.

## Usage / Example

Run the gallery demo:

```bash
cargo run --example subscription_store_views
```

Minimal usage:

```rust
use tontooui::prelude::*;

let group = GroupSubscriptionStoreView::new().group_id("kitty.pass.group");
let single = SingleSubscriptionStoreView::new().product_id("vip.kitty.pass");
let store = SubscriptionStoreViewElement::new();

let root = VStack::new()
    .spacing(8.0)
    .child(group.to_view())
    .child(store.to_view());
```

Category folder layout:

```
src/elements/subscription_store_views/
  mod.rs                    // category root
  custom_group.rs           // CustomGroupSubscriptionStoreView
  custom_header.rs          // CustomHeaderSubscriptionStoreView
  upgrade_only.rs           // UpgradeOnlySubscriptionStoreView
  group.rs                  // GroupSubscriptionStoreView
  single.rs                 // SingleSubscriptionStoreView
  subscription_store_view.rs // SubscriptionStoreViewElement
```

## Data and Callbacks

Every view renders from `StoreProduct` options with gallery defaults; cards are real buttons reporting their index and Subscribe is a real button. Selection is controlled state. There is no store backend — the hosting app connects its own kit inside the callbacks.

| Element | Data builders | Callbacks |
|---|---|---|
| `CustomGroupSubscriptionStoreView` | `group_id`, `options`, `option`, `clear_options`, `selected` | `on_select(index)`, `on_subscribe` |
| `CustomHeaderSubscriptionStoreView` | `group_id`, `header`, `options`, `option`, `clear_options`, `selected` | `on_select(index)`, `on_subscribe` |
| `UpgradeOnlySubscriptionStoreView` | `group_id`, `title`, `price`, `product` | `on_subscribe` |
| `GroupSubscriptionStoreView` | `group_id`, `options`, `option`, `clear_options`, `selected` | `on_select(index)`, `on_subscribe` |
| `SingleSubscriptionStoreView` | `product_id`, `title`, `price`, `product` | `on_subscribe` |
| `SubscriptionStoreViewElement` | `options`, `option`, `clear_options`, `selected` | `on_select(index)`, `on_subscribe` |

```rust
use tontooui::prelude::*;

let group = GroupSubscriptionStoreView::new()
    .group_id("kitty.pass.group")
    .selected(0)
    .on_select(|i| println!("option {}", i))
    .on_subscribe(|| println!("subscribe"));
```

## Cross References

- [StoreView.md](StoreView.md) -- one-time product collections; subscription views cover recurring groups
- [ProductView.md](ProductView.md) -- single-product views share pricing rows
