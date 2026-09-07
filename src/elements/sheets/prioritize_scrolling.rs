//! PrioritizeSheetContentScrolling — Configure the behavior of swipe gestures on a presentation.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// PrioritizeSheetContentScrolling — modifier — Configure the behavior of
/// swipe gestures on a presentation (content scrolling vs dismissal).
pub struct PrioritizeSheetContentScrolling {
    id: WidgetId,
    prioritize: bool,
    position_mode: PositionMode,
    position: Position,
}

impl PrioritizeSheetContentScrolling {
    pub fn new() -> Self {
        Self { id: next_widget_id(), prioritize: true, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn prioritize(mut self, v: bool) -> Self { self.prioritize = v; self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for PrioritizeSheetContentScrolling { fn default() -> Self { Self::new() } }

impl ViewContent for PrioritizeSheetContentScrolling {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let overlay = gtk::Overlay::new();
        overlay.set_size_request(180, 110);
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.add_css_class("shs-phone");
        uikit::widget::apply_css(&phone, ".shs-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        overlay.set_child(Some(&phone));
        let sheet = gtk::Box::new(gtk::Orientation::Vertical, 2);
        sheet.set_size_request(150, 92);
        sheet.set_halign(gtk::Align::Center);
        sheet.set_valign(gtk::Align::End);
        sheet.add_css_class("shs-sheet");
        uikit::widget::apply_css(&sheet, ".shs-sheet { background: #2c2c2e; border-radius: 14px 14px 12px 12px; padding: 8px; }");
        for i in 9..15 {
            let lbl = gtk::Label::new(Some(&format!("Bar {}", i)));
            lbl.set_halign(gtk::Align::Center);
            lbl.add_css_class("shs-bar");
            let alpha = if i == 11 { "0.9" } else { "0.45" };
            uikit::widget::apply_css(&lbl, &format!(".shs-bar {{ color: rgba(255,255,255,{}); font-family: 'SF Pro Display'; font-size: 7px; }}", alpha));
            sheet.append(&lbl);
        }
        overlay.add_overlay(&sheet);
        outer.append(&overlay);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for PrioritizeSheetContentScrolling {
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
    fn prioritize_exists() { let _ = PrioritizeSheetContentScrolling::new().prioritize(false); }
}
