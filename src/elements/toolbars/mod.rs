//! Toolbars — SwiftUI-style toolbar elements.
//!
//! One file per element, mirroring the SwiftUI API surface from the macOS 26
//! interface dumps:
//!
//! | File | Element |
//! |---|---|
//! | [`toolbar`] | [`Toolbar`] — glass capsule bar container |
//! | [`toolbar_item`] | [`ToolbarItem`] — flat item, shared glass background, placement |
//! | [`toolbar_spacer`] | [`ToolbarSpacer`] — fixed / flexible space (`SpacerSizing.Kind`) |
//! | [`common`] | shared enums and glass styling helpers |
//!
//! Consecutive items with a shared background merge into one glass capsule;
//! a [`ToolbarSpacer`] separates groups, mirroring the iOS 26 Liquid Glass
//! toolbar behavior.

pub mod common;
pub mod toolbar;
pub mod toolbar_item;
pub mod toolbar_spacer;

pub use toolbar::Toolbar;
pub use toolbar_item::ToolbarItem;
pub use toolbar_spacer::ToolbarSpacer;
pub use common::{ToolbarItemPlacement, ToolbarSpacerSizing};
