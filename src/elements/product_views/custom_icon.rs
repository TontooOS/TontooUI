//! CustomIconProductView — Creates a product view with a custom icon.

use std::sync::Arc;

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

use crate::elements::store_product::StoreProduct;

/// CustomIconProductView — initializer — Creates a view to load an individual
/// product from the App Store with a custom icon.
///
/// Usable: configure through [`StoreProduct`] builders and handle the buy
/// action through `on_buy` (the price pill is a real button). The hosting app
/// connects its own kit (e.g. StoreKit) inside the callback.
pub struct CustomIconProductView {
    id: WidgetId,
    product_id: String,
    product: StoreProduct,
    on_buy: Option<Arc<dyn Fn() + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl CustomIconProductView {
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

impl Default for CustomIconProductView { fn default() -> Self { Self::new() } }

impl ViewContent for CustomIconProductView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("pv-ci-phone");
        uikit::widget::apply_css(&phone, ".pv-ci-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        let title = gtk::Label::new(Some(self.product.title.as_str()));
        title.set_halign(gtk::Align::Center);
        title.add_css_class("pv-ci-title");
        uikit::widget::apply_css(&title, ".pv-ci-title { color: rgba(255,255,255,0.9); font-family: 'SF Pro Display'; font-size: 8px; font-weight: 600; margin-top: 22px; }");
        phone.append(&title);
        let sub = gtk::Label::new(Some(self.product.subtitle.as_str()));
        sub.set_halign(gtk::Align::Center);
        sub.add_css_class("pv-ci-sub");
        uikit::widget::apply_css(&sub, ".pv-ci-sub { color: rgba(255,255,255,0.45); font-family: 'SF Pro Display'; font-size: 6px; }");
        phone.append(&sub);
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        row.set_halign(gtk::Align::Center);
        row.set_size_request(180, 34);
        let cat = gtk::Box::new(gtk::Orientation::Vertical, 0);
        cat.set_size_request(22, 24);
        cat.set_valign(gtk::Align::Center);
        cat.add_css_class("pv-ci-cat");
        uikit::widget::apply_css(&cat, ".pv-ci-cat { background: #d8b48a; border-radius: 6px; min-width: 22px; min-height: 24px; margin-left: 40px; }");
        row.append(&cat);
        // Price pill is the real buy button.
        let buy = gtk::Button::new();
        buy.set_halign(gtk::Align::End);
        buy.set_valign(gtk::Align::Center);
        buy.set_hexpand(true);
        buy.add_css_class("pv-ci-buy");
        uikit::widget::apply_css(&buy, ".pv-ci-buy { background: transparent; border: none; outline: none; } .pv-ci-buy:focus { outline: none; }");
        let pill = gtk::Label::new(Some(self.product.price.as_str()));
        pill.add_css_class("pv-ci-pill");
        uikit::widget::apply_css(&pill, ".pv-ci-pill { color: #0A84FF; font-family: 'SF Pro Display'; font-size: 7px; margin-right: 30px; }");
        buy.set_child(Some(&pill));
        if let Some(cb) = &self.on_buy {
            let cb = cb.clone();
            buy.connect_clicked(move |_| cb());
        }
        row.append(&buy);
        phone.append(&row);
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for CustomIconProductView {
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
    fn custom_icon_exists() { let _ = CustomIconProductView::new().product_id("a.b.c"); }
    #[test]
    fn custom_icon_product_and_buy() {
        let v = CustomIconProductView::new().title("Pro").price("$4.99").on_buy(|| {});
        assert_eq!(v.product.title, "Pro");
        assert_eq!(v.product.price, "$4.99");
        assert!(v.on_buy.is_some());
    }
}
