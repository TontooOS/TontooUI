//! Semantic Colors — Primary / Secondary text colors.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// Semantic colors — mirrors SwiftUI `Color.primary` / `Color.secondary`
/// (adaptive: black/white and secondary opacity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticColor {
    Primary,
    Secondary,
}

impl SemanticColor {
    pub fn all() -> [Self; 2] { [Self::Primary, Self::Secondary] }

    pub fn label(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
        }
    }

    pub fn color(self, is_dark: bool) -> Color {
        match (self, is_dark) {
            (Self::Primary, false) => Color::from_rgb(0, 0, 0),
            (Self::Primary, true) => Color::from_rgb(255, 255, 255),
            (Self::Secondary, false) => Color::from_rgba(60, 60, 67, 153),
            (Self::Secondary, true) => Color::from_rgba(235, 235, 245, 153),
        }
    }
}

/// Palette widget — "Semantic Colors — Primary Text Color / Secondary Text Color".
pub struct SemanticColors {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl SemanticColors {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 160.0, 48.0)
    }
}

impl Default for SemanticColors {
    fn default() -> Self { Self::new() }
}

impl ViewContent for SemanticColors {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 48); }

        // Directly on window — no extra background
        let container = gtk::Box::new(gtk::Orientation::Vertical, 4);
        container.set_halign(gtk::Align::Center);
        container.set_valign(gtk::Align::Center);

        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        for variant in SemanticColor::all() {
            let text = match variant {
                SemanticColor::Primary => "Primary Text Color",
                SemanticColor::Secondary => "Secondary Text Color",
            };
            let size = if variant == SemanticColor::Primary { 11 } else { 10 };
            let lbl = gtk::Label::new(Some(text));
            lbl.set_halign(gtk::Align::Center);
            lbl.add_css_class("sem-lbl");
            uikit::widget::apply_css(
                &lbl,
                &format!(
                    ".sem-lbl {{ color: {}; font-family: 'SF Pro Display'; font-size: {}px; }}",
                    variant.color(is_dark).to_css(),
                    size
                ),
            );
            container.append(&lbl);
        }

        outer.append(&container);
        outer.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size { Size::new(160.0, 48.0) }
}

impl Widget for SemanticColors {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, 160.0, 48.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn semantic_primary_is_opaque() {
        assert_eq!(SemanticColor::Primary.color(false).a, 1.0);
        assert!(SemanticColor::Secondary.color(true).a < 1.0);
    }
}
