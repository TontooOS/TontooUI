//! SingleSubscriptionStoreView — Loads subscriptions for a single product.

use std::sync::Arc;

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

use crate::elements::store_product::StoreProduct;

/// SingleSubscriptionStoreView — initializer — Creates a view that loads
/// subscriptions based on a collection of products (single).
///
/// Usable: set the hero `product` and handle `on_subscribe` (Subscribe is
/// a real button).
pub struct SingleSubscriptionStoreView {
    id: WidgetId,
    product_id: String,
    product: StoreProduct,
    on_subscribe: Option<Arc<dyn Fn() + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl SingleSubscriptionStoreView {
    pub fn new() -> Self {
        Self { id: next_widget_id(), product_id: "vip.kitty.pass".to_string(), product: StoreProduct::new("Kitty Pass", "", "$0.99/month"), on_subscribe: None, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn product_id(mut self, v: impl Into<String>) -> Self { self.product_id = v.into(); self }
    /// Set the hero title.
    pub fn title(mut self, v: impl Into<String>) -> Self { self.product.title = v.into(); self }
    /// Set the hero price.
    pub fn price(mut self, v: impl Into<String>) -> Self { self.product.price = v.into(); self }
    /// Replace the whole product.
    pub fn product(mut self, v: StoreProduct) -> Self { self.product = v; self }
    /// Called when Subscribe is pressed.
    pub fn on_subscribe(mut self, f: impl Fn() + Send + Sync + 'static) -> Self { self.on_subscribe = Some(Arc::new(f)); self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for SingleSubscriptionStoreView { fn default() -> Self { Self::new() } }

impl ViewContent for SingleSubscriptionStoreView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 6);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("ss-sg-phone");
        uikit::widget::apply_css(&phone, ".ss-sg-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        let icon = gtk::Box::new(gtk::Orientation::Vertical, 0);
        icon.set_size_request(40, 40);
        icon.set_halign(gtk::Align::Center);
        icon.add_css_class("ss-sg-icon");
        uikit::widget::apply_css(&icon, ".ss-sg-icon { background: #1c1c1e; border: 1px solid rgba(255,255,255,0.12); border-radius: 10px; min-width: 40px; min-height: 40px; margin-top: 16px; }");
        let grid = gtk::Label::new(Some("▦"));
        grid.set_halign(gtk::Align::Center);
        grid.set_valign(gtk::Align::Center);
        grid.set_vexpand(true);
        grid.add_css_class("ss-sg-grid");
        uikit::widget::apply_css(&grid, ".ss-sg-grid { color: rgba(255,255,255,0.5); font-size: 14px; }");
        icon.append(&grid);
        phone.append(&icon);
        let title = gtk::Label::new(Some(self.product.title.as_str()));
        title.set_halign(gtk::Align::Center);
        title.add_css_class("ss-sg-title");
        uikit::widget::apply_css(&title, ".ss-sg-title { color: white; font-family: 'SF Pro Display'; font-size: 11px; font-weight: 700; }");
        phone.append(&title);
        let price = gtk::Label::new(Some(self.product.price.as_str()));
        price.set_halign(gtk::Align::Center);
        price.add_css_class("ss-sg-price");
        uikit::widget::apply_css(&price, ".ss-sg-price { color: rgba(255,255,255,0.6); font-family: 'SF Pro Display'; font-size: 7px; }");
        phone.append(&price);
        let sub_btn = gtk::Button::new();
        sub_btn.set_size_request(120, 18);
        sub_btn.set_halign(gtk::Align::Center);
        sub_btn.add_css_class("ss-sg-btn");
        uikit::widget::apply_css(&sub_btn, ".ss-sg-btn { background: #0A84FF; border: none; outline: none; border-radius: 9px; min-width: 120px; min-height: 18px; margin-bottom: 8px; } .ss-sg-btn:focus { outline: none; }");
        let bl = gtk::Label::new(Some("Subscribe"));
        bl.set_halign(gtk::Align::Center);
        bl.add_css_class("ss-sg-bl");
        uikit::widget::apply_css(&bl, ".ss-sg-bl { color: white; font-family: 'SF Pro Display'; font-size: 7px; font-weight: 600; }");
        sub_btn.set_child(Some(&bl));
        if let Some(cb) = &self.on_subscribe {
            let cb = cb.clone();
            sub_btn.connect_clicked(move |_| cb());
        }
        phone.append(&sub_btn);
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for SingleSubscriptionStoreView {
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
    fn single_exists() { let _ = SingleSubscriptionStoreView::new().product_id("a.b.c"); }
    #[test]
    fn single_product_and_subscribe() {
        let v = SingleSubscriptionStoreView::new().title("Pass").on_subscribe(|| {});
        assert_eq!(v.product.title, "Pass");
        assert!(v.on_subscribe.is_some());
    }
}
