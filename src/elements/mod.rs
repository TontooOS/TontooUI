//! UI elements for TontooUI.
//!
//! Each element family lives in its own subdirectory with one file per
//! element and a `mod.rs` aggregating the public API.

pub mod buttons;
pub mod sliders;
pub mod toolbars;

pub use buttons::*;
pub use sliders::*;
pub use toolbars::*;

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
