//! ManageSubscriptionsSheet — opens the manage subscriptions sheet.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use uikit::view_modifiers::ViewModifierExt;

/// ManageSubscriptionsSheet — modifier.
pub struct ManageSubscriptionsSheet {
    id: WidgetId,
    presented: bool,
    position_mode: PositionMode,
    position: Position,
}

impl ManageSubscriptionsSheet {
    pub fn new() -> Self { Self { id: next_widget_id(), presented: true, position_mode: PositionMode::Auto, position: Position::new() } }
    pub fn presented(mut self, v: bool) -> Self { self.presented = v; self }
    pub fn to_view(self) -> View { View::new(self).with_frame(0.0,0.0,220.0,40.0) }
}

impl Default for ManageSubscriptionsSheet { fn default() -> Self { Self::new() } }

impl ViewContent for ManageSubscriptionsSheet {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let fg_dim = if is_dark { "rgba(255,255,255,0.65)" } else { "rgba(60,60,67,0.6)" };
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 40); }
        let lbl = gtk::Label::new(Some("You don't have any subscriptions."));
        lbl.add_css_class("mss-lbl");
        uikit::widget::apply_css(&lbl, &format!(".mss-lbl {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; }}", fg_dim));
        lbl.set_halign(gtk::Align::Center);
        outer.append(&lbl);
        let _view_mod = uikit::view::View::empty().manageSubscriptionsSheet(self.presented);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(220.0,40.0) }
}

impl Widget for ManageSubscriptionsSheet {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,220.0,40.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}
