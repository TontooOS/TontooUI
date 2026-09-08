//! ControlSize — a control version that is the default size.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use uikit::view_modifiers::ViewModifierExt;

/// ControlSizeView — modifier preview.
///
/// A control version that is the default size. Renders a vertical stack of
/// "Tap Me" pills at varying sizes, directly on window.
pub struct ControlSizeView {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl ControlSizeView {
    pub fn new() -> Self { Self { id: next_widget_id(), position_mode: PositionMode::Auto, position: Position::new() } }
    pub fn to_view(self) -> View { View::new(self).with_frame(0.0,0.0,120.0,150.0) }
}

impl Default for ControlSizeView { fn default() -> Self { Self::new() } }

impl ViewContent for ControlSizeView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 6);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 150); }

        // 5 Tap Me pills with increasing size to hint ControlSize values
        let sizes = [(22,9),(26,10),(30,11),(36,12),(42,13)];
        for (h, fs) in sizes {
            let pill = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            pill.set_halign(gtk::Align::Center);
            pill.set_size_request(70, h);
            pill.add_css_class("cs-pill");
            let (bg, border) = if is_dark { ("#0A84FF","none") } else { ("#0A84FF","none") };
            uikit::widget::apply_css(&pill, &format!(".cs-pill {{ background: {}; border: {}; border-radius: 999px; min-height: {}px; min-width: 70px; }}", bg, border, h));
            let lbl = gtk::Label::new(Some("Tap Me"));
            lbl.set_halign(gtk::Align::Center);
            lbl.set_valign(gtk::Align::Center);
            lbl.set_hexpand(true);
            lbl.set_vexpand(true);
            lbl.add_css_class("cs-lbl");
            uikit::widget::apply_css(&lbl, &format!(".cs-lbl {{ color: white; font-family: 'SF Pro Display'; font-size: {}px; }}", fs));
            pill.append(&lbl);
            outer.append(&pill);
        }

        let _view_mod = uikit::view::View::empty().controlSize(uikit::view_modifiers::ControlSize::Regular);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(120.0,150.0) }
}

impl Widget for ControlSizeView {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,120.0,150.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}
