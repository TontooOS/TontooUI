//! StoreViewElement — Loads and merchandises a collection of products.

use std::sync::Arc;

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

use crate::elements::store_product::StoreProduct;

/// StoreViewElement — initializer — Creates a view to load and merchandise a
/// collection of products from the App Store.
///
/// Usable: set the `products`, mark `selected` and handle `on_select`
/// (rows are real buttons).
pub struct StoreViewElement {
    id: WidgetId,
    products: Vec<StoreProduct>,
    selected: usize,
    on_select: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl StoreViewElement {
    pub fn new() -> Self {
        Self { id: next_widget_id(), products: vec![StoreProduct::monthly_default(), StoreProduct::hat_default()], selected: 0, on_select: None, position_mode: PositionMode::Auto, position: Position::new() }
    }
    /// Replace the whole product list.
    pub fn products(mut self, v: Vec<StoreProduct>) -> Self { self.products = v; self }
    /// Append one product.
    pub fn product(mut self, title: impl Into<String>, subtitle: impl Into<String>, price: impl Into<String>) -> Self {
        self.products.push(StoreProduct::new(title, subtitle, price));
        self
    }
    /// Clear all products.
    pub fn clear_products(mut self) -> Self { self.products.clear(); self }
    /// Mark one product selected (controlled state, shown with a checkmark).
    pub fn selected(mut self, v: usize) -> Self { self.selected = v; self }
    /// Called with the row index when a product row is pressed.
    pub fn on_select(mut self, f: impl Fn(usize) + Send + Sync + 'static) -> Self { self.on_select = Some(Arc::new(f)); self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for StoreViewElement { fn default() -> Self { Self::new() } }

impl ViewContent for StoreViewElement {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 6);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("sv-sv-phone");
        uikit::widget::apply_css(&phone, ".sv-sv-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; padding: 12px 12px; }");
        for (i, product) in self.products.iter().enumerate() {
            let row_btn = gtk::Button::new();
            row_btn.add_css_class("sv-sv-rowbtn");
            uikit::widget::apply_css(&row_btn, ".sv-sv-rowbtn { background: transparent; border: none; outline: none; padding: 0; } .sv-sv-rowbtn:focus { outline: none; } .sv-sv-rowbtn:hover { background: rgba(255,255,255,0.04); }");
            let row = gtk::Box::new(gtk::Orientation::Vertical, 0);
            let head = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            let l = gtk::Label::new(Some(product.title.as_str()));
            l.set_halign(gtk::Align::Start);
            l.set_hexpand(true);
            l.add_css_class("sv-sv-l");
            uikit::widget::apply_css(&l, ".sv-sv-l { color: rgba(255,255,255,0.9); font-family: 'SF Pro Display'; font-size: 7px; font-weight: 600; }");
            head.append(&l);
            if i == self.selected {
                let check = gtk::Label::new(Some("✓"));
                check.add_css_class("sv-sv-check");
                uikit::widget::apply_css(&check, ".sv-sv-check { color: #0A84FF; font-size: 8px; font-weight: 700; }");
                head.append(&check);
            }
            row.append(&head);
            let s = gtk::Label::new(Some(product.subtitle.as_str()));
            s.set_halign(gtk::Align::Start);
            s.add_css_class("sv-sv-s");
            uikit::widget::apply_css(&s, ".sv-sv-s { color: rgba(255,255,255,0.45); font-family: 'SF Pro Display'; font-size: 5px; }");
            row.append(&s);
            let h = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            let spacer = gtk::Box::new(gtk::Orientation::Vertical, 0);
            spacer.set_hexpand(true);
            h.append(&spacer);
            let p = gtk::Label::new(Some(product.price.as_str()));
            p.add_css_class("sv-sv-p");
            uikit::widget::apply_css(&p, ".sv-sv-p { color: #0A84FF; font-family: 'SF Pro Display'; font-size: 6px; }");
            h.append(&p);
            row.append(&h);
            row_btn.set_child(Some(&row));
            if let Some(cb) = &self.on_select {
                let cb = cb.clone();
                row_btn.connect_clicked(move |_| cb(i));
            }
            phone.append(&row_btn);
        }
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for StoreViewElement {
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
    fn store_exists() { let _ = StoreViewElement::new(); }
    #[test]
    fn store_products_and_select() {
        let v = StoreViewElement::new().clear_products().product("A", "S", "$1").selected(0).on_select(|_| {});
        assert_eq!(v.products.len(), 1);
        assert!(v.on_select.is_some());
    }
}
