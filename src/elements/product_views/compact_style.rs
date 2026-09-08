//! CompactProductViewStyle — A compact style for product views.

use std::sync::Arc;

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

use crate::elements::store_product::StoreProduct;

/// CompactProductViewStyle — style — A style for a product view that is
/// suitable for layouts with less available space.
///
/// Usable: configure through [`StoreProduct`] builders and handle the buy
/// action through `on_buy` (the price is a real button).
pub struct CompactProductViewStyle {
    id: WidgetId,
    product: StoreProduct,
    on_buy: Option<Arc<dyn Fn() + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl CompactProductViewStyle {
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

impl Default for CompactProductViewStyle { fn default() -> Self { Self::new() } }

impl ViewContent for CompactProductViewStyle {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 0);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("pv-cp-phone");
        uikit::widget::apply_css(&phone, ".pv-cp-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; }");
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        row.set_size_request(160, 44);
        row.set_halign(gtk::Align::Center);
        row.set_valign(gtk::Align::Center);
        row.set_vexpand(true);
        row.add_css_class("pv-cp-row");
        uikit::widget::apply_css(&row, ".pv-cp-row { margin-top: 33px; margin-bottom: 33px; }");
        let thumb = gtk::Box::new(gtk::Orientation::Vertical, 0);
        thumb.set_size_request(28, 32);
        thumb.set_valign(gtk::Align::Center);
        thumb.add_css_class("pv-cp-thumb");
        uikit::widget::apply_css(&thumb, ".pv-cp-thumb { background: #d8b48a; border-radius: 6px; min-width: 28px; min-height: 32px; }");
        row.append(&thumb);
        let texts = gtk::Box::new(gtk::Orientation::Vertical, 0);
        texts.set_valign(gtk::Align::Center);
        texts.set_hexpand(true);
        let t = gtk::Label::new(Some(self.product.title.as_str()));
        t.set_halign(gtk::Align::Start);
        t.add_css_class("pv-cp-t");
        uikit::widget::apply_css(&t, ".pv-cp-t { color: rgba(255,255,255,0.9); font-family: 'SF Pro Display'; font-size: 7px; font-weight: 600; }");
        texts.append(&t);
        let s = gtk::Label::new(Some(self.product.subtitle.as_str()));
        s.set_halign(gtk::Align::Start);
        s.add_css_class("pv-cp-s");
        uikit::widget::apply_css(&s, ".pv-cp-s { color: rgba(255,255,255,0.45); font-family: 'SF Pro Display'; font-size: 5px; }");
        texts.append(&s);
        row.append(&texts);
        let buy = gtk::Button::new();
        buy.set_valign(gtk::Align::Center);
        buy.add_css_class("pv-cp-buy");
        uikit::widget::apply_css(&buy, ".pv-cp-buy { background: transparent; border: none; outline: none; } .pv-cp-buy:focus { outline: none; }");
        let price = gtk::Label::new(Some(self.product.price.as_str()));
        price.add_css_class("pv-cp-price");
        uikit::widget::apply_css(&price, ".pv-cp-price { color: #0A84FF; font-family: 'SF Pro Display'; font-size: 8px; font-weight: 700; }");
        buy.set_child(Some(&price));
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

impl Widget for CompactProductViewStyle {
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
    fn compact_exists() { let _ = CompactProductViewStyle::new(); }
    #[test]
    fn compact_product_and_buy() {
        let v = CompactProductViewStyle::new().title("Pro").on_buy(|| {});
        assert_eq!(v.product.title, "Pro");
        assert!(v.on_buy.is_some());
    }
}
