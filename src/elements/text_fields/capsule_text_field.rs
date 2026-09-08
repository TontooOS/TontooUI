//! CapsuleTextField — Gives your text field a capsule shape.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// CapsuleTextField — modifier — Gives your text field a capsule shape. This
/// only has a visible effect on macOS (Liquid Glass); on other platforms the
/// field falls back to a rounded style.
pub struct CapsuleTextField {
    id: WidgetId,
    placeholder: String,
    position_mode: PositionMode,
    position: Position,
}

impl CapsuleTextField {
    pub fn new() -> Self {
        Self { id: next_widget_id(), placeholder: "Enter text...".to_string(), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn placeholder(mut self, v: impl Into<String>) -> Self { self.placeholder = v.into(); self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for CapsuleTextField { fn default() -> Self { Self::new() } }

impl ViewContent for CapsuleTextField {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("tf-cap-phone");
        uikit::widget::apply_css(&phone, ".tf-cap-phone { background: #141a26; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        // Capsule-shaped field with Liquid Glass hint (frosted ring)
        let field = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        field.set_size_request(150, 32);
        field.set_halign(gtk::Align::Center);
        field.set_valign(gtk::Align::Center);
        field.set_vexpand(true);
        field.add_css_class("tf-cap-field");
        uikit::widget::apply_css(&field, ".tf-cap-field { background: rgba(255,255,255,0.08); border: 1px solid rgba(255,255,255,0.22); border-radius: 999px; min-width: 150px; min-height: 32px; margin-top: 39px; margin-bottom: 39px; padding: 4px 14px; box-shadow: 0 4px 16px rgba(0,0,0,0.25); }");
        let ph = gtk::Label::new(Some(self.placeholder.as_str()));
        ph.set_halign(gtk::Align::Start);
        ph.set_valign(gtk::Align::Center);
        ph.set_hexpand(true);
        ph.add_css_class("tf-cap-ph");
        uikit::widget::apply_css(&ph, ".tf-cap-ph { color: rgba(255,255,255,0.4); font-family: 'SF Pro Display'; font-size: 8px; }");
        field.append(&ph);
        // Caret hint
        let caret = gtk::Box::new(gtk::Orientation::Vertical, 0);
        caret.set_size_request(1, 12);
        caret.set_valign(gtk::Align::Center);
        caret.add_css_class("tf-cap-caret");
        uikit::widget::apply_css(&caret, ".tf-cap-caret { background: #0A84FF; min-width: 1px; min-height: 12px; }");
        field.append(&caret);
        phone.append(&field);
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for CapsuleTextField {
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
    fn capsule_exists() { let _ = CapsuleTextField::new().placeholder("Search"); }
}
