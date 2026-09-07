//! LabeledContent — Creates a labeled informational view.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// LabeledContent — initializer — Creates a labeled informational view.
pub struct LabeledContent {
    id: WidgetId,
    title: String,
    value: String,
    position_mode: PositionMode,
    position: Position,
}

impl LabeledContent {
    pub fn new() -> Self {
        Self { id: next_widget_id(), title: "Foo".to_string(), value: "Bar".to_string(), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn title(mut self, v: impl Into<String>) -> Self { self.title = v.into(); self }
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = v.into(); self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for LabeledContent { fn default() -> Self { Self::new() } }

impl ViewContent for LabeledContent {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("lc-plain-phone");
        uikit::widget::apply_css(&phone, ".lc-plain-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        let pill = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        pill.set_size_request(160, 30);
        pill.set_halign(gtk::Align::Center);
        pill.set_valign(gtk::Align::Center);
        pill.set_vexpand(true);
        pill.add_css_class("lc-plain-pill");
        uikit::widget::apply_css(&pill, ".lc-plain-pill { background: #1c1c1e; border-radius: 15px; min-width: 160px; min-height: 30px; margin-top: 40px; margin-bottom: 40px; padding: 4px 12px; }");
        let title = gtk::Label::new(Some(self.title.as_str()));
        title.set_halign(gtk::Align::Start);
        title.set_hexpand(true);
        title.add_css_class("lc-plain-title");
        uikit::widget::apply_css(&title, ".lc-plain-title { color: rgba(255,255,255,0.75); font-family: 'SF Pro Display'; font-size: 8px; }");
        pill.append(&title);
        let val = gtk::Label::new(Some(self.value.as_str()));
        val.set_halign(gtk::Align::End);
        val.add_css_class("lc-plain-val");
        uikit::widget::apply_css(&val, ".lc-plain-val { color: rgba(255,255,255,0.45); font-family: 'SF Pro Display'; font-size: 8px; }");
        pill.append(&val);
        phone.append(&pill);
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for LabeledContent {
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
    fn plain_exists() { let _ = LabeledContent::new().value("Baz"); }
}
