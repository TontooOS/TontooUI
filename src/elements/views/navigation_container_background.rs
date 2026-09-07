//! NavigationContainerBackground — sets the container background of the enclosing container.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use uikit::view_modifiers::ViewModifierExt;

/// NavigationContainerBackground — modifier.
pub struct NavigationContainerBackground {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl NavigationContainerBackground {
    pub fn new() -> Self { Self { id: next_widget_id(), position_mode: PositionMode::Auto, position: Position::new() } }
    pub fn to_view(self) -> View { View::new(self).with_frame(0.0,0.0,220.0,100.0) }
}

impl Default for NavigationContainerBackground { fn default() -> Self { Self::new() } }

impl ViewContent for NavigationContainerBackground {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 100); }

        let bg = gtk::Box::new(gtk::Orientation::Vertical, 0);
        bg.set_size_request(200, 80);
        bg.set_halign(gtk::Align::Center);
        bg.set_valign(gtk::Align::Center);
        bg.add_css_class("nc-bg");
        uikit::widget::apply_css(&bg, ".nc-bg { background: #2a7fff; border-radius: 10px; min-height: 80px; }");
        let lbl = gtk::Label::new(Some("Foo"));
        lbl.set_halign(gtk::Align::Center);
        lbl.set_valign(gtk::Align::Center);
        lbl.add_css_class("nc-foo");
        uikit::widget::apply_css(&lbl, ".nc-foo { color: rgba(255,255,255,0.90); font-family: 'SF Pro Display'; font-size: 11px; }");
        bg.append(&lbl);
        outer.append(&bg);

        let _view_mod = uikit::view::View::empty().navigationContainerBackground(Color::from_rgb(42,127,255));
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(220.0,100.0) }
}

impl Widget for NavigationContainerBackground {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,220.0,100.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}
