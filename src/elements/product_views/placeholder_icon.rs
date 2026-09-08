//! PlaceholderIconProductView — Creates a product view with a placeholder icon.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// PlaceholderIconProductView — initializer — Creates a view to load an
/// individual product from the App Store, with a placeholder icon.
pub struct PlaceholderIconProductView {
    id: WidgetId,
    product_id: String,
    position_mode: PositionMode,
    position: Position,
}

impl PlaceholderIconProductView {
    pub fn new() -> Self {
        Self { id: next_widget_id(), product_id: "vip.kitty.pass".to_string(), position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn product_id(mut self, v: impl Into<String>) -> Self { self.product_id = v.into(); self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for PlaceholderIconProductView { fn default() -> Self { Self::new() } }

impl ViewContent for PlaceholderIconProductView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("pv-ph-phone");
        uikit::widget::apply_css(&phone, ".pv-ph-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        // Skeleton bars hinting a loading product
        let bar1 = gtk::Box::new(gtk::Orientation::Vertical, 0);
        bar1.set_size_request(70, 8);
        bar1.set_halign(gtk::Align::Center);
        bar1.add_css_class("pv-ph-bar1");
        uikit::widget::apply_css(&bar1, ".pv-ph-bar1 { background: rgba(255,255,255,0.14); border-radius: 4px; min-width: 70px; min-height: 8px; margin-top: 26px; }");
        phone.append(&bar1);
        let bar2 = gtk::Box::new(gtk::Orientation::Vertical, 0);
        bar2.set_size_request(46, 6);
        bar2.set_halign(gtk::Align::Center);
        bar2.add_css_class("pv-ph-bar2");
        uikit::widget::apply_css(&bar2, ".pv-ph-bar2 { background: rgba(255,255,255,0.10); border-radius: 3px; min-width: 46px; min-height: 6px; margin-top: 6px; }");
        phone.append(&bar2);
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        row.set_halign(gtk::Align::Center);
        row.set_size_request(180, 30);
        let star = gtk::Label::new(Some("☆"));
        star.set_valign(gtk::Align::Center);
        star.add_css_class("pv-ph-star");
        uikit::widget::apply_css(&star, ".pv-ph-star { color: rgba(255,255,255,0.35); font-family: 'SF Pro Display'; font-size: 10px; margin-left: 30px; }");
        row.append(&star);
        let pill = gtk::Box::new(gtk::Orientation::Vertical, 0);
        pill.set_size_request(34, 14);
        pill.set_valign(gtk::Align::Center);
        pill.set_hexpand(true);
        pill.set_halign(gtk::Align::End);
        pill.add_css_class("pv-ph-pill");
        uikit::widget::apply_css(&pill, ".pv-ph-pill { background: rgba(255,255,255,0.14); border-radius: 7px; min-width: 34px; min-height: 14px; margin-right: 24px; }");
        row.append(&pill);
        phone.append(&row);
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for PlaceholderIconProductView {
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
    fn placeholder_exists() { let _ = PlaceholderIconProductView::new().product_id("a.b.c"); }
}
