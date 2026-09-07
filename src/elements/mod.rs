//! UI elements for TontooUI.
//!
//! Each element family lives in its own subdirectory with one file per
//! element and a `mod.rs` aggregating the public API.

pub mod buttons;
pub mod colors;
pub mod dividers;
pub mod gauges;
pub mod group_boxes;
pub mod lists;
pub mod materials;
pub mod menus;
pub mod scroll_views;
pub mod sliders;
pub mod text;
pub mod toggles;
pub mod toolbars;
pub mod view_that_fits;

pub use buttons::*;
pub use colors::{
    ColorGradient, ColorOpacity, ColorVariants, HierarchicalVariant, SemanticColor,
    SemanticColors, StandardColor, StandardColors, UIKitBackgroundColor,
    UIKitContentBackgroundColors, UIKitFillColor, UIKitFillColors, UIKitLabelColor,
    UIKitLabelColors, UIKitSeparatorColor, UIKitSeparatorColors, UIKitTextColor,
    UIKitTextColors,
};
pub use dividers::*;
pub use gauges::*;
pub use group_boxes::GroupBox;
pub use lists::*;
pub use materials::{Material, Materials};
pub use menus::*;
pub use scroll_views::{ScrollView, ScrollEdgeEffect};
pub use sliders::*;
pub use text::{TextFormat, TextFormatKind};
pub use toggles::{Toggle, ToggleStyle};
pub use toolbars::*;
pub use view_that_fits::*;

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
