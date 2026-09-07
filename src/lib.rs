//! # TontooUI
//!
//! A SwiftUI-inspired declarative UI layer for TontooOS.
//!
//! Built on top of [`TontooUIKit`](https://docs.rs/uikit) — provides
//! pre-made elements with a clean, SwiftUI-like builder API.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use tontooui::prelude::*;
//!
//! fn main() {
//!     let mut app = App::new("My App", 800, 600);
//!     app.set_root(
//!         VStack::new()
//!             .spacing(8.0)
//!             .child(Text::new("Hello, TontooOS!").font_size(24.0).bold())
//!             .child(TextInput::new("Enter text...")
//!                 .on_change(|text| println!("Changed: {}", text)))
//!     );
//!     app.run();
//! }
//! ```
//!
//! ## Elements
//!
//! | Element | Description |
//! |---|---|
//! | [`Button`] | SwiftUI-style push button: roles, styles, tint, border shapes, sizing |
//! | [`Toolbar`] | Glass capsule toolbar bar with items and spacers |
//! | [`Toggle`] | SwiftUI-style toggle: switch style and checkbox style |
//! | [`RenameButton`] | System button that triggers a standard rename action |
//! | [`EditButton`] | System button that toggles between Edit and Done |
//! | [`PasteButton`] | System button that reads text from the clipboard |
//! | [`TextInput`] | Single-line text input field |
//! | [`WheelPicker`] | macOS/iOS style scroll wheel picker |
//! | [`Picker`] | SwiftUI-style picker: all picker styles (wheel, segmented, palette, radio, menu, inline, tabs, navigation) + sections/dividers |
//! | [`PickerItem`] | Single option inside a `Picker` (title / image / systemImage / custom label) |
//! | [`PickerSection`] | Section inside a `Picker` |
//! | [`PickerStyle`] | Picker presentation style enum |
//! | [`Slider`] | Slider with spring physics and squish animation |
//! | [`ProgressView`] | Loading indicator: spinner/ring or linear bar |
//! | [`ProgressViewStyle`] | ProgressView style (Circular / Linear) |
//! | [`Sidebar`] | macOS-style sidebar with traffic lights, search, and item list |
//! | [`ContentUnavailableView`] | Empty state with icon, title and hint message |
//! | [`Gauge`] | SwiftUI-style gauge: linear/circular, capacity/marker, accessory styles, tint/gradient |
//! | [`GaugeStyle`] | Gauge presentation style enum |
//! | [`GaugeTint`] | Gauge tint (single color or gradient) |
//! | [`Menu`] | SwiftUI-style menu with Liquid Glass (items, dividers, sections, nested) |
//! | [`ContextMenu`] | Context menu (secondary gesture) with optional custom preview |
//! | [`ViewThatFits`] | Adaptive container that picks first child fitting available space |
//! | [`ViewThatFitsAxis`] | Axis for ViewThatFits (Horizontal/Vertical/Both) |
//! | [`Divider`] | SwiftUI-style separator line (Light/Dark, SF Pro context) |
//! | [`DividerOrientation`] | Divider orientation (Horizontal/Vertical) |
//! | [`List`] | SwiftUI-style list (sections, rows, disclosure, outline) |
//! | [`ListSection`] | Section with header/footer and rows |
//! | [`ListRow`] | Row with label, badge, tint, separators, swipe actions |
//! | [`ListStyle`] | List style (Plain/Inset/InsetGrouped/Sidebar/Elliptical/Carousel/Bordered) |
//! | [`OutlineGroup`] | Hierarchical disclosure outline |
//! | [`DisclosureGroup`] | Collapsible disclosure |
//! | [`ColorOpacity`] | Style: Color opacity modifier (5 swatches 1.0→0.12) |
//! | [`ColorGradient`] | Modifier: Color gradient (interpolated swatches + bar) |
//! | [`ColorVariants`] | Modifier: HierarchicalShapeStyle variants (Primary/Secondary/Tertiary/Quaternary) |
//! | [`HierarchicalVariant`] | Variant level for ColorVariants |
//! | [`UIKitSeparatorColors`] | Type: UIKit separator palette (separator / opaqueSeparator) |
//! | [`UIKitSeparatorColor`] | Enum for single separator color |
//! | [`UIKitContentBackgroundColors`] | Type: UIKit content background palette (6 system backgrounds) |
//! | [`UIKitBackgroundColor`] | Enum for single background color |
//! | [`UIKitTextColors`] | Type: UIKit text palette (placeholderText) |
//! | [`UIKitTextColor`] | Enum for single text color |
//! | [`UIKitFillColors`] | Type: UIKit fill palette (systemFill ×4) |
//! | [`UIKitFillColor`] | Enum for single fill color |
//! | [`UIKitLabelColors`] | Type: UIKit label palette (label ×4) |
//! | [`UIKitLabelColor`] | Enum for single label color |
//! | [`SemanticColors`] | Type: Semantic palette (primary/secondary) |
//! | [`SemanticColor`] | Enum for single semantic color |
//! | [`StandardColors`] | Type: Standard SwiftUI palette (12 colors grid) |
//! | [`StandardColor`] | Enum for single standard color |
//! | [`TextFormat`] | Initializer: Text that displays formatted representation of non-string type |
//! | [`TextFormatKind`] | Format kind enum for TextFormat |
//! | [`Materials`] | Type: All materials (ultraThinMaterial … ultraThickMaterial, bar) |
//! | [`Material`] | Enum for single material thickness |

