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
use crate::elements::resolve_scheme;

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
    on_blur: Option<Arc<dyn Fn() + Send + Sync>>,
    deselect_on_click_away: bool,
    position_mode: PositionMode,
    position: Position,
    width: f32,
    height: f32,
    transparent: bool,
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
            on_blur: None,
            deselect_on_click_away: true,
            position_mode: PositionMode::Auto,
            position: Position::new(),
            width: 300.0,
            height: 32.0,
            transparent: false,
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

    /// Paint no background or border so the field can sit on glass
    /// (e.g. inside a `GlassContainer`). Text, placeholder and caret
    /// colors are unchanged.
    pub fn transparent(mut self) -> Self {
        self.transparent = true;
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

    /// Callback when the field loses keyboard focus (click-away, Tab, ...).
    /// Fires after the text selection was cleared. Use it to commit
    /// (e.g. Finder inline rename) or collapse (e.g. search field).
    pub fn on_blur(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_blur = Some(Arc::new(handler));
        self
    }

    /// Whether a click anywhere outside the field drops focus and clears
    /// the selection (default `true`). Set to `false` to keep focus.
    pub fn deselect_on_click_away(mut self, enabled: bool) -> Self {
        self.deselect_on_click_away = enabled;
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

    /// Build the underlying GTK entry with all styling and handlers applied.
    fn build_entry(&self, width: f32) -> gtk::Entry {
        let entry = Entry::new();
        entry.set_placeholder_text(Some(&self.placeholder));

        if !self.text.is_empty() {
            entry.set_text(&self.text);
        }

        entry.set_visibility(!self.is_password);
        entry.set_sensitive(!self.is_disabled);

        entry.set_hexpand(false);
        entry.set_vexpand(false);
        entry.set_halign(gtk::Align::Start);

        if width > 0.0 {
            entry.set_width_request(width as i32);
        }

        let accent_hex = format!(
            "#{:02x}{:02x}{:02x}",
            (self.accent_color.r * 255.0) as u8,
            (self.accent_color.g * 255.0) as u8,
            (self.accent_color.b * 255.0) as u8,
        );

        let is_dark = resolve_scheme(None) == uikit::app::ColorScheme::Dark;
        // BG #1d1d1d dark / #ececec light per AGENTS.md — input field sits slightly above bg.
        // Transparent mode paints nothing so a GlassContainer behind shows through.
        let bg_color = if self.transparent {
            "transparent"
        } else if is_dark {
            if self.is_disabled { "#1a1a1c" } else { "#2a2a2c" }
        } else {
            if self.is_disabled { "#e8e8ea" } else { "#ffffff" }
        };
        let border_color = if self.transparent {
            "transparent"
        } else if is_dark {
            if self.is_disabled { "#2a2a2c" } else { "#3a3a3d" }
        } else {
            if self.is_disabled { "#e5e5e5" } else { "#d1d1d6" }
        };
        let text_color = if is_dark { "#ececec" } else { "#1d1d1d" };
        let placeholder_color = if is_dark { "#8e8e93" } else { "#aeaeb2" };
        let hover_color = if self.transparent {
            "transparent"
        } else if is_dark {
            "#4a4a4e"
        } else {
            "#aeaeb2"
        };

        let css = format!(
            "entry {{
                background-color: {bg_color};
                color: {text_color};
                border-radius: 8px;
                border: 1px solid {border_color};
                padding: 4px 10px;
                min-height: 0px;
                font-family: 'SF Pro Display';
                font-size: 13px;
                caret-color: {accent};
            }}
            entry placeholder {{
                color: {placeholder_color};
                opacity: 1;
            }}
            entry:hover {{
                border-color: {hover_color};
            }}
            entry:focus {{
                border-color: {accent};
            }}
            entry:disabled {{
                background-color: {bg_color};
                color: {placeholder_color};
                border-color: {border_color};
            }}",
            bg_color = bg_color,
            border_color = border_color,
            text_color = text_color,
            placeholder_color = placeholder_color,
            hover_color = hover_color,
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

        // Focus loss always clears the selection first, then notifies.
        // `on_blur` fires exactly once here; the click-away gesture below
        // only drops focus, which routes through this handler.
        {
            let weak = entry.downgrade();
            let blur = self.on_blur.clone();
            let focus = gtk::EventControllerFocus::new();
            focus.connect_leave(move |_| {
                if let Some(field) = weak.upgrade() {
                    field.select_region(0, 0);
                }
                if let Some(cb) = blur.as_ref() {
                    cb();
                }
            });
            entry.add_controller(focus);
        }

        // Clear the text selection and drop keyboard focus when the user
        // clicks anywhere else — including empty space. GTK only moves focus
        // to widgets that can take it, so a click on an ordinary box, a label
        // or window padding would otherwise leave the entry focused with its
        // focus ring and selected text intact. A capture-phase gesture on the
        // toplevel window sees every press before the target widget; a
        // hit-test keeps presses inside the entry untouched.
        if self.deselect_on_click_away {
            entry.connect_realize(|w| {
                install_click_away(w);
            });
        }

        entry
    }

    /// Create a View wrapping this TextInput.
    pub fn to_view(self) -> View {
        let w = self.width;
        let h = self.height;
        View::new(self).with_frame(0.0, 0.0, w, h)
    }
}

