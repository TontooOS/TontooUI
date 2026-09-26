pub mod basic;
pub(crate) mod clipboard;
pub mod editor;
pub mod large;
pub mod large_editor;
pub mod search;
pub mod secure;

pub use basic::{
    BasicTextField, TEXTFIELD_FONT_SIZE, TEXTFIELD_PAD_X, TEXTFIELD_PAD_Y,
    TEXTFIELD_RADIUS,
};
pub use editor::{
    TextEditor, EDITOR_FONT_SIZE, EDITOR_MIN_H, EDITOR_PAD, EDITOR_RADIUS,
    EDITOR_WRAP_W,
};
pub use large::{
    LargeTextField, LARGE_FIELD_FONT_SIZE, LARGE_FIELD_PAD_X, LARGE_FIELD_PAD_Y,
    LARGE_FIELD_RADIUS,
};
pub use large_editor::{
    LargeTextEditor, LARGE_EDITOR_BG_DARK, LARGE_EDITOR_BG_LIGHT,
    LARGE_EDITOR_FONT_SIZE, LARGE_EDITOR_MIN_H, LARGE_EDITOR_PAD,
    LARGE_EDITOR_RADIUS, LARGE_EDITOR_WRAP_W,
};
pub use search::{
    SearchField, SEARCH_FONT_SIZE, SEARCH_GAP, SEARCH_ICON_SIZE, SEARCH_PAD_X,
};
pub use secure::SecureField;

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use vello::Scene;
use vello::kurbo::{Affine, Rect, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use crate::elements::gestures::GESTURE_DOUBLE_TAP_SECONDS;
use crate::renderer::text::{CTFrame, FontSystem, draw_layout, point_to_caret};
use crate::renderer::window::Key;
use crate::theme::desaturate;

/// Field fill in dark mode.
pub const TEXTFIELD_BG_DARK: Color = Color::from_rgb8(0x2c, 0x2c, 0x2e);
/// Field fill in light mode.
pub const TEXTFIELD_BG_LIGHT: Color = Color::from_rgb8(0xff, 0xff, 0xff);
/// Idle 1 px border in dark mode.
pub const TEXTFIELD_BORDER_DARK: Color = Color::from_rgba8(255, 255, 255, 36);
/// Idle 1 px border in light mode.
pub const TEXTFIELD_BORDER_LIGHT: Color = Color::from_rgba8(0, 0, 0, 60);
/// Default accent (theme blue), like buttons.
pub const TEXTFIELD_ACCENT: Color = Color::from_rgb8(0x00, 0x7a, 0xff);
/// Focus ring width in logical px.
pub const TEXTFIELD_RING_W: f32 = 2.0;
/// Placeholder gray in dark mode.
pub const TEXTFIELD_PLACEHOLDER_DARK: Color = Color::from_rgb8(0x9a, 0x9a, 0x9e);
/// Placeholder gray in light mode.
pub const TEXTFIELD_PLACEHOLDER_LIGHT: Color = Color::from_rgb8(0x6e, 0x6e, 0x72);
/// Caret width in logical px.
pub const TEXTFIELD_CARET_W: f32 = 2.0;
/// Caret blink period in seconds (macOS-like).
pub const TEXTFIELD_BLINK_SECONDS: f64 = 1.06;
/// Undo/redo depth per field.
pub const TEXTFIELD_UNDO_LIMIT: usize = 100;
/// Selection highlight alpha (0-1 of the accent).
pub const TEXTFIELD_SELECTION_ALPHA: f32 = 0.3;

#[derive(Clone)]
struct UndoState {
    text: String,
    caret: usize,
    anchor: usize,
    sel: bool,
}

/// Cached CoreText line ranges for multiline geometry. Line breaks
/// only move when the text, size, wrap or scale change, so frames
/// reuse them instead of re-laying-out every draw (the old code
/// laid the whole text out per frame, which lagged past ~1K chars).
#[derive(Clone, Default)]
struct LinesCache {
    text: String,
    size: f32,
    wrap: f32,
    scale: f32,
    lines: Vec<(usize, usize, f32)>,
    valid: bool,
}

/// Shared single-line editing core behind the field variants:
/// text plus caret (always a char boundary), placeholder, selection
/// and accent. `masked` echoes bullets (secure fields), `multiline`
/// keeps newlines (editor). Layouts cache per content, color, size,
/// wrap and scale per the crisp text rules.
///
/// Text selection is a fixed `anchor` plus the moving `caret`,
/// visible while `sel` holds and both ends differ. Clicks and drags
/// record points (`press`/`drag`) that the variant resolves to carets
/// in `draw` (caret mapping needs fonts); `hovered` drives the
/// I-beam cursor. Every mutation pushes undo (capped); undo/redo
/// restore text plus caret and fire `on_change`.
pub(crate) struct FieldCore {
    text: String,
    caret: usize,
    anchor: usize,
    sel: bool,
    placeholder: String,
    selected: bool,
    masked: bool,
    multiline: bool,
    borderless: bool,
    align_right: bool,
    accent: Color,
    dark: bool,
    focused: bool,
    scroll: f32,
    pressing: bool,
    hovered: bool,
    press: Option<(f32, f32)>,
    drag: Option<(f32, f32)>,
    press_time: Option<Instant>,
    undo: Vec<UndoState>,
    redo: Vec<UndoState>,
    lines_cache: LinesCache,
    on_change: Option<Box<dyn FnMut(&str)>>,
    layout: Option<CTFrame>,
    layout_text: String,
    layout_color: Color,
    layout_size: f32,
    layout_wrap: Option<f32>,
    layout_scale: f32,
    dirty: bool,
}

impl FieldCore {
    pub(crate) fn new(placeholder: String) -> Self {
        Self {
            text: String::new(),
            caret: 0,
            anchor: 0,
            sel: false,
            placeholder,
            selected: false,
            masked: false,
            multiline: false,
            borderless: false,
            align_right: false,
            accent: TEXTFIELD_ACCENT,
            dark: true,
            focused: true,
            scroll: 0.0,
            pressing: false,
            hovered: false,
            press: None,
            drag: None,
            press_time: None,
            undo: Vec::new(),
            redo: Vec::new(),
            lines_cache: LinesCache::default(),
            on_change: None,
            layout: None,
            layout_text: String::new(),
            layout_color: Color::WHITE,
            layout_size: 0.0,
            layout_wrap: None,
            layout_scale: 0.0,
            dirty: true,
        }
    }

    /// Visible selection as a sorted byte range, or `None` when
    /// collapsed or hidden.
    pub(crate) fn selection_range(&self) -> Option<(usize, usize)> {
        if self.sel && self.anchor != self.caret {
            let a = self.anchor.min(self.text.len());
            let b = self.caret.min(self.text.len());
            Some((a.min(b), a.max(b)))
        } else {
            None
        }
    }

    /// Currently highlighted text (empty when nothing is selected).
    pub(crate) fn selected_text(&self) -> String {
        match self.selection_range() {
            Some((a, b)) => self.text[a..b].to_string(),
            None => String::new(),
        }
    }

    fn snapshot(&self) -> UndoState {
        UndoState {
            text: self.text.clone(),
            caret: self.caret,
            anchor: self.anchor,
            sel: self.sel,
        }
    }

    fn push_undo(&mut self) {
        let state = self.snapshot();
        if self
            .undo
            .last()
            .is_none_or(|top| top.text != state.text || top.caret != state.caret)
        {
            self.undo.push(state);
            if self.undo.len() > TEXTFIELD_UNDO_LIMIT {
                self.undo.remove(0);
            }
        }
        self.redo.clear();
    }

    fn restore(&mut self, state: UndoState) {
        self.text = state.text;
        self.caret = state.caret.min(self.text.len());
        self.anchor = state.anchor.min(self.text.len());
        self.sel = state.sel && self.anchor != self.caret;
        self.scroll = 0.0;
        self.dirty = true;
        self.notify();
    }

    /// Undo the last mutation. Returns false when the stack is empty.
    pub(crate) fn undo(&mut self) -> bool {
        let Some(top) = self.undo.pop() else {
            return false;
        };
        self.redo.push(self.snapshot());
        self.restore(top);
        true
    }

    /// Redo the last undone mutation. Returns false when empty.
    pub(crate) fn redo(&mut self) -> bool {
        let Some(top) = self.redo.pop() else {
            return false;
        };
        self.undo.push(self.snapshot());
        self.restore(top);
        true
    }

    /// Highlight everything.
    pub(crate) fn select_all(&mut self) {
        self.anchor = 0;
        self.caret = self.text.len();
        self.sel = !self.text.is_empty();
    }

    /// Collapse the selection, keeping the caret where it is.
    pub(crate) fn collapse(&mut self) {
        self.anchor = self.caret;
        self.sel = false;
    }

    /// Delete the highlighted range. Returns false when nothing is
    /// selected. Fires `on_change` on delete.
    pub(crate) fn delete_selection(&mut self) -> bool {
        let Some((a, b)) = self.selection_range() else {
            return false;
        };
        self.push_undo();
        self.text.replace_range(a..b, "");
        self.caret = a;
        self.anchor = a;
        self.sel = false;
        self.dirty = true;
        self.notify();
        true
    }

    /// Insert text at the caret, replacing the highlight first (one
    /// undo step). Single line skips control chars and newlines;
    /// multiline keeps `\n`. Fires `on_change`.
    pub(crate) fn insert(&mut self, content: &str) {
        let clean: String = content
            .chars()
            .filter(|c| !c.is_control() || (self.multiline && *c == '\n'))
            .collect();
        if clean.is_empty() {
            return;
        }
        self.push_undo();
        if let Some((a, b)) = self.selection_range() {
            self.text.replace_range(a..b, "");
            self.caret = a;
        }
        self.caret = self.caret.min(self.text.len());
        self.text.insert_str(self.caret, &clean);
        self.caret += clean.len();
        self.anchor = self.caret;
        self.sel = false;
        self.dirty = true;
        self.notify();
    }

    /// Delete the highlight, else the char before the caret (UTF-8
    /// safe). Fires `on_change` when something vanished.
    pub(crate) fn backspace(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.caret == 0 {
            return;
        }
        self.push_undo();
        let prev = self.text[..self.caret]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.text.remove(prev);
        self.caret = prev;
        self.anchor = prev;
        self.dirty = true;
        self.notify();
    }

    fn step_left(&mut self) {
        if self.caret > 0 {
            self.caret = self.text[..self.caret]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    fn step_right(&mut self) {
        if self.caret < self.text.len() {
            let rest = &self.text[self.caret..];
            let next = rest
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.caret + i)
                .unwrap_or(self.text.len());
            self.caret = next;
        }
    }

    pub(crate) fn move_left(&mut self, extend: bool) {
        if !extend {
            if let Some((a, _)) = self.selection_range() {
                self.caret = a;
                self.collapse();
                return;
            }
            self.step_left();
            self.collapse();
            return;
        }
        if !self.sel {
            self.anchor = self.caret;
        }
        self.step_left();
        self.sel = self.anchor != self.caret;
    }

    pub(crate) fn move_right(&mut self, extend: bool) {
        if !extend {
            if let Some((_, b)) = self.selection_range() {
                self.caret = b;
                self.collapse();
                return;
            }
            self.step_right();
            self.collapse();
            return;
        }
        if !self.sel {
            self.anchor = self.caret;
        }
        self.step_right();
        self.sel = self.anchor != self.caret;
    }

    /// Copy the highlight to the clipboard. Returns false when
    /// nothing is selected.
    pub(crate) fn copy(&mut self) -> bool {
        let selected = self.selected_text();
        if selected.is_empty() {
            return false;
        }
        clipboard::set(&selected);
        true
    }

    /// Cut the highlight to the clipboard. Returns false when
    /// nothing is selected. Fires `on_change` on cut.
    pub(crate) fn cut(&mut self) -> bool {
        if !self.copy() {
            return false;
        }
        self.delete_selection()
    }

    /// Paste clipboard text at the caret (replaces the highlight).
    /// Fires `on_change` on paste.
    pub(crate) fn paste(&mut self) {
        if let Some(text) = clipboard::get() {
            self.insert(&text);
        }
    }

    /// Shared single-line key handling: Backspace, caret motion and
    /// the Ctrl shortcuts (select all, clipboard, undo/redo).
    /// Returns true when consumed. Multiline editors route Up/Down
    /// and Enter through their own arms first.
    pub(crate) fn handle_key(&mut self, key: Key) -> bool {
        if !self.selected {
            return false;
        }
        match key {
            Key::Backspace => self.backspace(),
            Key::Left => self.move_left(false),
            Key::Right => self.move_right(false),
            Key::SelectLeft => self.move_left(true),
            Key::SelectRight => self.move_right(true),
            Key::SelectAll => self.select_all(),
            Key::Copy => {
                self.copy();
            }
            Key::Cut => {
                self.cut();
            }
            Key::Paste => self.paste(),
            Key::Undo => {
                self.undo();
            }
            Key::Redo => {
                self.redo();
            }
            Key::Escape => self.deselect(),
            _ => return false,
        }
        true
    }

    /// Record a press inside the field (field coords). Caret mapping
    /// needs fonts, so `draw` finishes it via `take_press` plus a
    /// variant caret lookup and `finish_press`.
    pub(crate) fn press(&mut self, x: f32, y: f32) {
        self.selected = true;
        self.pressing = true;
        self.press = Some((x, y));
        self.drag = None;
    }

    /// Extend the highlight toward a drag point while held (resolved
    /// in `draw` via `take_drag` plus `finish_drag`).
    pub(crate) fn drag_to_point(&mut self, x: f32, y: f32) {
        if self.pressing && self.selected {
            self.drag = Some((x, y));
        }
    }

    /// Release a hold (mouse up).
    pub(crate) fn release(&mut self) {
        self.pressing = false;
        self.press = None;
        self.drag = None;
    }

    pub(crate) fn take_press(&mut self) -> Option<(f32, f32)> {
        self.press.take()
    }

    pub(crate) fn take_drag(&mut self) -> Option<(f32, f32)> {
        self.drag.take()
    }

    /// Finish a press at a resolved caret: double-click highlights
    /// the word, single click collapses there.
    pub(crate) fn finish_press(&mut self, caret: usize, now: Instant) {
        let caret = caret.min(self.text.len());
        let double = matches!(self.press_time, Some(t)
            if now.duration_since(t).as_secs_f64() < GESTURE_DOUBLE_TAP_SECONDS);
        if double {
            let (a, b) = word_range(&self.text, caret);
            self.press_time = None;
            if a != b {
                self.anchor = a;
                self.caret = b;
                self.sel = true;
                return;
            }
        } else {
            self.press_time = Some(now);
        }
        self.caret = caret;
        self.anchor = caret;
        self.sel = false;
    }

    /// Finish a drag at a resolved caret: the anchor stays where the
    /// press landed, the caret follows the pointer.
    pub(crate) fn finish_drag(&mut self, caret: usize) {
        self.caret = caret.min(self.text.len());
        self.sel = self.anchor != self.caret;
    }

    pub(crate) fn deselect(&mut self) {
        self.selected = false;
        self.sel = false;
        self.release();
    }

    /// Parley line ranges for the current text, cached across
    /// frames: rebuilt only when text, size, wrap or scale changed.
    /// Glyph color never affects breaks, so it is not part of the
    /// key. Returns an owned copy (a few dozen triples).
    pub(crate) fn cached_editor_lines(
        &mut self,
        fonts: &mut FontSystem,
        size: f32,
        color: Color,
        wrap: f32,
    ) -> Vec<(usize, usize, f32)> {
        let cache = &self.lines_cache;
        if cache.valid
            && cache.text == self.text
            && cache.size == size
            && cache.wrap == wrap
            && cache.scale == fonts.scale
        {
            return cache.lines.clone();
        }
        let lines = editor_lines(fonts, &self.text, size, color, wrap);
        self.lines_cache = LinesCache {
            text: self.text.clone(),
            size,
            wrap,
            scale: fonts.scale,
            lines: lines.clone(),
            valid: true,
        };
        lines
    }

    pub(crate) fn set_text(&mut self, text: String) {
        if text != self.text {
            self.text = text;
            self.caret = self.text.len();
            self.anchor = self.caret;
            self.sel = false;
            self.scroll = 0.0;
            self.undo.clear();
            self.redo.clear();
            self.dirty = true;
        }
    }

    fn notify(&mut self) {
        if let Some(callback) = self.on_change.as_mut() {
            callback(&self.text.clone());
        }
    }

    /// Right shift in logical px for right-aligned text: pushes
    /// short content to the box end, zero once it scrolls. Uses the
    /// displayed content (placeholder when empty).
    pub(crate) fn align_shift(
        &self,
        fonts: &mut FontSystem,
        size: f32,
        color: Color,
        avail_w: f32,
    ) -> f32 {
        if !self.align_right {
            return 0.0;
        }
        let content = if self.text.is_empty() {
            self.placeholder.clone()
        } else {
            self.echo()
        };
        let layout = fonts.layout_text(&content, size, color, None);
        let (tw, _) = FontSystem::layout_size(&layout);
        (avail_w - tw / fonts.scale).max(0.0)
    }

    /// Displayed content: bullets per char when masked, the raw
    /// text otherwise.
    pub(crate) fn echo(&self) -> String {
        if self.masked {
            "•".repeat(self.text.chars().count())
        } else {
            self.text.clone()
        }
    }

    /// Laid-out content: the echo, or the placeholder when empty.
    /// Returns the layout plus whether it shows the placeholder.
    pub(crate) fn ensure_layout(
        &mut self,
        fonts: &mut FontSystem,
        size: f32,
        text_color: Color,
        placeholder_color: Color,
        wrap: Option<f32>,
    ) -> (&CTFrame, bool) {
        let empty = self.text.is_empty();
        let content = if empty {
            self.placeholder.clone()
        } else {
            self.echo()
        };
        let color = if empty { placeholder_color } else { text_color };
        if !self.dirty
            && self.layout.is_some()
            && self.layout_text == content
            && self.layout_color == color
            && self.layout_size == size
            && self.layout_wrap == wrap
            && self.layout_scale == fonts.scale
        {
            return (self.layout.as_ref().expect("layout built"), empty);
        }
        let layout = fonts.layout_text(&content, size, color, wrap);
        self.layout = Some(layout);
        self.layout_text = content;
        self.layout_color = color;
        self.layout_size = size;
        self.layout_wrap = wrap;
        self.layout_scale = fonts.scale;
        self.dirty = false;
        (self.layout.as_ref().expect("layout built"), empty)
    }

    /// Caret x in logical px (advance of the echo before the caret).
    pub(crate) fn caret_x(&self, fonts: &mut FontSystem, size: f32, color: Color) -> f32 {
        self.echo_advance(fonts, self.caret.min(self.text.len()), size, color)
    }

    /// Advance of the echo before byte `idx` in logical px
    /// (bullets for masked fields).
    pub(crate) fn echo_advance(
        &self,
        fonts: &mut FontSystem,
        idx: usize,
        size: f32,
        color: Color,
    ) -> f32 {
        let idx = idx.min(self.text.len());
        let prefix_chars = self.text[..idx].chars().count();
        let prefix = if self.masked {
            "•".repeat(prefix_chars)
        } else {
            self.text[..idx].to_string()
        };
        let layout = fonts.layout_text(&prefix, size, color, None);
        FontSystem::layout_size(&layout).0 / fonts.scale
    }

    /// Keep the caret visible inside `visible_w`: scrolls right when
    /// typing past the edge, back when moving left.
    pub(crate) fn track_caret(&mut self, caret_x: f32, visible_w: f32) {
        if caret_x - self.scroll > visible_w - 2.0 {
            self.scroll = (caret_x - visible_w + 2.0).max(0.0);
        }
        if caret_x - self.scroll < 0.0 {
            self.scroll = caret_x.max(0.0);
        }
    }
}

/// Per-variant metrics: font size, padding, radius and minimum
/// intrinsic width in logical px.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FieldMetrics {
    pub(crate) font_size: f32,
    pub(crate) pad_x: f32,
    pub(crate) pad_y: f32,
    pub(crate) radius: f32,
    pub(crate) min_width: f32,
}

/// (fill, border, text) for a core's mode and focus.
pub(crate) fn field_colors(core: &FieldCore) -> (Color, Color, Color) {
    let (fill, border, text) = if core.dark {
        (
            TEXTFIELD_BG_DARK,
            TEXTFIELD_BORDER_DARK,
            Color::from_rgb8(0xd8, 0xd9, 0xd9),
        )
    } else {
        (
            TEXTFIELD_BG_LIGHT,
            TEXTFIELD_BORDER_LIGHT,
            Color::from_rgb8(0x27, 0x27, 0x27),
        )
    };
    if core.focused {
        (fill, border, text)
    } else {
        (
            desaturate(fill),
            desaturate(border),
            desaturate(text),
        )
    }
}

/// Placeholder gray for a core's mode.
pub(crate) fn placeholder_color(core: &FieldCore) -> Color {
    if core.dark {
        TEXTFIELD_PLACEHOLDER_DARK
    } else {
        TEXTFIELD_PLACEHOLDER_LIGHT
    }
}

/// Accent caret/ring color for a core's focus.
pub(crate) fn accent_color(core: &FieldCore) -> Color {
    if core.focused {
        core.accent
    } else {
        desaturate(core.accent)
    }
}

/// Caret blink phase (macOS-like ~1 s period, on half the time).
pub(crate) fn caret_blink() -> bool {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() % (TEXTFIELD_BLINK_SECONDS * 1000.0) as u128)
        .map(|ms| ms < (TEXTFIELD_BLINK_SECONDS * 500.0) as u128)
        .unwrap_or(true)
}

