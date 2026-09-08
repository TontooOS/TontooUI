//! CustomGroupSubscriptionStoreView — Subscription store with custom grouping.

use std::sync::Arc;

use uikit::style::{Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

use crate::elements::store_product::StoreProduct;

/// CustomGroupSubscriptionStoreView — initializer — Creates a
/// SubscriptionStoreView with custom grouping.
///
/// Usable: set the `options`, mark `selected` and handle `on_select`
/// (cards are real buttons) plus `on_subscribe` (Subscribe is a button).
pub struct CustomGroupSubscriptionStoreView {
    id: WidgetId,
    group_id: String,
    options: Vec<StoreProduct>,
    selected: usize,
    on_select: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    on_subscribe: Option<Arc<dyn Fn() + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl CustomGroupSubscriptionStoreView {
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

impl Default for CustomGroupSubscriptionStoreView { fn default() -> Self { Self::new() } }

impl ViewContent for CustomGroupSubscriptionStoreView {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }
        let phone = gtk::Box::new(gtk::Orientation::Vertical, 4);
        phone.set_size_request(180, 110);
        phone.set_halign(gtk::Align::Center);
        phone.add_css_class("ss-cg-phone");
        uikit::widget::apply_css(&phone, ".ss-cg-phone { background: #0b0b0e; border-radius: 12px; min-width: 180px; min-height: 110px; padding: 8px 12px; }");
        // Monthly / Yearly segmented tabs
        let tabs = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        tabs.set_halign(gtk::Align::Center);
        tabs.add_css_class("ss-cg-tabs");
        uikit::widget::apply_css(&tabs, ".ss-cg-tabs { background: #1c1c1e; border-radius: 8px; padding: 2px; }");
        for (label, active) in [("Monthly", true), ("Yearly", false)] {
            let t = gtk::Label::new(Some(label));
            t.add_css_class(&format!("ss-cg-tab{}", if active { "a" } else { "b" }));
            if active {
                uikit::widget::apply_css(&t, ".ss-cg-taba { background: rgba(255,255,255,0.16); border-radius: 6px; color: white; font-family: 'SF Pro Display'; font-size: 6px; padding: 2px 10px; }");
            } else {
                uikit::widget::apply_css(&t, ".ss-cg-tabb { color: rgba(255,255,255,0.5); font-family: 'SF Pro Display'; font-size: 6px; padding: 2px 10px; }");
            }
            tabs.append(&t);
        }
        phone.append(&tabs);
        // Option cards (clickable) with blue border + check on the selected one.
        for (i, option) in self.options.iter().enumerate() {
            let card_btn = gtk::Button::new();
            card_btn.add_css_class("ss-cg-cardbtn");
            uikit::widget::apply_css(&card_btn, ".ss-cg-cardbtn { background: transparent; border: none; outline: none; padding: 0; } .ss-cg-cardbtn:focus { outline: none; }");
            let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
            card.set_size_request(156, 40);
            card.add_css_class(&format!("ss-cg-card{}", i));
            if i == self.selected {
                uikit::widget::apply_css(&card, &format!(".ss-cg-card{} {{ border: 1px solid #0A84FF; border-radius: 10px; min-width: 156px; min-height: 40px; padding: 4px 8px; }}", i));
            } else {
                uikit::widget::apply_css(&card, &format!(".ss-cg-card{} {{ border: 1px solid rgba(255,255,255,0.14); border-radius: 10px; min-width: 156px; min-height: 40px; padding: 4px 8px; }}", i));
            }
            let head = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            let l = gtk::Label::new(Some(option.title.as_str()));
            l.set_halign(gtk::Align::Start);
            l.set_hexpand(true);
            l.add_css_class("ss-cg-l");
            uikit::widget::apply_css(&l, ".ss-cg-l { color: white; font-family: 'SF Pro Display'; font-size: 7px; font-weight: 600; }");
            head.append(&l);
            let dot = gtk::Label::new(Some(if i == self.selected { "●" } else { "○" }));
            dot.add_css_class("ss-cg-dot");
            if i == self.selected {
                uikit::widget::apply_css(&dot, ".ss-cg-dot { color: #0A84FF; font-size: 9px; }");
            } else {
                uikit::widget::apply_css(&dot, ".ss-cg-dot { color: rgba(255,255,255,0.35); font-size: 9px; }");
            }
            head.append(&dot);
            card.append(&head);
            let p = gtk::Label::new(Some(option.price.as_str()));
            p.set_halign(gtk::Align::Start);
            p.add_css_class("ss-cg-p");
            uikit::widget::apply_css(&p, ".ss-cg-p { color: rgba(255,255,255,0.6); font-family: 'SF Pro Display'; font-size: 6px; }");
            card.append(&p);
            card_btn.set_child(Some(&card));
            if let Some(cb) = &self.on_select {
                let cb = cb.clone();
                card_btn.connect_clicked(move |_| cb(i));
            }
            phone.append(&card_btn);
        }
        // Subscribe button.
        let btn = gtk::Button::new();
        btn.set_size_request(156, 18);
        btn.set_halign(gtk::Align::Center);
        btn.add_css_class("ss-cg-btn");
        uikit::widget::apply_css(&btn, ".ss-cg-btn { background: #0A84FF; border: none; outline: none; border-radius: 9px; min-width: 156px; min-height: 18px; } .ss-cg-btn:focus { outline: none; }");
        let bl = gtk::Label::new(Some("Subscribe"));
        bl.set_halign(gtk::Align::Center);
        bl.set_valign(gtk::Align::Center);
        bl.set_vexpand(true);
        bl.add_css_class("ss-cg-bl");
        uikit::widget::apply_css(&bl, ".ss-cg-bl { color: white; font-family: 'SF Pro Display'; font-size: 7px; font-weight: 600; }");
        btn.set_child(Some(&bl));
        if let Some(cb) = &self.on_subscribe {
            let cb = cb.clone();
            btn.connect_clicked(move |_| cb());
        }
        phone.append(&btn);
        outer.append(&phone);
        outer.upcast()
    }
    fn size_that_fits(&self, _: Size) -> Size { Size::new(180.0, 110.0) }
}

impl Widget for CustomGroupSubscriptionStoreView {
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
    fn custom_group_exists() { let _ = CustomGroupSubscriptionStoreView::new().group_id("g1"); }
    #[test]
    fn custom_group_options_and_callbacks() {
        let v = CustomGroupSubscriptionStoreView::new()
            .clear_options()
            .option("A", "S", "$1")
            .selected(0)
            .on_select(|_| {})
            .on_subscribe(|| {});
        assert_eq!(v.options.len(), 1);
        assert!(v.on_select.is_some() && v.on_subscribe.is_some());
    }
}
