# ProductView

SwiftUI-style ProductView category for TontooUI, recreating App Store product views and styles. The category contains three `initializer` elements and three `style` elements. Rendering is directly on the window background (#1d1d1d dark / #ececec light) with no extra card, using `SF Pro Display`.

## PlaceholderIconProductView

```rust
pub struct PlaceholderIconProductView { /* ... */ }
impl PlaceholderIconProductView {
    pub fn new() -> Self;
    pub fn product_id(self, v: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a view to load an individual product from the App Store, with a placeholder icon.

- Preview: skeleton bars plus star and pill on a `180x110` phone card.
- Size `180x110`.

## CustomIconProductView

```rust
pub struct CustomIconProductView { /* ... */ }
impl CustomIconProductView {
    pub fn new() -> Self;
    pub fn product_id(self, v: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a view to load an individual product from the App Store with a custom icon.

- Preview: `VIP Kitty Pass` plus subtitle, cat icon and `$0.99/month`.
- Size `180x110`.

## ProductViewElement

```rust
pub struct ProductViewElement { /* ... */ }
impl ProductViewElement {
    pub fn new() -> Self;
    pub fn product_id(self, v: impl Into<String>) -> Self;
    pub fn to_view(self) -> View;
}
```

Initializer — creates a view to load and merchandise an individual product from the App Store.

- Preview: header plus price plus cats row with texts.
- Size `180x110`.

## CompactProductViewStyle

```rust
pub struct CompactProductViewStyle { /* ... */ }
impl CompactProductViewStyle {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Style — a style for a product view that is suitable for layouts with less available space.

- Preview: compact horizontal row with thumb, texts and `$0.99`.
- Size `180x110`.

## RegularProductViewStyle

```rust
pub struct RegularProductViewStyle { /* ... */ }
impl RegularProductViewStyle {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Style — a style for a product view that uses a standard, platform-appropriate layout.

- Preview: two cats plus texts plus `$0.99/month` pill.
- Size `180x110`.

## LargeProductViewStyle

```rust
pub struct LargeProductViewStyle { /* ... */ }
impl LargeProductViewStyle {
    pub fn new() -> Self;
    pub fn to_view(self) -> View;
}
```

Style — a style for a product view that is suitable for layouts where the in-app purchase is the hero content.

- Preview: large centered cats hero plus title, subtitle and price.
- Size `180x110`.

## Usage / Example

Run the gallery demo:

```bash
cargo run --example product_views
```

Minimal usage:

```rust
use tontooui::prelude::*;

let product = ProductViewElement::new().product_id("vip.kitty.pass");
let compact = CompactProductViewStyle::new();
let large = LargeProductViewStyle::new();

let root = VStack::new()
    .spacing(8.0)
    .child(product.to_view())
    .child(large.to_view());
```

Category folder layout:

```
src/elements/product_views/
  mod.rs              // category root
  placeholder_icon.rs // PlaceholderIconProductView
  custom_icon.rs      // CustomIconProductView
  product_view.rs     // ProductViewElement
  compact_style.rs    // CompactProductViewStyle
  regular_style.rs    // RegularProductViewStyle
  large_style.rs      // LargeProductViewStyle
```

## Data and Callbacks

All views render from `StoreProduct` data (title, subtitle, price) with gallery defaults, so existing code keeps compiling. Prices are real buttons. There is no store backend — the hosting app connects its own kit (e.g. StoreKit) inside the callbacks.

| Element | Data builders | Callbacks |
|---|---|---|
| `PlaceholderIconProductView` | `product_id` (loading state, no data) | none |
| `CustomIconProductView` | `product_id`, `title`, `subtitle`, `price`, `product` | `on_buy` |
| `ProductViewElement` | `product_id`, `title`, `subtitle`, `price`, `product` | `on_buy` |
| `CompactProductViewStyle` | `title`, `subtitle`, `price`, `product` | `on_buy` |
| `RegularProductViewStyle` | `title`, `subtitle`, `price`, `product` | `on_buy` |
| `LargeProductViewStyle` | `title`, `subtitle`, `price`, `product` | `on_buy` |

```rust
use tontooui::prelude::*;

let view = CustomIconProductView::new()
    .product_id("vip.kitty.pass")
    .title("VIP Kitty Pass")
    .price("$0.99/month")
    .on_buy(|| println!("buy pressed"));
```

## Cross References

- [AsyncImage.md](AsyncImage.md) -- product icons load like async images
- [Label.md](Label.md) -- product titles pair with label styles
