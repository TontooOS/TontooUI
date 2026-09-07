//! Navigation — category for navigation modifiers.
//!
//! The `Navigation` category groups navigation-related view modifiers.
//! Rendering is directly on the window background (#1d1d1d dark / #ececec light),
//! no extra card, SF Pro.
//!
//! | File | Element | Badge | Description |
//! |---|---|---|--->
//! | [`navigation_subtitle`] | [`NavigationSubtitle`] | `modifier` | Configures the view's subtitle for purposes of navigation, using a localized string key |

pub mod navigation_subtitle;

pub use navigation_subtitle::NavigationSubtitle;
