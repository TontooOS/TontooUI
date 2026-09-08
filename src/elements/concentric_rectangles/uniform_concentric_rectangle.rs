//! UniformConcentricRectangle — Create a rectangle with the same corner style on four corners.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// UniformConcentricRectangle — initializer — Create a rectangle with the
/// same corner style set on four corners.
pub struct UniformConcentricRectangle {
    id: WidgetId,
    corner_radius: f32,
    inset: f32,
    position_mode: PositionMode,
    position: Position,
}

impl UniformConcentricRectangle {
    pub fn new() -> Self {
        Self { id: next_widget_id(), corner_radius: 20.0, inset: 8.0, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn corner_radius(mut self, v: f32) -> Self { self.corner_radius = v; self }
    pub fn inset(mut self, v: f32) -> Self { self.inset = v; self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for UniformConcentricRectangle { fn default() -> Self { Self::new() } }

impl ViewContent for UniformConcentricRectangle {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(150, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("cr-uni-phone");
        uikit::widget::apply_css(&phone, ".cr-uni-phone { background: #0b0b0e; border-radius: 12px; min-width: 150px; min-height: 110px; }");
        let overlay = gtk::Overlay::new();
        overlay.set_size_request(150, 110);
        overlay.set_child(Some(&phone));
        // Translucent gray outer rect (uniform corners)
        let gray = gtk::Box::new(gtk::Orientation::Vertical, 0);
        gray.set_size_request(100, 78);
        gray.set_halign(gtk::Align::Center);
        gray.set_valign(gtk::Align::Start);
        gray.add_css_class("cr-uni-gray");
        uikit::widget::apply_css(&gray, &format!(".cr-uni-gray {{ background: rgba(200,200,210,0.35); border-radius: {}px; min-width: 100px; min-height: 78px; margin-top: 8px; }}", self.corner_radius as i32));
        overlay.add_overlay(&gray);
        // Red inner rect inset uniformly
        let inset = self.inset.max(0.0) as i32;
        let inner_r = (self.corner_radius - self.inset).max(2.0) as i32;
        let red = gtk::Box::new(gtk::Orientation::Vertical, 0);
        red.set_size_request(100 - inset * 2, 78 - inset * 2);
        red.set_halign(gtk::Align::Center);
        red.set_valign(gtk::Align::Start);
        red.add_css_class("cr-uni-red");
        uikit::widget::apply_css(&red, &format!(".cr-uni-red {{ background: #e5484d; border-radius: {}px; margin-top: {}; }}", inner_r, 8 + inset));
        overlay.add_overlay(&red);
        // Blue base capsule
        let blue = gtk::Box::new(gtk::Orientation::Vertical, 0);
        blue.set_size_request(120, 16);
        blue.set_halign(gtk::Align::Center);
        blue.set_valign(gtk::Align::End);
        blue.add_css_class("cr-uni-blue");
        uikit::widget::apply_css(&blue, ".cr-uni-blue { background: #0A84FF; border-radius: 999px; min-width: 120px; min-height: 16px; margin-bottom: 10px; }");
        overlay.add_overlay(&blue);
        outer.append(&overlay);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for UniformConcentricRectangle {
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
    fn uniform_exists() { let _ = UniformConcentricRectangle::new().corner_radius(12.0).inset(6.0); }
}
