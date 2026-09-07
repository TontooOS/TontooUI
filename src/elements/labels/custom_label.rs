//! CustomLabel — Creates a label with a custom title and icon.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// CustomLabel — initializer — Creates a label with a custom title and icon.
pub struct CustomLabel {
    id: WidgetId,
    title: String,
    position_mode: PositionMode,
    position: Position,
}

impl CustomLabel {
    pub fn new() -> Self {
        Self { id: next_widget_id(), title: "Foo Bar".to_string(), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn title(mut self, v: impl Into<String>) -> Self { self.title = v.into(); self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for CustomLabel { fn default() -> Self { Self::new() } }

impl ViewContent for CustomLabel {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("lbl-custom-phone");
        uikit::widget::apply_css(&phone, ".lbl-custom-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        row.set_halign(gtk::Align::Center);
        row.set_valign(gtk::Align::Center);
        row.set_vexpand(true);
        // Custom icon: two stacked rings hinting a custom glyph
        let icon = gtk::Box::new(gtk::Orientation::Vertical, 0);
        icon.set_size_request(14, 14);
        icon.set_valign(gtk::Align::Center);
        icon.add_css_class("lbl-custom-icon");
        uikit::widget::apply_css(&icon, ".lbl-custom-icon { border: 1px solid rgba(255,255,255,0.65); border-radius: 7px; min-width: 12px; min-height: 12px; }");
        row.append(&icon);
        let lbl = gtk::Label::new(Some(self.title.as_str()));
        lbl.add_css_class("lbl-custom-lbl");
        uikit::widget::apply_css(&lbl, ".lbl-custom-lbl { color: rgba(255,255,255,0.85); font-family: 'SF Pro Display'; font-size: 8px; }");
        row.append(&lbl);
        phone.append(&row);
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for CustomLabel {
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
    fn custom_exists() { let _ = CustomLabel::new().title("Hello"); }
}
