//! Label — category for label initializers and styles.
//!
//! Category `Label` groups label views. All elements render directly on the
//! window background (#1d1d1d dark / #ececec light), no extra card, SF Pro.
//!
//! | File | Element | Badge | Description |
//! |---|---|---|--->
//! | [`custom_label`] | [`CustomLabel`] | `initializer` | Creates a label with a custom title and icon. |
//! | [`image_label`] | [`ImageLabel`] | `initializer` | Creates a label with an icon image and a title generated from a localized string. |
//! | [`system_image_label`] | [`SystemImageLabel`] | `initializer` | Creates a label with a system icon image and a title generated from a localized string. |
//! | [`label_styles`] | [`LabelStyles`] | `style` | Sets the style for labels within this view. |

pub mod custom_label;
pub mod image_label;
pub mod system_image_label;
pub mod label_styles;

pub use custom_label::CustomLabel;
pub use image_label::ImageLabel;
pub use system_image_label::SystemImageLabel;
pub use label_styles::{LabelStyles, LabelStyleKind};
