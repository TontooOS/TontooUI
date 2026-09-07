//! UIKit Text Colors — placeholderText etc.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// UIKit text colors — mirrors `UIColor.placeholderText` and the
/// system text palette used in `UITextField` / `UILabel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIKitTextColor {
    PlaceholderText,
    Link,
}

impl UIKitTextColor {
    pub fn all() -> [Self; 2] { [Self::PlaceholderText, Self::Link] }

    pub fn label(self) -> &'static str {
        match self {
            Self::PlaceholderText => "placeholderText",
            Self::Link => "link",
        }
    }

    pub fn color(self, is_dark: bool) -> Color {
        match (self, is_dark) {
            (Self::PlaceholderText, false) => Color::from_rgba(60, 60, 67, 77), // 30%
            (Self::PlaceholderText, true) => Color::from_rgba(235, 235, 245, 77),
            (Self::Link, _) => Color::from_rgb(10, 132, 255),
        }
    }
}

/// Palette widget — "UIKit Text Colors — The UIKit text colors that are also
/// used in other components, such as the ...".
pub struct UIKitTextColors {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl UIKitTextColors {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 140.0, 56.0)
    }
}

impl Default for UIKitTextColors {
    fn default() -> Self { Self::new() }
}

impl ViewContent for UIKitTextColors {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 {
            outer.set_size_request(frame.width as i32, 56);
        }

        // Directly on window — no extra background
        let container = gtk::Box::new(gtk::Orientation::Vertical, 6);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let placeholder = UIKitTextColor::PlaceholderText.color(is_dark);
        // Two lines of placeholder text like screenshot "Placeholder Paseholder"
        for text in ["Placeholder", "Paseholder"] {
            let lbl = gtk::Label::new(Some(text));
            lbl.set_halign(gtk::Align::Center);
            lbl.add_css_class("txt-lbl");
            uikit::widget::apply_css(
                &lbl,
                &format!(
                    ".txt-lbl {{ color: {}; font-family: 'SF Pro Display'; font-size: 11px; }}",
                    placeholder.to_css()
                ),
            );
            container.append(&lbl);
        }

        outer.append(&container);
        outer.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size { Size::new(140.0, 56.0) }
}

impl Widget for UIKitTextColors {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, 140.0, 56.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn text_placeholder_alpha() {
        let c = UIKitTextColor::PlaceholderText.color(false);
        assert!(c.a < 0.35 && c.a > 0.25);
    }
}
