//! Shared toolbar types and glass styling helpers.
//!
//! The enums mirror the SwiftUI API surface: `SwiftUI.ToolbarItemPlacement`
//! (automatic, principal, primaryAction, accessoryBar) and
//! `SwiftUI.SpacerSizing.Kind` (flexible, fixed).

use uikit::style::Color;

/// Where a toolbar item is placed, mirroring
/// `SwiftUI.ToolbarItemPlacement.Role`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarItemPlacement {
    /// Default trailing bar area.
    Automatic,
    /// The title area of the bar (leading edge).
    Principal,
    /// Prominent action area.
    PrimaryAction,
    /// Secondary accessory bar below/aside the main bar.
    AccessoryBar,
}

/// How a [`crate::elements::toolbars::ToolbarSpacer`] sizes itself,
/// mirroring `SwiftUI.SpacerSizing.Kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarSpacerSizing {
    /// Takes up only the standard gap between groups.
    Fixed,
    /// Expands to fill all available space.
    Flexible,
}

pub(crate) const GLASS_BG_DARK: &str = "rgba(255,255,255,0.14)";
pub(crate) const GLASS_BG_LIGHT: &str = "rgba(0,0,0,0.06)";
pub(crate) const GLASS_BORDER_DARK: &str = "1px solid rgba(255,255,255,0.18)";
pub(crate) const GLASS_BORDER_LIGHT: &str = "1px solid rgba(0,0,0,0.10)";

/// Foreground color for toolbar glyphs (near-white in dark, near-black in light).
pub(crate) fn toolbar_glyph_color(dark: bool) -> (u8, u8, u8) {
    if dark {
        (238, 238, 240)
    } else {
        (24, 24, 27)
    }
}

/// CSS for one shared glass capsule wrapping a group of items.
pub(crate) fn glass_capsule_css(dark: bool) -> String {
    format!(
        ".tb-group {{
            background: {bg};
            border: {border};
            border-radius: 9999px;
            padding: 3px 5px;
        }}",
        bg = if dark { GLASS_BG_DARK } else { GLASS_BG_LIGHT },
        border = if dark { GLASS_BORDER_DARK } else { GLASS_BORDER_LIGHT },
    )
}

/// Tint for toolbar item glyphs as a `Color`.
pub(crate) fn toolbar_tint(dark: bool) -> Color {
    if dark {
        Color::from_rgb(238, 238, 240)
    } else {
        Color::from_rgb(24, 24, 27)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_color_per_scheme() {
        assert_eq!(toolbar_glyph_color(true), (238, 238, 240));
        assert_eq!(toolbar_glyph_color(false), (24, 24, 27));
    }

    #[test]
    fn capsule_css_contains_scheme_background() {
        assert!(glass_capsule_css(true).contains(GLASS_BG_DARK));
        assert!(glass_capsule_css(false).contains(GLASS_BG_LIGHT));
    }
}
