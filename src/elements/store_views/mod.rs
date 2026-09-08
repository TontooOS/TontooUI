//! StoreView — category for App Store product collection views.
//!
//! Category `StoreView` groups store views and buttons. All elements render
//! directly on the window background (#1d1d1d dark / #ececec light), no extra
//! card, SF Pro.
//!
//! | File | Element | Badge | Description |
//! |---|---|---|--->
//! | [`icon_phase`] | [`IconPhaseStoreView`] | `initializer` | Creates a view to load a collection of products from the App Store using icon phases. |
//! | [`placeholder_icon`] | [`PlaceholderIconStoreView`] | `initializer` | Creates a view to load a collection of products from the App Store using placeholder icons. |
//! | [`custom_icon`] | [`CustomIconStoreView`] | `initializer` | Creates a view to load a collection of products from the App Store using custom icons. |
//! | [`store_view`] | [`StoreViewElement`] | `initializer` | Creates a view to load and merchandise a collection of products from the App Store. |
//! | [`cancellation_button`] | [`StoreCancellationButton`] | `modifier` | A type of button that people use to dismiss the current store presentation. |
//! | [`restore_button`] | [`RestorePurchasesButton`] | `modifier` | A type of button that people use to restore purchases. |

pub mod icon_phase;
pub mod placeholder_icon;
pub mod custom_icon;
pub mod store_view;
pub mod cancellation_button;
pub mod restore_button;

pub use icon_phase::{IconPhaseStoreView, StoreIconPhase};
pub use placeholder_icon::PlaceholderIconStoreView;
pub use custom_icon::CustomIconStoreView;
pub use store_view::StoreViewElement;
pub use cancellation_button::StoreCancellationButton;
pub use restore_button::RestorePurchasesButton;
