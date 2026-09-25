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

    /// Borderless text (form rows): no fill, ring or border, text
    /// plus caret and highlight only.
    pub fn borderless(mut self, borderless: bool) -> Self {
        self.core.borderless = borderless;
        self
    }

    pub fn set_borderless(&mut self, borderless: bool) {
        self.core.borderless = borderless;
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
        // Pending press/drag points resolve here: caret mapping
        // needs fonts, which only `draw` has.
        let echo = self.core.echo();
        let (_, _, text) = field_colors(&self.core);
        let origin_x = self.x + TEXTFIELD_PAD_X - self.core.scroll;
        resolve_press_single(
            &mut self.core,
            fonts,
            &echo,
            TEXTFIELD_FONT_SIZE,
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
            &BASIC_METRICS,
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

    fn typed(text: &str) -> (BasicTextField, FontSystem) {
        let mut field = field();
        let mut fonts = FontSystem::new();
        let (_, h) = field.measure(&mut fonts);
        field.place(&mut fonts, 0.0, 0.0, 400.0, h.max(28.0));
        field.mouse_down(300.0, 10.0);
        field.type_text(text);
        (field, fonts)
    }

    #[test]
    fn select_all_shortcut_highlights_everything() {
        let (mut field, _) = typed("hello");
        assert!(field.key(Key::SelectAll));
        assert_eq!(field.selection_range(), Some((0, 5)));
        assert_eq!(field.selected_text(), "hello");
    }

    #[test]
    fn shift_arrows_extend_and_plain_collapses() {
        let (mut field, _) = typed("hello");
        field.key(Key::SelectLeft);
        field.key(Key::SelectLeft);
        assert_eq!(field.selection_range(), Some((3, 5)));
        assert_eq!(field.selected_text(), "lo");
        field.key(Key::Left);
        assert_eq!(field.selection_range(), None);
        field.type_text("X");
        assert_eq!(field.text_value(), "helXlo");
    }

    #[test]
    fn backspace_deletes_selection_and_undo_restores() {
        let (mut field, _) = typed("hello");
        field.key(Key::SelectAll);
        field.key(Key::Backspace);
        assert_eq!(field.text_value(), "");
        assert!(field.key(Key::Undo));
        assert_eq!(field.text_value(), "hello");
        assert!(field.key(Key::Redo));
        assert_eq!(field.text_value(), "");
        // `key` reports consumed; the direct stack call reports effect.
        assert!(!field.redo());
        assert!(!field.cut_selection());
    }

    #[test]
    fn typing_replaces_selection_in_one_undo_step() {
        let (mut field, _) = typed("hello");
        field.key(Key::SelectAll);
        field.type_text("hi");
        assert_eq!(field.text_value(), "hi");
        field.key(Key::Undo);
        assert_eq!(field.text_value(), "hello");
    }

    #[test]
    fn cut_copy_paste_roundtrip() {
        let (mut field, _) = typed("hello world");
        for _ in 0..5 {
            field.key(Key::SelectLeft);
        }
        assert_eq!(field.selected_text(), "world");
        assert!(field.key(Key::Copy));
        assert_eq!(field.text_value(), "hello world");
        assert!(field.key(Key::Cut));
        assert_eq!(field.text_value(), "hello ");
        assert!(field.key(Key::Paste));
        assert_eq!(field.text_value(), "hello world");
    }

    #[test]
    fn double_click_highlights_word() {
        use std::time::Instant;

        let (mut field, _) = typed("hello world");
        field.core.finish_press(3, Instant::now());
        assert_eq!(field.selection_range(), None);
        field.core.finish_press(3, Instant::now());
        assert_eq!(field.selection_range(), Some((0, 5)));
        assert_eq!(field.selected_text(), "hello");
    }

    #[test]
    fn drag_extends_from_press_anchor() {
        use std::time::Instant;

        let (mut field, _) = typed("hello world");
        field.core.finish_press(0, Instant::now());
        field.core.finish_drag(5);
        assert_eq!(field.selection_range(), Some((0, 5)));
    }

    #[test]
    fn shortcuts_ignore_deselected_field() {
        let mut field = field();
        assert!(!field.key(Key::SelectAll));
        assert!(!field.key(Key::Copy));
        assert!(!field.key(Key::Paste));
        assert!(!field.key(Key::Undo));
    }
}