/// True when the press at window-relative `(x, y)` landed inside `entry`.
/// Walks from the picked widget up to the entry; anything else (including
/// empty space with no picked widget) counts as outside.
fn press_inside_entry(gesture: &gtk::GestureClick, entry: &gtk::Entry, x: f64, y: f64) -> bool {
    let target = gesture
        .widget()
        .and_then(|root| root.pick(x, y, gtk::PickFlags::DEFAULT));
    let mut node = target;
    while let Some(widget) = node {
        if widget == entry.clone().upcast::<gtk::Widget>() {
            return true;
        }
        node = widget.parent();
    }
    false
}

/// Install the click-away gesture for one realized entry. Runs in the
/// capture phase on the toplevel window so empty areas (boxes, labels,
/// padding) deselect just like focusable widgets do. Presses inside the
/// entry are ignored; everything else drops window focus (the focus-leave
/// handler clears the selection and fires `on_blur`).
fn install_click_away(entry: &gtk::Entry) {
    let try_install = |field: &gtk::Entry| -> bool {
        let Some(root) = field.root() else {
            return false;
        };
        let Ok(window) = root.downcast::<gtk::Window>() else {
            return false;
        };
        let weak_entry = field.downgrade();
        let weak_win = window.downgrade();
        let press = gtk::GestureClick::new();
        // Button 0 = any mouse button (left/right/middle all deselect).
        press.set_button(0);
        press.set_propagation_phase(gtk::PropagationPhase::Capture);
        let weak_win_c = weak_win.clone();
        press.connect_pressed(move |gesture, _, x, y| {
            let Some(field) = weak_entry.upgrade() else {
                return;
            };
            let Some(win) = weak_win.upgrade() else {
                return;
            };
            if press_inside_entry(gesture, &field, x, y) {
                return;
            }
            if field.has_focus() {
                field.select_region(0, 0);
            }
            gtk::prelude::GtkWindowExt::set_focus(&win, None::<&gtk::Widget>);
        });
        let press_handle = press.clone();
        window.add_controller(press);
        // The controller lives on the window while the entry may be rebuilt
        // (e.g. Finder grid refresh); the weak entry keeps it a no-op after
        // teardown, and unrealize removes it so repeated realizes never pile
        // up.
        let win_c = weak_win_c;
        field.connect_unrealize(move |_| {
            if let Some(win) = win_c.upgrade() {
                win.remove_controller(&press_handle);
            }
        });
        true
    };
    if try_install(entry) {
        return;
    }
    // Not yet attached to a window (built before append): retry once idle.
    let weak = entry.downgrade();
    glib::idle_add_local_once(move || {
        if let Some(field) = weak.upgrade() {
            try_install(&field);
        }
    });
}

impl ViewContent for TextInput {
    fn render(&self, frame: Rect) -> gtk::Widget {
        let w = if self.width > 0.0 { self.width } else { frame.width };
        let entry = self.build_entry(w);
        entry.upcast()
    }

    fn can_become_first_responder(&self) -> bool {
        !self.is_disabled
    }

    fn size_that_fits(&self, _available: Size) -> Size {
        Size::new(self.width, 0.0)
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
        let w = if let Some(w) = self.position.width {
            w as f32
        } else {
            self.width
        };
        let entry = self.build_entry(w);

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
    fn text_input_transparent() {
        let input = TextInput::new("Search").transparent();
        assert!(input.transparent);
        let solid = TextInput::new("Search");
        assert!(!solid.transparent);
    }

    #[test]
    fn text_input_accent_color() {
        let input = TextInput::new("Search")
            .accent_color(Color::from_hex("#FF6B2B").unwrap());
        assert_eq!(input.accent_color, Color::from_rgb(255, 107, 43));
    }

    #[test]
    fn text_input_click_away_defaults_on() {
        let input = TextInput::new("Search");
        assert!(input.deselect_on_click_away);
        assert!(input.on_blur.is_none());
        let kept = TextInput::new("Search").deselect_on_click_away(false);
        assert!(!kept.deselect_on_click_away);
        let with_blur = TextInput::new("Search").on_blur(|| {});
        assert!(with_blur.on_blur.is_some());
    }
}
