//! TextFieldLink — a control that requests text input from the user when pressed.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use std::sync::Arc;

/// TextFieldLink — mirrors `SwiftUI.TextFieldLink`.
///
/// A control that requests text input from the user when pressed.
/// Renders as a pill button "Set Text" directly on window background,
/// SF Pro, no extra card.
pub struct TextFieldLink {
    id: WidgetId,
    title: String,
    prompt: String,
    on_submit: Option<Arc<dyn Fn(String) + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl TextFieldLink {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: next_widget_id(),
            title: title.into(),
            prompt: "Enter text".into(),
            on_submit: None,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = prompt.into();
        self
    }

    pub fn on_submit(mut self, handler: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_submit = Some(Arc::new(handler));
        self
    }

    pub fn to_view(self) -> View {
        View::new(self).with_frame(0.0, 0.0, 180.0, 44.0)
    }
}

impl Default for TextFieldLink {
    fn default() -> Self { Self::new("Set Text") }
}

impl ViewContent for TextFieldLink {
    fn render(&self, frame: Rect) -> gtk::Widget {
        // Directly on window — no extra background
        let outer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 44); }

        let btn = gtk::Button::new();
        btn.set_size_request(160, 36);
        let lbl = gtk::Label::new(Some(&self.title));
        lbl.add_css_class("tfl-lbl");
        uikit::widget::apply_css(&lbl, ".tfl-lbl { color: white; font-family: 'SF Pro Display'; font-size: 13px; font-weight: 500; }");
        btn.set_child(Some(&lbl));

        // Pill, dark grey as in screenshot (#3a3a3c / #2c2c2e) — keep dark even in light mode for contrast, but adaptive border
        let _is_dark = crate::elements::resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        // Use dark pill directly on window for both modes (as screenshot is dark)
        uikit::widget::apply_css(&btn, "button { background: #3a3a3c; border: 1px solid rgba(255,255,255,0.12); border-radius: 18px; min-width: 160px; min-height: 36px; padding: 0 16px; } button:hover { background: #4a4a4c; } button:active { background: #2c2c2e; }");

        if let Some(handler) = &self.on_submit {
            let prompt = self.prompt.clone();
            let h = handler.clone();
            btn.connect_clicked(move |_| {
                // In real usage this would present a text field; for preview we send prompt
                h(prompt.clone());
            });
        }

        outer.append(&btn);
        outer.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size { Size::new(180.0, 44.0) }
}

impl Widget for TextFieldLink {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,180.0,44.0)) }
    fn is_interactive(&self) -> bool { true }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn textfield_link_default_title() {
        let t = TextFieldLink::new("Set Text");
        assert_eq!(t.title, "Set Text");
    }
    #[test]
    fn textfield_link_custom() {
        let t = TextFieldLink::new("Edit").prompt("Name");
        assert_eq!(t.prompt, "Name");
    }
}
