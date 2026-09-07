//! UI elements for TontooUI.
//!
//! Each element family lives in its own subdirectory with one file per
//! element and a `mod.rs` aggregating the public API.

pub mod buttons;
pub mod colors;
pub mod control_groups;
pub mod dividers;
pub mod gauges;
pub mod group_boxes;
pub mod labeled_contents;
pub mod labels;
pub mod links;
pub mod lists;
pub mod materials;
pub mod menus;
pub mod navigation;
pub mod scroll_views;
pub mod shapes;
pub mod sliders;
pub mod sheets;
pub mod tab_views;
pub mod text;
pub mod toggles;
pub mod toolbars;
pub mod view_that_fits;
pub mod views;

pub use buttons::*;
pub use colors::{
    ColorGradient, ColorOpacity, ColorVariants, HierarchicalVariant, SemanticColor,
    SemanticColors, StandardColor, StandardColors, UIKitBackgroundColor,
    UIKitContentBackgroundColors, UIKitFillColor, UIKitFillColors, UIKitLabelColor,
    UIKitLabelColors, UIKitSeparatorColor, UIKitSeparatorColors, UIKitTextColor,
    UIKitTextColors,
};
pub use control_groups::{
    CompactMenuControlGroupStyle, ControlGroup, ControlGroupStyle, MenuControlGroupStyle,
    NavigationControlGroupStyle, PaletteControlGroupStyle,
};
pub use dividers::*;
pub use gauges::*;
pub use group_boxes::GroupBox;
pub use labeled_contents::{CustomLabeledContent, FormattedLabeledContent, LabeledContent};
pub use labels::{CustomLabel, ImageLabel, LabelStyleKind, LabelStyles, SystemImageLabel};
pub use links::{CustomPreviewShareLink, HelpLink, Link, ShareLink, TextFieldLink};
pub use lists::*;
pub use materials::{Material, Materials};
pub use menus::*;
pub use navigation::NavigationSubtitle;
pub use scroll_views::{ScrollView, ScrollEdgeEffect};
pub use shapes::{
    Capsule, Circle, ContainerRelativeShape, Ellipse, RectangleShape,
    RoundedRectangle, UnevenRoundedRectangle,
};
pub use sliders::*;
pub use sheets::{
    BooleanSheet, DisableSheetDismissSwipe, FittedSheetSizing, ItemSheet,
    PageScreenSheetSize, PrioritizeSheetContentScrolling, SheetBackground,
    SheetBackgroundInteraction, SheetBackgroundInteractionKind, SheetCornerRadius,
    SheetDetent, SheetDragIndicator, SheetDragIndicatorVisibility, SheetPlacement,
    SheetPlacementKind, SheetSize,
};
pub use tab_views::{
    BottomAccessory, DefaultAdaptableTabBarPlacement, DefaultCollapsedTabSection,
    GroupedTabViewStyle, HiddenIndexPageTabViewStyle, HideTabBarOnScrollDown, PageTabViewStyle,
    SearchTabRole, SidebarAdaptableTabViewStyle, TabBadge, TabBarOnlyTabViewStyle,
    TabBarSectionActions, TabSection, TabView, TabViewBottomAccessoryPlacement,
    TabViewCustomization, TabViewCustomizationBehavior, TabViewSideBarBottomBar,
    TabViewSideBarFooter, TabViewSideBarHeader, ValueTabView, VerticalPageTabViewStyle,
};
pub use text::{TextFormat, TextFormatKind};
pub use toggles::{Toggle, ToggleStyle};
pub use toolbars::*;
pub use view_that_fits::*;
pub use views::{
    AppStoreOverlay, BackgroundExtensionEffect, ControlSizeView, GlassEffect,
    ManageSubscriptionsSheet, MusicPicker, NavigationContainerBackground,
    NavigationSplitViewBackground, SwipeAction, SwipeContainer,
};

use uikit::app::ColorScheme;

/// Resolve the color scheme an element should render with: an explicit
/// override wins, then the running app's scheme, then the system detection.
pub(crate) fn resolve_scheme(explicit: Option<ColorScheme>) -> ColorScheme {
    if let Some(s) = explicit {
        return s;
    }
    if let Some(s) = uikit::app::current_color_scheme() {
        return s;
    }
    ColorScheme::detect_system()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_scheme_wins() {
        assert_eq!(
            resolve_scheme(Some(ColorScheme::Light)),
            ColorScheme::Light
        );
    }
}
