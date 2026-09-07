//! SwipeAction — adds custom swipe actions to a row in a list or container.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use uikit::view_modifiers::ViewModifierExt;

/// SwipeAction — modifier.
pub struct SwipeAction {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl SwipeAction {
    pub fn new() -> Self { Self { id: next_widget_id(), position_mode: PositionMode::Auto, position: Position::new() } }
    pub fn to_view(self) -> View { View::new(self).with_frame(0.0,0.0,220.0,70.0) }
}

impl Default for SwipeAction { fn default() -> Self { Self::new() } }

impl ViewContent for SwipeAction {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let fg = if is_dark { "#FFFFFF" } else { "#1d1d1d" };
        let fg_dim = if is_dark { "rgba(255,255,255,0.55)" } else { "rgba(60,60,67,0.6)" };
        let red = "#FF3B30";

        let outer = gtk::Box::new(gtk::Orientation::Vertical, 8);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 70); }

        // Header: Trailing Swipe ... Leading Swipe
        for txt in ["Trailing Swipe        Leading Swipe", "Stateful Swipe        Swipe state: active"] {
            let lbl = gtk::Label::new(Some(txt));
            lbl.set_halign(gtk::Align::Center);
            uikit::widget::apply_css(&lbl, &format!("label {{ color: {}; font-family: 'SF Pro Display'; font-size: 9px; }}", fg_dim));
            outer.append(&lbl);
        }

        // Row with Delete pill trailing
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        row.set_halign(gtk::Align::Center);
        row.set_size_request(180, 20);
        let mid = gtk::Label::new(Some("Swipe to Delete"));
        mid.set_hexpand(true);
        uikit::widget::apply_css(&mid, &format!("label {{ color: {}; font-family: 'SF Pro Display'; font-size: 9px; }}", fg));
        row.append(&mid);
        let del = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        del.set_size_request(44, 16);
        uikit::widget::apply_css(&del, &format!(".swipe-del {{ background: {}; border-radius: 4px; min-width: 44px; min-height: 16px; }}", red));
        del.add_css_class("swipe-del");
        let dl = gtk::Label::new(Some("Delete")); uikit::widget::apply_css(&dl, ".swipe-del-lbl { color: white; font-family: 'SF Pro Display'; font-size: 7px; }"); dl.set_halign(gtk::Align::Center); del.append(&dl);
        row.append(&del);
        outer.append(&row);

        let _view_mod = uikit::view::View::empty().swipeAction(uikit::view_modifiers::SwipeEdge::Trailing, "Delete", true);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(220.0,70.0) }
}

impl Widget for SwipeAction {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,220.0,70.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}
