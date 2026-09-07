//! Toggles — SwiftUI-style toggle elements.
//!
//! One file per element, mirroring the SwiftUI toggle styles from the macOS 26
//! dumps:
//!
//! | File | Element |
//! |---|---|
//! | [`toggle`] | [`Toggle`] — switch style (label + trailing switch) and checkbox style |
//! | [`common`] | [`ToggleStyle`], system palette and state painters |

pub mod common;
pub mod group;
pub mod toggle;

pub use toggle::Toggle;
pub use common::ToggleStyle;
pub use group::GroupBox;
