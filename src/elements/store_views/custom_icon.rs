//! CustomIconStoreView — Loads a product collection with custom icons.

use std::sync::Arc;

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

use crate::elements::store_product::StoreProduct;

/// CustomIconStoreView — initializer — Creates a view to load a collection of
/// products from the App Store using custom icons.
///
/// Usable: set the `products`, mark `selected` and handle `on_select`
/// (rows are real buttons).
pub struct CustomIconStoreView {
    id: WidgetId,
    products: Vec<StoreProduct>,
    selected: usize,
    on_select: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl CustomIconStoreView {
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

impl Default for CustomIconStoreView { fn default() -> Self { Self::new() } }

impl ViewContent for CustomIconStoreView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 6);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("sv-ci-phone");
        uikit::widget::apply_css(&phone, ".sv-ci-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; padding: 12px 12px; }");
        for (i, product) in self.products.iter().enumerate() {
            let row_btn = gtk::Button::new();
            row_btn.add_css_class("sv-ci-rowbtn");
            uikit::widget::apply_css(&row_btn, ".sv-ci-rowbtn { background: transparent; border: none; outline: none; padding: 0; } .sv-ci-rowbtn:focus { outline: none; } .sv-ci-rowbtn:hover { background: rgba(255,255,255,0.04); }");
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            let star = gtk::Label::new(Some("☆"));
            star.add_css_class("sv-ci-star");
            uikit::widget::apply_css(&star, ".sv-ci-star { color: rgba(255,255,255,0.6); font-family: 'SF Pro Display'; font-size: 10px; }");
            row.append(&star);
            let texts = gtk::Box::new(gtk::Orientation::Vertical, 0);
            texts.set_hexpand(true);
            let l = gtk::Label::new(Some(product.title.as_str()));
            l.set_halign(gtk::Align::Start);
            l.add_css_class("sv-ci-l");
            uikit::widget::apply_css(&l, ".sv-ci-l { color: rgba(255,255,255,0.9); font-family: 'SF Pro Display'; font-size: 7px; font-weight: 600; }");
            texts.append(&l);
            let s = gtk::Label::new(Some(product.subtitle.as_str()));
            s.set_halign(gtk::Align::Start);
            s.add_css_class("sv-ci-s");
            uikit::widget::apply_css(&s, ".sv-ci-s { color: rgba(255,255,255,0.45); font-family: 'SF Pro Display'; font-size: 5px; }");
            texts.append(&s);
            row.append(&texts);
            let p = gtk::Label::new(Some(product.price.as_str()));
            p.set_halign(gtk::Align::End);
            p.add_css_class("sv-ci-p");
            uikit::widget::apply_css(&p, ".sv-ci-p { color: #0A84FF; font-family: 'SF Pro Display'; font-size: 7px; font-weight: 700; }");
            row.append(&p);
            if i == self.selected {
                let check = gtk::Label::new(Some("✓"));
                check.add_css_class("sv-ci-check");
                uikit::widget::apply_css(&check, ".sv-ci-check { color: #0A84FF; font-size: 8px; font-weight: 700; }");
                row.append(&check);
            }
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

impl Widget for CustomIconStoreView {
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
    fn custom_icon_exists() { let _ = CustomIconStoreView::new(); }
    #[test]
    fn custom_icon_products_and_select() {
        let v = CustomIconStoreView::new().clear_products().product("A", "S", "$1").selected(0).on_select(|_| {});
        assert_eq!(v.products.len(), 1);
        assert!(v.on_select.is_some());
    }
}
