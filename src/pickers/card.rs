//! PickerCard — gallery card for the Picker demo.
//!
//! Encapsulates the GTK chrome (`#2c2c2e` card, badge, title, description)
//! so the `examples/pickers.rs` demo stays pure TontooUI without importing
//! `gtk` or `uikit` directly. UIKit remains hidden behind TontooUI.

use gtk::prelude::*;
use uikit::widget::{Widget, WidgetId, next_widget_id};
use uikit::style::Padding;

/// Dark gallery card showing a picker preview plus title/description.
///
/// Used only by `examples/pickers.rs` to match the SwiftUI screenshot layout.
pub struct PickerCard {
    id: WidgetId,
    badge: String,
    title: String,
    desc: String,
    preview: Box<dyn Widget>,
}

impl PickerCard {
    pub fn new(
        badge: impl Into<String>,
        title: impl Into<String>,
        desc: impl Into<String>,
        preview: impl Widget + 'static,
    ) -> Self {
        Self {
            id: next_widget_id(),
            badge: badge.into(),
            title: title.into(),
            desc: desc.into(),
            preview: Box::new(preview),
        }
    }
}

impl Widget for PickerCard {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn to_gtk(&self) -> gtk::Widget {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        outer.set_size_request(220, 220);
        outer.add_css_class("pk-card");
        uikit::widget::apply_css(
            &outer,
            ".pk-card { background: #2c2c2e; border-radius: 14px; border: 1px solid #3a3a3c; }",
        );

        let badge_lbl = gtk::Label::new(Some(self.badge.as_str()));
        badge_lbl.set_halign(gtk::Align::Start);
        badge_lbl.set_margin_start(8);
        badge_lbl.set_margin_top(8);
        badge_lbl.add_css_class("pk-badge-top");
        uikit::widget::apply_css(
            &badge_lbl,
            ".pk-badge-top { background: #0a84ff; color: white; font-family: 'SF Pro Display'; font-size: 9px; font-weight: 700; border-radius: 8px; padding: 2px 6px; }",
        );

        let preview_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        preview_box.set_hexpand(true);
        preview_box.set_vexpand(true);
        preview_box.set_halign(gtk::Align::Center);
        preview_box.set_valign(gtk::Align::Center);
        preview_box.set_margin_top(6);
        preview_box.set_margin_bottom(6);
        let g = self.preview.to_gtk();
        preview_box.append(&g);

        let title_lbl = gtk::Label::new(Some(self.title.as_str()));
        title_lbl.set_halign(gtk::Align::Start);
        title_lbl.set_margin_start(10);
        title_lbl.add_css_class("pk-card-title");
        uikit::widget::apply_css(
            &title_lbl,
            ".pk-card-title { color: #ececec; font-family: 'SF Pro Display'; font-size: 12px; font-weight: 700; }",
        );

        let desc_lbl = gtk::Label::new(Some(self.desc.as_str()));
        desc_lbl.set_halign(gtk::Align::Start);
        desc_lbl.set_wrap(true);
        desc_lbl.set_max_width_chars(28);
        desc_lbl.set_margin_start(10);
        desc_lbl.set_margin_end(10);
        desc_lbl.set_margin_bottom(10);
        desc_lbl.add_css_class("pk-card-desc");
        uikit::widget::apply_css(
            &desc_lbl,
            ".pk-card-desc { color: #8e8e93; font-family: 'SF Pro Display'; font-size: 10px; }",
        );

        let v = gtk::Box::new(gtk::Orientation::Vertical, 0);
        v.append(&badge_lbl);
        v.append(&preview_box);
        v.append(&title_lbl);
        v.append(&desc_lbl);
        outer.append(&v);

        outer.upcast()
    }

    fn position(&self) -> uikit::widget::Position {
        uikit::widget::Position::new()
    }

    fn position_mode(&self) -> uikit::widget::PositionMode {
        uikit::widget::PositionMode::Auto
    }

    fn padding(&self) -> Padding {
        Padding::ZERO
    }
}
