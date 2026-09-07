//! Text — category for text display elements.
//!
//! The `Text` category groups text-related elements. The pre-existing
//! `Text`/`Label` widgets from UIKit live here logically (`Text` is a
//! thin SF Pro wrapper over `GtkLabel` with Pango rendering). The new
//! element in this category is `TextFormat`.
//!
//! | File | Element | Badge | Description |
//! |---|---|---|--->
//! | [`text_format`] | [`TextFormat`] | `initializer` | Creates a text view that displays the formatted representation of a non-string type |

pub mod text_format;

pub use text_format::{TextFormat, TextFormatKind};

// Re-export UIKit Text/Label under this category for discoverability.
// Users can continue to use `uikit::prelude::Text` / `Label`, but also
// `tontooui::Text` via this module.
pub use uikit::widgets::{Label, Text};
