//! PageScreenSheetSize — On devices smaller than a page of paper, page sizing fills.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// PageScreenSheetSize — modifier — On devices smaller than a page of paper,
/// such as iPhone or Apple Watch, page sizing fills the screen.
pub struct PageScreenSheetSize {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl PageScreenSheetSize {
    pub fn new() -> Self {
        Self { id: next_widget_id(), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for PageScreenSheetSize { fn default() -> Self { Self::new() } }

impl ViewContent for PageScreenSheetSize {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let overlay = gtk::Overlay::new();
        overlay.set_size_request(180, 110);
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.add_css_class("shp-phone");
        uikit::widget::apply_css(&phone, ".shp-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        overlay.set_child(Some(&phone));
        let sheet = gtk::Box::new(gtk::Orientation::Vertical, 0);
        sheet.set_size_request(150, 88);
        sheet.set_halign(gtk::Align::Center);
        sheet.set_valign(gtk::Align::Center);
        sheet.add_css_class("shp-sheet");
        uikit::widget::apply_css(&sheet, ".shp-sheet { background: #1c1c1e; border: 1px solid rgba(255,255,255,0.08); border-radius: 10px; }");
        let lbl = gtk::Label::new(Some("Bar"));
        lbl.set_halign(gtk::Align::Center);
        lbl.set_valign(gtk::Align::Center);
        lbl.set_vexpand(true);
        lbl.add_css_class("shp-bar");
        uikit::widget::apply_css(&lbl, ".shp-bar { color: rgba(255,255,255,0.75); font-family: 'SF Pro Display'; font-size: 8px; }");
        sheet.append(&lbl);
        overlay.add_overlay(&sheet);
        outer.append(&overlay);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for PageScreenSheetSize {
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
    fn page_exists() { let _ = PageScreenSheetSize::new(); }
}