/// Line (start, end, height) ranges of `text` wrapped at `wrap`.
/// Physical heights; empty text yields one empty line.
pub(crate) fn editor_lines(
    fonts: &mut FontSystem,
    text: &str,
    size: f32,
    color: Color,
    wrap: f32,
) -> Vec<(usize, usize, f32)> {
    if text.is_empty() {
        return vec![(0, 0, size * 1.25)];
    }
    let frame = fonts.layout_text(text, size, color, Some(wrap.max(0.0)));
    let scale = fonts.scale;
    let mut out = Vec::new();
    for line in frame.lines() {
        out.push((line.range.start, line.range.end, line.height * scale));
    }
    if out.is_empty() {
        out.push((0, 0, size * 1.25));
    }
    out
}

/// Advance of `text` in logical px (single line probe).
pub(crate) fn advance_of(fonts: &mut FontSystem, text: &str, size: f32, color: Color) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let probe = fonts.layout_text(text, size, color, None);
    FontSystem::layout_size(&probe).0 / fonts.scale
}

/// Word boundaries around a byte caret: alphanumeric plus `_`
/// counts as word chars. Collapsed when the caret sits between
/// words (double-click on a gap just places the caret).
pub(crate) fn word_range(text: &str, caret: usize) -> (usize, usize) {
    let caret = caret.min(text.len());
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let mut a = caret;
    while a > 0 {
        match text[..a].chars().next_back() {
            Some(c) if is_word(c) => a -= c.len_utf8(),
            _ => break,
        }
    }
    let mut b = caret;
    while b < text.len() {
        match text[b..].chars().next() {
            Some(c) if is_word(c) => b += c.len_utf8(),
            _ => break,
        }
    }
    (a, b)
}

