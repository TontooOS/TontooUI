//! BooleanSheet — Presents a sheet when a binding to a Boolean value is true.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// BooleanSheet — modifier — Presents a sheet when a binding to a Boolean
/// value that you provide is true.
pub struct BooleanSheet {
    id: WidgetId,
    presented: bool,
    position_mode: PositionMode,
    position: Position,
}

impl BooleanSheet {
    pub fn new() -> Self {
        Self { id: next_widget_id(), presented: true, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn presented(mut self, v: bool) -> Self { self.presented = v; self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for BooleanSheet { fn default() -> Self { Self::new() } }

impl ViewContent for BooleanSheet {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let overlay = gtk::Overlay::new();
        overlay.set_size_request(180, 110);
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.add_css_class("shb-phone");
        uikit::widget::apply_css(&phone, ".shb-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        overlay.set_child(Some(&phone));
        if self.presented {
            let sheet = gtk::Box::new(gtk::Orientation::Vertical, 0);
            sheet.set_size_request(160, 84);
            sheet.set_halign(gtk::Align::Center);
            sheet.set_valign(gtk::Align::End);
            sheet.add_css_class("shb-sheet");
            uikit::widget::apply_css(&sheet, ".shb-sheet { background: #2c2c2e; border-radius: 14px 14px 12px 12px; }");
            let lbl = gtk::Label::new(Some("Bar"));
            lbl.set_halign(gtk::Align::Center);
            lbl.add_css_class("shb-bar");
            uikit::widget::apply_css(&lbl, ".shb-bar { color: rgba(255,255,255,0.75); font-family: 'SF Pro Display'; font-size: 8px; margin-top: 10px; }");
            sheet.append(&lbl);
            overlay.add_overlay(&sheet);
        }
        outer.append(&overlay);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for BooleanSheet {
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
    fn boolean_exists() { let _ = BooleanSheet::new().presented(false); }
}
