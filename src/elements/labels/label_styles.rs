//! LabelStyles — Sets the style for labels within this view.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// Label style variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LabelStyleKind {
    #[default]
    TitleAndIcon,
    TitleOnly,
    IconOnly,
}

/// LabelStyles — style — Sets the style for labels within this view.
pub struct LabelStyles {
    id: WidgetId,
    style: LabelStyleKind,
    position_mode: PositionMode,
    position: Position,
}

impl LabelStyles {
    pub fn new() -> Self {
        Self { id: next_widget_id(), style: LabelStyleKind::TitleAndIcon, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn style(mut self, v: LabelStyleKind) -> Self { self.style = v; self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for LabelStyles { fn default() -> Self { Self::new() } }

impl ViewContent for LabelStyles {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("lbl-style-phone");
        uikit::widget::apply_css(&phone, ".lbl-style-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        let col = gtk::Box::new(gtk::Orientation::Vertical, 4);
        col.set_halign(gtk::Align::Center);
        col.set_valign(gtk::Align::Center);
        col.set_vexpand(true);
        let show_icon = self.style != LabelStyleKind::TitleOnly;
        let show_title = self.style != LabelStyleKind::IconOnly;
        for title in ["Foo", "Foo", "Foo"] {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 5);
            row.set_halign(gtk::Align::Center);
            if show_icon {
                let icon = gtk::Label::new(Some("☰"));
                icon.add_css_class("lbl-style-icon");
                uikit::widget::apply_css(&icon, ".lbl-style-icon { color: rgba(255,255,255,0.55); font-family: 'SF Pro Display'; font-size: 8px; }");
                row.append(&icon);
            }
            if show_title {
                let lbl = gtk::Label::new(Some(title));
                lbl.add_css_class("lbl-style-lbl");
                uikit::widget::apply_css(&lbl, ".lbl-style-lbl { color: rgba(255,255,255,0.75); font-family: 'SF Pro Display'; font-size: 8px; }");
                row.append(&lbl);
            }
            col.append(&row);
        }
        phone.append(&col);
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for LabelStyles {
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
    fn styles_exists() { let _ = LabelStyles::new().style(LabelStyleKind::TitleOnly); }
}
