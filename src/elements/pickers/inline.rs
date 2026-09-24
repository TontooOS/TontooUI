use std::any::Any;
use std::time::Instant;

use vello::Scene;
use vello::kurbo::{Affine, Circle};
use vello::peniko::{Brush, Color, Fill};

use super::super::layout::View;
use crate::animation::{Easing, Repeat, Tween, TweenAnim};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::desaturate;

/// Radio dot outer radius in logical px.
pub const INLINE_RADIO_R: f32 = 9.0;
/// Selected inner dot radius in logical px (white center).
pub const INLINE_DOT_R: f32 = 3.75;
/// Row height in logical px.
pub const INLINE_ROW_H: f32 = 24.0;
/// Vertical gap between rows in logical px.
pub const INLINE_ROW_SPACING: f32 = 3.0;
/// Gap between the leading label and the options column in logical px.
pub const INLINE_GAP_X: f32 = 12.0;
/// Gap between a radio dot and its option text in logical px.
pub const INLINE_RADIO_GAP: f32 = 7.5;
/// Option/leading label size in logical px (settings-row measure).
pub const INLINE_FONT_SIZE: f32 = 13.0;
/// Dot pop animation time in seconds.
pub const INLINE_ANIM_SECONDS: f32 = 0.15;
/// Radio off fill for light mode.
pub const INLINE_OFF_LIGHT: Color = Color::from_rgb8(0xe5, 0xe5, 0xe5);
/// Radio off fill for dark mode.
pub const INLINE_OFF_DARK: Color = Color::from_rgb8(0x3a, 0x3a, 0x3c);
/// Default selected radio fill (theme accent blue).
pub const INLINE_ACCENT: Color = Color::from_rgb8(0x00, 0x7a, 0xff);

/// Inline picker (SwiftUI `Picker` with `.inline` style): an optional
/// leading label beside a vertical radio list. The selected row draws
/// an accent dot with a white center, the rest draw plain gray dots.
///
/// Clicks select on release inside a row (`mouse_down` arms,
/// `View::mouse_up` fires); the shell forwards both. The leading label
/// shares the first row baseline, like the Settings `Size` picker.
pub struct InlinePicker {
    label: String,
    options: Vec<String>,
    selected: usize,
    dot_scale: f32,
    accent: Color,
    accent_manual: bool,
    dark: bool,
    text_color: Color,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    options_x: f32,
    label_x: f32,
    label_y: f32,
    armed: Option<usize>,
    hovered: Option<usize>,
    disabled: bool,
    focused: bool,
    anim: Option<TweenAnim<f32>>,
    anim_time: f32,
    last_draw: Option<Instant>,
    on_select: Option<Box<dyn FnMut(usize)>>,
}

impl InlinePicker {
    pub fn new(label: impl Into<String>, options: Vec<String>) -> Self {
        Self {
            label: label.into(),
            options,
            selected: 0,
            dot_scale: 1.0,
            accent: INLINE_ACCENT,
            accent_manual: false,
            dark: true,
            text_color: Color::WHITE,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            options_x: 0.0,
            label_x: 0.0,
            label_y: 0.0,
            armed: None,
            hovered: None,
            disabled: false,
            focused: true,
            anim: None,
            anim_time: 0.0,
            last_draw: None,
            on_select: None,
        }
    }

    /// Convenience constructor from a slice (e.g. `["Small", "Medium"]`).
    pub fn from_slice(label: impl Into<String>, options: &[&str]) -> Self {
        Self::new(
            label,
            options.iter().map(|s| s.to_string()).collect(),
        )
    }

    /// Initial selection without animation and without firing `on_select`.
    pub fn selected(mut self, index: usize) -> Self {
        let clamped = self.clamp_index(index);
        self.selected = clamped;
        self.dot_scale = 1.0;
        self.anim = None;
        self
    }

