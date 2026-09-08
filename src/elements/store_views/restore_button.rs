//! RestorePurchasesButton — Restores previously purchased products.

use std::sync::Arc;

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// RestorePurchasesButton — modifier — A type of button that people use to
/// restore purchases.
///
/// Usable: handle `on_restore` (the blue pill is a real button).
pub struct RestorePurchasesButton {
    id: WidgetId,
    on_restore: Option<Arc<dyn Fn() + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl RestorePurchasesButton {
    pub fn new() -> Self {
        Self { id: next_widget_id(), on_restore: None, position_mode: PositionMode::Auto, position: Position::new() }
    }
    /// Called when the restore button is pressed.
    pub fn on_restore(mut self, f: impl Fn() + Send + Sync + 'static) -> Self { self.on_restore = Some(Arc::new(f)); self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for RestorePurchasesButton { fn default() -> Self { Self::new() } }

impl ViewContent for RestorePurchasesButton {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("sv-rb-phone");
        uikit::widget::apply_css(&phone, ".sv-rb-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        let btn = gtk::Button::new();
        btn.set_size_request(150, 30);
        btn.set_halign(gtk::Align::Center);
        btn.set_valign(gtk::Align::Center);
        btn.set_vexpand(true);
        btn.add_css_class("sv-rb-btn");
        uikit::widget::apply_css(&btn, ".sv-rb-btn { background: #0A84FF; border: none; outline: none; border-radius: 15px; min-width: 150px; min-height: 30px; margin-top: 40px; margin-bottom: 40px; } .sv-rb-btn:focus { outline: none; }");
        let lbl = gtk::Label::new(Some("Restore Missing Purchases"));
        lbl.set_halign(gtk::Align::Center);
        lbl.set_valign(gtk::Align::Center);
        lbl.set_vexpand(true);
        lbl.add_css_class("sv-rb-lbl");
        uikit::widget::apply_css(&lbl, ".sv-rb-lbl { color: white; font-family: 'SF Pro Display'; font-size: 8px; font-weight: 600; }");
        btn.set_child(Some(&lbl));
        if let Some(cb) = &self.on_restore {
            let cb = cb.clone();
            btn.connect_clicked(move |_| cb());
        }
        phone.append(&btn);
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for RestorePurchasesButton {
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
    fn restore_exists() { let _ = RestorePurchasesButton::new(); }
    #[test]
    fn restore_callback() {
        assert!(RestorePurchasesButton::new().on_restore(|| {}).on_restore.is_some());
    }
}
