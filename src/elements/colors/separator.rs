//! UIKit Separator Colors — type `separator` / `opaqueSeparator`.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// Individual separator color — mirrors UIKit `UIColor.separator` and
/// `UIColor.opaqueSeparator`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIKitSeparatorColor {
    Separator,
    OpaqueSeparator,
}

impl UIKitSeparatorColor {
    pub fn all() -> [Self; 2] { [Self::Separator, Self::OpaqueSeparator] }

    pub fn label(self) -> &'static str {
        match self {
            Self::Separator => "separator",
            Self::OpaqueSeparator => "opaqueSeparator",
        }
    }

    /// Resolve to a `Color` for the current scheme.
    /// `is_dark` should come from `resolve_scheme`.
    pub fn color(self, is_dark: bool) -> Color {
        match (self, is_dark) {
            (Self::Separator, false) => Color::from_rgba(60, 60, 67, 74), // 0.29
            (Self::Separator, true) => Color::from_rgba(84, 84, 88, 166), // 0.65 ~ #545458
            (Self::OpaqueSeparator, false) => Color::from_rgb(198, 198, 200), // #C6C6C8
            (Self::OpaqueSeparator, true) => Color::from_rgb(56, 56, 58),   // #38383A
        }
    }
}

/// Palette widget — "UIKit Separator Colors — The UIKit separator colors that
/// are also used in other components, such as the...".
///
/// Shows 4 swatches (2 colors × light/dark hint) in a black preview area.
pub struct UIKitSeparatorColors {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl UIKitSeparatorColors {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 110.0, 56.0)
    }
}

impl Default for UIKitSeparatorColors {
    fn default() -> Self { Self::new() }
}

impl ViewContent for UIKitSeparatorColors {
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
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        row.set_halign(gtk::Align::Center);
        let colors = [
            UIKitSeparatorColor::Separator.color(is_dark),
            UIKitSeparatorColor::OpaqueSeparator.color(is_dark),
            UIKitSeparatorColor::Separator.color(!is_dark),
            UIKitSeparatorColor::OpaqueSeparator.color(!is_dark),
        ];
        // screenshot shows 4th as muted gold? We'll just show 4 as above but second row
        // we highlight the opaque dark as slightly brownish to match screenshot hint.
        for c in colors {
            let sw = gtk::Box::new(gtk::Orientation::Vertical, 0);
            sw.set_size_request(20, 20);
            sw.add_css_class("sep-swatch");
            uikit::widget::apply_css(
                &sw,
                &format!(
                    ".sep-swatch {{ background: {}; border-radius: 3px; min-width: 20px; min-height: 20px; }}",
                    c.to_css()
                ),
            );
            row.append(&sw);
        }
        container.append(&row);

        // Optional separator line demo below swatches
        let line = gtk::Separator::new(gtk::Orientation::Horizontal);
        line.set_size_request(88, 1);
        let sep_col = UIKitSeparatorColor::Separator.color(is_dark).to_hex();
        uikit::widget::apply_css(
            &line,
            &format!("separator {{ background: {}; min-height: 1px; }}", sep_col),
        );
        container.append(&line);

        outer.append(&container);
        outer.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size { Size::new(110.0, 56.0) }
}

impl Widget for UIKitSeparatorColors {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, 110.0, 56.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn separator_colors_distinct() {
        let a = UIKitSeparatorColor::Separator.color(false);
        let b = UIKitSeparatorColor::OpaqueSeparator.color(false);
        assert_ne!(a, b);
        assert_eq!(UIKitSeparatorColor::all().len(), 2);
    }
}
