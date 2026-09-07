//! Link — category for navigation and sharing links.
//!
//! Category `Link` groups link/sharing controls. All elements render directly on
//! the window background (#1d1d1d dark / #ececec light) with SF Pro, no extra card.
//!
//! | File | Element | Badge | Description |
//! |---|---|---|--->
//! | [`help_link`] | [`HelpLink`] | `initializer` | A button with a standard appearance that opens app-specific help |
//! | [`text_field_link`] | [`TextFieldLink`] | `initializer` | A control that requests text input from the user when pressed |
//! | [`custom_preview_share_link`] | [`CustomPreviewShareLink`] | `initializer` | Creates an instance, with a custom label, that presents the share interface |
//! | [`share_link`] | [`ShareLink`] | `initializer` | A view that controls a sharing presentation |
//! | [`link`] | [`Link`] | `initializer` | A control for navigating to a URL |

pub mod help_link;
pub mod text_field_link;
pub mod custom_preview_share_link;
pub mod share_link;
pub mod link;

pub use help_link::HelpLink;
pub use text_field_link::TextFieldLink;
pub use custom_preview_share_link::CustomPreviewShareLink;
pub use share_link::ShareLink;
pub use self::link::Link;
