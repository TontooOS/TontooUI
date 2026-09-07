//! ContainerRelativeShape — A shape replaced by an inset version of the container shape.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// ContainerRelativeShape — modifier — A shape that is replaced by an inset
/// version of the current container shape.
pub struct ContainerRelativeShape {
    id: WidgetId,
    inset: f32,
    position_mode: PositionMode,
    position: Position,
}

impl ContainerRelativeShape {
    pub fn new() -> Self {
        Self { id: next_widget_id(), inset: 8.0, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn inset(mut self, v: f32) -> Self { self.inset = v; self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for ContainerRelativeShape { fn default() -> Self { Self::new() } }

impl ViewContent for ContainerRelativeShape {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(140, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("shp-crs-phone");
        uikit::widget::apply_css(&phone, ".shp-crs-phone { background: #0b0b0e; border-radius: 12px; min-width: 140px; min-height: 110px; }");
        // Outer container-relative blue shape
        let blue = gtk::Box::new(gtk::Orientation::Vertical, 0);
        blue.set_size_request(84, 88);
        blue.set_halign(gtk::Align::Center);
        blue.set_valign(gtk::Align::Center);
        blue.set_vexpand(true);
        blue.add_css_class("shp-crs-blue");
        uikit::widget::apply_css(&blue, ".shp-crs-blue { background: #0A84FF; border-radius: 22px; min-width: 84px; min-height: 88px; margin-top: 11px; margin-bottom: 11px; }");
        let top = gtk::Label::new(Some("Foo"));
        top.set_halign(gtk::Align::Center);
        top.add_css_class("shp-crs-top");
        uikit::widget::apply_css(&top, ".shp-crs-top { color: white; font-family: 'SF Pro Display'; font-size: 7px; margin-top: 6px; }");
        blue.append(&top);
        // Middle inset red shape
        let red = gtk::Box::new(gtk::Orientation::Vertical, 0);
        red.set_size_request(56, 56);
        red.set_halign(gtk::Align::Center);
        red.add_css_class("shp-crs-red");
        uikit::widget::apply_css(&red, ".shp-crs-red { background: #e5484d; border-radius: 14px; min-width: 56px; min-height: 56px; }");
        let mid = gtk::Label::new(Some("Bar"));
        mid.set_halign(gtk::Align::Center);
        mid.add_css_class("shp-crs-mid");
        uikit::widget::apply_css(&mid, ".shp-crs-mid { color: white; font-family: 'SF Pro Display'; font-size: 7px; margin-top: 4px; }");
        red.append(&mid);
        // Inner inset green shape
        let green = gtk::Box::new(gtk::Orientation::Vertical, 0);
        green.set_size_request(36, 28);
        green.set_halign(gtk::Align::Center);
        green.add_css_class("shp-crs-green");
        uikit::widget::apply_css(&green, ".shp-crs-green { background: #30d158; border-radius: 8px; min-width: 36px; min-height: 28px; }");
        let inner = gtk::Label::new(Some("Foo"));
        inner.set_halign(gtk::Align::Center);
        inner.set_valign(gtk::Align::Center);
        inner.set_vexpand(true);
        inner.add_css_class("shp-crs-inner");
        uikit::widget::apply_css(&inner, ".shp-crs-inner { color: white; font-family: 'SF Pro Display'; font-size: 7px; }");
        green.append(&inner);
        red.append(&green);
        blue.append(&red);
        phone.append(&blue);
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for ContainerRelativeShape {
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
    fn crs_exists() { let _ = ContainerRelativeShape::new().inset(4.0); }
}
