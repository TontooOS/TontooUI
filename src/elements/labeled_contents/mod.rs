//! LabeledContent — category for labeled informational views.
//!
//! Category `LabeledContent` groups labeled content views. All elements render
//! directly on the window background (#1d1d1d dark / #ececec light), no extra
//! card, SF Pro.
//!
//! | File | Element | Badge | Description |
//! |---|---|---|--->
//! | [`custom_labeled_content`] | [`CustomLabeledContent`] | `initializer` | Creates a standard labeled element, with a view that conveys the value of the content. |
//! | [`formatted_labeled_content`] | [`FormattedLabeledContent`] | `initializer` | Creates a labeled informational view from a formatted value. |
//! | [`labeled_content`] | [`LabeledContent`] | `initializer` | Creates a labeled informational view. |

pub mod custom_labeled_content;
pub mod formatted_labeled_content;
pub mod labeled_content;

pub use custom_labeled_content::CustomLabeledContent;
pub use formatted_labeled_content::FormattedLabeledContent;
pub use labeled_content::LabeledContent;
