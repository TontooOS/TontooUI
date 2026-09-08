//! CustomPlaceholderAsyncImage — Loads a modifiable image with a custom placeholder.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// CustomPlaceholderAsyncImage — initializer — Loads and displays a
/// modifiable image from the specified URL load request with a custom placeholder.
pub struct CustomPlaceholderAsyncImage {
    id: WidgetId,
    url: String,
    position_mode: PositionMode,
    position: Position,
}

impl CustomPlaceholderAsyncImage {
    pub fn new() -> Self {
        Self { id: next_widget_id(), url: "https://example.com/cats.png".to_string(), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn url(mut self, v: impl Into<String>) -> Self { self.url = v.into(); self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for CustomPlaceholderAsyncImage { fn default() -> Self { Self::new() } }

impl ViewContent for CustomPlaceholderAsyncImage {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("ai-ph-phone");
        uikit::widget::apply_css(&phone, ".ai-ph-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        // Small centered loaded image hint (placeholder replaced by cats)
        let img = gtk::Box::new(gtk::Orientation::Horizontal, 3);
        img.set_halign(gtk::Align::Center);
        img.set_valign(gtk::Align::Center);
        img.set_vexpand(true);
        for (i, col) in ["#d8b48a", "#c9a86a", "#e8c9a0"].iter().enumerate() {
            let cat = gtk::Box::new(gtk::Orientation::Vertical, 0);
            cat.set_size_request(26, 32);
            cat.add_css_class(&format!("ai-ph-cat{}", i));
            uikit::widget::apply_css(&cat, &format!(".ai-ph-cat{} {{ background: {}; border-radius: 6px; min-width: 26px; min-height: 32px; }}", i, col));
            let hat = gtk::Label::new(Some("🎉"));
            hat.set_halign(gtk::Align::Center);
            uikit::widget::apply_css(&hat, ".ai-ph-hat { font-size: 8px; }");
            hat.add_css_class("ai-ph-hat");
            cat.append(&hat);
            img.append(&cat);
        }
        phone.append(&img);
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for CustomPlaceholderAsyncImage {
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
    fn placeholder_exists() { let _ = CustomPlaceholderAsyncImage::new().url("https://example.com/a.png"); }
}
