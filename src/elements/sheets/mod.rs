//! Sheet — category for sheet presentations and modifiers.
//!
//! Category `Sheet` groups sheet presentation modifiers. All elements render
//! directly on the window background (#1d1d1d dark / #ececec light), no extra
//! card, SF Pro.
//!
//! | File | Element | Badge | Description |
//! |---|---|---|--->
//! | [`sheet_placement`] | [`SheetPlacement`] | `modifier` | Sets the placement of a presentation within the presenting view. |
//! | [`disable_dismiss`] | [`DisableSheetDismissSwipe`] | `modifier` | Conditionally prevents interactive dismissal of presentations like popover. |
//! | [`page_size`] | [`PageScreenSheetSize`] | `modifier` | On devices smaller than a page of paper, such as iPhone, Apple Watch, page sizing fills. |
//! | [`fitted_sizing`] | [`FittedSheetSizing`] | `modifier` | Sets the sizing of the containing presentation. |
//! | [`corner_radius`] | [`SheetCornerRadius`] | `modifier` | Requests that the presentation have a specific corner radius. |
//! | [`prioritize_scrolling`] | [`PrioritizeSheetContentScrolling`] | `modifier` | Configure the behavior of swipe gestures on a presentation. |
//! | [`background_interaction`] | [`SheetBackgroundInteraction`] | `modifier` | Controls whether people can interact with the view behind a presentation. |
//! | [`background`] | [`SheetBackground`] | `modifier` | Sets the presentation background of the enclosing sheet using a shape style. |
//! | [`drag_indicator`] | [`SheetDragIndicatorVisibility`] | `modifier` | Sets the visibility of the drag indicator on top of a sheet. |
//! | [`size`] | [`SheetSize`] | `modifier` | Sets the available detents for the enclosing sheet. |
//! | [`item_sheet`] | [`ItemSheet`] | `modifier` | Presents a sheet using the given item as a data source for the sheet content. |
//! | [`boolean_sheet`] | [`BooleanSheet`] | `modifier` | Presents a sheet when a binding to a Boolean value is true. |

pub mod sheet_placement;
pub mod disable_dismiss;
pub mod page_size;
pub mod fitted_sizing;
pub mod corner_radius;
pub mod prioritize_scrolling;
pub mod background_interaction;
pub mod background;
pub mod drag_indicator;
pub mod size;
pub mod item_sheet;
pub mod boolean_sheet;

pub use sheet_placement::{SheetPlacement, SheetPlacementKind};
pub use disable_dismiss::DisableSheetDismissSwipe;
pub use page_size::PageScreenSheetSize;
pub use fitted_sizing::FittedSheetSizing;
pub use corner_radius::SheetCornerRadius;
pub use prioritize_scrolling::PrioritizeSheetContentScrolling;
pub use background_interaction::{SheetBackgroundInteraction, SheetBackgroundInteractionKind};
pub use background::SheetBackground;
pub use drag_indicator::{SheetDragIndicatorVisibility, SheetDragIndicator};
pub use size::{SheetSize, SheetDetent};
pub use item_sheet::ItemSheet;
pub use boolean_sheet::BooleanSheet;
