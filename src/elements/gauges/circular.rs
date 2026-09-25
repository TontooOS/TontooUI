use std::any::Any;
use std::f64::consts::PI;
use std::time::Instant;

use vello::Scene;
use vello::kurbo::{Affine, Arc, Cap, Circle, Point, Stroke, Vec2};
use vello::peniko::{Brush, Color, Fill};

use super::super::layout::View;
use super::gauge::GAUGE_ANIM_SECONDS;
use crate::animation::{Easing, Repeat, Tween, TweenAnim};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::desaturate;

/// Ring centerline radius in logical px.
pub const CIRC_RING_R: f32 = 70.0;
/// Ring stroke width in logical px.
pub const CIRC_TRACK_W: f32 = 12.0;
/// Ring start angle in radians (bottom-left, screen coords).
pub const CIRC_START: f64 = 0.75 * PI;
/// Ring sweep in radians (270 degrees, gap at the bottom).
pub const CIRC_SWEEP: f64 = 1.5 * PI;
/// Value text size in logical px.
pub const CIRC_VALUE_SIZE: f32 = 34.0;
/// Caption label size in logical px.
pub const CIRC_LABEL_SIZE: f32 = 15.0;
/// Knob outer radius in logical px (fixed, independent of size).
pub const CIRC_KNOB_R: f32 = 8.0;
/// Knob middle ring radius in logical px (fixed).
pub const CIRC_KNOB_RING_R: f32 = 5.5;
/// Knob center dot radius in logical px (fixed).
pub const CIRC_DOT_R: f32 = 2.5;
/// Default ring fill (theme accent blue).
pub const CIRC_FILL: Color = Color::from_rgb8(0x00, 0x7a, 0xff);

/// Circular gauge: 270-degree ring with a knob marker at the value
/// fraction, value text centered and a caption below it, like the
/// reference. Display-only (no mouse handling); value changes tween
/// the knob along the arc. The ring stays monochrome (label color)
/// unless the dev sets a fill by hand.
pub struct CircularGauge {
    min: f64,
    max: f64,
    value: f64,
    shown: f64,
    label: Option<String>,
    value_text: Option<Box<dyn Fn(f64) -> String>>,
    fill: Color,
    fill_manual: bool,
    text_color: Color,
    bg: Color,
    dark: bool,
    focused: bool,
    anim: Option<TweenAnim<f64>>,
    anim_time: f32,
    last_draw: Option<Instant>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl CircularGauge {
    pub fn new(value: f64, min: f64, max: f64) -> Self {
        let min = min.min(max);
        let max = max.max(min);
        let value = value.clamp(min, max);
        Self {
            min,
            max,
            value,
            shown: value,
            label: None,
            value_text: None,
            fill: Color::WHITE,
            fill_manual: false,
            text_color: Color::WHITE,
            bg: Color::from_rgb8(0x1d, 0x1d, 0x1d),
            dark: true,
            focused: true,
            anim: None,
            anim_time: 0.0,
            last_draw: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    /// Caption below the value (e.g. `"Battery"`).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Center value text, rebuilt live (e.g. `|v| format!("{v:.0}%")`).
    pub fn value_text(mut self, f: impl Fn(f64) -> String + 'static) -> Self {
        self.value_text = Some(Box::new(f));
        self
    }

    /// Manual ring fill: wins over the monochrome default. The ring
    /// follows the label color (black in light mode, like the
    /// reference) unless the dev sets it by hand.
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = color;
        self.fill_manual = true;
        self
    }

    /// Live theme: label color, knob background and mode. The ring
    /// keeps the label color unless set manually with `fill`.
    pub fn set_theme(&mut self, _accent: Color, dark: bool) {
        self.dark = dark;
        if dark {
            self.text_color = Color::WHITE;
            self.bg = Color::from_rgb8(0x1d, 0x1d, 0x1d);
            if !self.fill_manual {
                self.fill = Color::WHITE;
            }
        } else {
            self.text_color = Color::BLACK;
            self.bg = Color::from_rgb8(0xec, 0xec, 0xec);
            if !self.fill_manual {
                self.fill = Color::BLACK;
            }
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn fraction(&self) -> f32 {
        if self.max <= self.min {
            return 0.0;
        }
        ((self.value - self.min) / (self.max - self.min)) as f32
    }

    /// Set the value (clamped); knob and arc tween there.
    pub fn set_value(&mut self, value: f64) {
        let clamped = value.clamp(self.min, self.max);
        if clamped != self.value {
            self.value = clamped;
            self.anim = Some(TweenAnim::new(
                Tween::new(self.shown, clamped, GAUGE_ANIM_SECONDS)
                    .easing(Easing::CubicOut)
                    .repeat(Repeat::Never),
            ));
            self.anim_time = 0.0;
        }
    }

    fn shown_fraction(&self) -> f32 {
        if self.max <= self.min {
            return 0.0;
        }
        ((self.shown - self.min) / (self.max - self.min)) as f32
    }

    /// Knob angle in radians for a fraction along the arc.
    fn angle_at(fraction: f32) -> f64 {
        CIRC_START + CIRC_SWEEP * fraction.clamp(0.0, 1.0) as f64
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
            self.shown = *anim.value();
            if done {
                self.anim = None;
                self.shown = self.value;
            }
        } else {
            self.shown = self.value;
        }
    }

    fn eff(&self, color: Color) -> Color {
        if self.focused {
            color
        } else {
            desaturate(color)
        }
    }

    fn size() -> f32 {
        (CIRC_RING_R + CIRC_TRACK_W / 2.0 + 4.0) * 2.0
    }
}

impl View for CircularGauge {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        let s = Self::size();
        (s, s)
    }

    fn place(&mut self, _fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w;
        self.height = h;
    }

    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, _images: &mut ImageLoader<'_>) {
        self.advance(Instant::now());
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        // Center the fixed-size dial in the placed rect.
        let cx = self.x + self.width / 2.0;
        let cy = self.y + self.height / 2.0;

        // Full 270-degree arc, like the reference (no gray track).
        let arc = Arc::new(
            Point::new(px(cx), px(cy)),
            Vec2::new(px(CIRC_RING_R), px(CIRC_RING_R)),
            CIRC_START,
            CIRC_SWEEP,
            0.0,
        );
        let mut stroke = Stroke::new(px(CIRC_TRACK_W));
        stroke.start_cap = Cap::Round;
        stroke.end_cap = Cap::Round;
        scene.stroke(
            &stroke,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.fill)),
            None,
            &arc,
        );

