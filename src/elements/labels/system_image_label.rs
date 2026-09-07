//! SystemImageLabel — Creates a label with a system icon and a localized title.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// SystemImageLabel — initializer — Creates a label with a system icon image
/// and a title generated from a localized string.
pub struct SystemImageLabel {
    id: WidgetId,
    system_name: String,
    title_key: String,
    position_mode: PositionMode,
    position: Position,
}

impl SystemImageLabel {
    pub fn new() -> Self {
        Self { id: next_widget_id(), system_name: "square.grid.2x2".to_string(), title_key: "Foo".to_string(), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn system_name(mut self, v: impl Into<String>) -> Self { self.system_name = v.into(); self }
    pub fn title_key(mut self, v: impl Into<String>) -> Self { self.title_key = v.into(); self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for SystemImageLabel { fn default() -> Self { Self::new() } }

impl ViewContent for SystemImageLabel {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("lbl-sys-phone");
        uikit::widget::apply_css(&phone, ".lbl-sys-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        row.set_halign(gtk::Align::Center);
        row.set_valign(gtk::Align::Center);
        row.set_vexpand(true);
        let icon = gtk::Label::new(Some("▦"));
        icon.set_valign(gtk::Align::Center);
        icon.add_css_class("lbl-sys-icon");
        uikit::widget::apply_css(&icon, ".lbl-sys-icon { color: rgba(255,255,255,0.6); font-family: 'SF Pro Display'; font-size: 11px; }");
        row.append(&icon);
        let lbl = gtk::Label::new(Some(self.title_key.as_str()));
        lbl.set_valign(gtk::Align::Center);
        lbl.add_css_class("lbl-sys-lbl");
        uikit::widget::apply_css(&lbl, ".lbl-sys-lbl { color: rgba(255,255,255,0.6); font-family: 'SF Pro Display'; font-size: 8px; }");
        row.append(&lbl);
        phone.append(&row);
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for SystemImageLabel {
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
    fn sys_exists() { let _ = SystemImageLabel::new().system_name("star"); }
}
