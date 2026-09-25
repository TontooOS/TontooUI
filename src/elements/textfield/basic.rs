use std::any::Any;

use vello::Scene;
use vello::peniko::Color;

use super::super::layout::View;
use super::{
    FieldCore, FieldMetrics, draw_field, measure_field,
};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::renderer::window::Key;

/// Slim single-line field text size in logical px.
pub const TEXTFIELD_FONT_SIZE: f32 = 13.0;
/// Slim horizontal/vertical padding in logical px.
pub const TEXTFIELD_PAD_X: f32 = 8.0;
/// Slim horizontal/vertical padding in logical px.
pub const TEXTFIELD_PAD_Y: f32 = 6.0;
/// Slim corner radius in logical px.
pub const TEXTFIELD_RADIUS: f32 = 8.0;

/// Slim single-line text field (like the top reference row):
/// placeholder, caret, accent focus ring. Click inside to select
/// (caret to end); ESC or a click outside deselects. Typing,
/// Backspace and caret keys arrive through `type_text` and `key`.
pub struct BasicTextField {
    core: FieldCore,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl BasicTextField {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            core: FieldCore::new(placeholder.into()),
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        }
    }

    /// Accent focus ring and caret color, like a normal button
    /// (`set_theme(accent, dark)`).
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        self.core.accent = accent;
        self.core.dark = dark;
        self.core.dirty = true;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.core.focused = focused;
        self.core.dirty = true;
    }

    /// Fires with the full text on every user edit.
    pub fn on_change(mut self, callback: impl FnMut(&str) + 'static) -> Self {
        self.core.on_change = Some(Box::new(callback));
        self
    }

    /// Programmatic text (caret to end, no `on_change`).
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.core.set_text(text.into());
    }

    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        let placeholder = placeholder.into();
        if placeholder != self.core.placeholder {
            self.core.placeholder = placeholder;
            self.core.dirty = true;
        }
    }

    pub fn text_value(&self) -> &str {
        &self.core.text
    }

    pub fn placeholder_value(&self) -> &str {
        &self.core.placeholder
    }

    pub fn is_selected(&self) -> bool {
        self.core.selected
    }

    /// Type printable text at the caret (the app forwards its
    /// `text` here while selected).
    pub fn type_text(&mut self, content: &str) {
        if self.core.selected {
            self.core.insert(content);
        }
    }

    /// Key handling while selected: Backspace deletes, Left/Right
    /// move the caret, ESC deselects. Returns true when consumed.
    /// The app forwards its `key` here.
    pub fn key(&mut self, key: Key) -> bool {
        if !self.core.selected {
            return false;
        }
        match key {
            Key::Backspace => self.core.backspace(),
            Key::Left => self.core.move_left(),
            Key::Right => self.core.move_right(),
            Key::Escape => self.core.deselect(),
            _ => return false,
        }
        true
    }

    /// Press handling: click inside selects (caret to end), anywhere
    /// else deselects. The app forwards every press here.
    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.hit(x as f32, y as f32) {
            self.core.select();
        } else {
            self.core.deselect();
        }
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }

    fn hit(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.placed_w && y >= self.y && y <= self.y + self.placed_h
    }
}

const BASIC_METRICS: FieldMetrics = FieldMetrics {
    font_size: TEXTFIELD_FONT_SIZE,
    pad_x: TEXTFIELD_PAD_X,
    pad_y: TEXTFIELD_PAD_Y,
    radius: TEXTFIELD_RADIUS,
    min_width: 120.0,
};

impl View for BasicTextField {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        measure_field(fonts, &mut self.core, &BASIC_METRICS)
    }

    fn place(&mut self, _fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.placed_w = w;
        self.placed_h = h;
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        _images: &mut ImageLoader<'_>,
    ) {
        draw_field(
            scene,
            fonts,
            &mut self.core,
            self.x,
            self.y,
            self.placed_w,
            self.placed_h,
            &BASIC_METRICS,
        );
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::text::FontSystem;

    fn field() -> BasicTextField {
        BasicTextField::new("Enter text here")
    }

    #[test]
    fn typing_inserts_and_backspace_deletes() {
        let mut field = field();
        field.mouse_down(5.0, 5.0);
        // No placed rect yet: outside clicks deselect.
        assert!(!field.is_selected());
        let mut fonts = FontSystem::new();
        let (_w, h) = field.measure(&mut fonts);
        field.place(&mut fonts, 0.0, 0.0, 300.0, h.max(28.0));
        field.mouse_down(10.0, 10.0);
        assert!(field.is_selected());
        field.type_text("hi");
        assert_eq!(field.text_value(), "hi");
        field.key(Key::Backspace);
        assert_eq!(field.text_value(), "h");
        field.key(Key::Left);
        field.type_text("X");
        assert_eq!(field.text_value(), "Xh");
    }

    #[test]
    fn escape_and_outside_click_deselect() {
        let mut field = field();
        let mut fonts = FontSystem::new();
        let (_, h) = field.measure(&mut fonts);
        field.place(&mut fonts, 0.0, 0.0, 300.0, h);
        field.mouse_down(10.0, 10.0);
        assert!(field.is_selected());
        assert!(field.key(Key::Escape));
        assert!(!field.is_selected());
        assert!(!field.key(Key::Backspace));
        field.mouse_down(10.0, 10.0);
        field.mouse_down(500.0, 500.0);
        assert!(!field.is_selected());
    }

    #[test]
    fn control_chars_filtered_and_utf8_safe() {
        let mut field = field();
        let mut fonts = FontSystem::new();
        let (_, h) = field.measure(&mut fonts);
        field.place(&mut fonts, 0.0, 0.0, 300.0, h);
        field.mouse_down(10.0, 10.0);
        field.type_text("a\nb\tc");
        assert_eq!(field.text_value(), "abc");
        field.type_text("é");
        field.key(Key::Backspace);
        assert_eq!(field.text_value(), "abc");
        field.key(Key::Left);
        field.key(Key::Left);
        field.type_text("X");
        assert_eq!(field.text_value(), "aXbc");
    }

    #[test]
    fn accent_ring_follows_theme_accent() {
        let mut field = field();
        let pink = Color::from_rgb8(0xff, 0x2d, 0x55);
        field.set_theme(pink, true);
        assert_eq!(field.core.accent, pink);
    }
}
