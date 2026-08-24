//! Buttons — SwiftUI-style button elements.
//!
//! One file per element, mirroring the SwiftUI API surface from the macOS 26
//! interface dumps:
//!
//! | File | Element |
//! |---|---|
//! | [`button`] | [`Button`] with roles, styles, tint, border shapes, sizing |
//! | [`rename_button`] | [`RenameButton`] — standard rename action |
//! | [`edit_button`] | [`EditButton`] — Edit/Done toggle |
//! | [`paste_button`] | [`PasteButton`] — clipboard read |
//! | [`common`] | shared enums, system palette and render helpers |

pub mod common;
pub mod button;
pub mod rename_button;
pub mod edit_button;
pub mod paste_button;

pub use button::Button;
pub use common::{ButtonRole, ButtonStyle, ButtonBorderShape, ButtonSizing};
pub use rename_button::RenameButton;
pub use edit_button::EditButton;
pub use paste_button::PasteButton;
