//! SheetCornerRadius — Requests that the presentation have a specific corner radius.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// SheetCornerRadius — modifier — Requests that the presentation have a
/// specific corner radius.
pub struct SheetCornerRadius {
    id: WidgetId,
    radius: f32,
    position_mode: PositionMode,
    position: Position,
}

impl SheetCornerRadius {
    pub fn new() -> Self {
        Self { id: next_widget_id(), radius: 28.0, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn radius(mut self, v: f32) -> Self { self.radius = v; self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for SheetCornerRadius { fn default() -> Self { Self::new() } }

impl ViewContent for SheetCornerRadius {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let overlay = gtk::Overlay::new();
        overlay.set_size_request(180, 110);
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.add_css_class("shc-phone");
        uikit::widget::apply_css(&phone, ".shc-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        overlay.set_child(Some(&phone));
        let sheet = gtk::Box::new(gtk::Orientation::Vertical, 0);
        sheet.set_size_request(180, 72);
        sheet.set_halign(gtk::Align::Center);
        sheet.set_valign(gtk::Align::End);
        sheet.add_css_class("shc-sheet");
        uikit::widget::apply_css(&sheet, &format!(".shc-sheet {{ background: #2c2c2e; border-radius: {}px {}px 12px 12px; }}", self.radius as i32, self.radius as i32));
        let lbl = gtk::Label::new(Some("Bar"));
        lbl.set_halign(gtk::Align::Center);
        lbl.add_css_class("shc-bar");
        uikit::widget::apply_css(&lbl, ".shc-bar { color: rgba(255,255,255,0.75); font-family: 'SF Pro Display'; font-size: 8px; margin-top: 10px; }");
        sheet.append(&lbl);
        overlay.add_overlay(&sheet);
        outer.append(&overlay);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for SheetCornerRadius {
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
    fn corner_exists() { let _ = SheetCornerRadius::new().radius(12.0); }
}
