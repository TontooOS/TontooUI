//! TextInput — SwiftUI-style text input element.
//!
//! A pre-made text input field with a declarative builder API.
//!
//! ```rust,no_run
//! use tontooui::prelude::*;
//!
//! let input = TextInput::new("Search...")
//!     .text("Hello")
//!     .on_change(|text| println!("Changed: {}", text))
//!     .on_submit(|text| println!("Submitted: {}", text))
//!     .frame(300.0, 44.0);
//!
//! let view = View::new(input);
//! ```

use std::sync::Arc;
use uikit::style::{Color, Rect, Size};
use uikit::view::{View, ViewContent};
use uikit::widget::{Position, PositionMode, Widget, WidgetId, next_widget_id};
use gtk::prelude::*;
use gtk::{self, Entry};

/// A SwiftUI-style text input field.
pub struct TextInput {
    id: WidgetId,
    placeholder: String,
    text: String,
    is_password: bool,
    is_disabled: bool,
    accent_color: Color,
    on_change: Option<Arc<dyn Fn(String) + Send + Sync>>,
    on_submit: Option<Arc<dyn Fn(String) + Send + Sync>>,
    position_mode: PositionMode,
    position: Position,
    width: f32,
    height: f32,
}

impl TextInput {
    /// Create a new text input with a placeholder.
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            id: next_widget_id(),
            placeholder: placeholder.into(),
            text: String::new(),
            is_password: false,
            is_disabled: false,
            accent_color: Color::new(0.047, 0.522, 0.937, 1.0), // TontooOS blue
            on_change: None,
            on_submit: None,
            position_mode: PositionMode::Auto,
            position: Position::new(),
            width: 300.0,
            height: 44.0,
        }
    }

    /// Set the initial text value.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Make this a password field (hidden text).
    pub fn password(mut self) -> Self {
        self.is_password = true;
        self
    }

    /// Disable the text input.
    pub fn disabled(mut self) -> Self {
        self.is_disabled = true;
        self
    }

    /// Set the accent (focus) color.
    pub fn accent_color(mut self, color: Color) -> Self {
        self.accent_color = color;
        self
    }

    /// Set the size of the text input.
    pub fn frame(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set the width of the text input.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Callback when the text changes.
    pub fn on_change(mut self, handler: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }

    /// Callback when the user presses Enter.
    pub fn on_submit(mut self, handler: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_submit = Some(Arc::new(handler));
        self
    }

    /// Position this input at absolute coordinates.
    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.position_mode = PositionMode::Absolute;
        self.position.x = Some(x);
        self.position.y = Some(y);
        self
    }

    /// Get the current text value.
    pub fn text_value(&self) -> &str {
        &self.text
    }

    /// Get the placeholder text.
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    /// Create a View wrapping this TextInput.
    pub fn to_view(self) -> View {
        let w = self.width;
        let h = self.height;
        View::new(self).with_frame(0.0, 0.0, w, h)
    }
}

impl ViewContent for TextInput {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let entry = Entry::new();
        entry.set_placeholder_text(Some(&self.placeholder));

        if !self.text.is_empty() {
            entry.set_text(&self.text);
        }

        entry.set_visibility(!self.is_password);
        entry.set_sensitive(!self.is_disabled);

        let w = if self.width > 0.0 { self.width } else { frame.width };
        let h = if self.height > 0.0 { self.height } else { frame.height };

        if w > 0.0 {
            entry.set_width_request(w as i32);
        }
        if h > 0.0 {
            entry.set_height_request(h as i32);
        }

        let accent_hex = format!(
            "#{:02x}{:02x}{:02x}",
            (self.accent_color.r * 255.0) as u8,
            (self.accent_color.g * 255.0) as u8,
            (self.accent_color.b * 255.0) as u8,
        );

        let bg_color = if self.is_disabled { "#1a1a1c" } else { "#2a2a2c" };
        let border_color = if self.is_disabled { "#2a2a2c" } else { "#3a3a3d" };

