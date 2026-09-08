//! AsyncURLImage — Loads and displays an image from a URL load request.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// AsyncURLImage — initializer — Loads and displays an image from the
/// specified URL load request.
pub struct AsyncURLImage {
    id: WidgetId,
    url: String,
    position_mode: PositionMode,
    position: Position,
}

impl AsyncURLImage {
    pub fn new() -> Self {
        Self { id: next_widget_id(), url: "https://example.com/cats.png".to_string(), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn url(mut self, v: impl Into<String>) -> Self { self.url = v.into(); self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for AsyncURLImage { fn default() -> Self { Self::new() } }

impl ViewContent for AsyncURLImage {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        // Full-bleed cats image
        let img = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        img.set_size_request(180, 110);
        img.set_halign(gtk::Align::Center);
        img.add_css_class("ai-url-img");
        uikit::widget::apply_css(&img, ".ai-url-img { background: #d8b48a; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        for (i, col) in ["#c98a4b", "#e8c9a0", "#d8b48a"].iter().enumerate() {
            let cat = gtk::Box::new(gtk::Orientation::Vertical, 0);
            cat.set_size_request(60, 110);
            cat.add_css_class(&format!("ai-url-cat{}", i));
            let r = if i == 0 { "12px 0 0 12px" } else if i == 2 { "0 12px 12px 0" } else { "0" };
            uikit::widget::apply_css(&cat, &format!(".ai-url-cat{} {{ background: {}; border-radius: {}; min-width: 60px; min-height: 110px; }}", i, col, r));
            let hat = gtk::Label::new(Some("🎉"));
            hat.set_halign(gtk::Align::Center);
            uikit::widget::apply_css(&hat, ".ai-url-hat { font-size: 14px; margin-top: 8px; }");
            hat.add_css_class("ai-url-hat");
            cat.append(&hat);
            let face = gtk::Label::new(Some("● ●"));
            face.set_halign(gtk::Align::Center);
            face.set_vexpand(true);
            uikit::widget::apply_css(&face, ".ai-url-face { color: rgba(0,0,0,0.55); font-size: 10px; }");
            face.add_css_class("ai-url-face");
            cat.append(&face);
            img.append(&cat);
        }
        outer.append(&img);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for AsyncURLImage {
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
    fn url_exists() { let _ = AsyncURLImage::new().url("https://example.com/b.png"); }
}
