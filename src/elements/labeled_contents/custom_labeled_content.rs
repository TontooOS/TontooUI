//! CustomLabeledContent — Creates a standard labeled element with a custom value view.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// CustomLabeledContent — initializer — Creates a standard labeled element,
/// with a view that conveys the value of the content.
pub struct CustomLabeledContent {
    id: WidgetId,
    title: String,
    subtitle: String,
    value: String,
    position_mode: PositionMode,
    position: Position,
}

impl CustomLabeledContent {
    pub fn new() -> Self {
        Self { id: next_widget_id(), title: "Foo".to_string(), subtitle: "Sub".to_string(), value: "bar".to_string(), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn title(mut self, v: impl Into<String>) -> Self { self.title = v.into(); self }
    pub fn subtitle(mut self, v: impl Into<String>) -> Self { self.subtitle = v.into(); self }
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = v.into(); self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for CustomLabeledContent { fn default() -> Self { Self::new() } }

impl ViewContent for CustomLabeledContent {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("lc-custom-phone");
        uikit::widget::apply_css(&phone, ".lc-custom-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        let pill = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        pill.set_size_request(160, 34);
        pill.set_halign(gtk::Align::Center);
        pill.set_valign(gtk::Align::Center);
        pill.set_vexpand(true);
        pill.add_css_class("lc-custom-pill");
        uikit::widget::apply_css(&pill, ".lc-custom-pill { background: #1c1c1e; border-radius: 17px; min-width: 160px; min-height: 34px; margin-top: 38px; margin-bottom: 38px; padding: 4px 10px; }");
        let star = gtk::Label::new(Some("☆"));
        star.set_valign(gtk::Align::Center);
        star.add_css_class("lc-custom-star");
        uikit::widget::apply_css(&star, ".lc-custom-star { color: #0A84FF; font-family: 'SF Pro Display'; font-size: 10px; }");
        pill.append(&star);
        let texts = gtk::Box::new(gtk::Orientation::Vertical, 0);
        texts.set_valign(gtk::Align::Center);
        texts.set_hexpand(true);
        let title = gtk::Label::new(Some(self.title.as_str()));
        title.set_halign(gtk::Align::Start);
        title.add_css_class("lc-custom-title");
        uikit::widget::apply_css(&title, ".lc-custom-title { color: rgba(255,255,255,0.9); font-family: 'SF Pro Display'; font-size: 8px; }");
        texts.append(&title);
        let sub = gtk::Label::new(Some(self.subtitle.as_str()));
        sub.set_halign(gtk::Align::Start);
        sub.add_css_class("lc-custom-sub");
        uikit::widget::apply_css(&sub, ".lc-custom-sub { color: rgba(255,255,255,0.45); font-family: 'SF Pro Display'; font-size: 6px; }");
        texts.append(&sub);
        pill.append(&texts);
        let val = gtk::Label::new(Some(self.value.as_str()));
        val.set_valign(gtk::Align::Center);
        val.add_css_class("lc-custom-val");
        uikit::widget::apply_css(&val, ".lc-custom-val { color: #0A84FF; font-family: 'SF Pro Display'; font-size: 8px; }");
        pill.append(&val);
        phone.append(&pill);
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for CustomLabeledContent {
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
    fn custom_exists() { let _ = CustomLabeledContent::new().value("baz"); }
}
