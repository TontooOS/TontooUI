use parley::Layout;
use vello::Scene;
use vello::kurbo::{Affine, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use crate::renderer::text::{FontSystem, SolidBrush, draw_layout};

const PADDING: f32 = 12.0;
const RADIUS: f32 = 10.0;
const CURSOR_WIDTH: f32 = 2.0;

/// Single-line text field. Click to focus, type to edit, `Backspace` and
/// arrow keys move the byte cursor. Click-to-position and selection are not
/// implemented yet; clicking focuses and moves the cursor to the end.
pub struct TextInput {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    text: String,
    cursor: usize,
    focused: bool,
    placeholder: String,
    on_change: Option<Box<dyn FnMut(&str)>>,
    layout: Option<Layout<SolidBrush>>,
    dirty: bool,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 300.0,
            height: 40.0,
            text: String::new(),
            cursor: 0,
            focused: false,
            placeholder: String::new(),
            on_change: None,
            layout: None,
            dirty: true,
        }
    }

    pub fn bounds(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
        self.dirty = true;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self.dirty = true;
        self
    }

    pub fn on_change(mut self, callback: impl FnMut(&str) + 'static) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    /// Move/resize. Rebuilds the layout only when the size changed.
    pub fn set_bounds(&mut self, x: f32, y: f32, width: f32, height: f32) {
        if width != self.width || height != self.height {
            self.dirty = true;
        }
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    fn notify(&mut self) {
        if let Some(callback) = self.on_change.as_mut() {
            callback(&self.text);
        }
    }

    /// Focus when the click lands inside the field, blur otherwise.
    pub fn mouse_down(&mut self, x: f64, y: f64) {
        let x = x as f32;
        let y = y as f32;
        let inside =
            x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height;
        self.focused = inside;
        if inside {
            self.cursor = self.text.len();
        }
    }

    /// Insert printable text at the cursor.
    pub fn insert(&mut self, text: &str) {
        if !self.focused || text.is_empty() {
            return;
        }
        // Ignore control characters; keep newlines out (single line).
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        if clean.is_empty() {
            return;
        }
        self.text.insert_str(self.cursor, &clean);
        self.cursor += clean.len();
        self.dirty = true;
        self.notify();
    }

    pub fn backspace(&mut self) {
        if !self.focused || self.cursor == 0 {
            return;
        }
        let prev = self.text[..self.cursor]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.text.drain(prev..self.cursor);
        self.cursor = prev;
        self.dirty = true;
        self.notify();
    }

    pub fn move_left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor = self.text[..self.cursor]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);
    }

    pub fn move_right(&mut self) {
        if self.cursor >= self.text.len() {
            return;
        }
        let c = self.text[self.cursor..].chars().next().expect("char exists");
        self.cursor += c.len_utf8();
    }

    fn ensure_layout(&mut self, fonts: &mut FontSystem) {
        if self.dirty || self.layout.is_none() {
            let shown = if self.text.is_empty() {
                self.placeholder.clone()
            } else {
                self.text.clone()
            };
            let color = if self.text.is_empty() {
                Color::from_rgb8(0x8e, 0x8e, 0x93)
            } else {
                Color::WHITE
            };
            self.layout = Some(fonts.layout_text(
                &shown,
                15.0,
                color,
                Some(self.width - PADDING * 2.0),
            ));
            self.dirty = false;
        }
    }

    pub fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, blink_on: bool) {
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;

        let rect = RoundedRect::new(
            px(self.x),
            px(self.y),
            px(self.x + self.width),
            px(self.y + self.height),
            px(RADIUS),
        );
        let bg = if self.focused {
            Color::from_rgb8(0x3a, 0x3a, 0x3c)
        } else {
            Color::from_rgb8(0x2c, 0x2c, 0x2e)
        };
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(bg),
            None,
            &rect,
        );

        if self.focused {
            let accent = Color::from_rgb8(0xff, 0x9f, 0x0a);
            scene.stroke(
                &Stroke::new(1.5 * scale),
                Affine::IDENTITY,
                &Brush::Solid(accent),
                None,
                &rect,
            );
        }

        self.ensure_layout(fonts);
        let layout = self.layout.as_ref().expect("layout built");
        let text_h = FontSystem::layout_size(layout).1 / fonts.scale;
        let text_y = self.y + (self.height - text_h) / 2.0;
        draw_layout(scene, layout, self.x + PADDING, text_y, fonts.scale);

        if self.focused && blink_on && !self.text.is_empty() {
            let cursor_x = self.x + PADDING + self.cursor_offset(fonts);
            let cursor = RoundedRect::new(
                px(cursor_x),
                px(text_y + 2.0),
                px(cursor_x + CURSOR_WIDTH),
                px(text_y + text_h - 2.0),
                px(1.0),
            );
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(Color::WHITE),
                None,
                &cursor,
            );
        } else if self.focused && blink_on {
            let cursor = RoundedRect::new(
                px(self.x + PADDING),
                px(text_y + 2.0),
                px(self.x + PADDING + CURSOR_WIDTH),
                px(text_y + text_h - 2.0),
                px(1.0),
            );
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(Color::WHITE),
                None,
                &cursor,
            );
        }
    }

    /// Logical x offset of the cursor relative to the text origin. Measures
    /// the prefix before the cursor with a scratch layout so shaping matches.
    fn cursor_offset(&mut self, fonts: &mut FontSystem) -> f32 {
        let end = self.cursor.min(self.text.len());
        if end == 0 {
            return 0.0;
        }
        let prefix = self.text[..end].to_owned();
        let scratch = fonts.layout_text(&prefix, 15.0, Color::WHITE, None);
        FontSystem::layout_size(&scratch).0 / fonts.scale
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}
