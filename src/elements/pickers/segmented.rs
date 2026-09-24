use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, Line, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::layout::View;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::desaturate;

/// Segmented control height in logical px (macOS segmented measure).
pub const SEGMENTED_HEIGHT: f32 = 16.0;
/// Outer track corner radius in logical px.
pub const SEGMENTED_RADIUS: f32 = 4.0;
/// Selected pill inset inside the track in logical px.
pub const SEGMENTED_PAD: f32 = 1.0;
/// Selected pill corner radius in logical px.
pub const SEGMENTED_PILL_RADIUS: f32 = 3.25;
/// Segment label size in logical px.
pub const SEGMENTED_FONT_SIZE: f32 = 7.5;
/// Leading label size in logical px (settings-row measure).
pub const SEGMENTED_LABEL_SIZE: f32 = 8.5;
/// Gap between the leading label and the track in logical px.
pub const SEGMENTED_GAP: f32 = 6.0;
/// Horizontal text padding inside a segment in logical px.
pub const SEGMENTED_PAD_X: f32 = 8.0;
/// Minimum segment width in logical px.
pub const SEGMENTED_MIN_SEG_W: f32 = 36.0;
/// Pressed-segment fill for light mode (shown while held, like macOS).
pub const SEGMENTED_PRESSED_LIGHT: Color = Color::from_rgb8(0xd1, 0xd1, 0xd6);
/// Pressed-segment fill for dark mode (shown while held, like macOS).
pub const SEGMENTED_PRESSED_DARK: Color = Color::from_rgb8(0x63, 0x63, 0x66);
/// Track fill for light mode.
pub const SEGMENTED_TRACK_LIGHT: Color = Color::from_rgb8(0xe5, 0xe5, 0xe5);
/// Track fill for dark mode.
pub const SEGMENTED_TRACK_DARK: Color = Color::from_rgb8(0x3a, 0x3a, 0x3c);
/// Default selected fill (theme accent blue).
pub const SEGMENTED_ACCENT: Color = Color::from_rgb8(0x00, 0x7a, 0xff);

/// Segmented picker (SwiftUI `Picker` with `.segmented` style): an
/// optional leading label plus a macOS-style segmented track. The
/// selected segment draws as an accent pill, the rest as plain labels
/// with hairline dividers between unselected neighbors. Like macOS
/// there is no hover state and no selection animation: pressing a
/// segment shows a gray hold highlight and releasing there switches
/// to it instantly.
///
/// Clicks select on release inside the track (`mouse_down` arms,
/// `View::mouse_up` fires); the shell forwards both.
pub struct SegmentedPicker {
    label: String,
    options: Vec<String>,
    selected: usize,
    accent: Color,
    accent_manual: bool,
    dark: bool,
    text_color: Color,
    text_dim: Color,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    track_x: f32,
    track_y: f32,
    track_w: f32,
    seg_w: f32,
    label_x: f32,
    label_y: f32,
    armed: bool,
    pressed: Option<usize>,
    disabled: bool,
    focused: bool,
    on_select: Option<Box<dyn FnMut(usize)>>,
}

impl SegmentedPicker {
    pub fn new(label: impl Into<String>, options: Vec<String>) -> Self {
        let selected = 0;
        Self {
            label: label.into(),
            options,
            selected,
            accent: SEGMENTED_ACCENT,
            accent_manual: false,
            dark: true,
            text_color: Color::WHITE,
            text_dim: Color::from_rgb8(0x9a, 0x9a, 0x9e),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            track_x: 0.0,
            track_y: 0.0,
            track_w: 0.0,
            seg_w: SEGMENTED_MIN_SEG_W,
            label_x: 0.0,
            label_y: 0.0,
            armed: false,
            pressed: None,
            disabled: false,
            focused: true,
            on_select: None,
        }
    }

    /// Convenience constructor from a slice (e.g. `["One", "Two", "Three"]`).
    pub fn from_slice(label: impl Into<String>, options: &[&str]) -> Self {
        Self::new(
            label,
            options.iter().map(|s| s.to_string()).collect(),
        )
    }

