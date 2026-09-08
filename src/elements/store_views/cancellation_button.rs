//! StoreCancellationButton — Dismisses the current store presentation.

use std::sync::Arc;

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

use crate::elements::store_product::StoreProduct;

/// StoreCancellationButton — modifier — A type of button that people use to
/// dismiss the current store presentation.
///
/// Usable: set the listed `products` and handle `on_cancel` (the X circle is
/// a real button).
pub struct StoreCancellationButton {
    id: WidgetId,
    products: Vec<StoreProduct>,
    on_cancel: Option<Arc<dyn Fn() + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl StoreCancellationButton {
    pub fn new() -> Self {
        Self { id: next_widget_id(), products: vec![StoreProduct::monthly_default(), StoreProduct::hat_default()], on_cancel: None, position_mode: PositionMode::Auto, position: Position::new() }
    }
    /// Replace the listed products.
    pub fn products(mut self, v: Vec<StoreProduct>) -> Self { self.products = v; self }
    /// Append one listed product.
    pub fn product(mut self, title: impl Into<String>, subtitle: impl Into<String>, price: impl Into<String>) -> Self {
        self.products.push(StoreProduct::new(title, subtitle, price));
        self
    }
    /// Clear the listed products.
    pub fn clear_products(mut self) -> Self { self.products.clear(); self }
    /// Called when the X button is pressed.
    pub fn on_cancel(mut self, f: impl Fn() + Send + Sync + 'static) -> Self { self.on_cancel = Some(Arc::new(f)); self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for StoreCancellationButton { fn default() -> Self { Self::new() } }

impl ViewContent for StoreCancellationButton {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let overlay = gtk::Overlay::new();
        overlay.set_size_request(180, 110);
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 6);
        phone.set_size_request(180, 110);
        phone.add_css_class("sv-cb-phone");
        uikit::widget::apply_css(&phone, ".sv-cb-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; padding: 12px 12px; }");
        for product in &self.products {
            let row = gtk::Box::new(gtk::Orientation::Vertical, 0);
            let l = gtk::Label::new(Some(product.title.as_str()));
            l.set_halign(gtk::Align::Start);
            l.add_css_class("sv-cb-l");
            uikit::widget::apply_css(&l, ".sv-cb-l { color: rgba(255,255,255,0.9); font-family: 'SF Pro Display'; font-size: 7px; font-weight: 600; }");
            row.append(&l);
            let s = gtk::Label::new(Some(product.subtitle.as_str()));
            s.set_halign(gtk::Align::Start);
            s.add_css_class("sv-cb-s");
            uikit::widget::apply_css(&s, ".sv-cb-s { color: rgba(255,255,255,0.45); font-family: 'SF Pro Display'; font-size: 5px; }");
            row.append(&s);
            let p = gtk::Label::new(Some(product.price.as_str()));
            p.set_halign(gtk::Align::Start);
            p.add_css_class("sv-cb-p");
            uikit::widget::apply_css(&p, ".sv-cb-p { color: #0A84FF; font-family: 'SF Pro Display'; font-size: 6px; }");
            row.append(&p);
            phone.append(&row);
        }
        overlay.set_child(Some(&phone));
        // Dismiss X top-right — a real button.
        let x = gtk::Button::new();
        x.set_size_request(20, 20);
        x.set_halign(gtk::Align::End);
        x.set_valign(gtk::Align::Start);
        x.add_css_class("sv-cb-x");
        uikit::widget::apply_css(&x, ".sv-cb-x { background: rgba(255,255,255,0.14); border: none; outline: none; border-radius: 10px; min-width: 20px; min-height: 20px; margin: 8px; } .sv-cb-x:focus { outline: none; }");
        let xl = gtk::Label::new(Some("✕"));
        xl.set_halign(gtk::Align::Center);
        xl.set_valign(gtk::Align::Center);
        xl.set_vexpand(true);
        xl.add_css_class("sv-cb-xl");
        uikit::widget::apply_css(&xl, ".sv-cb-xl { color: rgba(255,255,255,0.8); font-size: 8px; }");
        x.set_child(Some(&xl));
        if let Some(cb) = &self.on_cancel {
            let cb = cb.clone();
            x.connect_clicked(move |_| cb());
        }
        overlay.add_overlay(&x);
        outer.append(&overlay);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for StoreCancellationButton {
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
    fn cancel_exists() { let _ = StoreCancellationButton::new(); }
    #[test]
    fn cancel_products_and_callback() {
        let v = StoreCancellationButton::new().clear_products().product("A", "S", "$1").on_cancel(|| {});
        assert_eq!(v.products.len(), 1);
        assert!(v.on_cancel.is_some());
    }
}
