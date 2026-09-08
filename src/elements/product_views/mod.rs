//! ProductView — category for App Store product views.
//!
//! Category `ProductView` groups product views and styles. All elements render
//! directly on the window background (#1d1d1d dark / #ececec light), no extra
//! card, SF Pro.
//!
//! | File | Element | Badge | Description |
//! |---|---|---|--->
//! | [`placeholder_icon`] | [`PlaceholderIconProductView`] | `initializer` | Creates a view to load an individual product from the App Store, with a placeholder icon. |
//! | [`custom_icon`] | [`CustomIconProductView`] | `initializer` | Creates a view to load an individual product from the App Store with a custom icon. |
//! | [`product_view`] | [`ProductViewElement`] | `initializer` | Creates a view to load and merchandise an individual product from the App Store. |
//! | [`compact_style`] | [`CompactProductViewStyle`] | `style` | A style for a product view that is suitable for layouts with less available space. |
//! | [`regular_style`] | [`RegularProductViewStyle`] | `style` | A style for a product view that uses a standard, platform-appropriate layout. |
//! | [`large_style`] | [`LargeProductViewStyle`] | `style` | A style for a product view that is suitable for layouts where the in-app purchase is the hero content. |

pub mod placeholder_icon;
pub mod custom_icon;
pub mod product_view;
pub mod compact_style;
pub mod regular_style;
pub mod large_style;

pub use placeholder_icon::PlaceholderIconProductView;
pub use custom_icon::CustomIconProductView;
pub use product_view::ProductViewElement;
pub use compact_style::CompactProductViewStyle;
pub use regular_style::RegularProductViewStyle;
pub use large_style::LargeProductViewStyle;