    /// Initial selection without firing `on_select`.
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = self.clamp_index(index);
        self
    }

    /// Manual selected-pill color: wins over the system accent until
    /// cleared. The pill follows the system accent unless set by hand.
    pub fn accent(mut self, color: Color) -> Self {
        self.accent = color;
        self.accent_manual = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select(mut self, callback: impl FnMut(usize) + 'static) -> Self {
        self.on_select = Some(Box::new(callback));
        self
    }

    /// Live theme: accent pill, mode track grays and label colors. A
    /// manually set accent wins over the system accent.
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        if !self.accent_manual {
            self.accent = accent;
        }
        self.dark = dark;
        if dark {
            self.text_color = Color::WHITE;
            self.text_dim = Color::from_rgb8(0x9a, 0x9a, 0x9e);
        } else {
            self.text_color = Color::BLACK;
            self.text_dim = Color::from_rgb8(0x6e, 0x6e, 0x72);
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected_label(&self) -> Option<&str> {
        self.options.get(self.selected).map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.options.len()
    }

    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }

    fn clamp_index(&self, index: usize) -> usize {
        if self.options.is_empty() {
            0
        } else {
            index.min(self.options.len() - 1)
        }
    }

    /// Select instantly, like macOS (no slide animation). Fires
    /// `on_select` when the selection changed.
    pub fn select(&mut self, index: usize) {
        let clamped = self.clamp_index(index);
        if clamped != self.selected {
            self.selected = clamped;
            self.notify();
        }
    }

    /// Set the selection instantly. Fires `on_select` when the
    /// selection changed.
    pub fn set_selected(&mut self, index: usize) {
        self.select(index);
    }

    fn notify(&mut self) {
        if let Some(callback) = self.on_select.as_mut() {
            callback(self.selected);
        }
    }

    fn track_color(&self) -> Color {
        if self.dark {
            SEGMENTED_TRACK_DARK
        } else {
            SEGMENTED_TRACK_LIGHT
        }
    }

    fn divider_color(&self) -> Color {
        if self.dark {
            Color::from_rgba8(255, 255, 255, 40)
        } else {
            Color::from_rgba8(0, 0, 0, 36)
        }
    }

    fn eff(&self, color: Color) -> Color {
        let mut color = if self.focused {
            color
        } else {
            desaturate(color)
        };
        if self.disabled {
            let c = color.to_rgba8();
            color = Color::from_rgba8(c.r, c.g, c.b, (c.a as f32 * 0.4).round() as u8);
        }
        color
    }

    fn label_size(&self, fonts: &mut FontSystem) -> (f32, f32) {
        if self.label.is_empty() {
            return (0.0, 0.0);
        }
        let layout = fonts.layout_text(&self.label, SEGMENTED_LABEL_SIZE, Color::WHITE, None);
        let (tw, th) = FontSystem::layout_size(&layout);
        (tw / fonts.scale, th / fonts.scale)
    }

    fn seg_text_w(&self, fonts: &mut FontSystem, text: &str) -> f32 {
        let layout = fonts.layout_text(text, SEGMENTED_FONT_SIZE, Color::WHITE, None);
        FontSystem::layout_size(&layout).0 / fonts.scale
    }

    fn intrinsic_track_w(&self, fonts: &mut FontSystem) -> f32 {
        if self.options.is_empty() {
            return 0.0;
        }
        let mut w = 0.0;
        for option in &self.options {
            w += (self.seg_text_w(fonts, option) + SEGMENTED_PAD_X * 2.0).max(SEGMENTED_MIN_SEG_W);
        }
        w
    }

    fn segment_at(&self, x: f32, y: f32) -> Option<usize> {
        if self.options.is_empty() || self.seg_w <= 0.0 {
            return None;
        }
        if x < self.track_x
            || x > self.track_x + self.track_w
            || y < self.track_y
            || y > self.track_y + SEGMENTED_HEIGHT
        {
            return None;
        }
        let index = ((x - self.track_x) / self.seg_w).floor() as usize;
        Some(index.min(self.options.len() - 1))
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        if let Some(index) = self.segment_at(x as f32, y as f32) {
            self.armed = true;
            self.pressed = Some(index);
        }
    }

    /// While held the gray highlight follows the pointer; otherwise
    /// there is no hover state, like macOS.
    pub fn mouse_move(&mut self, x: f64, y: f64) {
        if self.armed {
            self.pressed = self.segment_at(x as f32, y as f32);
        }
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_up(x, y);
    }

    fn finish_up(&mut self, x: f64, y: f64) {
        let was_armed = self.armed;
        let was_pressed = self.pressed.take();
        self.armed = false;
        if was_armed && !self.disabled {
            if let Some(index) = was_pressed {
                if self.segment_at(x as f32, y as f32) == Some(index) {
                    self.select(index);
                }
            }
        }
    }

    fn pressed_color(&self) -> Color {
        if self.dark {
            SEGMENTED_PRESSED_DARK
        } else {
            SEGMENTED_PRESSED_LIGHT
        }
    }
}

