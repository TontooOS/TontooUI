//! DisableSheetDismissSwipe — Conditionally prevents interactive dismissal of presentations.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// DisableSheetDismissSwipe — modifier — Conditionally prevents interactive
/// dismissal of presentations like popover. Directly on window, SF Pro.
pub struct DisableSheetDismissSwipe {
    id: WidgetId,
    disabled: bool,
    position_mode: PositionMode,
    position: Position,
}

impl DisableSheetDismissSwipe {
    pub fn new() -> Self {
        Self { id: next_widget_id(), disabled: true, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn disabled(mut self, v: bool) -> Self { self.disabled = v; self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for DisableSheetDismissSwipe { fn default() -> Self { Self::new() } }

impl ViewContent for DisableSheetDismissSwipe {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let overlay = gtk::Overlay::new();
        overlay.set_size_request(180, 110);
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.add_css_class("shd-phone");
        uikit::widget::apply_css(&phone, ".shd-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        overlay.set_child(Some(&phone));
        let sheet = gtk::Box::new(gtk::Orientation::Vertical, 0);
        sheet.set_size_request(180, 62);
        sheet.set_halign(gtk::Align::Center);
        sheet.set_valign(gtk::Align::End);
        sheet.add_css_class("shd-sheet");
        uikit::widget::apply_css(&sheet, ".shd-sheet { background: #2c2c2e; border-radius: 14px 14px 12px 12px; }");
        // Lock hint row: small bar + disabled state keeps swipe blocked
        let bar = gtk::Label::new(Some(if self.disabled { "—  swipe locked" } else { "—  swipe enabled" }));
        bar.set_halign(gtk::Align::Center);
        bar.set_valign(gtk::Align::Center);
        bar.add_css_class("shd-bar");
        uikit::widget::apply_css(&bar, ".shd-bar { color: rgba(255,255,255,0.55); font-family: 'SF Pro Display'; font-size: 8px; margin-top: 22px; }");
        sheet.append(&bar);
        overlay.add_overlay(&sheet);
        outer.append(&overlay);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for DisableSheetDismissSwipe {
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
    fn disable_exists() { let _ = DisableSheetDismissSwipe::new().disabled(false); }
}
