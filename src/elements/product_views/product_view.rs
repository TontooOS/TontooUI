//! ProductViewElement — Creates a view to load and merchandise a product.

use std::sync::Arc;

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

use crate::elements::store_product::StoreProduct;

/// ProductViewElement — initializer — Creates a view to load and merchandise
/// an individual product from the App Store.
///
/// Usable: configure through [`StoreProduct`] builders and handle the buy
/// action through `on_buy` (the price line is a real button).
pub struct ProductViewElement {
    id: WidgetId,
    product_id: String,
    product: StoreProduct,
    on_buy: Option<Arc<dyn Fn() + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl ProductViewElement {
    pub fn new() -> Self {
        Self { id: next_widget_id(), product_id: "vip.kitty.pass".to_string(), product: StoreProduct::monthly_default(), on_buy: None, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn product_id(mut self, v: impl Into<String>) -> Self { self.product_id = v.into(); self }
    /// Set the displayed title.
    pub fn title(mut self, v: impl Into<String>) -> Self { self.product.title = v.into(); self }
    /// Set the displayed subtitle.
    pub fn subtitle(mut self, v: impl Into<String>) -> Self { self.product.subtitle = v.into(); self }
    /// Set the displayed price.
    pub fn price(mut self, v: impl Into<String>) -> Self { self.product.price = v.into(); self }
    /// Replace the whole product.
    pub fn product(mut self, v: StoreProduct) -> Self { self.product = v; self }
    /// Called when the buy (price) button is pressed.
    pub fn on_buy(mut self, f: impl Fn() + Send + Sync + 'static) -> Self { self.on_buy = Some(Arc::new(f)); self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for ProductViewElement { fn default() -> Self { Self::new() } }

impl ViewContent for ProductViewElement {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("pv-pv-phone");
        uikit::widget::apply_css(&phone, ".pv-pv-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        let head = gtk::Label::new(Some(self.product.title.as_str()));
        head.set_halign(gtk::Align::Center);
        head.add_css_class("pv-pv-head");
        uikit::widget::apply_css(&head, ".pv-pv-head { color: rgba(255,255,255,0.9); font-family: 'SF Pro Display'; font-size: 7px; font-weight: 600; margin-top: 10px; }");
        phone.append(&head);
        let sub = gtk::Label::new(Some(self.product.subtitle.as_str()));
        sub.set_halign(gtk::Align::Center);
        sub.add_css_class("pv-pv-sub");
        uikit::widget::apply_css(&sub, ".pv-pv-sub { color: rgba(255,255,255,0.45); font-family: 'SF Pro Display'; font-size: 6px; }");
        phone.append(&sub);
        // Price line is the real buy button.
        let buy = gtk::Button::new();
        buy.set_halign(gtk::Align::Center);
        buy.add_css_class("pv-pv-buy");
        uikit::widget::apply_css(&buy, ".pv-pv-buy { background: transparent; border: none; outline: none; } .pv-pv-buy:focus { outline: none; }");
        let price_top = gtk::Label::new(Some(self.product.price.as_str()));
        price_top.add_css_class("pv-pv-top");
        uikit::widget::apply_css(&price_top, ".pv-pv-top { color: #30d158; font-family: 'SF Pro Display'; font-size: 6px; }");
        buy.set_child(Some(&price_top));
        if let Some(cb) = &self.on_buy {
            let cb = cb.clone();
            buy.connect_clicked(move |_| cb());
        }
        phone.append(&buy);
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        row.set_halign(gtk::Align::Center);
        row.set_size_request(180, 40);
        for (i, col) in ["#d8b48a", "#c9a86a"].iter().enumerate() {
            let cat = gtk::Box::new(gtk::Orientation::Vertical, 0);
            cat.set_size_request(26, 30);
            cat.set_valign(gtk::Align::Center);
            cat.add_css_class(&format!("pv-pv-cat{}", i));
            uikit::widget::apply_css(&cat, &format!(".pv-pv-cat{} {{ background: {}; border-radius: 6px; min-width: 26px; min-height: 30px; }}", i, col));
            row.append(&cat);
        }
        let texts = gtk::Box::new(gtk::Orientation::Vertical, 0);
        texts.set_valign(gtk::Align::Center);
        let t = gtk::Label::new(Some(self.product.title.as_str()));
        t.set_halign(gtk::Align::Start);
        t.add_css_class("pv-pv-t");
        uikit::widget::apply_css(&t, ".pv-pv-t { color: rgba(255,255,255,0.9); font-family: 'SF Pro Display'; font-size: 7px; font-weight: 600; }");
        texts.append(&t);
        let s = gtk::Label::new(Some(self.product.subtitle.as_str()));
        s.set_halign(gtk::Align::Start);
        s.add_css_class("pv-pv-s");
        uikit::widget::apply_css(&s, ".pv-pv-s { color: rgba(255,255,255,0.45); font-family: 'SF Pro Display'; font-size: 5px; }");
        texts.append(&s);
        row.append(&texts);
        phone.append(&row);
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for ProductViewElement {
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
    fn product_exists() { let _ = ProductViewElement::new().product_id("a.b.c"); }
    #[test]
    fn product_builders_and_buy() {
        let v = ProductViewElement::new().title("Pro").subtitle("Best").price("$4.99").on_buy(|| {});
        assert_eq!(v.product.title, "Pro");
        assert_eq!(v.product.subtitle, "Best");
        assert!(v.on_buy.is_some());
    }
}
