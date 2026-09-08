//! SubscriptionStoreView — category for subscription store views.
//!
//! Category `SubscriptionStoreView` groups subscription store views. All
//! elements render directly on the window background (#1d1d1d dark / #ececec
//! light), no extra card, SF Pro.
//!
//! | File | Element | Badge | Description |
//! |---|---|---|--->
//! | [`custom_group`] | [`CustomGroupSubscriptionStoreView`] | `initializer` | Creates a SubscriptionStoreView with custom grouping. |
//! | [`custom_header`] | [`CustomHeaderSubscriptionStoreView`] | `initializer` | Creates a view that loads all the subscriptions in a subscription group with a custom header. |
//! | [`upgrade_only`] | [`UpgradeOnlySubscriptionStoreView`] | `initializer` | Creates a view that loads all subscriptions from a subscription group, upgrade options only. |
//! | [`group`] | [`GroupSubscriptionStoreView`] | `initializer` | Creates a view that loads all subscriptions in a subscription group. |
//! | [`single`] | [`SingleSubscriptionStoreView`] | `initializer` | Creates a view that loads subscriptions based on a collection of products (single). |
//! | [`subscription_store_view`] | [`SubscriptionStoreViewElement`] | `initializer` | Creates a view that loads subscriptions based on a collection of products. |

pub mod custom_group;
pub mod custom_header;
pub mod upgrade_only;
pub mod group;
pub mod single;
pub mod subscription_store_view;

pub use custom_group::CustomGroupSubscriptionStoreView;
pub use custom_header::CustomHeaderSubscriptionStoreView;
pub use upgrade_only::UpgradeOnlySubscriptionStoreView;
pub use group::GroupSubscriptionStoreView;
pub use single::SingleSubscriptionStoreView;
pub use subscription_store_view::SubscriptionStoreViewElement;
