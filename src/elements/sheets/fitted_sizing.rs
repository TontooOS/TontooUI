//! FittedSheetSizing — Sets the sizing of the containing presentation.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// FittedSheetSizing — modifier — Sets the sizing of the containing
/// presentation to fitted content size.
pub struct FittedSheetSizing {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl FittedSheetSizing {
    pub fn new() -> Self {
        Self { id: next_widget_id(), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for FittedSheetSizing { fn default() -> Self { Self::new() } }

impl ViewContent for FittedSheetSizing {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let overlay = gtk::Overlay::new();
        overlay.set_size_request(180, 110);
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.add_css_class("shf-phone");
        uikit::widget::apply_css(&phone, ".shf-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        overlay.set_child(Some(&phone));
        // Tiny fitted pill centered
        let sheet = gtk::Box::new(gtk::Orientation::Vertical, 0);
        sheet.set_size_request(44, 34);
        sheet.set_halign(gtk::Align::Center);
        sheet.set_valign(gtk::Align::Center);
        sheet.add_css_class("shf-sheet");
        uikit::widget::apply_css(&sheet, ".shf-sheet { background: #2c2c2e; border-radius: 10px; }");
        let lbl = gtk::Label::new(Some("Bar"));
        lbl.set_halign(gtk::Align::Center);
        lbl.set_valign(gtk::Align::Center);
        lbl.set_vexpand(true);
        lbl.add_css_class("shf-bar");
        uikit::widget::apply_css(&lbl, ".shf-bar { color: rgba(255,255,255,0.6); font-family: 'SF Pro Display'; font-size: 6px; }");
        sheet.append(&lbl);
        overlay.add_overlay(&sheet);
        outer.append(&overlay);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for FittedSheetSizing {
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
    fn fitted_exists() { let _ = FittedSheetSizing::new(); }
}
