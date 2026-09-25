use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, Rect, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::layout::View;
use super::{
    FieldCore, TEXTFIELD_CARET_W, TEXTFIELD_RING_W, accent_color, caret_blink,
    field_colors, placeholder_color,
};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
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
    lines: Vec<(usize, usize, f32)>,
    lines_wrap: f32,
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
            lines: Vec::new(),
            lines_wrap: 0.0,
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
    /// the caret (Up/Down keep the column), Enter breaks the line,
    /// ESC deselects. Returns true when consumed. Needs fonts for
    /// the multiline caret geometry.
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

    /// Vertical caret motion with a goal column. Pure helper for
    /// tests (needs fonts for advances).
    pub(crate) fn move_vertical(
        &mut self,
        fonts: &mut FontSystem,
        up: bool,
        wrap: f32,
    ) {
        let (line, goal_x) = self.caret_line_col(fonts, wrap);
        if up && line > 0 {
            self.core.caret = self.column_caret(fonts, line - 1, goal_x, wrap);
        } else if !up && line + 1 < self.lines.len() {
            self.core.caret = self.column_caret(fonts, line + 1, goal_x, wrap);
        }
        // Down on the last line and Up on the first are no-ops.
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
        (self.placed_w - EDITOR_PAD * 2.0).max(0.0)
    }

    /// Line (start, end, height) ranges of the full text at `wrap`,
    /// refreshed with the layout.
    fn refresh_lines(&mut self, fonts: &mut FontSystem, wrap: f32) {
        if !self.lines.is_empty() && (self.lines_wrap - wrap).abs() < f32::EPSILON && !self.core.dirty
        {
            return;
        }
        let (_, _, text) = field_colors(&self.core);
        let content = if self.core.text.is_empty() {
            self.core.placeholder.clone()
        } else {
            self.core.text.clone()
        };
        let probe = fonts.layout_text(&content, EDITOR_FONT_SIZE, text, Some(wrap.max(0.0)));
        self.lines.clear();
        for index in 0..probe.len() {
            if let Some(line) = probe.get(index) {
                let range = line.text_range();
                let height = line.metrics().line_height;
                self.lines.push((range.start, range.end, height));
            }
        }
        if self.lines.is_empty() {
            self.lines.push((0, 0, EDITOR_FONT_SIZE * 1.25));
        }
        self.lines_wrap = wrap;
        self.core.dirty = false;
    }

    /// Caret line index plus goal x in logical px.
    fn caret_line_col(&mut self, fonts: &mut FontSystem, wrap: f32) -> (usize, f32) {
        self.refresh_lines(fonts, wrap);
        let caret = self.core.caret.min(self.core.text.len());
        let mut line = self.lines.len().saturating_sub(1);
        for (index, (start, end, _)) in self.lines.iter().enumerate() {
            if caret >= *start && caret <= *end {
                line = index;
                break;
            }
        }
        // x: advance of the caret within its line (char walk, LTR).
        let (_, _, text) = field_colors(&self.core);
        let (start, end, _) = self.lines[line];
        let end = end.min(self.core.text.len());
        let mut x = 0.0;
        let mut at = start.min(end);
        while at < caret.min(end) {
            let next = self.core.text[at..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| at + i)
                .unwrap_or(end);
            let ch = &self.core.text[at..next.min(end)];
            let probe = fonts.layout_text(ch, EDITOR_FONT_SIZE, text, None);
            x += FontSystem::layout_size(&probe).0 / fonts.scale;
            at = next.min(end);
            if at >= end {
                break;
            }
        }
        // Caret exactly at a line end before `\n` belongs to the line
        // start below in standard editors; keep it on its line here
        // (documented v1 behavior).
        (line, x)
    }

    /// Byte index in `line` at column `goal_x` (nearest advance).
    fn column_caret(
        &mut self,
        fonts: &mut FontSystem,
        line: usize,
        goal_x: f32,
        wrap: f32,
    ) -> usize {
        self.refresh_lines(fonts, wrap);
        let line = line.min(self.lines.len().saturating_sub(1));
        let (_, _, text) = field_colors(&self.core);
        let (start, mut end, _) = self.lines[line];
        end = end.min(self.core.text.len());
        // A trailing newline belongs to the break, not the column walk.
        let walk_end = if self.core.text[start.min(end)..end].ends_with('\n') {
            end.saturating_sub(1).max(start.min(end))
        } else {
            end
        };
        let mut x = 0.0;
        let mut at = start.min(walk_end);
        while at < walk_end {
            let next = self.core.text[at..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| at + i)
                .unwrap_or(walk_end);
            let ch = &self.core.text[at..next.min(walk_end)];
            let probe = fonts.layout_text(ch, EDITOR_FONT_SIZE, text, None);
            let adv = FontSystem::layout_size(&probe).0 / fonts.scale;
            if x + adv / 2.0 >= goal_x {
                break;
            }
            x += adv;
            at = next.min(walk_end);
        }
        // Past the last line end lands at the very end.
        if line == self.lines.len().saturating_sub(1) && goal_x >= x {
            return self.core.text.len();
        }
        at
    }

    /// Caret (x, y, height) in logical px relative to the text origin.
    fn caret_geometry(&mut self, fonts: &mut FontSystem, wrap: f32) -> (f32, f32, f32) {
        let (line, x) = self.caret_line_col(fonts, wrap);
        let mut y = 0.0;
        for (_, _, height) in self.lines.iter().take(line) {
            y += *height / fonts.scale;
        }
        let line_h = self.lines.get(line).map(|(_, _, h)| *h).unwrap_or(EDITOR_FONT_SIZE * 1.25)
            / fonts.scale;
        (x, y, line_h)
    }
}

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
        if self.placed_w <= 0.0 || self.placed_h <= 0.0 {
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let (fill, border, text) = field_colors(&self.core);
        let body = RoundedRect::new(
            px(self.x),
            px(self.y),
            px(self.x + self.placed_w),
            px(self.y + self.placed_h),
            px(EDITOR_RADIUS),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(fill),
            None,
            &body,
        );
        if self.core.selected {
            scene.stroke(
                &Stroke::new(px(TEXTFIELD_RING_W)),
                Affine::IDENTITY,
                &Brush::Solid(accent_color(&self.core)),
                None,
                &body,
            );
        } else {
            scene.stroke(
                &Stroke::new(1.0 * scale),
                Affine::IDENTITY,
                &Brush::Solid(border),
                None,
                &body,
            );
        }
        // Wrapped text clipped to the padded box with vertical caret
        // tracking.
        let ix = self.x + EDITOR_PAD;
        let iy = self.y + EDITOR_PAD;
        let iw = self.inner_width();
        let ih = (self.placed_h - EDITOR_PAD * 2.0).max(0.0);
        let clip = RoundedRect::new(px(ix), px(iy), px(ix + iw), px(iy + ih), px(4.0));
        scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &clip);
        let placeholder = placeholder_color(&self.core);
        {
            let (layout, _) = self.core.ensure_layout(
                fonts,
                EDITOR_FONT_SIZE,
                text,
                placeholder,
                Some(iw.max(0.0)),
            );
            draw_layout(scene, layout, ix, iy - self.scroll_y, fonts.scale);
        }
        let (cx, cy, ch) = self.caret_geometry(fonts, iw.max(0.0));
        // Track after measuring so the caret stays visible.
        if cy - self.scroll_y + ch > ih {
            self.scroll_y = (cy + ch - ih).max(0.0);
        }
        if cy - self.scroll_y < 0.0 {
            self.scroll_y = cy.max(0.0);
        }
        if self.core.selected && caret_blink() {
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(accent_color(&self.core)),
                None,
                &Rect::new(
                    px(ix + cx),
                    px(iy + cy - self.scroll_y),
                    px(ix + cx + TEXTFIELD_CARET_W),
                    px(iy + cy - self.scroll_y + ch),
                ),
            );
        }
        scene.pop_layer();
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
        editor.move_vertical(&mut fonts, true, 300.0);
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
    fn measure_grows_with_wrapped_lines() {
        let mut fonts = FontSystem::new();
        let mut one = editor();
        let mut many = editor();
        many.set_text("one\ntwo\nthree\nfour\nfive\nsix\nseven\neight");
        assert!(many.measure(&mut fonts).1 > one.measure(&mut fonts).1);
    }
}
