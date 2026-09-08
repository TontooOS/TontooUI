//! PlaceholderIconStoreView — Loads a product collection with placeholder icons.

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// PlaceholderIconStoreView — initializer — Creates a view to load a
/// collection of products from the App Store using placeholder icons.
///
/// Usable as a loading state: `rows` controls how many skeleton bars render.
pub struct PlaceholderIconStoreView {
    id: WidgetId,
    rows: usize,
    position_mode: PositionMode,
    position: Position,
}

impl PlaceholderIconStoreView {
    pub fn new() -> Self {
        Self { id: next_widget_id(), rows: 3, position_mode: PositionMode::Auto, position: Position::new() }
    }
    /// How many skeleton bars to render (default 3).
    pub fn rows(mut self, v: usize) -> Self { self.rows = v; self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for PlaceholderIconStoreView { fn default() -> Self { Self::new() } }

impl ViewContent for PlaceholderIconStoreView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 8);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("sv-pi-phone");
        uikit::widget::apply_css(&phone, ".sv-pi-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; padding: 16px 20px; }");
        let widths = [120, 90, 60];
        let alphas = [0.14, 0.10, 0.08];
        for i in 0..self.rows {
            let w = widths[i % widths.len()];
            let o = alphas[i % alphas.len()];
            let bar = gtk::Box::new(gtk::Orientation::Vertical, 0);
            bar.set_size_request(w, 8);
            bar.set_halign(gtk::Align::Start);
            bar.add_css_class(&format!("sv-pi-bar{}", w));
            uikit::widget::apply_css(&bar, &format!(".sv-pi-bar{} {{ background: rgba(255,255,255,{}); border-radius: 4px; min-width: {}px; min-height: 8px; }}", w, o, w));
            phone.append(&bar);
        }
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for PlaceholderIconStoreView {
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
    fn placeholder_exists() { let _ = PlaceholderIconStoreView::new(); }
    #[test]
    fn placeholder_rows() {
        assert_eq!(PlaceholderIconStoreView::new().rows(5).rows, 5);
    }
}
