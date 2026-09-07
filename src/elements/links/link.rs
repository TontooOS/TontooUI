//! Link — a control for navigating to a URL.

use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use std::sync::Arc;

/// Link — mirrors `SwiftUI.Link`.
///
/// A control for navigating to a URL. Renders as blue SF Pro text
/// "Explore SwiftUI" (or custom) directly on window, no extra card.
/// Clicking invokes the handler (in real app would open URL).
pub struct Link {
    id: WidgetId,
    title: String,
    destination: String,
    on_activate: Option<Arc<dyn Fn(String) + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
}

impl Link {
    pub fn new(title: impl Into<String>, destination: impl Into<String>) -> Self {
        Self {
            id: next_widget_id(),
            title: title.into(),
            destination: destination.into(),
            on_activate: None,
            position_mode: PositionMode::Auto,
            position: Position::new(),
        }
    }

    pub fn destination(mut self, url: impl Into<String>) -> Self {
        self.destination = url.into();
        self
    }

    pub fn on_activate(mut self, handler: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_activate = Some(Arc::new(handler));
        self
    }

    pub fn to_view(self) -> View {
        let w = (self.title.len() as f32 * 7.0 + 20.0).max(80.0);
        View::new(self).with_frame(0.0, 0.0, w, 24.0)
    }
}

impl Default for Link {
    fn default() -> Self { Self::new("Explore SwiftUI", "https://explore.swiftui.com") }
}

impl ViewContent for Link {
    fn render(&self, frame: Rect) -> gtk::Widget {
        // Directly on window — no extra background
        let outer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        outer.set_halign(gtk::Align::Center);
        outer.set_valign(gtk::Align::Center);
        if frame.width > 0.0 { outer.set_size_request(frame.width as i32, 24); }

        let btn = gtk::Button::new();
        btn.set_has_frame(false);
        // Link style: blue text, no background, underline on hover via CSS
        let title = self.title.clone();
        let lbl = gtk::Label::new(Some(&title));
        lbl.add_css_class("link-lbl");
        uikit::widget::apply_css(&lbl, ".link-lbl { color: #0A84FF; font-family: 'SF Pro Display'; font-size: 11px; }");
        btn.set_child(Some(&lbl));
        uikit::widget::apply_css(&btn, "button { background: transparent; border: none; padding: 2px 4px; } button:hover label { text-decoration: underline; }");

        let dest = self.destination.clone();
        let handler = self.on_activate.clone();
        btn.connect_clicked(move |_| {
            if let Some(h) = &handler {
                h(dest.clone());
            } else {
                // Fallback: try to open URL via gtk ShowUri? For preview just print
                #[cfg(target_os = "linux")]
                {
                    let _ = std::process::Command::new("xdg-open").arg(&dest).spawn();
                }
                println!("Link activated: {}", dest);
            }
        });

        outer.append(&btn);
        outer.upcast()
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        let w = (self.title.len() as f32 * 7.0 + 20.0).max(80.0);
        Size::new(w, 24.0)
    }
}

impl Widget for Link {
    fn id(&self) -> WidgetId { self.id }
    fn position_mode(&self) -> PositionMode { self.position_mode }
    fn position(&self) -> Position { self.position }
    fn to_gtk(&self) -> gtk::Widget { self.render(Rect::new(0.0,0.0,120.0,24.0)) }
    fn is_interactive(&self) -> bool { true }
    fn padding(&self) -> uikit::style::Padding { uikit::style::Padding::ZERO }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn link_default() {
        let l = Link::default();
        assert_eq!(l.title, "Explore SwiftUI");
        assert!(l.destination.contains("swiftui"));
    }
    #[test]
    fn link_custom() {
        let l = Link::new("Foo", "https://example.com");
        assert_eq!(l.destination, "https://example.com");
    }
}
