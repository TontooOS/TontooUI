use std::any::Any;

use vello::Scene;

use super::super::layout::View;
use super::{
    EditorMetrics, FieldCore, draw_editor_multiline, editor_caret_pos,
    editor_column, editor_lines, field_colors,
};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::renderer::window::Key;
use vello::peniko::Color;

/// Large editor text size in logical px.
pub const LARGE_EDITOR_FONT_SIZE: f32 = 15.0;
/// Large editor padding in logical px.
pub const LARGE_EDITOR_PAD: f32 = 14.0;
/// Large editor corner radius in logical px.
pub const LARGE_EDITOR_RADIUS: f32 = 12.0;
/// Wrap width for intrinsic measure in logical px.
pub const LARGE_EDITOR_WRAP_W: f32 = 320.0;
/// Minimum intrinsic height in logical px.
pub const LARGE_EDITOR_MIN_H: f32 = 220.0;
/// Large editor fill in dark mode (near-black inset).
pub const LARGE_EDITOR_BG_DARK: Color = Color::from_rgb8(0x14, 0x14, 0x16);
/// Large editor fill in light mode.
pub const LARGE_EDITOR_BG_LIGHT: Color = Color::from_rgb8(0xf2, 0xf2, 0xf5);

const LARGE_EDITOR_METRICS: EditorMetrics = EditorMetrics {
    font_size: LARGE_EDITOR_FONT_SIZE,
    pad: LARGE_EDITOR_PAD,
    radius: LARGE_EDITOR_RADIUS,
};

/// Large multi-line text editor (like the reference inset box):
/// roomier type and padding than `TextEditor`, near-black fill,
/// wrapped text, Enter for newlines, Up/Down/Left/Right caret motion
/// with column memory, vertical caret tracking. Same modal contract:
/// click inside selects, ESC or outside clicks deselect, accent ring
/// while selected. `key` takes fonts for the multiline caret
/// geometry (buffer keys to `draw` when the app has none in its
/// `key` handler).
pub struct LargeTextEditor {
    core: FieldCore,
    scroll_y: f32,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl LargeTextEditor {
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
    /// the caret (Up/Down keep the column), Enter breaks the line,
    /// ESC deselects. Returns true when consumed.
    pub fn key(&mut self, fonts: &mut FontSystem, key: Key) -> bool {
        if !self.core.selected {
            return false;
        }
        match key {
            Key::Backspace => self.core.backspace(),
            Key::Left => self.core.move_left(),
            Key::Right => self.core.move_right(),
            Key::Up => self.move_vertical(fonts, true, self.inner_width().max(0.0)),
            Key::Down => self.move_vertical(fonts, false, self.inner_width().max(0.0)),
            Key::Enter => self.core.insert("\n"),
            Key::Escape => self.core.deselect(),
        }
        true
    }

    /// Vertical caret motion with a goal column.
    pub(crate) fn move_vertical(
        &mut self,
        fonts: &mut FontSystem,
        up: bool,
        wrap: f32,
    ) {
        let (_, _, text) = field_colors(&self.core);
        let lines = editor_lines(fonts, &self.core.text, LARGE_EDITOR_FONT_SIZE, text, wrap);
        let (line, goal_x) = editor_caret_pos(
            fonts,
            &self.core.text,
            self.core.caret,
            LARGE_EDITOR_FONT_SIZE,
            text,
            &lines,
        );
        if up && line > 0 {
            self.core.caret = editor_column(
                fonts,
                &self.core.text,
                line - 1,
                goal_x,
                LARGE_EDITOR_FONT_SIZE,
                text,
                &lines,
            );
        } else if !up && line + 1 < lines.len() {
            self.core.caret = editor_column(
                fonts,
                &self.core.text,
                line + 1,
                goal_x,
                LARGE_EDITOR_FONT_SIZE,
                text,
                &lines,
            );
        }
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

    fn inner_width(&self) -> f32 {
        (self.placed_w - LARGE_EDITOR_PAD * 2.0).max(0.0)
    }

    fn fill(&self) -> Color {
        if self.core.dark {
            LARGE_EDITOR_BG_DARK
        } else {
            LARGE_EDITOR_BG_LIGHT
        }
    }
}

impl View for LargeTextEditor {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let (_, _, text) = field_colors(&self.core);
        let content = if self.core.text.is_empty() {
            self.core.placeholder.clone()
        } else {
            self.core.text.clone()
        };
        let layout = fonts.layout_text(&content, LARGE_EDITOR_FONT_SIZE, text, Some(LARGE_EDITOR_WRAP_W));
        let (_, th) = FontSystem::layout_size(&layout);
        (
            LARGE_EDITOR_WRAP_W + LARGE_EDITOR_PAD * 2.0,
            (th / fonts.scale + LARGE_EDITOR_PAD * 2.0).max(LARGE_EDITOR_MIN_H),
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
        let (_, border, _) = field_colors(&self.core);
        let (x, y, w, h) = (self.x, self.y, self.placed_w, self.placed_h);
        let fill = self.fill();
        let scroll = self.scroll_y;
        self.scroll_y = draw_editor_multiline(
            scene,
            fonts,
            &mut self.core,
            x,
            y,
            w,
            h,
            &LARGE_EDITOR_METRICS,
            fill,
            border,
            scroll,
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

    fn editor() -> LargeTextEditor {
        LargeTextEditor::new("Start typing here…")
    }

    #[test]
    fn typing_and_newlines() {
        let mut editor = editor();
        let mut fonts = FontSystem::new();
        editor.place(&mut fonts, 0.0, 0.0, 500.0, 300.0);
        editor.mouse_down(10.0, 10.0);
        assert!(editor.is_selected());
        editor.type_text("hello");
        editor.key(&mut fonts, Key::Enter);
        editor.type_text("world");
        assert_eq!(editor.text_value(), "hello\nworld");
    }

    #[test]
    fn vertical_motion_keeps_column() {
        let mut editor = editor();
        let mut fonts = FontSystem::new();
        editor.place(&mut fonts, 0.0, 0.0, 500.0, 300.0);
        editor.mouse_down(10.0, 10.0);
        editor.type_text("ab\nab");
        editor.move_vertical(&mut fonts, true, 400.0);
        let (_, _, text) = field_colors(&editor.core);
        let lines = editor_lines(&mut fonts, &editor.core.text, LARGE_EDITOR_FONT_SIZE, text, 400.0);
        let (line, _) =
            editor_caret_pos(&mut fonts, &editor.core.text, editor.core.caret, LARGE_EDITOR_FONT_SIZE, text, &lines);
        assert_eq!(line, 0);
        assert_eq!(editor.core.caret, 2);
    }

    #[test]
    fn large_runs_roomier_than_editor() {
        use super::super::editor::{EDITOR_MIN_H, TextEditor};

        assert!(LARGE_EDITOR_MIN_H > EDITOR_MIN_H);
        let mut fonts = FontSystem::new();
        let mut large = editor();
        let mut plain = TextEditor::new("x");
        assert!(large.measure(&mut fonts).1 >= plain.measure(&mut fonts).1);
    }
}