pub mod pickers;
pub mod text_input;
// Shims kept for backwards compatibility — real code lives in `pickers/`
pub mod wheel_picker;
pub mod picker;
pub mod progress_view;
pub mod sidebar;
pub mod content_unavailable_view;
pub mod elements;

pub use text_input::TextInput;
pub use pickers::wheel_picker::WheelPicker;
pub use pickers::picker::{Picker, PickerItem, PickerSection, PickerStyle};
pub use pickers::card::PickerCard;
pub use progress_view::{ProgressView, ProgressViewStyle};
pub use sidebar::Sidebar;
pub use content_unavailable_view::ContentUnavailableView;
pub use elements::buttons::{Button, ButtonRole, ButtonStyle, ButtonBorderShape, ButtonSizing, RenameButton, EditButton, PasteButton};
pub use elements::colors::{
    ColorGradient, ColorOpacity, ColorVariants, HierarchicalVariant, SemanticColor,
    SemanticColors, StandardColor, StandardColors, UIKitBackgroundColor,
    UIKitContentBackgroundColors, UIKitFillColor, UIKitFillColors, UIKitLabelColor,
    UIKitLabelColors, UIKitSeparatorColor, UIKitSeparatorColors, UIKitTextColor,
    UIKitTextColors,
};
pub use elements::materials::{Material, Materials};
pub use elements::text::{TextFormat, TextFormatKind};
pub use elements::dividers::{Divider, DividerOrientation};
pub use elements::group_boxes::GroupBox;
pub use elements::lists::{List, ListSection, ListRow, ListStyle, OutlineGroup, DisclosureGroup};
pub use elements::scroll_views::{ScrollView, ScrollEdgeEffect};
pub use elements::sliders::Slider;
pub use elements::gauges::{Gauge, GaugeStyle, GaugeTint};
pub use elements::menus::{Menu, MenuItem, MenuEntry, MenuRole, ContextMenu};
pub use elements::toggles::{Toggle, ToggleStyle};
pub use elements::toolbars::{Toolbar, ToolbarItem, ToolbarSpacer, ToolbarItemPlacement, ToolbarSpacerSizing};
pub use elements::view_that_fits::{ViewThatFits, ViewThatFitsAxis};
#[cfg(feature = "coreicon")]
pub use sidebar::SidebarIcon;

pub const TONTOO_UI_VERSION: (u32, u32, u32) = (0, 1, 0);

pub mod prelude {
    pub use crate::{
        ColorGradient, ColorOpacity, ColorVariants, HierarchicalVariant, Material, Materials,
        SemanticColor, SemanticColors, StandardColor, StandardColors, TextFormat,
        TextFormatKind, UIKitBackgroundColor, UIKitContentBackgroundColors, UIKitFillColor,
        UIKitFillColors, UIKitLabelColor, UIKitLabelColors, UIKitSeparatorColor,
        UIKitSeparatorColors, UIKitTextColor, UIKitTextColors, TextInput, WheelPicker, Picker,
        PickerItem, PickerSection, PickerStyle, PickerCard, ProgressView, ProgressViewStyle,
        Sidebar, ContentUnavailableView, TONTOO_UI_VERSION,
    };
    pub use crate::elements::dividers::{Divider, DividerOrientation};
    pub use crate::elements::group_boxes::GroupBox;
    pub use crate::elements::lists::{List, ListSection, ListRow, ListStyle, OutlineGroup, DisclosureGroup};
    pub use crate::elements::sliders::Slider;
    pub use crate::elements::gauges::{Gauge, GaugeStyle, GaugeTint};
    pub use crate::elements::menus::{Menu, MenuItem, MenuEntry, MenuRole, ContextMenu};
    pub use crate::elements::view_that_fits::{ViewThatFits, ViewThatFitsAxis};
    pub use crate::elements::buttons::{Button, ButtonRole, ButtonStyle, ButtonBorderShape, ButtonSizing, RenameButton, EditButton, PasteButton};
    pub use crate::elements::toggles::{Toggle, ToggleStyle};
    pub use crate::elements::toolbars::{Toolbar, ToolbarItem, ToolbarSpacer, ToolbarItemPlacement, ToolbarSpacerSizing};
    #[cfg(feature = "coreicon")]
    pub use crate::SidebarIcon;

    // Re-export UIKit types for convenience
    pub use uikit::prelude::*;
}

mod ffi;
