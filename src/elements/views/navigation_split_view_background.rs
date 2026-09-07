//! NavigationSplitViewBackground — background placement behind NavigationSplitView content.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use uikit::view_modifiers::ViewModifierExt;

/// NavigationSplitViewContainerBackground — modifier.
pub struct NavigationSplitViewBackground {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl NavigationSplitViewBackground {
    pub fn new() -> Self { Self { id: next_widget_id(), position_mode: PositionMode::Auto, position: Position::new() } }
    pub fn to_view(self) -> View { View::new(self).with_frame(0.0,0.0,220.0,100.0) }
}

impl Default for NavigationSplitViewBackground { fn default() -> Self { Self::new() } }

impl ViewContent for NavigationSplitViewBackground {
    fn render(&self, frame: Rect) -> gtk::Widget {
        // Blue container background as in screenshot (solid #0A84FF / similar)
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 100); }

        // Background placement behind content — simulate with blue box + small "a" overlay
        let bg = gtk::Box::new(gtk::Orientation::Vertical, 0);
        bg.set_size_request(200, 80);
        bg.set_halign(gtk::Align::Center);
        bg.add_css_class("nsv-bg");
        uikit::widget::apply_css(&bg, ".nsv-bg { background: #2a7fff; border-radius: 10px; min-height: 80px; }");
        // small "a" at bottom left to hint content behind
        let lbl = gtk::Label::new(Some("a"));
        lbl.set_halign(gtk::Align::Start);
        lbl.set_valign(gtk::Align::End);
        lbl.set_margin_start(8); lbl.set_margin_bottom(6);
        uikit::widget::apply_css(&lbl, ".nsv-a { color: rgba(255,255,255,0.65); font-family: 'SF Pro Display'; font-size: 9px; }");
        lbl.add_css_class("nsv-a");
        bg.append(&lbl);
        outer.append(&bg);

        let _view_mod = uikit::view::View::empty().navigationSplitViewBackground(uikit::view_modifiers::ContainerBackground::Blue);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(220.0,100.0) }
}

impl Widget for NavigationSplitViewBackground {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,220.0,100.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}
