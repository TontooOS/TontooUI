//! SheetPlacement — Sets the placement of a presentation within the presenting view.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// Sheet placement within the presenting view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SheetPlacementKind {
    #[default]
    Sheet,
    Popover,
    FullScreen,
}

/// SheetPlacement — modifier — Sets the placement of a presentation within the presenting view.
/// Directly on window (#1d1d1d dark / #ececec light), no extra card, SF Pro.
pub struct SheetPlacement {
    id: WidgetId,
    placement: SheetPlacementKind,
    position_mode: PositionMode,
    position: Position,
}

impl SheetPlacement {
    pub fn new() -> Self {
        Self { id: next_widget_id(), placement: SheetPlacementKind::Sheet, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn placement(mut self, v: SheetPlacementKind) -> Self { self.placement = v; self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for SheetPlacement { fn default() -> Self { Self::new() } }

impl ViewContent for SheetPlacement {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let fg = if is_dark { "#FFFFFF" } else { "#1d1d1d" };
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.add_css_class("sh-phone");
        uikit::widget::apply_css(&phone, ".sh-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        let overlay = gtk::Overlay::new();
        overlay.set_size_request(180, 110);
        overlay.set_child(Some(&phone));
        // Small sheet docked top-left to hint placement
        let sheet = gtk::Box::new(gtk::Orientation::Vertical, 0);
        match self.placement {
            SheetPlacementKind::Sheet => { sheet.set_size_request(110, 70); sheet.set_halign(gtk::Align::Start); sheet.set_valign(gtk::Align::Center); }
            SheetPlacementKind::Popover => { sheet.set_size_request(80, 56); sheet.set_halign(gtk::Align::Center); sheet.set_valign(gtk::Align::Center); }
            SheetPlacementKind::FullScreen => { sheet.set_size_request(168, 98); sheet.set_halign(gtk::Align::Center); sheet.set_valign(gtk::Align::Center); }
        }
        sheet.add_css_class("sh-sheet");
        uikit::widget::apply_css(&sheet, ".sh-sheet { background: #2c2c2e; border-radius: 10px; margin: 8px; }");
        let dot = gtk::Box::new(gtk::Orientation::Vertical, 0);
        dot.set_size_request(10, 6);
        dot.set_halign(gtk::Align::Center);
        dot.set_valign(gtk::Align::Center);
        dot.add_css_class("sh-dot");
        uikit::widget::apply_css(&dot, ".sh-dot { background: rgba(255,255,255,0.35); border-radius: 3px; min-width: 10px; min-height: 6px; margin-top: 18px; }");
        sheet.append(&dot);
        overlay.add_overlay(&sheet);
        outer.append(&overlay);
        let _ = fg;
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for SheetPlacement {
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
    fn placement_exists() { let _ = SheetPlacement::new().placement(SheetPlacementKind::Popover); }
}
