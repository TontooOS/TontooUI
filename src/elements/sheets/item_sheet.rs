//! ItemSheet — Presents a sheet using the given item as a data source for the sheet's content.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// ItemSheet — modifier — Presents a sheet using the given item as a data
/// source for the sheet's content.
pub struct ItemSheet {
    id: WidgetId,
    item: Option<String>,
    position_mode: PositionMode,
    position: Position,
}

impl ItemSheet {
    pub fn new() -> Self {
        Self { id: next_widget_id(), item: Some("390BD69D-3D10-47EB-B046-4865133A36497".to_string()), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn item(mut self, v: Option<String>) -> Self { self.item = v; self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for ItemSheet { fn default() -> Self { Self::new() } }

impl ViewContent for ItemSheet {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let overlay = gtk::Overlay::new();
        overlay.set_size_request(180, 110);
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.add_css_class("shi-phone");
        uikit::widget::apply_css(&phone, ".shi-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        overlay.set_child(Some(&phone));
        let uuid = gtk::Label::new(Some(self.item.as_deref().unwrap_or("nil")));
        uuid.set_halign(gtk::Align::Center);
        uuid.set_valign(gtk::Align::Start);
        uuid.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        uuid.set_max_width_chars(22);
        uuid.add_css_class("shi-uuid");
        uikit::widget::apply_css(&uuid, ".shi-uuid { color: rgba(255,255,255,0.65); font-family: 'SF Pro Display'; font-size: 6px; margin-top: 8px; }");
        overlay.add_overlay(&uuid);
        if self.item.is_some() {
            let sheet = gtk::Box::new(gtk::Orientation::Vertical, 0);
            sheet.set_size_request(180, 62);
            sheet.set_halign(gtk::Align::Center);
            sheet.set_valign(gtk::Align::End);
            sheet.add_css_class("shi-sheet");
            uikit::widget::apply_css(&sheet, ".shi-sheet { background: #2c2c2e; border-radius: 14px 14px 12px 12px; }");
            let lbl = gtk::Label::new(Some("Bar"));
            lbl.set_halign(gtk::Align::Center);
            lbl.set_valign(gtk::Align::Center);
            lbl.set_vexpand(true);
            lbl.add_css_class("shi-bar");
            uikit::widget::apply_css(&lbl, ".shi-bar { color: rgba(255,255,255,0.75); font-family: 'SF Pro Display'; font-size: 8px; }");
            sheet.append(&lbl);
            overlay.add_overlay(&sheet);
        }
        outer.append(&overlay);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for ItemSheet {
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
    fn item_exists() { let _ = ItemSheet::new().item(None); }
}
