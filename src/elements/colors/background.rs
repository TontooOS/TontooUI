//! UIKit Content Background colors — systemGroupedBackground etc.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// Individual background color — mirrors UIKit `UIColor.systemBackground`,
/// `secondarySystemBackground`, `tertiarySystemBackground`,
/// `systemGroupedBackground`, `secondarySystemGroupedBackground`,
/// `tertiarySystemGroupedBackground`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIKitBackgroundColor {
    SystemBackground,
    SecondarySystemBackground,
    TertiarySystemBackground,
    SystemGroupedBackground,
    SecondarySystemGroupedBackground,
    TertiarySystemGroupedBackground,
}

impl UIKitBackgroundColor {
    pub fn all() -> [Self; 6] {
        [
            Self::SystemBackground,
            Self::SecondarySystemBackground,
            Self::TertiarySystemBackground,
            Self::SystemGroupedBackground,
            Self::SecondarySystemGroupedBackground,
            Self::TertiarySystemGroupedBackground,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::SystemBackground => "systemBackground",
            Self::SecondarySystemBackground => "secondarySystemBackground",
            Self::TertiarySystemBackground => "tertiarySystemBackground",
            Self::SystemGroupedBackground => "systemGroupedBackground",
            Self::SecondarySystemGroupedBackground => "secondarySystemGroupedBackground",
            Self::TertiarySystemGroupedBackground => "tertiarySystemGroupedBackground",
        }
    }

    pub fn color(self, is_dark: bool) -> Color {
        match (self, is_dark) {
            (Self::SystemBackground, false) => Color::from_rgb(255, 255, 255),
            (Self::SystemBackground, true) => Color::from_hex("#1d1d1d").unwrap(),
            (Self::SecondarySystemBackground, false) => Color::from_hex("#f2f2f7").unwrap(),
            (Self::SecondarySystemBackground, true) => Color::from_hex("#2c2c2e").unwrap(),
            (Self::TertiarySystemBackground, false) => Color::from_rgb(255, 255, 255),
            (Self::TertiarySystemBackground, true) => Color::from_hex("#3a3a3c").unwrap(),
            (Self::SystemGroupedBackground, false) => Color::from_hex("#f2f2f7").unwrap(),
            (Self::SystemGroupedBackground, true) => Color::from_rgb(0, 0, 0),
            (Self::SecondarySystemGroupedBackground, false) => Color::from_rgb(255, 255, 255),
            (Self::SecondarySystemGroupedBackground, true) => Color::from_hex("#1d1d1d").unwrap(),
            (Self::TertiarySystemGroupedBackground, false) => Color::from_hex("#f2f2f7").unwrap(),
            (Self::TertiarySystemGroupedBackground, true) => Color::from_hex("#2c2c2e").unwrap(),
        }
    }
}

/// Palette widget — "UIKit Content Background colors — The UIKit colors that
/// are also used in other components like List and...".
pub struct UIKitContentBackgroundColors {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl UIKitContentBackgroundColors {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 140.0, 64.0)
    }
}

impl Default for UIKitContentBackgroundColors {
    fn default() -> Self { Self::new() }
}

impl ViewContent for UIKitContentBackgroundColors {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 {
            outer.set_size_request(frame.width as i32, 64);
        }

        // Directly on window — no extra background
        let container = gtk::Box::new(gtk::Orientation::Vertical, 6);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        row.set_halign(gtk::Align::Center);
        row.set_valign(gtk::Align::Center);
        let swatches = [
            UIKitBackgroundColor::SystemBackground.color(is_dark),
            UIKitBackgroundColor::SecondarySystemBackground.color(is_dark),
            UIKitBackgroundColor::SystemGroupedBackground.color(is_dark),
            UIKitBackgroundColor::SecondarySystemGroupedBackground.color(is_dark),
        ];

        for col in swatches {
            let sw = gtk::Box::new(gtk::Orientation::Vertical, 0);
            sw.set_size_request(20, 20);
            sw.add_css_class("bg-swatch");
            let border = if is_dark {
                if col.r < 0.12 && col.g < 0.12 && col.b < 0.12 { "1px solid rgba(255,255,255,0.12)" } else { "none" }
            } else {
                if col.r > 0.95 && col.g > 0.95 && col.b > 0.95 { "1px solid rgba(0,0,0,0.12)" } else { "none" }
            };
            uikit::widget::apply_css(
                &sw,
                &format!(
                    ".bg-swatch {{ background: {}; border: {}; border-radius: 3px; min-width: 20px; min-height: 20px; }}",
                    col.to_css(),
                    border
                ),
            );
            row.append(&sw);
        }

        container.append(&row);

        // Content chip below — adaptive
        let chip = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        chip.set_halign(gtk::Align::Center);
        chip.add_css_class("bg-chip");
        let chip_bg = if is_dark { "#2c2c2e" } else { "#ffffff" };
        let chip_border = if is_dark { "none" } else { "1px solid rgba(0,0,0,0.08)" };
        uikit::widget::apply_css(
            &chip,
            &format!(".bg-chip {{ background: {}; border: {}; border-radius: 6px; padding: 3px 10px; }}", chip_bg, chip_border),
        );
        let lbl = gtk::Label::new(Some("content"));
        lbl.add_css_class("bg-chip-lbl");
        let lbl_col = if is_dark { "rgba(255,255,255,0.55)" } else { "rgba(60,60,67,0.6)" };
        uikit::widget::apply_css(
            &lbl,
            &format!(".bg-chip-lbl {{ color: {}; font-family: 'SF Pro Display'; font-size: 9px; }}", lbl_col),
        );
        chip.append(&lbl);
        container.append(&chip);

        outer.append(&container);
        outer.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size { Size::new(140.0, 64.0) }
}

impl Widget for UIKitContentBackgroundColors {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, 140.0, 64.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn background_distinct() {
        let a = UIKitBackgroundColor::SystemBackground.color(true);
        let b = UIKitBackgroundColor::SecondarySystemBackground.color(true);
        assert_ne!(a, b);
        assert_eq!(UIKitBackgroundColor::all().len(), 6);
    }
}
