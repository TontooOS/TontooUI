//! BackgroundExtensionEffect — adds the background extension effect.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use uikit::view_modifiers::ViewModifierExt;

/// BackgroundExtensionEffect — modifier.
///
/// Adds the background extension effect to the view. The view will be
/// duplicated behind the extended background. Preview shows cats with hats
/// duplicated vertically, directly on window.
pub struct BackgroundExtensionEffect {
    id: WidgetId,
    position_mode: PositionMode,
    position: Position,
}

impl BackgroundExtensionEffect {
    pub fn new() -> Self { Self { id: next_widget_id(), position_mode: PositionMode::Auto, position: Position::new() } }
    pub fn to_view(self) -> View { View::new(self).with_frame(0.0,0.0,220.0,110.0) }
}

impl Default for BackgroundExtensionEffect { fn default() -> Self { Self::new() } }

impl ViewContent for BackgroundExtensionEffect {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }

        // Two rows of cats — top row normal, bottom row slightly faded to hint duplication/extension
        for opacity in [1.0, 0.45] {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            row.set_halign(gtk::Align::Center);
            row.set_opacity(opacity as f64);
            for i in 0..3 {
                let cat = gtk::Box::new(gtk::Orientation::Vertical, 0);
                cat.set_size_request(60, 44);
                // Use colored placeholder with cat emoji
                let colors = ["#d8b48a", "#a8c0d0", "#e8d5b8"];
                cat.add_css_class("bee-cat");
                uikit::widget::apply_css(&cat, &format!(".bee-cat {{ background: {}; border-radius: 8px; min-width: 60px; min-height: 44px; }}", colors[i]));
                let lbl = gtk::Label::new(Some("🐱"));
                lbl.set_halign(gtk::Align::Center); lbl.set_valign(gtk::Align::Center);
                uikit::widget::apply_css(&lbl, "label { font-size: 20px; }");
                cat.append(&lbl);
                row.append(&cat);
            }
            outer.append(&row);
        }

        let _view_mod = uikit::view::View::empty().backgroundExtensionEffect(true);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(220.0,110.0) }
}

impl Widget for BackgroundExtensionEffect {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,220.0,110.0)) }
    fn is_interactive(&self) -> bool { false }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}