/// Byte index at `goal_x` (text-origin coords) in single-line
/// `echo`: nearest advance boundary (ties to the earlier one),
/// clamped to both ends. CoreText binary-searches over grapheme
/// boundaries, so huge lines need O(log n) layouts, not O(n).
pub(crate) fn single_line_caret(
    fonts: &mut FontSystem,
    echo: &str,
    size: f32,
    color: Color,
    goal_x: f32,
) -> usize {
    point_to_caret(fonts.framesetter(), echo, size, color, goal_x)
}

/// Resolve pending press/drag points of a single-line field.
/// `origin_x` is the text-origin x (field x plus padding minus
/// scroll); `echo` is the displayed content (bullets when masked).
/// Drag extends from the press anchor; press collapses or
/// word-selects on double-click.
pub(crate) fn resolve_press_single(
    core: &mut FieldCore,
    fonts: &mut FontSystem,
    echo: &str,
    size: f32,
    color: Color,
    origin_x: f32,
) {
    if let Some((px, _)) = core.take_press() {
        let caret = single_line_caret(fonts, echo, size, color, px - origin_x);
        core.finish_press(caret, Instant::now());
    }
    if let Some((dx, _)) = core.take_drag() {
        let caret = single_line_caret(fonts, echo, size, color, dx - origin_x);
        core.finish_drag(caret);
    }
}