    /// Manual selected-dot color: wins over the system accent until
    /// cleared. The dot follows the system accent unless set by hand.
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

    /// Live theme: accent dot plus mode grays and label color. A
    /// manually set accent wins over the system accent.
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        if !self.accent_manual {
            self.accent = accent;
        }
        self.dark = dark;
        self.text_color = if dark { Color::WHITE } else { Color::BLACK };
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

    /// Select with a dot pop animation. Fires `on_select` when the
    /// selection changed.
    pub fn select(&mut self, index: usize) {
        let clamped = self.clamp_index(index);
        if clamped != self.selected {
            self.selected = clamped;
            self.anim = Some(TweenAnim::new(
                Tween::new(0.4, 1.0, INLINE_ANIM_SECONDS)
                    .easing(Easing::CubicOut)
                    .repeat(Repeat::Never),
            ));
            self.anim_time = 0.0;
            self.dot_scale = 0.4;
            self.notify();
        }
    }

    /// Set the selection immediately (no animation). Fires `on_select`
    /// when the selection changed.
    pub fn set_selected(&mut self, index: usize) {
        let clamped = self.clamp_index(index);
        if clamped != self.selected {
            self.selected = clamped;
            self.dot_scale = 1.0;
            self.anim = None;
            self.notify();
        }
    }

    fn notify(&mut self) {
        if let Some(callback) = self.on_select.as_mut() {
            callback(self.selected);
        }
    }

