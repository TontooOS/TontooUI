pub mod basic;
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

use std::time::{SystemTime, UNIX_EPOCH};

use parley::Layout;
use vello::Scene;
use vello::kurbo::{Affine, Rect, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use crate::renderer::text::{FontSystem, SolidBrush, draw_layout};
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

/// Shared single-line editing core behind the field variants:
/// text plus caret (always a char boundary), placeholder, selection
/// and accent. `masked` echoes bullets (secure fields), `multiline`
/// keeps newlines (editor). Layouts cache per content, color, size,
/// wrap and scale per the crisp text rules.
pub(crate) struct FieldCore {
    text: String,
    caret: usize,
    placeholder: String,
    selected: bool,
    masked: bool,
    multiline: bool,
    accent: Color,
    dark: bool,
    focused: bool,
    scroll: f32,
    on_change: Option<Box<dyn FnMut(&str)>>,
    layout: Option<Layout<SolidBrush>>,
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
            placeholder,
            selected: false,
            masked: false,
            multiline: false,
            accent: TEXTFIELD_ACCENT,
            dark: true,
            focused: true,
            scroll: 0.0,
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

    /// Insert text at the caret. Single line skips control chars and
    /// newlines; multiline keeps `\n`. Fires `on_change`.
    pub(crate) fn insert(&mut self, content: &str) {
        let clean: String = content
            .chars()
            .filter(|c| !c.is_control() || (self.multiline && *c == '\n'))
            .collect();
        if clean.is_empty() {
            return;
        }
        self.text.insert_str(self.caret, &clean);
        self.caret += clean.len();
        self.dirty = true;
        self.notify();
    }

    /// Delete the char before the caret (UTF-8 safe). Fires
    /// `on_change` when something vanished.
    pub(crate) fn backspace(&mut self) {
        if self.caret == 0 {
            return;
        }
        let prev = self.text[..self.caret]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.text.remove(prev);
        self.caret = prev;
        self.dirty = true;
        self.notify();
    }

    pub(crate) fn move_left(&mut self) {
        if self.caret > 0 {
            self.caret = self.text[..self.caret]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    pub(crate) fn move_right(&mut self) {
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

    pub(crate) fn select(&mut self) {
        self.selected = true;
        self.caret = self.text.len();
    }

    pub(crate) fn deselect(&mut self) {
        self.selected = false;
    }

    pub(crate) fn set_text(&mut self, text: String) {
        if text != self.text {
            self.text = text;
            self.caret = self.text.len();
            self.scroll = 0.0;
            self.dirty = true;
        }
    }

    fn notify(&mut self) {
        if let Some(callback) = self.on_change.as_mut() {
            callback(&self.text.clone());
        }
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
    ) -> (&Layout<SolidBrush>, bool) {
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
        let prefix_chars = self.text[..self.caret.min(self.text.len())].chars().count();
        let prefix = if self.masked {
            "•".repeat(prefix_chars)
        } else {
            self.text[..self.caret.min(self.text.len())].to_string()
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
    let probe = fonts.layout_text(text, size, color, Some(wrap.max(0.0)));
    let mut out = Vec::new();
    for index in 0..probe.len() {
        if let Some(line) = probe.get(index) {
            let range = line.text_range();
            out.push((range.start, range.end, line.metrics().line_height));
        }
    }
    if out.is_empty() {
        out.push((0, 0, size * 1.25));
    }
    out
}

/// Advance of `text` in logical px (single line probe).
fn advance_of(fonts: &mut FontSystem, text: &str, size: f32, color: Color) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let probe = fonts.layout_text(text, size, color, None);
    FontSystem::layout_size(&probe).0 / fonts.scale
}

/// Caret line index plus goal x in logical px. The caret belongs to
/// the first line spanning it; a caret exactly at a line end before
/// `\n` stays on its line (documented v1 behavior).
pub(crate) fn editor_caret_pos(
    fonts: &mut FontSystem,
    text: &str,
    caret: usize,
    size: f32,
    color: Color,
    lines: &[(usize, usize, f32)],
) -> (usize, f32) {
    let caret = caret.min(text.len());
    let mut line = lines.len().saturating_sub(1);
    for (index, (start, end, _)) in lines.iter().enumerate() {
        if caret >= *start && caret <= *end {
            line = index;
            break;
        }
    }
    let (start, end, _) = lines[line];
    let end = end.min(text.len());
    let mut x = 0.0;
    let mut at = start.min(end);
    while at < caret.min(end) {
        let next = text[at..]
            .char_indices()
            .nth(1)
            .map(|(i, _)| at + i)
            .unwrap_or(end);
        x += advance_of(fonts, &text[at..next.min(end)], size, color);
        at = next.min(end);
        if at >= end {
            break;
        }
    }
    (line, x)
}

/// Byte index in `line` at column `goal_x` (nearest advance). A
/// trailing newline belongs to the break, not the walk. Past the
/// last line end lands at the very end.
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
    let walk_end = if text[start.min(end)..end].ends_with('\n') {
        end.saturating_sub(1).max(start.min(end))
    } else {
        end
    };
    let mut x = 0.0;
    let mut at = start.min(walk_end);
    while at < walk_end {
        let next = text[at..]
            .char_indices()
            .nth(1)
            .map(|(i, _)| at + i)
            .unwrap_or(walk_end);
        let adv = advance_of(fonts, &text[at..next.min(walk_end)], size, color);
        if x + adv / 2.0 >= goal_x {
            break;
        }
        x += adv;
        at = next.min(walk_end);
    }
    if line == lines.len().saturating_sub(1) && goal_x >= x {
        return text.len();
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
    let lines = editor_lines(fonts, &content, metrics.font_size, text, iw.max(0.0));
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
    {
        let (layout, _) =
            core.ensure_layout(fonts, metrics.font_size, text, placeholder, None);
        draw_layout(scene, layout, ix - scroll, ty, fonts.scale);
    }
    // Blinking caret while selected.
    if core.selected && caret_blink() {
        let cx = ix - scroll + caret_x;
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