        let css = format!(
            "entry {{
                background-color: {bg_color};
                color: #ececec;
                border-radius: 8px;
                border: 1px solid {border_color};
                padding: 8px 12px;
                font-family: 'SF Pro Display';
                font-size: 13px;
                caret-color: {accent};
            }}
            entry:hover {{
                border-color: #4a4a4e;
            }}
            entry:focus {{
                border-color: {accent};
            }}",
            bg_color = bg_color,
            border_color = border_color,
            accent = accent_hex,
        );
        uikit::widget::apply_css(&entry, &css);

        if let Some(handler) = &self.on_change {
            let handler = handler.clone();
            entry.connect_changed(move |e| {
                let value = e.text().to_string();
                handler(value);
            });
        }

        if let Some(handler) = &self.on_submit {
            let handler = handler.clone();
            entry.connect_activate(move |e| {
                let value = e.text().to_string();
                handler(value);
            });
        }

        entry.upcast()
    }

    fn can_become_first_responder(&self) -> bool {
        !self.is_disabled
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        Size::new(self.width, self.height)
    }
}

impl Widget for TextInput {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn position_mode(&self) -> PositionMode {
        self.position_mode
    }

    fn position(&self) -> Position {
        self.position
    }

    fn to_gtk(&self) -> gtk::Widget {
        let entry = Entry::new();
        entry.set_placeholder_text(Some(&self.placeholder));

        if !self.text.is_empty() {
            entry.set_text(&self.text);
        }

        entry.set_visibility(!self.is_password);
        entry.set_sensitive(!self.is_disabled);

        if let Some(w) = self.position.width {
            entry.set_width_request(w as i32);
        } else if self.width > 0.0 {
            entry.set_width_request(self.width as i32);
        }

        if let Some(h) = self.position.height {
            entry.set_height_request(h as i32);
        } else if self.height > 0.0 {
            entry.set_height_request(self.height as i32);
        }

        let accent_hex = format!(
            "#{:02x}{:02x}{:02x}",
            (self.accent_color.r * 255.0) as u8,
            (self.accent_color.g * 255.0) as u8,
            (self.accent_color.b * 255.0) as u8,
        );

        let bg_color = if self.is_disabled { "#1a1a1c" } else { "#2a2a2c" };
        let border_color = if self.is_disabled { "#2a2a2c" } else { "#3a3a3d" };

        let css = format!(
            "entry {{
                background-color: {bg_color};
                color: #ececec;
                border-radius: 8px;
                border: 1px solid {border_color};
                padding: 8px 12px;
                font-family: 'SF Pro Display';
                font-size: 13px;
                caret-color: {accent};
            }}
            entry:hover {{
                border-color: #4a4a4e;
            }}
            entry:focus {{
                border-color: {accent};
            }}",
            bg_color = bg_color,
            border_color = border_color,
            accent = accent_hex,
        );
        uikit::widget::apply_css(&entry, &css);

        if let Some(handler) = &self.on_change {
            let handler = handler.clone();
            entry.connect_changed(move |e| {
                let value = e.text().to_string();
                handler(value);
            });
        }

        if let Some(handler) = &self.on_submit {
            let handler = handler.clone();
            entry.connect_activate(move |e| {
                let value = e.text().to_string();
                handler(value);
            });
        }

        if self.position_mode == PositionMode::Absolute {
            let mut css = String::from("entry {");
            if let Some(x) = self.position.x {
                css.push_str(&format!("margin-left: {}px;", x));
            }
            if let Some(y) = self.position.y {
                css.push_str(&format!("margin-top: {}px;", y));
            }
            css.push('}');
            uikit::widget::apply_css(&entry, &css);
        }

        entry.upcast()
    }

    fn is_interactive(&self) -> bool {
        !self.is_disabled
    }

    fn padding(&self) -> uikit::style::Padding {
        uikit::style::Padding::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_input_builder() {
        let input = TextInput::new("Email...")
            .text("test@example.com")
            .password()
            .frame(400.0, 48.0);

        assert_eq!(input.placeholder(), "Email...");
        assert_eq!(input.text_value(), "test@example.com");
        assert!(input.is_password);
        assert_eq!(input.width, 400.0);
        assert_eq!(input.height, 48.0);
    }

    #[test]
    fn text_input_disabled() {
        let input = TextInput::new("Read only").disabled();
        assert!(input.is_disabled);
        assert!(!input.is_interactive());
    }

    #[test]
    fn text_input_accent_color() {
        let input = TextInput::new("Search")
            .accent_color(Color::from_hex("#FF6B2B").unwrap());
        assert_eq!(input.accent_color, Color::from_rgb(255, 107, 43));
    }
}
