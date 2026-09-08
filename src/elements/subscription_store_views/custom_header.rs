//! CustomHeaderSubscriptionStoreView — Subscription store with a custom header.

use std::sync::Arc;

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

use crate::elements::store_product::StoreProduct;

/// CustomHeaderSubscriptionStoreView — initializer — Creates a view that loads
/// all the subscriptions in a subscription group with a custom header.
///
/// Usable: set the `options`, mark `selected` and handle `on_select`
/// (cards are real buttons) plus `on_subscribe` (Subscribe is a button).
pub struct CustomHeaderSubscriptionStoreView {
    id: WidgetId,
    group_id: String,
    header: String,
    options: Vec<StoreProduct>,
    selected: usize,
    on_select: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    on_subscribe: Option<Arc<dyn Fn() + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl CustomHeaderSubscriptionStoreView {
    pub fn new() -> Self {
        Self { id: next_widget_id(), group_id: "kitty.pass.group".to_string(), header: "Foo".to_string(), options: vec![StoreProduct::monthly_default()], selected: 0, on_select: None, on_subscribe: None, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn group_id(mut self, v: impl Into<String>) -> Self { self.group_id = v.into(); self }
    pub fn header(mut self, v: impl Into<String>) -> Self { self.header = v.into(); self }
    /// Replace the whole option list.
    pub fn options(mut self, v: Vec<StoreProduct>) -> Self { self.options = v; self }
    /// Append one option.
    pub fn option(mut self, title: impl Into<String>, subtitle: impl Into<String>, price: impl Into<String>) -> Self {
        self.options.push(StoreProduct::new(title, subtitle, price));
        self
    }
    /// Clear all options.
    pub fn clear_options(mut self) -> Self { self.options.clear(); self }
    /// Mark one option selected (controlled state).
    pub fn selected(mut self, v: usize) -> Self { self.selected = v; self }
    /// Called with the option index when a card is pressed.
    pub fn on_select(mut self, f: impl Fn(usize) + Send + Sync + 'static) -> Self { self.on_select = Some(Arc::new(f)); self }
    /// Called when Subscribe is pressed.
    pub fn on_subscribe(mut self, f: impl Fn() + Send + Sync + 'static) -> Self { self.on_subscribe = Some(Arc::new(f)); self }
    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 110.0)
    }
}

impl Default for CustomHeaderSubscriptionStoreView { fn default() -> Self { Self::new() } }

impl ViewContent for CustomHeaderSubscriptionStoreView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 6);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("ss-ch-phone");
        uikit::widget::apply_css(&phone, ".ss-ch-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; padding: 12px; }");
        // Custom header
        let h = gtk::Label::new(Some(self.header.as_str()));
        h.set_halign(gtk::Align::Center);
        h.add_css_class("ss-ch-h");
        uikit::widget::apply_css(&h, ".ss-ch-h { color: rgba(255,255,255,0.85); font-family: 'SF Pro Display'; font-size: 9px; font-weight: 600; margin-top: 18px; }");
        phone.append(&h);
        // Option cards below the header (clickable).
        for (i, option) in self.options.iter().enumerate() {
            let card_btn = gtk::Button::new();
            card_btn.set_halign(gtk::Align::Center);
            card_btn.add_css_class("ss-ch-cardbtn");
            uikit::widget::apply_css(&card_btn, ".ss-ch-cardbtn { background: transparent; border: none; outline: none; padding: 0; } .ss-ch-cardbtn:focus { outline: none; }");
            let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
            card.set_size_request(156, 34);
            card.add_css_class(&format!("ss-ch-card{}", i));
            if i == self.selected {
                uikit::widget::apply_css(&card, &format!(".ss-ch-card{} {{ border: 1px solid #0A84FF; border-radius: 10px; min-width: 156px; min-height: 34px; padding: 4px 8px; }}", i));
            } else {
                uikit::widget::apply_css(&card, &format!(".ss-ch-card{} {{ border: 1px solid rgba(255,255,255,0.14); border-radius: 10px; min-width: 156px; min-height: 34px; padding: 4px 8px; }}", i));
            }
            let l = gtk::Label::new(Some(option.title.as_str()));
            l.set_halign(gtk::Align::Start);
            l.add_css_class("ss-ch-l");
            uikit::widget::apply_css(&l, ".ss-ch-l { color: white; font-family: 'SF Pro Display'; font-size: 7px; font-weight: 600; }");
            card.append(&l);
            let p = gtk::Label::new(Some(option.price.as_str()));
            p.set_halign(gtk::Align::Start);
            p.add_css_class("ss-ch-p");
            uikit::widget::apply_css(&p, ".ss-ch-p { color: rgba(255,255,255,0.6); font-family: 'SF Pro Display'; font-size: 6px; }");
            card.append(&p);
            card_btn.set_child(Some(&card));
            if let Some(cb) = &self.on_select {
                let cb = cb.clone();
                card_btn.connect_clicked(move |_| cb(i));
            }
            phone.append(&card_btn);
        }
        // Subscribe button.
        let sub_btn = gtk::Button::new();
        sub_btn.set_size_request(156, 18);
        sub_btn.set_halign(gtk::Align::Center);
        sub_btn.add_css_class("ss-ch-btn");
        uikit::widget::apply_css(&sub_btn, ".ss-ch-btn { background: #0A84FF; border: none; outline: none; border-radius: 9px; min-width: 156px; min-height: 18px; } .ss-ch-btn:focus { outline: none; }");
        let bl = gtk::Label::new(Some("Subscribe"));
        bl.set_halign(gtk::Align::Center);
        bl.add_css_class("ss-ch-bl");
        uikit::widget::apply_css(&bl, ".ss-ch-bl { color: white; font-family: 'SF Pro Display'; font-size: 7px; font-weight: 600; }");
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

impl Widget for CustomHeaderSubscriptionStoreView {
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
    fn custom_header_exists() { let _ = CustomHeaderSubscriptionStoreView::new().header("Bar"); }
    #[test]
    fn custom_header_options_and_callbacks() {
        let v = CustomHeaderSubscriptionStoreView::new().option("A", "S", "$1").on_select(|_| {}).on_subscribe(|| {});
        assert_eq!(v.options.len(), 2);
        assert!(v.on_select.is_some() && v.on_subscribe.is_some());
    }
}
