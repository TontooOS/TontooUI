//! IconPhaseStoreView — Loads a product collection with icon phases.

use std::sync::Arc;

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

use crate::elements::store_product::StoreProduct;

/// Icon phase for store products.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StoreIconPhase {
    #[default]
    Icon,
    Placeholder,
    Unavailable,
}

/// IconPhaseStoreView — initializer — Creates a view to load a collection of
/// products from the App Store using icon phases.
///
/// Usable: set the `products`, read the `selected` index through `on_select`
/// (rows are real buttons) and switch the `phase` (`Placeholder` renders
/// skeleton rows, `Unavailable` marks every row unavailable).
pub struct IconPhaseStoreView {
    id: WidgetId,
    phase: StoreIconPhase,
    products: Vec<StoreProduct>,
    selected: usize,
    on_select: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl IconPhaseStoreView {
    pub fn new() -> Self {
        Self { id: next_widget_id(), phase: StoreIconPhase::Icon, products: vec![StoreProduct::monthly_default(), StoreProduct::hat_default()], selected: 0, on_select: None, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn phase(mut self, v: StoreIconPhase) -> Self { self.phase = v; self }
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

impl Default for IconPhaseStoreView { fn default() -> Self { Self::new() } }

impl ViewContent for IconPhaseStoreView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 4);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("sv-ip-phone");
        uikit::widget::apply_css(&phone, ".sv-ip-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; padding: 10px 12px; }");
        if self.phase == StoreIconPhase::Placeholder {
            // Skeleton rows while loading.
            for (w, o) in [(140, 0.14), (110, 0.10)] {
                let bar = gtk::Box::new(gtk::Orientation::Vertical, 0);
                bar.set_size_request(w, 10);
                bar.set_halign(gtk::Align::Start);
                bar.add_css_class(&format!("sv-ip-sk{}", w));
                uikit::widget::apply_css(&bar, &format!(".sv-ip-sk{} {{ background: rgba(255,255,255,{}); border-radius: 5px; min-width: {}px; min-height: 10px; }}", w, o, w));
                phone.append(&bar);
            }
            outer.append(&phone);
            return outer.upcast();
        }
        let unavailable = self.phase == StoreIconPhase::Unavailable;
        for (i, product) in self.products.iter().enumerate() {
            let row_btn = gtk::Button::new();
            row_btn.add_css_class("sv-ip-rowbtn");
            uikit::widget::apply_css(&row_btn, ".sv-ip-rowbtn { background: transparent; border: none; outline: none; padding: 0; } .sv-ip-rowbtn:focus { outline: none; } .sv-ip-rowbtn:hover { background: rgba(255,255,255,0.04); }");
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            let cat = gtk::Box::new(gtk::Orientation::Vertical, 0);
            cat.set_size_request(24, 28);
            cat.add_css_class("sv-ip-cat");
            uikit::widget::apply_css(&cat, ".sv-ip-cat { background: #d8b48a; border-radius: 6px; min-width: 24px; min-height: 28px; }");
            row.append(&cat);
            let texts = gtk::Box::new(gtk::Orientation::Vertical, 0);
            texts.set_hexpand(true);
            let l = gtk::Label::new(Some(product.title.as_str()));
            l.set_halign(gtk::Align::Start);
            l.add_css_class("sv-ip-l");
            uikit::widget::apply_css(&l, ".sv-ip-l { color: rgba(255,255,255,0.9); font-family: 'SF Pro Display'; font-size: 7px; font-weight: 600; }");
            texts.append(&l);
            let sub = if unavailable { "Unavailable" } else { product.subtitle.as_str() };
            let s = gtk::Label::new(Some(sub));
            s.set_halign(gtk::Align::Start);
            s.add_css_class("sv-ip-s");
            uikit::widget::apply_css(&s, ".sv-ip-s { color: rgba(255,255,255,0.45); font-family: 'SF Pro Display'; font-size: 5px; }");
            texts.append(&s);
            row.append(&texts);
            let p = gtk::Label::new(Some(product.price.as_str()));
            p.set_halign(gtk::Align::End);
            p.add_css_class("sv-ip-p");
            uikit::widget::apply_css(&p, ".sv-ip-p { color: #0A84FF; font-family: 'SF Pro Display'; font-size: 6px; }");
            row.append(&p);
            if i == self.selected {
                let check = gtk::Label::new(Some("✓"));
                check.add_css_class("sv-ip-check");
                uikit::widget::apply_css(&check, ".sv-ip-check { color: #0A84FF; font-size: 8px; font-weight: 700; }");
                row.append(&check);
            }
            row_btn.set_child(Some(&row));
            if !unavailable {
                if let Some(cb) = &self.on_select {
                    let cb = cb.clone();
                    row_btn.connect_clicked(move |_| cb(i));
                }
            }
            phone.append(&row_btn);
        }
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for IconPhaseStoreView {
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
    fn icon_phase_exists() { let _ = IconPhaseStoreView::new().phase(StoreIconPhase::Unavailable); }
    #[test]
    fn icon_phase_products_and_select() {
        let v = IconPhaseStoreView::new()
            .clear_products()
            .product("A", "Sub", "$1")
            .selected(0)
            .on_select(|_| {});
        assert_eq!(v.products.len(), 1);
        assert!(v.on_select.is_some());
    }
}