    fn off_color(&self) -> Color {
        if self.dark {
            INLINE_OFF_DARK
        } else {
            INLINE_OFF_LIGHT
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
        let layout = fonts.layout_text(&self.label, INLINE_FONT_SIZE, Color::WHITE, None);
        let (tw, th) = FontSystem::layout_size(&layout);
        (tw / fonts.scale, th / fonts.scale)
    }

    fn option_text_w(&self, fonts: &mut FontSystem, text: &str) -> f32 {
        let layout = fonts.layout_text(text, INLINE_FONT_SIZE, Color::WHITE, None);
        FontSystem::layout_size(&layout).0 / fonts.scale
    }

    fn row_stride(&self) -> f32 {
        INLINE_ROW_H + INLINE_ROW_SPACING
    }

    fn row_center_y(&self, row: usize) -> f32 {
        self.y + row as f32 * self.row_stride() + INLINE_ROW_H / 2.0
    }

    fn row_at(&self, x: f32, y: f32) -> Option<usize> {
        if self.options.is_empty() || self.disabled {
            return None;
        }
        if x < self.options_x || x > self.x + self.width {
            return None;
        }
        if y < self.y || y > self.y + self.height {
            return None;
        }
        let row = ((y - self.y) / self.row_stride()).floor() as usize;
        if row >= self.options.len() {
            return None;
        }
        // Ignore hits in the spacing gap between rows.
        let within = (y - self.y) - row as f32 * self.row_stride();
        if within > INLINE_ROW_H {
            return None;
        }
        Some(row)
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        if let Some(row) = self.row_at(x as f32, y as f32) {
            self.armed = Some(row);
        }
    }

    pub fn mouse_move(&mut self, x: f64, y: f64) {
        self.hovered = self.row_at(x as f32, y as f32);
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_up(x, y);
    }

    fn finish_up(&mut self, x: f64, y: f64) {
        let was_armed = self.armed.take();
        if self.disabled {
            return;
        }
        if let Some(armed) = was_armed {
            if self.row_at(x as f32, y as f32) == Some(armed) {
                self.select(armed);
            }
        }
    }

    fn advance(&mut self, now: Instant) {
        let dt = match self.last_draw {
            Some(last) => now.saturating_duration_since(last).as_secs_f32().min(0.1),
            None => 0.0,
        };
        self.last_draw = Some(now);
        if let Some(anim) = self.anim.as_mut() {
            self.anim_time += dt;
            let done = anim.update(self.anim_time);
            self.dot_scale = *anim.value();
            if done {
                self.anim = None;
                self.dot_scale = 1.0;
            }
        }
    }
}

impl View for InlinePicker {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        if self.options.is_empty() {
            return (0.0, 0.0);
        }
        let (label_w, _) = self.label_size(fonts);
        let mut widest: f32 = 0.0;
        for option in &self.options {
            widest = widest.max(self.option_text_w(fonts, option));
        }
        let options_w = INLINE_RADIO_R * 2.0 + INLINE_RADIO_GAP + widest;
        let label_part = if label_w > 0.0 {
            label_w + INLINE_GAP_X
        } else {
            0.0
        };
        let rows = self.options.len() as f32;
        (
            label_part + options_w,
            rows * INLINE_ROW_H + (rows - 1.0) * INLINE_ROW_SPACING,
        )
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w;
        self.height = h;
        let (label_w, label_h) = self.label_size(fonts);
        self.label_x = x;
        // Leading label shares the first row baseline.
        self.label_y = y + (INLINE_ROW_H - label_h) / 2.0;
        self.options_x = if label_w > 0.0 {
            x + label_w + INLINE_GAP_X
        } else {
            x
        };
    }

    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, _images: &mut ImageLoader<'_>) {
        self.advance(Instant::now());
        if self.options.is_empty() {
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;

        if !self.label.is_empty() {
            let layout = fonts.layout_text_weighted(
                &self.label,
                INLINE_FONT_SIZE,
                self.eff(self.text_color),
                400.0,
                None,
            );
            draw_layout(scene, &layout, self.label_x, self.label_y, fonts.scale);
        }

        for (i, option) in self.options.clone().iter().enumerate() {
            let cy = self.row_center_y(i);
            let cx = self.options_x + INLINE_RADIO_R;
            let is_selected = i == self.selected;

            // Small shadow under the selected dot: active elements
            // carry a shadow.
            if is_selected {
                scene.draw_blurred_rounded_rect(
                    Affine::IDENTITY,
                    vello::kurbo::Rect::new(
                        px(cx - INLINE_RADIO_R),
                        px(cy - INLINE_RADIO_R),
                        px(cx + INLINE_RADIO_R),
                        px(cy + INLINE_RADIO_R),
                    ),
                    Color::from_rgba8(0, 0, 0, 40),
                    px(INLINE_RADIO_R),
                    3.0 * scale,
                );
            }

            // Radio dot: accent when selected, mode gray otherwise.
            let fill = if is_selected {
                self.accent
            } else {
                self.off_color()
            };
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(self.eff(fill)),
                None,
                &Circle::new((px(cx), px(cy)), px(INLINE_RADIO_R)),
            );

            // Hover ring on unselected rows.
            if Some(i) == self.hovered && !is_selected && !self.disabled {
                let ring = if self.dark {
                    Color::from_rgba8(255, 255, 255, 60)
                } else {
                    Color::from_rgba8(0, 0, 0, 40)
                };
                scene.stroke(
                    &vello::kurbo::Stroke::new(1.5 * scale),
                    Affine::IDENTITY,
                    &Brush::Solid(self.eff(ring)),
                    None,
                    &Circle::new((px(cx), px(cy)), px(INLINE_RADIO_R)),
                );
            }

            // White center dot with pop scale on fresh selection.
            if is_selected {
                let r = INLINE_DOT_R * self.dot_scale;
                if r > 0.3 {
                    scene.fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        &Brush::Solid(self.eff(Color::WHITE)),
                        None,
                        &Circle::new((px(cx), px(cy)), px(r)),
                    );
                }
            }

            let layout = fonts.layout_text_weighted(
                option,
                INLINE_FONT_SIZE,
                self.eff(self.text_color),
                400.0,
                None,
            );
            let (_, th) = FontSystem::layout_size(&layout);
            let tx = self.options_x + INLINE_RADIO_R * 2.0 + INLINE_RADIO_GAP;
            draw_layout(
                scene,
                &layout,
                tx,
                cy - (th / fonts.scale) / 2.0,
                fonts.scale,
            );
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