impl View for SegmentedPicker {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let (label_w, label_h) = self.label_size(fonts);
        let track_w = self.intrinsic_track_w(fonts);
        let label_part = if label_w > 0.0 {
            label_w + SEGMENTED_GAP
        } else {
            0.0
        };
        (
            label_part + track_w,
            SEGMENTED_HEIGHT.max(label_h),
        )
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w;
        self.height = h;
        let (label_w, label_h) = self.label_size(fonts);
        let intrinsic = self.intrinsic_track_w(fonts);
        let label_part = if label_w > 0.0 {
            label_w + SEGMENTED_GAP
        } else {
            0.0
        };
        let track_w = (w - label_part).max(0.0).max(intrinsic.min(w - label_part));
        let n = self.options.len().max(1) as f32;
        // Segments share the track equally; at intrinsic size each keeps
        // at least its minimum width.
        self.seg_w = if track_w > 0.0 { track_w / n } else { 0.0 };
        self.track_w = self.seg_w * self.options.len() as f32;
        self.label_x = x;
        self.label_y = y + (h - label_h) / 2.0;
        self.track_x = x + label_part;
        self.track_y = y + (h - SEGMENTED_HEIGHT) / 2.0;
    }

    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, _images: &mut ImageLoader<'_>) {
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        if self.options.is_empty() {
            return;
        }

        if !self.label.is_empty() {
            let layout = fonts.layout_text_weighted(
                &self.label,
                SEGMENTED_LABEL_SIZE,
                self.eff(self.text_color),
                400.0,
                None,
            );
            draw_layout(scene, &layout, self.label_x, self.label_y, fonts.scale);
        }

        // Outer track.
        let track = RoundedRect::new(
            px(self.track_x),
            px(self.track_y),
            px(self.track_x + self.track_w),
            px(self.track_y + SEGMENTED_HEIGHT),
            px(SEGMENTED_RADIUS),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.track_color())),
            None,
            &track,
        );

        // Selected pill sits on the selected segment (no animation,
        // like macOS).
        let pill_x0 = self.track_x + self.selected as f32 * self.seg_w + SEGMENTED_PAD;
        let pill_x1 = pill_x0 + self.seg_w - SEGMENTED_PAD * 2.0;
        let pill = RoundedRect::new(
            px(pill_x0),
            px(self.track_y + SEGMENTED_PAD),
            px(pill_x1),
            px(self.track_y + SEGMENTED_HEIGHT - SEGMENTED_PAD),
            px(SEGMENTED_PILL_RADIUS),
        );
        scene.draw_blurred_rounded_rect(
            Affine::IDENTITY,
            vello::kurbo::Rect::new(
                px(pill_x0),
                px(self.track_y + SEGMENTED_PAD),
                px(pill_x1),
                px(self.track_y + SEGMENTED_HEIGHT - SEGMENTED_PAD),
            ),
            Color::from_rgba8(0, 0, 0, 40),
            px(SEGMENTED_PILL_RADIUS),
            3.0 * scale,
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.accent)),
            None,
            &pill,
        );

        // Hairline dividers between unselected neighbors only, like the
        // macOS segmented control (no divider touches the pill).
        let selected = self.selected;
        for i in 1..self.options.len() {
            if i == selected || i == selected + 1 {
                continue;
            }
            let dx = self.track_x + i as f32 * self.seg_w;
            let line = Line::new(
                (px(dx), px(self.track_y + 4.0)),
                (px(dx), px(self.track_y + SEGMENTED_HEIGHT - 4.0)),
            );
            scene.stroke(
                &Stroke::new(1.0 * scale),
                Affine::IDENTITY,
                &Brush::Solid(self.eff(self.divider_color())),
                None,
                &line,
            );
        }

        // Gray hold highlight on the pressed segment (not on the
        // already-selected one, which keeps its accent pill).
        if let Some(held) = self.pressed {
            if held != selected && !self.disabled {
                let hx0 = self.track_x + held as f32 * self.seg_w + SEGMENTED_PAD;
                let hx1 = hx0 + self.seg_w - SEGMENTED_PAD * 2.0;
                let hold = RoundedRect::new(
                    px(hx0),
                    px(self.track_y + SEGMENTED_PAD),
                    px(hx1),
                    px(self.track_y + SEGMENTED_HEIGHT - SEGMENTED_PAD),
                    px(SEGMENTED_PILL_RADIUS),
                );
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(self.eff(self.pressed_color())),
                    None,
                    &hold,
                );
            }
        }

        // Segment labels centered in each cell. No hover state, like
        // macOS: unselected labels stay dim.
        for (i, option) in self.options.clone().iter().enumerate() {
            let is_selected = i == selected;
            let color = if is_selected {
                Color::WHITE
            } else {
                self.text_dim
            };
            let weight = if is_selected { 600.0 } else { 400.0 };
            let layout =
                fonts.layout_text_weighted(option, SEGMENTED_FONT_SIZE, self.eff(color), weight, None);
            let (tw, th) = FontSystem::layout_size(&layout);
            let tw = tw / fonts.scale;
            let th = th / fonts.scale;
            let cx = self.track_x + i as f32 * self.seg_w + (self.seg_w - tw) / 2.0;
            let cy = self.track_y + (SEGMENTED_HEIGHT - th) / 2.0;
            draw_layout(scene, &layout, cx, cy, fonts.scale);
        }
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_up(x, y);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    fn picker() -> SegmentedPicker {
        SegmentedPicker::from_slice("Options", &["One", "Two", "Three"])
    }

    #[test]
    fn defaults_to_first_segment() {
        let p = picker();
        assert_eq!(p.selected_index(), 0);
        assert_eq!(p.selected_label(), Some("One"));
    }

    #[test]
    fn click_selects_segment_on_release_inside() {
        let mut p = picker();
        let mut fonts = FontSystem::new();
        let (w, h) = p.measure(&mut fonts);
        p.place(&mut fonts, 0.0, 0.0, w, h);
        assert!(!p.is_empty());
        assert_eq!(p.len(), 3);
        // Release without press does nothing.
        p.mouse_up(1000.0, 1000.0);
        assert_eq!(p.selected_index(), 0);
        // Press and release inside the last segment selects it
        // instantly (no animation, like macOS).
        let x = (p.track_x + p.track_w - 2.0) as f64;
        let y = (p.track_y + SEGMENTED_HEIGHT / 2.0) as f64;
        p.mouse_down(x, y);
        // While held the pressed segment shows the gray highlight.
        assert_eq!(p.pressed, Some(2));
        p.mouse_up(x, y);
        assert_eq!(p.selected_index(), 2);
        assert_eq!(p.selected_label(), Some("Three"));
        assert_eq!(p.pressed, None);
        // Press inside, release outside keeps the selection.
        p.mouse_down(x, y);
        p.mouse_up(5000.0, 5000.0);
        assert_eq!(p.selected_index(), 2);
    }

    #[test]
    fn hold_highlight_follows_pointer_and_has_no_hover() {
        let mut p = picker();
        let mut fonts = FontSystem::new();
        let (w, h) = p.measure(&mut fonts);
        p.place(&mut fonts, 0.0, 0.0, w, h);
        let y = (p.track_y + SEGMENTED_HEIGHT / 2.0) as f64;
        // Hovering without pressing highlights nothing.
        let x = (p.track_x + p.seg_w * 1.5) as f64;
        p.mouse_move(x, y);
        assert_eq!(p.pressed, None);
        // Pressing highlights the segment under the pointer...
        p.mouse_down(x, y);
        assert_eq!(p.pressed, Some(1));
        // ...follows it while held...
        let x2 = (p.track_x + p.seg_w * 2.5) as f64;
        p.mouse_move(x2, y);
        assert_eq!(p.pressed, Some(2));
        // ...and releases there.
        p.mouse_up(x2, y);
        assert_eq!(p.selected_index(), 2);
    }

    #[test]
    fn select_fires_callback_only_on_change() {
        let fires = Rc::new(Cell::new(0));
        let count = fires.clone();
        let mut p = picker().on_select(move |_| count.set(count.get() + 1));
        p.select(0);
        assert_eq!(fires.get(), 0);
        p.select(1);
        assert_eq!(p.selected_index(), 1);
        assert_eq!(fires.get(), 1);
    }

    #[test]
    fn set_selected_is_immediate_and_clamped() {
        let mut p = picker();
        p.set_selected(99);
        assert_eq!(p.selected_index(), 2);
        assert_eq!(p.selected_label(), Some("Three"));
    }

    #[test]
    fn manual_accent_wins_over_theme_accent() {
        let mut p = SegmentedPicker::from_slice("Options", &["One", "Two"])
            .accent(Color::from_rgb8(0x34, 0xc7, 0x59));
        p.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        assert_eq!(p.accent, Color::from_rgb8(0x34, 0xc7, 0x59));
        let mut plain = SegmentedPicker::from_slice("Options", &["One", "Two"]);
        plain.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        assert_eq!(plain.accent, Color::from_rgb8(0xff, 0x2d, 0x55));
    }

    #[test]
    fn empty_options_never_selects() {
        let mut p = SegmentedPicker::new("Options", Vec::new());
        let mut fonts = FontSystem::new();
        let (w, h) = p.measure(&mut fonts);
        p.place(&mut fonts, 0.0, 0.0, w.max(1.0), h.max(1.0));
        p.mouse_down(2.0, 2.0);
        p.mouse_up(2.0, 2.0);
        assert_eq!(p.selected_label(), None);
    }
}