/// Accent selection wash for highlighted text.
pub(crate) fn selection_brush(core: &FieldCore) -> Color {
    let c = accent_color(core).to_rgba8();
    Color::from_rgba8(c.r, c.g, c.b, (c.a as f32 * TEXTFIELD_SELECTION_ALPHA).round() as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_range_covers_word_chars_only() {
        assert_eq!(word_range("hello world", 3), (0, 5));
        assert_eq!(word_range("hello world", 5), (0, 5));
        assert_eq!(word_range("hello world", 6), (6, 11));
        assert_eq!(word_range("foo_bar baz", 4), (0, 7));
        assert_eq!(word_range("hi", 99), (0, 2));
        assert_eq!(word_range("", 0), (0, 0));
    }

    #[test]
    fn single_line_caret_maps_ends_and_middle() {
        let mut fonts = FontSystem::new();
        let color = Color::WHITE;
        let echo = "hello";
        assert_eq!(single_line_caret(&mut fonts, echo, 13.0, color, -10.0), 0);
        assert_eq!(single_line_caret(&mut fonts, echo, 13.0, color, 10000.0), 5);
        let hel_x = advance_of(&mut fonts, "hel", 13.0, color);
        let middle = single_line_caret(&mut fonts, echo, 13.0, color, hel_x);
        assert_eq!(middle, 3);
        assert_eq!(single_line_caret(&mut fonts, "", 13.0, color, 50.0), 0);
    }
}

/// Caret line index plus goal x in logical px. A caret exactly on a
/// line start belongs to that line, so carets after an empty line
/// land on the next line instead of inside the gap; otherwise the
/// first line spanning the caret wins. The x advance comes from a
/// single layout of the line prefix (not per-char probes), so long
/// lines stay cheap.
pub(crate) fn editor_caret_pos(
    fonts: &mut FontSystem,
    text: &str,
    caret: usize,
    size: f32,
    color: Color,
    lines: &[(usize, usize, f32)],
) -> (usize, f32) {
    if lines.is_empty() {
        return (0, 0.0);
    }
    let caret = caret.min(text.len());
    let mut line = lines.len().saturating_sub(1);
    let mut found = false;
    for (index, (start, _, _)) in lines.iter().enumerate() {
        if caret == *start {
            line = index;
            found = true;
            break;
        }
    }
    if !found {
        for (index, (start, end, _)) in lines.iter().enumerate() {
            if caret >= *start && caret <= *end {
                line = index;
                break;
            }
        }
    }
    let (start, end, _) = lines[line];
    let end = end.min(text.len());
    let start = start.min(end);
    let at = caret.min(end).max(start);
    let x = match text.get(start..at) {
        Some(slice) => advance_of(fonts, slice, size, color),
        None => 0.0,
    };
    (line, x)
}

/// Byte index in `line` at column `goal_x` (nearest advance, ties
/// to the earlier boundary, like the old half-advance walk). A
/// trailing newline belongs to the break, not the walk. Past the
/// last line end lands at the very end. Binary searches over char
/// boundaries, so long lines need O(log n) layouts, not O(n).
pub(crate) fn editor_column(
    fonts: &mut FontSystem,
    text: &str,
    line: usize,
    goal_x: f32,
    size: f32,
    color: Color,
    lines: &[(usize, usize, f32)],
) -> usize {
    if lines.is_empty() {
        return text.len();
    }
    let line = line.min(lines.len().saturating_sub(1));
    let (start, end, _) = lines[line];
    let end = end.min(text.len());
    let start = start.min(end);
    let walk_end = match text.get(start..end) {
        Some(slice) if slice.ends_with('\n') => end.saturating_sub(1).max(start),
        _ => end,
    };
    // Char boundaries in the walk range (no layouts here).
    let mut bounds = vec![start];
    let mut at = start;
    while at < walk_end {
        match text[at..].chars().next() {
            Some(c) => {
                at = (at + c.len_utf8()).min(walk_end);
                bounds.push(at);
                if at >= walk_end {
                    break;
                }
            }
            None => break,
        }
    }
    let goal = goal_x.max(0.0);
    let advance_at = |k: usize, fonts: &mut FontSystem| -> f32 {
        match text.get(start..bounds[k]) {
            Some(slice) => advance_of(fonts, slice, size, color),
            None => 0.0,
        }
    };
    let mut lo = 0usize;
    let mut hi = bounds.len().saturating_sub(1);
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        if advance_at(mid, fonts) <= goal {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    let mut at = bounds[lo];
    if lo + 1 < bounds.len() {
        let a0 = advance_at(lo, fonts);
        let a1 = advance_at(lo + 1, fonts);
        if (a0 + a1) / 2.0 < goal {
            at = bounds[lo + 1];
        }
    }
    if line == lines.len().saturating_sub(1) {
        let ax = match text.get(start..at) {
            Some(slice) => advance_of(fonts, slice, size, color),
            None => 0.0,
        };
        if goal >= ax {
            return text.len();
        }
    }
    at
}

/// Caret (x, y, height) in logical px relative to the text origin.
pub(crate) fn editor_caret_geometry(
    fonts: &mut FontSystem,
    text: &str,
    caret: usize,
    size: f32,
    color: Color,
    lines: &[(usize, usize, f32)],
) -> (f32, f32, f32) {
    let (line, x) = editor_caret_pos(fonts, text, caret, size, color, lines);
    let mut y = 0.0;
    for (_, _, height) in lines.iter().take(line) {
        y += *height / fonts.scale;
    }
    let line_h = lines
        .get(line)
        .map(|(_, _, h)| *h)
        .unwrap_or(size * 1.25)
        / fonts.scale;
    (x, y, line_h)
}

/// Per-editor metrics for the shared multiline draw.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EditorMetrics {
    pub(crate) font_size: f32,
    pub(crate) pad: f32,
    pub(crate) radius: f32,
}

/// Multiline draw shared by both editors: fill, accent ring while
/// selected (subtle border otherwise), wrapped text clipped to the
/// padded box with vertical caret tracking and a blinking caret.
/// Returns the new vertical scroll.
pub(crate) fn draw_editor_multiline(
    scene: &mut Scene,
    fonts: &mut FontSystem,
    core: &mut FieldCore,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    metrics: &EditorMetrics,
    bg: Color,
    border: Color,
    scroll_y: f32,
) -> f32 {
    if w <= 0.0 || h <= 0.0 {
        return scroll_y;
    }
    let scale = fonts.scale as f64;
    let px = |v: f32| v as f64 * scale;
    let (fill, border_line, text) = (bg, border, {
        let (_, _, t) = field_colors(core);
        t
    });
    let body = RoundedRect::new(px(x), px(y), px(x + w), px(y + h), px(metrics.radius));
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(fill),
        None,
        &body,
    );
    if core.selected {
        scene.stroke(
            &Stroke::new(px(TEXTFIELD_RING_W)),
            Affine::IDENTITY,
            &Brush::Solid(accent_color(core)),
            None,
            &body,
        );
    } else {
        scene.stroke(
            &Stroke::new(1.0 * scale),
            Affine::IDENTITY,
            &Brush::Solid(border_line),
            None,
            &body,
        );
    }
    let ix = x + metrics.pad;
    let iy = y + metrics.pad;
    let iw = (w - metrics.pad * 2.0).max(0.0);
    let ih = (h - metrics.pad * 2.0).max(0.0);
    let clip = RoundedRect::new(px(ix), px(iy), px(ix + iw), px(iy + ih), px(4.0));
    scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &clip);
    let placeholder = placeholder_color(core);
    // Caret geometry first (needs fonts but not the layout borrow),
    // then track, then paint text and caret at the new scroll.
    let content = core.text.clone();
    let caret = core.caret;
    let lines = core.cached_editor_lines(fonts, metrics.font_size, text, iw.max(0.0));
    let (cx, cy, ch) = editor_caret_geometry(
        fonts,
        &content,
        caret,
        metrics.font_size,
        text,
        &lines,
    );
    let mut scroll = scroll_y;
    if cy - scroll + ch > ih {
        scroll = (cy + ch - ih).max(0.0);
    }
    if cy - scroll < 0.0 {
        scroll = cy.max(0.0);
    }
    {
        let (layout, _) = core.ensure_layout(
            fonts,
            metrics.font_size,
            text,
            placeholder,
            Some(iw.max(0.0)),
        );
        draw_layout(scene, layout, ix, iy - scroll, fonts.scale);
    }
    // Highlighted range wash, segmented per wrapped line.
    if let Some((sa, sb)) = core.selection_range() {
        let wash = selection_brush(core);
        let mut ly = 0.0;
        for (start, end, height) in &lines {
            let h = *height / fonts.scale;
            let end = (*end).min(content.len());
            let start = (*start).min(end);
            let lo = sa.max(start).min(end);
            let hi = sb.max(start).min(end);
            if lo < hi {
                if let (Some(before), Some(within)) =
                    (content.get(start..lo), content.get(start..hi))
                {
                    let x0 = advance_of(fonts, before, metrics.font_size, text);
                    let x1 = advance_of(fonts, within, metrics.font_size, text);
                    scene.fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        &Brush::Solid(wash),
                        None,
                        &Rect::new(
                            px(ix + x0),
                            px(iy + ly - scroll),
                            px(ix + x1),
                            px(iy + ly - scroll + h),
                        ),
                    );
                }
            }
            ly += h;
        }
    }
    if core.selected && caret_blink() {
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(accent_color(core)),
            None,
            &Rect::new(
                px(ix + cx),
                px(iy + cy - scroll),
                px(ix + cx + TEXTFIELD_CARET_W),
                px(iy + cy - scroll + ch),
            ),
        );
    }
    scene.pop_layer();
    scroll
}

