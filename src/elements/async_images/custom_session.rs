//! CustomSessionAsyncImage — Modifier adding a URL session for async images.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// CustomSessionAsyncImage — modifier — A modifier that adds a URL session
/// for asynchronous images contained in the view.
pub struct CustomSessionAsyncImage {
    id: WidgetId,
    session: String,
    position_mode: PositionMode,
    position: Position,
}

impl CustomSessionAsyncImage {
    pub fn new() -> Self {
        Self { id: next_widget_id(), session: "default".to_string(), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn session(mut self, v: impl Into<String>) -> Self { self.session = v.into(); self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for CustomSessionAsyncImage { fn default() -> Self { Self::new() } }

impl ViewContent for CustomSessionAsyncImage {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        // Same full-bleed cats with a subtle session tint overlay
        let overlay = gtk::Overlay::new();
        overlay.set_size_request(180, 110);
        let img = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        img.set_size_request(180, 110);
        img.add_css_class("ai-sess-img");
        uikit::widget::apply_css(&img, ".ai-sess-img { background: #c98a4b; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        for (i, col) in ["#c98a4b", "#e8c9a0", "#d8b48a"].iter().enumerate() {
            let cat = gtk::Box::new(gtk::Orientation::Vertical, 0);
            cat.set_size_request(60, 110);
            cat.add_css_class(&format!("ai-sess-cat{}", i));
            let r = if i == 0 { "12px 0 0 12px" } else if i == 2 { "0 12px 12px 0" } else { "0" };
            uikit::widget::apply_css(&cat, &format!(".ai-sess-cat{} {{ background: {}; border-radius: {}; min-width: 60px; min-height: 110px; }}", i, col, r));
            let hat = gtk::Label::new(Some("🎉"));
            hat.set_halign(gtk::Align::Center);
            uikit::widget::apply_css(&hat, ".ai-sess-hat { font-size: 14px; margin-top: 8px; }");
            hat.add_css_class("ai-sess-hat");
            cat.append(&hat);
            img.append(&cat);
        }
        overlay.set_child(Some(&img));
        // Session tint to hint a custom configuration
        let tint = gtk::Box::new(gtk::Orientation::Vertical, 0);
        tint.set_size_request(180, 110);
        tint.add_css_class("ai-sess-tint");
        uikit::widget::apply_css(&tint, ".ai-sess-tint { background: rgba(10,132,255,0.12); border-radius: 12px; min-width: 180px; min-height: 110px; }");
        overlay.add_overlay(&tint);
        outer.append(&overlay);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for CustomSessionAsyncImage {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0, 0.0, 180.0, 110.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn session_exists() { let _ = CustomSessionAsyncImage::new().session("ephemeral"); }
}
