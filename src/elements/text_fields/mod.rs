//! TextField — category for text field styles.
//!
//! Category `TextField` groups text field styles. All elements render directly
//! on the window background (#1d1d1d dark / #ececec light), no extra card,
//! SF Pro.
//!
//! | File | Element | Badge | Description |
//! |---|---|---|--->
//! | [`capsule_text_field`] | [`CapsuleTextField`] | `modifier` | Gives your text field a capsule shape. This only has a visible effect on macOS. |

pub mod capsule_text_field;

pub use capsule_text_field::CapsuleTextField;
