//! SheetBackground — Sets the presentation background of the enclosing sheet using a shape style.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// SheetBackground — modifier — Sets the presentation background of the
/// enclosing sheet using a shape style.
pub struct SheetBackground {
    id: WidgetId,
    color: Color,
    position_mode: PositionMode,
    position: Position,
}

impl SheetBackground {
    pub fn new() -> Self {
        Self { id: next_widget_id(), color: Color::from_hex("#0A84FF").unwrap_or(Color::new(0.04, 0.52, 1.0, 1.0)), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn color(mut self, c: Color) -> Self { self.color = c; self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for SheetBackground { fn default() -> Self { Self::new() } }

impl ViewContent for SheetBackground {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let overlay = gtk::Overlay::new();
        overlay.set_size_request(180, 110);
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.add_css_class("shbg-phone");
        uikit::widget::apply_css(&phone, ".shbg-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        overlay.set_child(Some(&phone));
        let sheet = gtk::Box::new(gtk::Orientation::Vertical, 0);
        sheet.set_size_request(180, 78);
        sheet.set_halign(gtk::Align::Center);
        sheet.set_valign(gtk::Align::End);
        sheet.add_css_class("shbg-sheet");
        uikit::widget::apply_css(&sheet, &format!(".shbg-sheet {{ background: {}; border-radius: 14px 14px 12px 12px; }}", self.color.to_css()));
        let lbl = gtk::Label::new(Some("Bar"));
        lbl.set_halign(gtk::Align::Center);
        lbl.add_css_class("shbg-bar");
        uikit::widget::apply_css(&lbl, ".shbg-bar { color: rgba(255,255,255,0.9); font-family: 'SF Pro Display'; font-size: 8px; margin-top: 10px; }");
        sheet.append(&lbl);
        overlay.add_overlay(&sheet);
        outer.append(&overlay);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for SheetBackground {
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
    fn background_exists() { let _ = SheetBackground::new(); }
}
