//! LargeProductViewStyle — A large hero style for product views.

use std::sync::Arc;

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

use crate::elements::store_product::StoreProduct;

/// LargeProductViewStyle — style — A style for a product view that is suitable
/// for layouts where the in-app purchase is the hero content.
///
/// Usable: configure through [`StoreProduct`] builders and handle the buy
/// action through `on_buy` (the price is a real button).
pub struct LargeProductViewStyle {
    id: WidgetId,
    product: StoreProduct,
    on_buy: Option<Arc<dyn Fn() + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl LargeProductViewStyle {
    pub fn new() -> Self {
        Self { id: next_widget_id(), product: StoreProduct::monthly_default(), on_buy: None, position_mode: PositionMode::Auto, position: Position::new() }
    }
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

impl Default for LargeProductViewStyle { fn default() -> Self { Self::new() } }

impl ViewContent for LargeProductViewStyle {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 2);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("pv-lg-phone");
        uikit::widget::apply_css(&phone, ".pv-lg-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        let cats = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        cats.set_halign(gtk::Align::Center);
        cats.add_css_class("pv-lg-cats");
        uikit::widget::apply_css(&cats, ".pv-lg-cats { margin-top: 12px; }");
        for (i, col) in ["#d8b48a", "#c9a86a", "#e8c9a0"].iter().enumerate() {
            let cat = gtk::Box::new(gtk::Orientation::Vertical, 0);
            cat.set_size_request(30, 36);
            cat.add_css_class(&format!("pv-lg-cat{}", i));
            uikit::widget::apply_css(&cat, &format!(".pv-lg-cat{} {{ background: {}; border-radius: 8px; min-width: 30px; min-height: 36px; }}", i, col));
            let hat = gtk::Label::new(Some("🎉"));
            hat.set_halign(gtk::Align::Center);
            uikit::widget::apply_css(&hat, ".pv-lg-hat { font-size: 9px; }");
            hat.add_css_class("pv-lg-hat");
            cat.append(&hat);
            cats.append(&cat);
        }
        phone.append(&cats);
        let title = gtk::Label::new(Some(self.product.title.as_str()));
        title.set_halign(gtk::Align::Center);
        title.add_css_class("pv-lg-title");
        uikit::widget::apply_css(&title, ".pv-lg-title { color: rgba(255,255,255,0.95); font-family: 'SF Pro Display'; font-size: 8px; font-weight: 700; }");
        phone.append(&title);
        let sub = gtk::Label::new(Some(self.product.subtitle.as_str()));
        sub.set_halign(gtk::Align::Center);
        sub.add_css_class("pv-lg-sub");
        uikit::widget::apply_css(&sub, ".pv-lg-sub { color: rgba(255,255,255,0.5); font-family: 'SF Pro Display'; font-size: 6px; }");
        phone.append(&sub);
        let buy = gtk::Button::new();
        buy.set_halign(gtk::Align::Center);
        buy.add_css_class("pv-lg-buy");
        uikit::widget::apply_css(&buy, ".pv-lg-buy { background: transparent; border: none; outline: none; } .pv-lg-buy:focus { outline: none; }");
        let pill = gtk::Label::new(Some(self.product.price.as_str()));
        pill.add_css_class("pv-lg-pill");
        uikit::widget::apply_css(&pill, ".pv-lg-pill { color: #0A84FF; font-family: 'SF Pro Display'; font-size: 6px; margin-bottom: 8px; }");
        buy.set_child(Some(&pill));
        if let Some(cb) = &self.on_buy {
            let cb = cb.clone();
            buy.connect_clicked(move |_| cb());
        }
        phone.append(&buy);
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for LargeProductViewStyle {
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
    fn large_exists() { let _ = LargeProductViewStyle::new(); }
    #[test]
    fn large_product_and_buy() {
        let v = LargeProductViewStyle::new().subtitle("Best").on_buy(|| {});
        assert_eq!(v.product.subtitle, "Best");
        assert!(v.on_buy.is_some());
    }
}
