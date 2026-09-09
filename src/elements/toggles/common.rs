//! Shared toggle types, the iOS system palette and state painters.
//!
//! The style enum mirrors the SwiftUI toggle style hierarchy
//! (`SwitchToggleStyle`, `CheckboxToggleStyle`).

use gtk::prelude::*;
use gtk::Label as GtkLabel;

/// The visual style of a [`crate::elements::toggles::Toggle`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleStyle {
    /// Leading label with a trailing switch (`SwitchToggleStyle`).
    Switch,
    /// A checkbox followed by its label (`CheckboxToggleStyle`).
    Checkbox,
}

pub(crate) const SWITCH_ON_LIGHT: &str = "#34C759";
pub(crate) const SWITCH_ON_DARK: &str = "#30D158";
pub(crate) const SWITCH_OFF_LIGHT: &str = "#E9E9EA";
pub(crate) const SWITCH_OFF_DARK: &str = "#39393D";
pub(crate) const CHECK_ON_LIGHT: &str = "#007AFF";
pub(crate) const CHECK_ON_DARK: &str = "#0A84FF";

pub(crate) fn label_color(dark: bool) -> &'static str {
    if dark {
        "#ececec"
    } else {
        "#1d1d1d"
    }
}

/// Paint the switch track and knob for the given state.
pub(crate) fn paint_switch(track: &gtk::Box, _knob: &gtk::Box, on: bool, dark: bool) {
    let bg = if on {
        if dark { SWITCH_ON_DARK } else { SWITCH_ON_LIGHT }
    } else if dark {
        SWITCH_OFF_DARK
    } else {
        SWITCH_OFF_LIGHT
    };
    uikit::widget::apply_css(
        track,
        &format!(
            "box {{ background: {bg}; border-radius: 15.5px; transition: background 200ms cubic-bezier(0.32,0.72,0,1); }}",
            bg = bg,
        ),
    );
}

pub(crate) const CHECK_OFF_DARK: &str = "#2C2C2E";
pub(crate) const CHECK_OFF_LIGHT: &str = "#E0E0E4";

/// Paint the checkbox for the given state (blue filled + white checkmark
/// when on, light-gray solid when off — no border on either).
pub(crate) fn paint_checkbox(box_w: &gtk::Box, check: &GtkLabel, on: bool, dark: bool) {
    if on {
        let bg = if dark { CHECK_ON_DARK } else { CHECK_ON_LIGHT };
        uikit::widget::apply_css(
            box_w,
            &format!("box {{ background: {bg}; border-radius: 6px; }}", bg = bg),
        );
        check.set_visible(true);
    } else {
        let bg = if dark { CHECK_OFF_DARK } else { CHECK_OFF_LIGHT };
        uikit::widget::apply_css(
            box_w,
            &format!("box {{ background: {bg}; border-radius: 6px; }}", bg = bg),
        );
        check.set_visible(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_per_scheme() {
        assert_eq!(label_color(true), "#ececec");
        assert_eq!(label_color(false), "#1d1d1d");
        assert_eq!(SWITCH_ON_DARK, "#30D158");
        assert_eq!(CHECK_ON_LIGHT, "#007AFF");
    }
}
