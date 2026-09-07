//! HelpLink — a button with a standard appearance that opens app-specific help.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use std::sync::Arc;

/// HelpLink — mirrors `SwiftUI.HelpLink`.
///
/// A button with a standard appearance that opens app-specific help.
/// In TontooOS it renders as a circular `?` button directly on the
/// window background (#1d1d1d dark / #ececec light), no extra card, SF Pro.
pub struct HelpLink {
    id: WidgetId,
    label: String,
    on_activate: Option<Arc<dyn Fn() + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl HelpLink {
    pub fn new() -> Self {
        Self {
            id: next_widget_id(),
            label: "Help".into(),
            on_activate: None,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn on_activate(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_activate = Some(Arc::new(handler));
        self
    }

    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 80.0, 36.0)
    }
}

impl Default for HelpLink {
    fn default() -> Self { Self::new() }
}

impl ViewContent for HelpLink {
    fn render(&self, frame: Rect) -> gtk::Widget {
        // Directly on window — no extra background card
        let outer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 36); }

        let btn = gtk::Button::new();
        btn.set_size_request(36, 36);
        // Circular help button with "?" — SF Pro
        let lbl = gtk::Label::new(Some("?"));
        lbl.add_css_class("help-lbl");
        let is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        let fg = if is_dark { "#FFFFFF" } else { "#1d1d1d" };
        let bg = if is_dark { "rgba(255,255,255,0.12)" } else { "rgba(0,0,0,0.06)" };
        let border = if is_dark { "1px solid rgba(255,255,255,0.14)" } else { "1px solid rgba(0,0,0,0.08)" };
        // Apply CSS directly via widget
        uikit::widget::apply_css(&lbl, &format!(".help-lbl {{ color: {}; font-family: 'SF Pro Display'; font-size: 14px; font-weight: 600; }}", fg));
        btn.set_child(Some(&lbl));
        uikit::widget::apply_css(&btn, &format!(
            "button {{ background: {}; border: {}; border-radius: 18px; min-width: 36px; min-height: 36px; padding: 0; }} button:hover {{ background: {}; }}",
            bg, border, if is_dark { "rgba(255,255,255,0.18)" } else { "rgba(0,0,0,0.10)" }
        ));

        if let Some(handler) = &self.on_activate {
            let h = handler.clone();
            btn.connect_clicked(move |_| h());
        }

        // Also show text label beside if not default "Help"?? Screenshot shows maybe just button
        // For palette directly on window, show only button centred
        outer.append(&btn);

        // If label is custom and not "Help", show label next to button
        if self.label != "Help" && !self.label.is_empty() {
            let txt = gtk::Label::new(Some(&self.label));
            txt.set_margin_start(8);
            txt.add_css_class("help-txt");
            uikit::widget::apply_css(&txt, &format!(".help-txt {{ color: {}; font-family: 'SF Pro Display'; font-size: 11px; }}", fg));
            outer.append(&txt);
        }

        outer.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size { Size::new(80.0, 36.0) }
}

impl Widget for HelpLink {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,80.0,36.0)) }
    fn is_interactive(&self) -> bool { true }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn help_link_default() {
        let h = HelpLink::new();
        assert_eq!(h.label, "Help");
    }
    #[test]
    fn help_link_custom_label() {
        let h = HelpLink::new().label("Get Help");
        assert_eq!(h.label, "Get Help");
    }
}