        // Knob at the value angle: fill disc, background ring, fill dot.
        let a = Self::angle_at(self.shown_fraction());
        let kx = cx + CIRC_RING_R * a.cos() as f32;
        let ky = cy + CIRC_RING_R * a.sin() as f32;
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.fill)),
            None,
            &Circle::new((px(kx), px(ky)), px(CIRC_KNOB_R)),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.bg)),
            None,
            &Circle::new((px(kx), px(ky)), px(CIRC_KNOB_RING_R)),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.fill)),
            None,
            &Circle::new((px(kx), px(ky)), px(CIRC_DOT_R)),
        );

        // Center value text.
        if let Some(f) = self.value_text.as_ref() {
            let layout = fonts.layout_text_weighted(
                &f(self.value),
                CIRC_VALUE_SIZE,
                self.eff(self.text_color),
                600.0,
                None,
            );
            let (tw, th) = FontSystem::layout_size(&layout);
            draw_layout(
                scene,
                &layout,
                cx - (tw / fonts.scale) / 2.0,
                cy - 10.0 - (th / fonts.scale) / 2.0,
                fonts.scale,
            );
        }
        // Caption below the value, inside the ring gap.
        if let Some(label) = self.label.clone() {
            let layout = fonts.layout_text_weighted(
                &label,
                CIRC_LABEL_SIZE,
                self.eff(self.text_color),
                400.0,
                None,
            );
            let (tw, th) = FontSystem::layout_size(&layout);
            draw_layout(
                scene,
                &layout,
                cx - (tw / fonts.scale) / 2.0,
                cy + 26.0 - (th / fonts.scale) / 2.0,
                fonts.scale,
            );
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_square_size() {
        let mut gauge = CircularGauge::new(70.0, 0.0, 100.0);
        let mut fonts = FontSystem::new();
        assert_eq!(gauge.measure(&mut fonts), (160.0, 160.0));
    }

    #[test]
    fn clamps_value_into_range() {
        let gauge = CircularGauge::new(500.0, 0.0, 100.0);
        assert_eq!(gauge.value(), 100.0);
        assert_eq!(gauge.fraction(), 1.0);
    }

    #[test]
    fn set_value_tweens_knob() {
        let mut gauge = CircularGauge::new(70.0, 0.0, 100.0).value_text(|v| format!("{v:.0}%"));
        gauge.set_value(20.0);
        assert_eq!(gauge.value(), 20.0);
        assert!(gauge.anim.is_some());
    }

    #[test]
    fn knob_angle_spans_gap() {
        // 0 sits at the start (bottom-left), 1 at the end (bottom-right).
        assert!((CircularGauge::angle_at(0.0) - CIRC_START).abs() < 1e-9);
        assert!((CircularGauge::angle_at(1.0) - (CIRC_START + CIRC_SWEEP)).abs() < 1e-9);
        // 70 percent lands on the right side, like the reference dot.
        let a = CircularGauge::angle_at(0.7);
        let (x, y) = (a.cos(), a.sin());
        assert!(x > 0.0 && y < 0.0);
    }

    #[test]
    fn manual_fill_wins_over_monochrome() {
        let mut gauge = CircularGauge::new(70.0, 0.0, 100.0)
            .fill(Color::from_rgb8(0x34, 0xc7, 0x59));
        gauge.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        assert_eq!(gauge.fill, Color::from_rgb8(0x34, 0xc7, 0x59));
        let mut plain = CircularGauge::new(70.0, 0.0, 100.0);
        plain.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), false);
        assert_eq!(plain.fill, Color::BLACK);
    }
}
