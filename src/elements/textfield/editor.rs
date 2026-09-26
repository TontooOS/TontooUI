use std::any::Any;
use std::time::Instant;

use vello::Scene;
use vello::peniko::Color;

use super::super::layout::View;
use super::{
    EditorMetrics, FieldCore, draw_editor_multiline, editor_caret_pos, editor_column,
    field_colors,
};
#[cfg(test)]
use super::{editor_caret_geometry, editor_lines};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::renderer::window::Key;

/// Editor text size in logical px.
pub const EDITOR_FONT_SIZE: f32 = 14.0;
/// Editor padding in logical px.
pub const EDITOR_PAD: f32 = 12.0;
/// Editor corner radius in logical px.
pub const EDITOR_RADIUS: f32 = 10.0;
/// Wrap width for intrinsic measure in logical px.
pub const EDITOR_WRAP_W: f32 = 240.0;
/// Minimum intrinsic height in logical px.
pub const EDITOR_MIN_H: f32 = 120.0;

/// Large multi-line text editor: wrapped text, Enter for newlines,
/// Up/Down/Left/Right caret motion, vertical caret tracking. The
/// caret geometry comes from parley line ranges, so wrapped lines
/// behave. Same modal contract as the fields: click inside selects,
/// ESC or outside clicks deselect, accent ring while selected.
/// Multiline geometry needs fonts, so `key` takes them (unlike the
/// single-line fields).
pub struct TextEditor {
    core: FieldCore,
    scroll_y: f32,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl TextEditor {
    pub fn new(placeholder: impl Into<String>) -> Self {
        let mut core = FieldCore::new(placeholder.into());
        core.multiline = true;
        Self {
            core,
            scroll_y: 0.0,
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

    /// Type text at the caret (the app forwards its `text` here
    /// while selected; `\n` starts a new line).
    pub fn type_text(&mut self, content: &str) {
        if self.core.selected {
            self.core.insert(content);
        }
    }

    /// Key handling while selected: Backspace deletes, arrows move
    /// the caret (Up/Down keep the column, Shift extends), Enter
    /// breaks the line, Ctrl+A/C/X/V/Z/Y select, copy, cut, paste,
    /// undo, redo, ESC deselects. Returns true when consumed. Needs
    /// fonts for the multiline caret geometry.
    pub fn key(&mut self, fonts: &mut FontSystem, key: Key) -> bool {
        if !self.core.selected {
            return false;
        }
        match key {
            Key::Up => self.move_vertical(fonts, true, self.inner_width().max(0.0), false),
            Key::Down => self.move_vertical(fonts, false, self.inner_width().max(0.0), false),
            Key::SelectUp => self.move_vertical(fonts, true, self.inner_width().max(0.0), true),
            Key::SelectDown => {
                self.move_vertical(fonts, false, self.inner_width().max(0.0), true)
            }
            Key::Enter => self.core.insert("\n"),
            _ => return self.core.handle_key(key),
        }
        true
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

    /// True while the pointer hovers the editor: the app returns
    /// the I-beam cursor from `App::cursor` then.
    pub fn wants_text_cursor(&self) -> bool {
        self.core.hovered
    }

    /// Vertical caret motion with a goal column. Pure helper for
    /// tests (needs fonts for advances).
    pub(crate) fn move_vertical(
        &mut self,
        fonts: &mut FontSystem,
        up: bool,
        wrap: f32,
        extend: bool,
    ) {
        let before = self.core.caret;
        let was_sel = self.core.sel;
        let (_, _, text) = field_colors(&self.core);
        let lines = self.core.cached_editor_lines(fonts, EDITOR_FONT_SIZE, text, wrap);
        let (line, goal_x) = editor_caret_pos(
            fonts,
            &self.core.text,
            self.core.caret,
            EDITOR_FONT_SIZE,
            text,
            &lines,
        );
        if up && line > 0 {
            self.core.caret = editor_column(
                fonts,
                &self.core.text,
                line - 1,
                goal_x,
                EDITOR_FONT_SIZE,
                text,
                &lines,
            );
        } else if !up && line + 1 < lines.len() {
            self.core.caret = editor_column(
                fonts,
                &self.core.text,
                line + 1,
                goal_x,
                EDITOR_FONT_SIZE,
                text,
                &lines,
            );
        }
        // Down on the last line and Up on the first are no-ops.
        if extend {
            if !was_sel {
                self.core.anchor = before;
            }
            self.core.sel = self.core.anchor != self.core.caret;
        } else {
            self.core.collapse();
        }
    }

    /// Byte caret at a field-coords point (click/drag mapping).
    fn caret_at_point(&mut self, fonts: &mut FontSystem, qx: f32, qy: f32) -> usize {
        let (_, _, text) = field_colors(&self.core);
        let iw = self.inner_width().max(0.0);
        let lines = self.core.cached_editor_lines(fonts, EDITOR_FONT_SIZE, text, iw);
        let rel_y = (qy - self.y - EDITOR_PAD + self.scroll_y).max(0.0);
        let mut line = lines.len().saturating_sub(1);
        let mut acc = 0.0;
        for (i, (_, _, h)) in lines.iter().enumerate() {
            let lh = *h / fonts.scale;
            if rel_y < acc + lh {
                line = i;
                break;
            }
            acc += lh;
        }
        editor_column(
            fonts,
            &self.core.text,
            line,
            (qx - self.x - EDITOR_PAD).max(0.0),
            EDITOR_FONT_SIZE,
            text,
            &lines,
        )
    }

    /// Resolve pending press/drag points to carets (needs fonts and
    /// the placed geometry, so it runs at the top of `draw`).
    fn resolve_press(&mut self, fonts: &mut FontSystem) {
        if let Some((qx, qy)) = self.core.take_press() {
            let caret = self.caret_at_point(fonts, qx, qy);
            self.core.finish_press(caret, Instant::now());
        }
        if let Some((qx, qy)) = self.core.take_drag() {
            let caret = self.caret_at_point(fonts, qx, qy);
            self.core.finish_drag(caret);
        }
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

    fn inner_width(&self) -> f32 {
        (self.placed_w - EDITOR_PAD * 2.0).max(0.0)
    }

    /// Caret line index plus goal x in logical px (test helper).
    #[cfg(test)]
    fn caret_line_col(&mut self, fonts: &mut FontSystem, wrap: f32) -> (usize, f32) {
        let (_, _, text) = field_colors(&self.core);
        let lines = editor_lines(fonts, &self.core.text, EDITOR_FONT_SIZE, text, wrap);
        editor_caret_pos(
            fonts,
            &self.core.text,
            self.core.caret,
            EDITOR_FONT_SIZE,
            text,
            &lines,
        )
    }
}

const EDITOR_METRICS: EditorMetrics = EditorMetrics {
    font_size: EDITOR_FONT_SIZE,
    pad: EDITOR_PAD,
    radius: EDITOR_RADIUS,
};

impl View for TextEditor {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let (_, _, text) = field_colors(&self.core);
        let content = if self.core.text.is_empty() {
            self.core.placeholder.clone()
        } else {
            self.core.text.clone()
        };
        let layout = fonts.layout_text(&content, EDITOR_FONT_SIZE, text, Some(EDITOR_WRAP_W));
        let (_, th) = FontSystem::layout_size(&layout);
        (
            EDITOR_WRAP_W + EDITOR_PAD * 2.0,
            (th / fonts.scale + EDITOR_PAD * 2.0).max(EDITOR_MIN_H),
        )
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
        self.resolve_press(fonts);
        let (fill, border, _) = field_colors(&self.core);
        self.scroll_y = draw_editor_multiline(
            scene,
            fonts,
            &mut self.core,
            self.x,
            self.y,
            self.placed_w,
            self.placed_h,
            &EDITOR_METRICS,
            fill,
            border,
            self.scroll_y,
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

    fn editor() -> TextEditor {
        TextEditor::new("Write something…")
    }

    #[test]
    fn enter_breaks_lines_and_backspace_joins() {
        let mut editor = editor();
        let mut fonts = FontSystem::new();
        editor.mouse_down(10.0, 10.0);
        // No placed rect yet: outside clicks deselect.
        assert!(!editor.is_selected());
        editor.place(&mut fonts, 0.0, 0.0, 400.0, 200.0);
        editor.mouse_down(10.0, 10.0);
        assert!(editor.is_selected());
        editor.type_text("ab");
        editor.key(&mut fonts, Key::Enter);
        editor.type_text("ab");
        assert_eq!(editor.text_value(), "ab\nab");
        // Caret sits on line 1: Up keeps the column on line 0.
        let (line, x) = editor.caret_line_col(&mut fonts, 300.0);
        assert_eq!(line, 1);
        editor.move_vertical(&mut fonts, true, 300.0, false);
        let (up_line, up_x) = editor.caret_line_col(&mut fonts, 300.0);
        assert_eq!(up_line, 0);
        assert!((up_x - x).abs() < 1e-4);
        // Backspace at line start joins the lines.
        editor.key(&mut fonts, Key::Enter);
        editor.key(&mut fonts, Key::Backspace);
        assert_eq!(editor.text_value(), "ab\nab");
        let _ = line;
    }

    #[test]
    fn escape_and_outside_click_deselect() {
        let mut editor = editor();
        let mut fonts = FontSystem::new();
        editor.place(&mut fonts, 0.0, 0.0, 400.0, 200.0);
        editor.mouse_down(10.0, 10.0);
        assert!(editor.is_selected());
        assert!(editor.key(&mut fonts, Key::Escape));
        assert!(!editor.is_selected());
        assert!(!editor.key(&mut fonts, Key::Backspace));
    }

    #[test]
    fn control_chars_filtered_except_newline() {
        let mut editor = editor();
        let mut fonts = FontSystem::new();
        editor.place(&mut fonts, 0.0, 0.0, 400.0, 200.0);
        editor.mouse_down(10.0, 10.0);
        editor.type_text("a\tb\rc\nd");
        assert_eq!(editor.text_value(), "abc\nd");
    }

    #[test]
    fn select_up_extends_highlight() {
        let mut editor = editor();
        let mut fonts = FontSystem::new();
        editor.place(&mut fonts, 0.0, 0.0, 400.0, 200.0);
        editor.mouse_down(10.0, 10.0);
        editor.type_text("ab\ncd");
        assert!(editor.key(&mut fonts, Key::SelectUp));
        assert_eq!(editor.selection_range(), Some((2, 5)));
        assert_eq!(editor.selected_text(), "\ncd");
        editor.key(&mut fonts, Key::Down);
        assert_eq!(editor.selection_range(), None);
    }

    #[test]
    fn select_all_and_cut_paste_multiline() {
        let _guard = crate::elements::textfield::clipboard::test_lock();
        let mut editor = editor();
        let mut fonts = FontSystem::new();
        editor.place(&mut fonts, 0.0, 0.0, 400.0, 200.0);
        editor.mouse_down(10.0, 10.0);
        editor.type_text("one\ntwo");
        assert!(editor.key(&mut fonts, Key::SelectAll));
        assert_eq!(editor.selected_text(), "one\ntwo");
        assert!(editor.key(&mut fonts, Key::Cut));
        assert_eq!(editor.text_value(), "");
        assert!(editor.key(&mut fonts, Key::Paste));
        assert_eq!(editor.text_value(), "one\ntwo");
    }

    #[test]
    fn caret_after_empty_line_lands_on_next_line() {
        let mut editor = editor();
        let mut fonts = FontSystem::new();
        editor.place(&mut fonts, 0.0, 0.0, 400.0, 300.0);
        editor.mouse_down(10.0, 10.0);
        editor.type_text("a\n\nb");
        let (_, _, text) = field_colors(&editor.core);
        let wrap = 300.0;
        let lines = editor_lines(&mut fonts, editor.text_value(), EDITOR_FONT_SIZE, text, wrap);
        assert!(lines.len() >= 3);
        // End of "a": first line.
        let (first, _) =
            editor_caret_pos(&mut fonts, editor.text_value(), 1, EDITOR_FONT_SIZE, text, &lines);
        assert_eq!(first, 0);
        // Start of the empty line: the middle line.
        let (empty, _) =
            editor_caret_pos(&mut fonts, editor.text_value(), 2, EDITOR_FONT_SIZE, text, &lines);
        assert_eq!(empty, 1);
        // Start of "b": last line at x ~ 0, not inside the gap.
        let (last, x) =
            editor_caret_pos(&mut fonts, editor.text_value(), 3, EDITOR_FONT_SIZE, text, &lines);
        assert_eq!(last, lines.len() - 1);
        assert!(x < 5.0);
        // Geometry drops one line lower per step.
        let (_, y1, _) = editor_caret_geometry(
            &mut fonts,
            editor.text_value(),
            1,
            EDITOR_FONT_SIZE,
            text,
            &lines,
        );
        let (_, y2, _) = editor_caret_geometry(
            &mut fonts,
            editor.text_value(),
            2,
            EDITOR_FONT_SIZE,
            text,
            &lines,
        );
        let (_, y3, _) = editor_caret_geometry(
            &mut fonts,
            editor.text_value(),
            3,
            EDITOR_FONT_SIZE,
            text,
            &lines,
        );
        assert!(y2 > y1 && y3 > y2);
    }

    #[test]
    fn large_text_caret_ops_stay_correct() {
        let mut editor = editor();
        let mut fonts = FontSystem::new();
        editor.place(&mut fonts, 0.0, 0.0, 400.0, 300.0);
        editor.mouse_down(10.0, 10.0);
        let chunk = "lorem ipsum dolor sit amet ".repeat(80);
        editor.type_text(&format!("{chunk}\n\n{chunk}"));
        assert!(editor.text_value().len() > 4000);
        let (_, _, text) = field_colors(&editor.core);
        let wrap = 300.0;
        let lines = editor_lines(&mut fonts, editor.text_value(), EDITOR_FONT_SIZE, text, wrap);
        let len = editor.text_value().len();
        for caret in [0, 100, 1000, 2160, 2162, 3000, len] {
            let caret = caret.min(len);
            let (line, x) = editor_caret_pos(
                &mut fonts,
                editor.text_value(),
                caret,
                EDITOR_FONT_SIZE,
                text,
                &lines,
            );
            assert!(line < lines.len());
            assert!(x >= 0.0);
            let col = editor_column(
                &mut fonts,
                editor.text_value(),
                line,
                x + 3.0,
                EDITOR_FONT_SIZE,
                text,
                &lines,
            );
            assert!(col <= len);
        }
        // Cached lines reuse: same breaks without re-laying-out.
        let again = editor
            .core
            .cached_editor_lines(&mut fonts, EDITOR_FONT_SIZE, text, wrap);
        assert_eq!(again, lines);
    }

    #[test]
    fn measure_grows_with_wrapped_lines() {
        let mut fonts = FontSystem::new();
        let mut one = editor();
        let mut many = editor();
        many.set_text("one\ntwo\nthree\nfour\nfive\nsix\nseven\neight");
        assert!(many.measure(&mut fonts).1 > one.measure(&mut fonts).1);
    }
}