    fn picker() -> InlinePicker {
        InlinePicker::from_slice("Size", &["Small", "Medium", "Large", "Extra Large"])
    }

    #[test]
    fn defaults_to_first_option() {
        let p = picker();
        assert_eq!(p.selected_index(), 0);
        assert_eq!(p.selected_label(), Some("Small"));
    }

    #[test]
    fn click_selects_row_on_release_inside() {
        let mut p = picker();
        let mut fonts = FontSystem::new();
        let (w, h) = p.measure(&mut fonts);
        p.place(&mut fonts, 0.0, 0.0, w, h);
        assert_eq!(p.len(), 4);
        assert!(!p.is_empty());
        // Release without press does nothing.
        p.mouse_up(1000.0, 1000.0);
        assert_eq!(p.selected_index(), 0);
        // Press and release inside the third row selects it.
        let x = (p.options_x + INLINE_RADIO_R) as f64;
        let y = p.row_center_y(2) as f64;
        p.mouse_down(x, y);
        p.mouse_up(x, y);
        assert_eq!(p.selected_index(), 2);
        assert_eq!(p.selected_label(), Some("Large"));
        assert!(p.anim.is_some());
        // Press inside, release outside keeps the selection.
        p.mouse_down(x, y);
        p.mouse_up(5000.0, 5000.0);
        assert_eq!(p.selected_index(), 2);
    }

    #[test]
    fn gap_between_rows_is_not_clickable() {
        let mut p = picker();
        let mut fonts = FontSystem::new();
        let (w, h) = p.measure(&mut fonts);
        p.place(&mut fonts, 0.0, 0.0, w, h);
        let gap_y = (p.y + INLINE_ROW_H + INLINE_ROW_SPACING / 2.0) as f64;
        let x = (p.options_x + INLINE_RADIO_R) as f64;
        p.mouse_down(x, gap_y);
        p.mouse_up(x, gap_y);
        assert_eq!(p.selected_index(), 0);
    }

    #[test]
    fn select_fires_callback_only_on_change() {
        let fires = Rc::new(Cell::new(0));
        let count = fires.clone();
        let mut p = picker().on_select(move |_| count.set(count.get() + 1));
        p.select(0);
        assert_eq!(fires.get(), 0);
        p.select(3);
        assert_eq!(p.selected_index(), 3);
        assert_eq!(p.selected_label(), Some("Extra Large"));
        assert_eq!(fires.get(), 1);
    }

    #[test]
    fn set_selected_is_immediate_and_clamped() {
        let mut p = picker();
        p.set_selected(99);
        assert_eq!(p.selected_index(), 3);
        assert!(p.anim.is_none());
        assert_eq!(p.dot_scale, 1.0);
    }

    #[test]
    fn manual_accent_wins_over_theme_accent() {
        let mut p = InlinePicker::from_slice("Size", &["Small", "Medium"])
            .accent(Color::from_rgb8(0x34, 0xc7, 0x59));
        p.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        assert_eq!(p.accent, Color::from_rgb8(0x34, 0xc7, 0x59));
        let mut plain = InlinePicker::from_slice("Size", &["Small", "Medium"]);
        plain.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        assert_eq!(plain.accent, Color::from_rgb8(0xff, 0x2d, 0x55));
    }

    #[test]
    fn empty_options_never_selects() {
        let mut p = InlinePicker::new("Size", Vec::new());
        let mut fonts = FontSystem::new();
        p.place(&mut fonts, 0.0, 0.0, 200.0, 100.0);
        p.mouse_down(20.0, 16.0);
        p.mouse_up(20.0, 16.0);
        assert_eq!(p.selected_label(), None);
    }
}
