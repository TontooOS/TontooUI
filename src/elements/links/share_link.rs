//! ShareLink — a view that controls a sharing presentation.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use std::sync::Arc;

/// ShareLink — mirrors `SwiftUI.ShareLink`.
///
/// A view that controls a sharing presentation. Renders three share targets
/// ("Share...", "Explore SwiftUI", "Foo") plus a link preview directly on
/// window background, no extra card, SF Pro.
pub struct ShareLink {
    id: WidgetId,
    items: Vec<String>,
    preview_title: Option<String>,
    preview_subtitle: Option<String>,
    on_share: Option<Arc<dyn Fn(String) + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl ShareLink {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            id: next_widget_id(),
            items,
            preview_title: Some("Visual Library for SwiftUI Compon...".into()),
            preview_subtitle: Some("explore.swiftui.com".into()),
            on_share: None,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn item(item: impl Into<String>) -> Self {
        Self::new(vec![item.into()])
    }

    pub fn items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }

    pub fn preview(mut self, title: impl Into<String>, subtitle: impl Into<String>) -> Self {
        self.preview_title = Some(title.into());
        self.preview_subtitle = Some(subtitle.into());
        self
    }

    pub fn on_share(mut self, handler: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_share = Some(Arc::new(handler));
        self
    }

    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 260.0, 110.0)
    }
}

impl Default for ShareLink {
    fn default() -> Self {
        Self::new(vec!["Share ...".into(), "Explore SwiftUI".into(), "Foo".into()])
    }
}

impl ViewContent for ShareLink {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let fg = if is_dark { "#FFFFFF" } else { "#1d1d1d" };
        let fg_sub = if is_dark { "rgba(255,255,255,0.55)" } else { "rgba(60,60,67,0.6)" };
        let blue = "#0A84FF";

        // Directly on window — no extra background
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 8);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 110); }

        // Row of share items with icon "↗"
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        row.set_halign(gtk::Align::Center);
        for item in &self.items {
            let item_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
            item_box.set_halign(gtk::Align::Center);
            let icon = gtk::Label::new(Some("↗"));
            icon.add_css_class("share-icon");
            uikit::widget::apply_css(&icon, &format!(".share-icon {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; }}", blue));
            item_box.append(&icon);
            let lbl = gtk::Label::new(Some(item));
            lbl.add_css_class("share-lbl");
            uikit::widget::apply_css(&lbl, &format!(".share-lbl {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; }}", blue));
            item_box.append(&lbl);

            if let Some(handler) = &self.on_share {
                let h = handler.clone();
                let it = item.clone();
                let gesture = gtk::GestureClick::new();
                gesture.connect_pressed(move |_, _, _, _| h(it.clone()));
                item_box.add_controller(gesture);
            }

            row.append(&item_box);
        }
        outer.append(&row);

        // Link preview — app icon placeholder + title/subtitle, frosted look but directly on window
        if let (Some(title), Some(subtitle)) = (&self.preview_title, &self.preview_subtitle) {
            let preview = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            preview.set_halign(gtk::Align::Center);
            preview.set_size_request(240, 36);
            // frosted directly on window — use semi-transparent bg + border, no card
            let bg = if is_dark { "rgba(44,44,46,0.6)" } else { "rgba(255,255,255,0.72)" };
            let border = if is_dark { "1px solid rgba(255,255,255,0.10)" } else { "1px solid rgba(0,0,0,0.08)" };
            preview.add_css_class("share-preview");
            uikit::widget::apply_css(&preview, &format!(".share-preview {{ background: {}; border: {}; border-radius: 10px; padding: 6px 8px; }}", bg, border));

            let icon_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
            icon_box.set_size_request(28, 28);
            icon_box.add_css_class("share-preview-icon");
            uikit::widget::apply_css(&icon_box, ".share-preview-icon { background: #0A84FF; border-radius: 6px; min-width: 28px; min-height: 28px; }");
            let icon_lbl = gtk::Label::new(Some("◈"));
            icon_lbl.add_css_class("spi-lbl");
            uikit::widget::apply_css(&icon_lbl, ".spi-lbl { color: white; font-size: 14px; }");
            icon_box.append(&icon_lbl);
            preview.append(&icon_box);

            let text_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
            let t = gtk::Label::new(Some(title));
            t.set_halign(gtk::Align::Start);
            t.set_ellipsize(gtk::pango::EllipsizeMode::End);
            t.set_max_width_chars(24);
            t.add_css_class("share-preview-title");
            uikit::widget::apply_css(&t, &format!(".share-preview-title {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; font-weight: 600; }}", fg));
            text_box.append(&t);
            let st = gtk::Label::new(Some(subtitle));
            st.set_halign(gtk::Align::Start);
            st.add_css_class("share-preview-sub");
            uikit::widget::apply_css(&st, &format!(".share-preview-sub {{ color: {}; font-family: 'SF Pro Display'; font-size: 8px; }}", fg_sub));
            text_box.append(&st);
            preview.append(&text_box);

            outer.append(&preview);
        }

        outer.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size { Size::new(260.0, 110.0) }
}

impl Widget for ShareLink {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,260.0,110.0)) }
    fn is_interactive(&self) -> bool { true }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn share_link_items() {
        let s = ShareLink::new(vec!["A".into(), "B".into()]);
        assert_eq!(s.items.len(), 2);
    }
    #[test]
    fn share_link_default() {
        let s = ShareLink::default();
        assert!(s.preview_title.is_some());
    }
}
