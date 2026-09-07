//! Capsule — A capsule shape (stadium) filling its frame.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// Capsule — modifier — A capsule (stadium) shape filling its frame.
pub struct Capsule {
    id: WidgetId,
    fill: Color,
    position_mode: PositionMode,
    position: Position,
}

impl Capsule {
    pub fn new() -> Self {
        Self { id: next_widget_id(), fill: Color::from_hex("#0A84FF").unwrap_or(Color::new(0.04, 0.52, 1.0, 1.0)), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn fill(mut self, c: Color) -> Self { self.fill = c; self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for Capsule { fn default() -> Self { Self::new() } }

impl ViewContent for Capsule {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(140, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("shp-capsule-phone");
        uikit::widget::apply_css(&phone, ".shp-capsule-phone { background: #0b0b0e; border-radius: 12px; min-width: 140px; min-height: 110px; }");
        let shape = gtk::Box::new(gtk::Orientation::Vertical, 0);
        shape.set_size_request(112, 44);
        shape.set_halign(gtk::Align::Center);
        shape.set_valign(gtk::Align::Center);
        shape.set_vexpand(true);
        shape.add_css_class("shp-capsule");
        uikit::widget::apply_css(&shape, &format!(".shp-capsule {{ background: {}; border-radius: 999px; min-width: 112px; min-height: 44px; margin-top: 33px; margin-bottom: 33px; }}", self.fill.to_css()));
        let lbl = gtk::Label::new(Some("Foo"));
        lbl.set_halign(gtk::Align::Center);
        lbl.set_valign(gtk::Align::Center);
        lbl.set_vexpand(true);
        lbl.add_css_class("shp-capsule-lbl");
        uikit::widget::apply_css(&lbl, ".shp-capsule-lbl { color: white; font-family: 'SF Pro Display'; font-size: 8px; }");
        shape.append(&lbl);
        phone.append(&shape);
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for Capsule {
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
    fn capsule_exists() { let _ = Capsule::new(); }
}
