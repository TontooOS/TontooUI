//! SwipeContainer — only allows a single active swipe within a container.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use uikit::view_modifiers::ViewModifierExt;

/// SwipeContainer — modifier.
pub struct SwipeContainer {
    id: WidgetId,
    single_active: bool,
    position_mode: PositionMode,
    position: Position,
}

impl SwipeContainer {
    pub fn new() -> Self { Self { id: next_widget_id(), single_active: true, position_mode: PositionMode::Auto, position: Position::new() } }
    pub fn single_active(mut self, v: bool) -> Self { self.single_active = v; self }
    pub fn to_view(self) -> View { View::new(self).with_frame(0.0,0.0,220.0,80.0) }
}

impl Default for SwipeContainer { fn default() -> Self { Self::new() } }

impl ViewContent for SwipeContainer {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let fg = if is_dark { "#FFFFFF" } else { "#1d1d1d" };
        let fg_dim = if is_dark { "rgba(255,255,255,0.55)" } else { "rgba(60,60,67,0.6)" };
        let (bg, border) = if is_dark { ("#2c2c2e","1px solid rgba(255,255,255,0.10)") } else { ("#ffffff","1px solid rgba(0,0,0,0.08)") };
        let delete_bg = "#FF3B30";

        let outer = gtk::Box::new(gtk::Orientation::Vertical, 6);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 80); }

        for group in ["Group1", "Group1"] {
            let _ = group;
            let card = gtk::Box::new(gtk::Orientation::Vertical, 4);
            card.set_halign(gtk::Align::Center);
            card.set_size_request(200, 34);
            card.add_css_class("swc-card");
            uikit::widget::apply_css(&card, &format!(".swc-card {{ background: {}; border: {}; border-radius: 10px; padding: 4px 8px; }}", bg, border));

            // Two rows Foo1/Bar1, Foo2/Bar2 with Delete pills
            for (label, val) in [("Foo","Bar1"), ("Bar","Bar2")] {
                let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                row.set_hexpand(true);
                let kind = if label=="Foo" { "Foo1" } else { "Foo2" };
                // prefix
                let prefix = gtk::Label::new(Some(group));
                uikit::widget::apply_css(&prefix, &format!("label {{ color: {}; font-family: 'SF Pro Display'; font-size: 8px; }}", fg_dim));
                row.append(&prefix);
                let mid = gtk::Label::new(Some(kind));
                mid.set_hexpand(true); mid.set_halign(gtk::Align::Center);
                uikit::widget::apply_css(&mid, &format!("label {{ color: {}; font-family: 'SF Pro Display'; font-size: 9px; }}", fg));
                row.append(&mid);
                let del = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                del.set_size_request(32, 14);
                uikit::widget::apply_css(&del, &format!(".swc-del {{ background: {}; border-radius: 4px; min-width: 32px; min-height: 14px; }}", delete_bg));
                del.add_css_class("swc-del");
                let dl = gtk::Label::new(Some("Delete")); uikit::widget::apply_css(&dl, ".swc-del-lbl { color: white; font-family: 'SF Pro Display'; font-size: 7px; }"); dl.set_halign(gtk::Align::Center); dl.set_valign(gtk::Align::Center); del.append(&dl);
                row.append(&del);
                card.append(&row);
            }
            outer.append(&card);
        }

        let _view_mod = uikit::view::View::empty().swipeContainer(self.single_active);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(220.0,80.0) }
}

impl Widget for SwipeContainer {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,220.0,80.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}
