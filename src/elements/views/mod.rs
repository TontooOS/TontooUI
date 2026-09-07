//! View — category for view modifiers.
//!
//! Category `View` groups view modifiers. All elements render directly on the
//! window background (#1d1d1d dark / #ececec light), no extra card, SF Pro.
//! Core backing is in `uikit::view_modifiers::ViewModifierExt` — TontooUI
//! provides the palette previews.
//!
//! | File | Element | Badge | Description |
//! |---|---|---|--->
//! | [`music_picker`] | [`MusicPicker`] | `modifier` | Presents a music picker to select items from the Apple Music catalog |
//! | [`app_store_overlay`] | [`AppStoreOverlay`] | `modifier` | Presents a StoreKit overlay when a given condition is true |
//! | [`manage_subscriptions_sheet`] | [`ManageSubscriptionsSheet`] | `modifier` | Opens the manage subscriptions sheet |
//! | [`swipe_container`] | [`SwipeContainer`] | `modifier` | Only allows a single active swipe within a container |
//! | [`swipe_action`] | [`SwipeAction`] | `modifier` | Adds custom swipe actions to a row in a list or container |
//! | [`navigation_split_view_background`] | [`NavigationSplitViewBackground`] | `modifier` | A background placement behind the content of a NavigationSplitView |
//! | [`navigation_container_background`] | [`NavigationContainerBackground`] | `modifier` | Sets the container background of the enclosing container using a view |
//! | [`control_size`] | [`ControlSizeView`] | `modifier` | A control version that is the default size |
//! | [`background_extension_effect`] | [`BackgroundExtensionEffect`] | `modifier` | Adds the background extension effect to the view |
//! | [`glass_effect`] | [`GlassEffect`] | `modifier` | Applies the Liquid Glass effect to a view |

pub mod music_picker;
pub mod app_store_overlay;
pub mod manage_subscriptions_sheet;
pub mod swipe_container;
pub mod swipe_action;
pub mod navigation_split_view_background;
pub mod navigation_container_background;
pub mod control_size;
pub mod background_extension_effect;
pub mod glass_effect;

pub use music_picker::MusicPicker;
pub use app_store_overlay::AppStoreOverlay;
pub use manage_subscriptions_sheet::ManageSubscriptionsSheet;
pub use swipe_container::SwipeContainer;
pub use swipe_action::SwipeAction;
pub use navigation_split_view_background::NavigationSplitViewBackground;
pub use navigation_container_background::NavigationContainerBackground;
pub use control_size::ControlSizeView;
pub use background_extension_effect::BackgroundExtensionEffect;
pub use glass_effect::GlassEffect;

// Re-export core control size for convenience
pub use uikit::view_modifiers::ControlSize;
