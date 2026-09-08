//! ConcentricRectangle — A concentric rectangle whose corner radii come from the same circle.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// ConcentricRectangle — initializer — A concentric rectangle whose corner
/// radii are defined from the same circle.
pub struct ConcentricRectangle {
    id: WidgetId,
    top_leading: f32,
    top_trailing: f32,
    bottom_leading: f32,
    bottom_trailing: f32,
    inset: f32,
    position_mode: PositionMode,
    position: Position,
}

impl ConcentricRectangle {
    pub fn new() -> Self {
        Self { id: next_widget_id(), top_leading: 22.0, top_trailing: 22.0, bottom_leading: 22.0, bottom_trailing: 22.0, inset: 8.0, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn corners(mut self, tl: f32, tr: f32, bl: f32, br: f32) -> Self {
        self.top_leading = tl; self.top_trailing = tr; self.bottom_leading = bl; self.bottom_trailing = br; self
    }
    pub fn inset(mut self, v: f32) -> Self { self.inset = v; self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for ConcentricRectangle { fn default() -> Self { Self::new() } }

impl ViewContent for ConcentricRectangle {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(150, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("cr-phone");
        uikit::widget::apply_css(&phone, ".cr-phone { background: #0b0b0e; border-radius: 12px; min-width: 150px; min-height: 110px; }");
        let overlay = gtk::Overlay::new();
        overlay.set_size_request(150, 110);
        overlay.set_child(Some(&phone));
        // Gray outer with per-corner radii
        let gray = gtk::Box::new(gtk::Orientation::Vertical, 0);
        gray.set_size_request(100, 78);
        gray.set_halign(gtk::Align::Center);
        gray.set_valign(gtk::Align::Start);
        gray.add_css_class("cr-gray");
        uikit::widget::apply_css(&gray, &format!(
            ".cr-gray {{ background: rgba(200,200,210,0.35); border-top-left-radius: {}px; border-top-right-radius: {}px; border-bottom-left-radius: {}px; border-bottom-right-radius: {}px; min-width: 100px; min-height: 78px; margin-top: 8px; }}",
            self.top_leading as i32, self.top_trailing as i32, self.bottom_leading as i32, self.bottom_trailing as i32
        ));
        overlay.add_overlay(&gray);
        // Red inner: same-circle radii minus inset
        let i = self.inset.max(0.0);
        let r = |v: f32| (v - i).max(2.0) as i32;
        let red = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let inset_px = i as i32;
        red.set_size_request(100 - inset_px * 2, 78 - inset_px * 2);
        red.set_halign(gtk::Align::Center);
        red.set_valign(gtk::Align::Start);
        red.add_css_class("cr-red");
        uikit::widget::apply_css(&red, &format!(
            ".cr-red {{ background: #e5484d; border-top-left-radius: {}px; border-top-right-radius: {}px; border-bottom-left-radius: {}px; border-bottom-right-radius: {}px; margin-top: {}; }}",
            r(self.top_leading), r(self.top_trailing), r(self.bottom_leading), r(self.bottom_trailing), 8 + inset_px
        ));
        overlay.add_overlay(&red);
        // Blue base capsule
        let blue = gtk::Box::new(gtk::Orientation::Vertical, 0);
        blue.set_size_request(120, 16);
        blue.set_halign(gtk::Align::Center);
        blue.set_valign(gtk::Align::End);
        blue.add_css_class("cr-blue");
        uikit::widget::apply_css(&blue, ".cr-blue { background: #0A84FF; border-radius: 999px; min-width: 120px; min-height: 16px; margin-bottom: 10px; }");
        overlay.add_overlay(&blue);
        outer.append(&overlay);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for ConcentricRectangle {
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
    fn concentric_exists() { let _ = ConcentricRectangle::new().corners(16.0, 8.0, 8.0, 16.0).inset(6.0); }
}
