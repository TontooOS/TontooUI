//! Custom Preview Item ShareLink — with a custom label/preview.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;

/// CustomPreviewShareLink — mirrors ShareLink initializer with custom preview.
///
/// Creates an instance, with a custom label, that presents the share interface.
/// Preview shows "Share Cats" with share icon on top, and an image placeholder
/// with "Derpy Cats" below, directly on window.
pub struct CustomPreviewShareLink {
    id: WidgetId,
    share_label: String,
    title: String,
    subtitle: Option<String>,
    position_mode: PositionMode,
    position: Position,
}

impl CustomPreviewShareLink {
    pub fn new(share_label: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: next_widget_id(),
            share_label: share_label.into(),
            title: title.into(),
            subtitle: None,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 220.0, 120.0)
    }
}

impl Default for CustomPreviewShareLink {
    fn default() -> Self { Self::new("Share Cats", "Derpy Cats") }
}

impl ViewContent for CustomPreviewShareLink {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let fg = if is_dark { "#FFFFFF" } else { "#1d1d1d" };
        let fg_sub = if is_dark { "rgba(255,255,255,0.65)" } else { "rgba(60,60,67,0.6)" };
        let blue = "#0A84FF";

        // Directly on window — no extra card
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 10);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 120); }

        // Top share label with icon
        let top = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        top.set_halign(gtk::Align::Center);
        let icon = gtk::Label::new(Some("↗"));
        icon.add_css_class("cps-icon");
        uikit::widget::apply_css(&icon, &format!(".cps-icon {{ color: {}; font-family: 'SF Pro Display'; font-size: 11px; }}", blue));
        top.append(&icon);
        let share_lbl = gtk::Label::new(Some(&self.share_label));
        share_lbl.add_css_class("cps-share-lbl");
        uikit::widget::apply_css(&share_lbl, &format!(".cps-share-lbl {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; }}", blue));
        top.append(&share_lbl);
        outer.append(&top);

        // Image preview + title — simulate cats image with gradient placeholder
        let card = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        card.set_halign(gtk::Align::Center);
        card.set_size_request(200, 56);
        let bg = if is_dark { "rgba(44,44,46,0.55)" } else { "rgba(255,255,255,0.70)" };
        let border = if is_dark { "1px solid rgba(255,255,255,0.10)" } else { "1px solid rgba(0,0,0,0.08)" };
        card.add_css_class("cps-card");
        uikit::widget::apply_css(&card, &format!(".cps-card {{ background: {}; border: {}; border-radius: 12px; padding: 8px; }}", bg, border));

        // Image placeholder — 3 cats? Use colored boxes as image
        let img = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        img.set_size_request(64, 40);
        img.set_halign(gtk::Align::Center);
        img.set_valign(gtk::Align::Center);
        // Simulate image with three small cat boxes
        for i in 0..3 {
            let cat = gtk::Box::new(gtk::Orientation::Vertical, 0);
            cat.set_size_request(18, 28);
            let cols = ["#c8a882", "#8a9ba8", "#b0a898"];
            uikit::widget::apply_css(&cat, &format!(".cat{{ background: {}; border-radius: 4px; min-width: 18px; min-height: 28px; }}", cols[i]));
            cat.add_css_class("cat");
            img.append(&cat);
        }
        card.append(&img);

        let text_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        text_box.set_valign(gtk::Align::Center);
        let title = gtk::Label::new(Some(&self.title));
        title.set_halign(gtk::Align::Start);
        title.add_css_class("cps-title");
        uikit::widget::apply_css(&title, &format!(".cps-title {{ color: {}; font-family: 'SF Pro Display'; font-size: 10px; font-weight: 600; }}", fg));
        text_box.append(&title);
        if let Some(sub) = &self.subtitle {
            let s = gtk::Label::new(Some(sub));
            s.set_halign(gtk::Align::Start);
            s.add_css_class("cps-sub");
            uikit::widget::apply_css(&s, &format!(".cps-sub {{ color: {}; font-family: 'SF Pro Display'; font-size: 8px; }}", fg_sub));
            text_box.append(&s);
        }
        card.append(&text_box);
        outer.append(&card);

        outer.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size { Size::new(220.0, 120.0) }
}

impl Widget for CustomPreviewShareLink {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,220.0,120.0)) }
    fn is_interactive(&self) -> bool { true }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn custom_preview_defaults() {
        let c = CustomPreviewShareLink::default();
        assert_eq!(c.share_label, "Share Cats");
        assert_eq!(c.title, "Derpy Cats");
    }
}
