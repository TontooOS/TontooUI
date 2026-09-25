use std::any::Any;

use vello::Scene;
use vello::peniko::Color;

use super::super::layout::View;
use super::{
    FieldCore, FieldMetrics, draw_field, field_colors, measure_field, resolve_press_single,
};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::renderer::window::Key;

/// Large single-line field text size in logical px.
pub const LARGE_FIELD_FONT_SIZE: f32 = 15.0;
/// Large horizontal/vertical padding in logical px.
pub const LARGE_FIELD_PAD_X: f32 = 12.0;
/// Large horizontal/vertical padding in logical px.
pub const LARGE_FIELD_PAD_Y: f32 = 10.0;
/// Large corner radius in logical px.
pub const LARGE_FIELD_RADIUS: f32 = 10.0;

/// Large single-line text field (like the bottom reference row):
/// roomier padding and type than the basic variant, same behavior —
/// placeholder, caret, accent focus ring, ESC and outside clicks
/// deselect, typing through `type_text` and `key`.
pub struct LargeTextField {
    core: FieldCore,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl LargeTextField {
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
    /// move the caret (Shift extends), Ctrl+A/C/X/V/Z/Y select,
    /// copy, cut, paste, undo, redo, ESC deselects. Returns true
    /// when consumed. The app forwards its `key` here.
    pub fn key(&mut self, key: Key) -> bool {
        self.core.handle_key(key)
    }

    /// Highlight everything (Ctrl+A equivalent).
    pub fn select_all(&mut self) {
        if self.core.selected {
            self.core.select_all();
        }
    }

    /// Currently highlighted text (empty when nothing is selected).
    pub fn selected_text(&self) -> String {
        self.core.selected_text()
    }

    /// Visible selection as a sorted byte range, if any.
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.core.selection_range()
    }

    /// Undo the last edit. Returns false when the stack is empty.
    pub fn undo(&mut self) -> bool {
        self.core.undo()
    }

    /// Redo the last undone edit. Returns false when empty.
    pub fn redo(&mut self) -> bool {
        self.core.redo()
    }

    /// Copy the highlight to the clipboard. Returns false when
    /// nothing is selected.
    pub fn copy_selection(&mut self) -> bool {
        self.core.copy()
    }

    /// Cut the highlight to the clipboard. Returns false when
    /// nothing is selected.
    pub fn cut_selection(&mut self) -> bool {
        self.core.cut()
    }

    /// Paste clipboard text at the caret.
    pub fn paste_clipboard(&mut self) {
        if self.core.selected {
            self.core.paste();
        }
    }

    /// True while the pointer hovers the field: the app returns the
    /// I-beam cursor from `App::cursor` then.
    pub fn wants_text_cursor(&self) -> bool {
        self.core.hovered
    }

    /// Press handling: click inside focuses (the caret lands at the
    /// click in `draw`, double-click highlights the word, dragging
    /// extends), anywhere else deselects. The app forwards every
    /// press here.
    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.hit(x as f32, y as f32) {
            self.core.press(x as f32, y as f32);
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

const LARGE_METRICS: FieldMetrics = FieldMetrics {
    font_size: LARGE_FIELD_FONT_SIZE,
    pad_x: LARGE_FIELD_PAD_X,
    pad_y: LARGE_FIELD_PAD_Y,
    radius: LARGE_FIELD_RADIUS,
    min_width: 160.0,
};

impl View for LargeTextField {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        measure_field(fonts, &mut self.core, &LARGE_METRICS)
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
        let echo = self.core.echo();
        let (_, _, text) = field_colors(&self.core);
        let iw = (self.placed_w - LARGE_FIELD_PAD_X * 2.0).max(0.0);
        let origin_x = self.x + LARGE_FIELD_PAD_X - self.core.scroll
            + self.core.align_shift(fonts, LARGE_FIELD_FONT_SIZE, text, iw);
        resolve_press_single(
            &mut self.core,
            fonts,
            &echo,
            LARGE_FIELD_FONT_SIZE,
            text,
            origin_x,
        );
        draw_field(
            scene,
            fonts,
            &mut self.core,
            self.x,
            self.y,
            self.placed_w,
            self.placed_h,
            &LARGE_METRICS,
        );
    }

    fn mouse_up(&mut self, _x: f64, _y: f64) {
        self.core.release();
    }

    fn set_hover(&mut self, x: f32, y: f32) {
        self.core.hovered = self.hit(x, y);
        if self.core.hovered || self.core.pressing {
            self.core.drag_to_point(x, y);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::text::FontSystem;

    fn field() -> LargeTextField {
        LargeTextField::new("Placeholder")
    }

    #[test]
    fn typing_selects_and_edits() {
        let mut field = field();
        let mut fonts = FontSystem::new();
        let (_, h) = field.measure(&mut fonts);
        field.place(&mut fonts, 0.0, 0.0, 400.0, h);
        assert!(!field.is_selected());
        field.mouse_down(20.0, (h / 2.0) as f64);
        assert!(field.is_selected());
        field.type_text("abc");
        assert_eq!(field.text_value(), "abc");
        field.key(Key::Backspace);
        assert_eq!(field.text_value(), "ab");
    }

    #[test]
    fn large_runs_roomier_than_basic() {
        use super::super::basic::BasicTextField;

        assert!(LARGE_FIELD_FONT_SIZE > super::super::basic::TEXTFIELD_FONT_SIZE);
        assert!(LARGE_FIELD_PAD_Y > super::super::basic::TEXTFIELD_PAD_Y);
        let mut fonts = FontSystem::new();
        let mut large = field();
        let mut basic = BasicTextField::new("Placeholder");
        assert!(large.measure(&mut fonts).1 > basic.measure(&mut fonts).1);
    }

    #[test]
    fn accent_ring_follows_theme_accent() {
        let mut field = field();
        let pink = Color::from_rgb8(0xff, 0x2d, 0x55);
        field.set_theme(pink, true);
        assert_eq!(field.core.accent, pink);
    }
}
