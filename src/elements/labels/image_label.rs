//! ImageLabel — Creates a label with an icon image and a localized title.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// ImageLabel — initializer — Creates a label with an icon image and a title
/// generated from a localized string.
pub struct ImageLabel {
    id: WidgetId,
    title_key: String,
    position_mode: PositionMode,
    position: Position,
}

impl ImageLabel {
    pub fn new() -> Self {
        Self { id: next_widget_id(), title_key: "Foo".to_string(), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn title_key(mut self, v: impl Into<String>) -> Self { self.title_key = v.into(); self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for ImageLabel { fn default() -> Self { Self::new() } }

impl ViewContent for ImageLabel {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("lbl-img-phone");
        uikit::widget::apply_css(&phone, ".lbl-img-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row.set_halign(gtk::Align::Center);
        row.set_valign(gtk::Align::Center);
        row.set_vexpand(true);
        // Icon image hint: two cat boxes with party hats
        let cats = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        cats.set_valign(gtk::Align::Center);
        for (i, col) in ["#d8b48a", "#c9a86a"].iter().enumerate() {
            let cat = gtk::Box::new(gtk::Orientation::Vertical, 0);
            cat.set_size_request(34, 40);
            cat.add_css_class(&format!("lbl-img-cat{}", i));
            uikit::widget::apply_css(&cat, &format!(".lbl-img-cat{} {{ background: {}; border-radius: 8px; min-width: 34px; min-height: 40px; }}", i, col));
            let hat = gtk::Label::new(Some("🎉"));
            hat.set_halign(gtk::Align::Center);
            uikit::widget::apply_css(&hat, ".lbl-img-hat { font-size: 10px; }");
            hat.add_css_class("lbl-img-hat");
            cat.append(&hat);
            cats.append(&cat);
        }
        row.append(&cats);
        let lbl = gtk::Label::new(Some(self.title_key.as_str()));
        lbl.set_valign(gtk::Align::Center);
        lbl.add_css_class("lbl-img-lbl");
        uikit::widget::apply_css(&lbl, ".lbl-img-lbl { color: rgba(255,255,255,0.85); font-family: 'SF Pro Display'; font-size: 8px; }");
        row.append(&lbl);
        phone.append(&row);
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for ImageLabel {
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
    fn image_exists() { let _ = ImageLabel::new().title_key("Bar"); }
}