/// Intrinsic size: text height plus vertical padding.
pub(crate) fn measure_field(
    fonts: &mut FontSystem,
    core: &mut FieldCore,
    metrics: &FieldMetrics,
) -> (f32, f32) {
    let (_, _, text) = field_colors(core);
    let (layout, _) =
        core.ensure_layout(fonts, metrics.font_size, text, text, None);
    let (_, th) = FontSystem::layout_size(layout);
    (
        metrics.min_width + metrics.pad_x * 2.0,
        th / fonts.scale + metrics.pad_y * 2.0,
    )
}

/// Full field draw shared by both variants: fill, accent ring while
/// selected (subtle border otherwise), clipped text with caret
/// tracking and a blinking caret.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_field(
    scene: &mut Scene,
    fonts: &mut FontSystem,
    core: &mut FieldCore,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    metrics: &FieldMetrics,
) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let scale = fonts.scale as f64;
    let px = |v: f32| v as f64 * scale;
    let (fill, border, text) = field_colors(core);
    let body = RoundedRect::new(px(x), px(y), px(x + w), px(y + h), px(metrics.radius));
    // Borderless fields (form rows) paint text only, no fill, ring
    // or border.
    if !core.borderless {
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(fill),
            None,
            &body,
        );
        // Accent ring while selected, subtle border otherwise.
        if core.selected {
            scene.stroke(
                &Stroke::new(px(TEXTFIELD_RING_W)),
                Affine::IDENTITY,
                &Brush::Solid(accent_color(core)),
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
    }
    // Text clipped to the padded box with caret tracking.
    let ix = x + metrics.pad_x;
    let iy = y + metrics.pad_y;
    let iw = (w - metrics.pad_x * 2.0).max(0.0);
    let ih = (h - metrics.pad_y * 2.0).max(0.0);
    let clip = RoundedRect::new(px(ix), px(iy), px(ix + iw), px(iy + ih), px(4.0));
    scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &clip);
    let placeholder = placeholder_color(core);
    let th = {
        let (layout, _) =
            core.ensure_layout(fonts, metrics.font_size, text, placeholder, None);
        FontSystem::layout_size(layout).1
    };
    let ty = iy + (ih - th / fonts.scale) / 2.0;
    let caret_x = core.caret_x(fonts, metrics.font_size, text);
    core.track_caret(caret_x, iw);
    let scroll = core.scroll;
    let shift = core.align_shift(fonts, metrics.font_size, text, iw);
    {
        let (layout, _) =
            core.ensure_layout(fonts, metrics.font_size, text, placeholder, None);
        draw_layout(scene, layout, ix - scroll + shift, ty, fonts.scale);
    }
    // Highlighted range wash under the text.
    if let Some((a, b)) = core.selection_range() {
        let x0 = ix - scroll + shift + core.echo_advance(fonts, a, metrics.font_size, text);
        let x1 = ix - scroll + shift + core.echo_advance(fonts, b, metrics.font_size, text);
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(selection_brush(core)),
            None,
            &Rect::new(px(x0), px(ty), px(x1), px(ty + th / fonts.scale)),
        );
    }
    // Blinking caret while selected.
    if core.selected && caret_blink() {
        let cx = ix - scroll + shift + caret_x;
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(accent_color(core)),
            None,
            &Rect::new(
                px(cx),
                px(ty),
                px(cx + TEXTFIELD_CARET_W),
                px(ty + th / fonts.scale),
            ),
        );
    }
    scene.pop_layer();
}
