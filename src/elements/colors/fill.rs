//! UIKit Fill Colors — systemFill variants.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// UIKit fill colors — mirrors `UIColor.systemFill`,
/// `secondarySystemFill`, `tertiarySystemFill`, `quaternarySystemFill`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIKitFillColor {
    SystemFill,
    SecondarySystemFill,
    TertiarySystemFill,
    QuaternarySystemFill,
}

impl UIKitFillColor {
    pub fn all() -> [Self; 4] {
        [
            Self::SystemFill,
            Self::SecondarySystemFill,
            Self::TertiarySystemFill,
            Self::QuaternarySystemFill,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::SystemFill => "systemFill",
            Self::SecondarySystemFill => "secondarySystemFill",
            Self::TertiarySystemFill => "tertiarySystemFill",
            Self::QuaternarySystemFill => "quaternarySystemFill",
        }
    }

    pub fn color(self, is_dark: bool) -> Color {
        match (self, is_dark) {
            (Self::SystemFill, false) => Color::from_rgba(120, 120, 128, 51), // 20%
            (Self::SystemFill, true) => Color::from_rgba(120, 120, 128, 92),  // 36%
            (Self::SecondarySystemFill, false) => Color::from_rgba(120, 120, 128, 41), // 16%
            (Self::SecondarySystemFill, true) => Color::from_rgba(120, 120, 128, 82),  // 32%
            (Self::TertiarySystemFill, false) => Color::from_rgba(118, 118, 128, 31), // 12%
            (Self::TertiarySystemFill, true) => Color::from_rgba(118, 118, 128, 61),  // 24%
            (Self::QuaternarySystemFill, false) => Color::from_rgba(116, 116, 128, 20), // 8%
            (Self::QuaternarySystemFill, true) => Color::from_rgba(116, 116, 128, 46), // 18%
        }
    }
}

/// Palette widget — "UIKit Fill Colors".
pub struct UIKitFillColors {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl UIKitFillColors {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 120.0, 48.0)
    }
}

impl Default for UIKitFillColors {
    fn default() -> Self { Self::new() }
}

impl ViewContent for UIKitFillColors {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 48); }

        // Directly on window — no extra background
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        for variant in UIKitFillColor::all() {
            let c = variant.color(is_dark);
            let sw = gtk::Box::new(gtk::Orientation::Vertical, 0);
            sw.set_size_request(22, 22);
            sw.add_css_class("fill-swatch");
            uikit::widget::apply_css(
                &sw,
                &format!(
                    ".fill-swatch {{ background: {}; border-radius: 4px; min-width: 22px; min-height: 22px; }}",
                    c.to_css()
                ),
            );
            container.append(&sw);
        }

        outer.append(&container);
        outer.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size { Size::new(120.0, 48.0) }
}

impl Widget for UIKitFillColors {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, 120.0, 48.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fill_distinct() {
        let a = UIKitFillColor::SystemFill.color(true);
        let b = UIKitFillColor::QuaternarySystemFill.color(true);
        assert_ne!(a, b);
        assert!(a.a > b.a);
    }
}
