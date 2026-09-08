//! GroupSubscriptionStoreView — Loads all subscriptions in a group.

use std::sync::Arc;

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

use crate::elements::store_product::StoreProduct;

/// GroupSubscriptionStoreView — initializer — Creates a view that loads all
/// subscriptions in a subscription group.
///
/// Usable: set the `options`, mark `selected` and handle `on_select`
/// (cards are real buttons) plus `on_subscribe` (Subscribe is a button).
pub struct GroupSubscriptionStoreView {
    id: WidgetId,
    group_id: String,
    options: Vec<StoreProduct>,
    selected: usize,
    on_select: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    on_subscribe: Option<Arc<dyn Fn() + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl GroupSubscriptionStoreView {
    pub fn new() -> Self {
        Self { id: next_widget_id(), group_id: "kitty.pass.group".to_string(), options: vec![StoreProduct::monthly_default(), StoreProduct::yearly_default()], selected: 0, on_select: None, on_subscribe: None, position_mode: PositionMode::Auto, position: Position::new() }
    }
    pub fn group_id(mut self, v: impl Into<String>) -> Self { self.group_id = v.into(); self }
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

impl Default for GroupSubscriptionStoreView { fn default() -> Self { Self::new() } }

fn option_card(selected: bool, title: &str, price: &str, sub: &str, prefix: &str) -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
    card.set_size_request(156, 38);
    card.add_css_class(&format!("{}-card", prefix));
    if selected {
        uikit::widget::apply_css(&card, &format!(".{}-card {{ border: 1px solid #0A84FF; border-radius: 10px; min-width: 156px; min-height: 38px; padding: 3px 8px; }}", prefix));
    } else {
        uikit::widget::apply_css(&card, &format!(".{}-card {{ border: 1px solid rgba(255,255,255,0.14); border-radius: 10px; min-width: 156px; min-height: 38px; padding: 3px 8px; }}", prefix));
    }
    let head = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let l = gtk::Label::new(Some(title));
    l.set_halign(gtk::Align::Start);
    l.set_hexpand(true);
    l.add_css_class(&format!("{}-l", prefix));
    uikit::widget::apply_css(&l, &format!(".{}-l {{ color: white; font-family: 'SF Pro Display'; font-size: 7px; font-weight: 600; }}", prefix));
    head.append(&l);
    let dot = gtk::Label::new(Some(if selected { "●" } else { "○" }));
    dot.add_css_class(&format!("{}-dot", prefix));
    if selected {
        uikit::widget::apply_css(&dot, &format!(".{}-dot {{ color: #0A84FF; font-size: 9px; }}", prefix));
    } else {
        uikit::widget::apply_css(&dot, &format!(".{}-dot {{ color: rgba(255,255,255,0.35); font-size: 9px; }}", prefix));
    }
    head.append(&dot);
    card.append(&head);
    let p = gtk::Label::new(Some(price));
    p.set_halign(gtk::Align::Start);
    p.add_css_class(&format!("{}-p", prefix));
    uikit::widget::apply_css(&p, &format!(".{}-p {{ color: rgba(255,255,255,0.6); font-family: 'SF Pro Display'; font-size: 6px; }}", prefix));
    card.append(&p);
    let s = gtk::Label::new(Some(sub));
    s.set_halign(gtk::Align::Start);
    s.add_css_class(&format!("{}-s", prefix));
    uikit::widget::apply_css(&s, &format!(".{}-s {{ color: rgba(255,255,255,0.4); font-family: 'SF Pro Display'; font-size: 5px; }}", prefix));
    card.append(&s);
    card
}

impl ViewContent for GroupSubscriptionStoreView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 4);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("ss-gr-phone");
        uikit::widget::apply_css(&phone, ".ss-gr-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; padding: 8px 12px; }");
        for (i, option) in self.options.iter().enumerate() {
            let card_btn = gtk::Button::new();
            card_btn.add_css_class("ss-gr-cardbtn");
            uikit::widget::apply_css(&card_btn, ".ss-gr-cardbtn { background: transparent; border: none; outline: none; padding: 0; } .ss-gr-cardbtn:focus { outline: none; }");
            let prefix = format!("ss-gr-{}", i);
            let card = option_card(i == self.selected, &option.title, &option.price, &option.subtitle, &prefix);
            card_btn.set_child(Some(&card));
            if let Some(cb) = &self.on_select {
                let cb = cb.clone();
                card_btn.connect_clicked(move |_| cb(i));
            }
            phone.append(&card_btn);
        }
        let sub_btn = gtk::Button::new();
        sub_btn.set_size_request(156, 18);
        sub_btn.set_halign(gtk::Align::Center);
        sub_btn.add_css_class("ss-gr-btn");
        uikit::widget::apply_css(&sub_btn, ".ss-gr-btn { background: #0A84FF; border: none; outline: none; border-radius: 9px; min-width: 156px; min-height: 18px; } .ss-gr-btn:focus { outline: none; }");
        let bl = gtk::Label::new(Some("Subscribe"));
        bl.set_halign(gtk::Align::Center);
        bl.add_css_class("ss-gr-bl");
        uikit::widget::apply_css(&bl, ".ss-gr-bl { color: white; font-family: 'SF Pro Display'; font-size: 7px; font-weight: 600; }");
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

impl Widget for GroupSubscriptionStoreView {
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
    fn group_exists() { let _ = GroupSubscriptionStoreView::new().group_id("g1"); }
    #[test]
    fn group_options_and_callbacks() {
        let v = GroupSubscriptionStoreView::new().clear_options().option("A", "S", "$1").selected(0).on_select(|_| {}).on_subscribe(|| {});
        assert_eq!(v.options.len(), 1);
        assert!(v.on_select.is_some() && v.on_subscribe.is_some());
    }
}
