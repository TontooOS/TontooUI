use std::any::Any;
use std::f64::consts::PI;
use std::time::Instant;

use vello::Scene;
use vello::kurbo::{Affine, Arc, Cap, Point, Stroke, Vec2};
use vello::peniko::{Brush, Color};
use vello::peniko::Fill;

use super::super::layout::View;
use super::gauge::GAUGE_ANIM_SECONDS;
use crate::animation::{Easing, Repeat, Tween, TweenAnim};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::desaturate;

/// Ring centerline radius in logical px.
pub const CAP_RING_R: f32 = 60.0;
/// Ring stroke width in logical px.
pub const CAP_TRACK_W: f32 = 10.0;
/// Arc start angle in radians (top, screen coords).
pub const CAP_START: f64 = -0.5 * PI;
/// Full circle sweep in radians.
pub const CAP_SWEEP: f64 = 2.0 * PI;
/// Value text size in logical px.
pub const CAP_VALUE_SIZE: f32 = 30.0;
/// Track fill for light mode.
pub const CAP_TRACK_LIGHT: Color = Color::from_rgb8(0xe5, 0xe5, 0xe5);
/// Track fill for dark mode.
pub const CAP_TRACK_DARK: Color = Color::from_rgb8(0x3a, 0x3a, 0x3c);
/// Default value fill (theme accent blue).
pub const CAP_FILL: Color = Color::from_rgb8(0x00, 0x7a, 0xff);

/// Capacity gauge: full-circle ring with a gray track and a value
/// arc sweeping clockwise from the top, value text centered, like
/// the reference. Display-only (no mouse handling); value changes
/// tween the arc there. The arc stays monochrome (label color)
/// unless the dev sets a fill by hand.
pub struct CapacityGauge {
    min: f64,
    max: f64,
    value: f64,
    shown: f64,
    value_text: Option<Box<dyn Fn(f64) -> String>>,
    fill: Color,
    fill_manual: bool,
    track: Color,
    track_manual: bool,
    text_color: Color,
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

impl CapacityGauge {
    pub fn new(value: f64, min: f64, max: f64) -> Self {
        let min = min.min(max);
        let max = max.max(min);
        let value = value.clamp(min, max);
        Self {
            min,
            max,
            value,
            shown: value,
            value_text: None,
            fill: Color::WHITE,
            fill_manual: false,
            track: CAP_TRACK_DARK,
            track_manual: false,
            text_color: Color::WHITE,
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

    /// Center value text, rebuilt live (e.g. `|v| format!("{v:.0}%")`).
    pub fn value_text(mut self, f: impl Fn(f64) -> String + 'static) -> Self {
        self.value_text = Some(Box::new(f));
        self
    }

    /// Manual value-arc fill: wins over the monochrome default. The
    /// arc follows the label color (black in light mode, like the
    /// reference) unless the dev sets it by hand.
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = color;
        self.fill_manual = true;
        self
    }

    pub fn track_color(mut self, color: Color) -> Self {
        self.track = color;
        self.track_manual = true;
        self
    }

    /// Live theme: label color and mode track gray. The arc keeps the
    /// label color unless set manually with `fill`.
    pub fn set_theme(&mut self, _accent: Color, dark: bool) {
        self.dark = dark;
        if dark {
            self.text_color = Color::WHITE;
            if !self.track_manual {
                self.track = CAP_TRACK_DARK;
            }
            if !self.fill_manual {
                self.fill = Color::WHITE;
            }
        } else {
            self.text_color = Color::BLACK;
            if !self.track_manual {
                self.track = CAP_TRACK_LIGHT;
            }
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

    /// Set the value (clamped); the arc tweens there.
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
        (CAP_RING_R + CAP_TRACK_W / 2.0 + 4.0) * 2.0
    }
}

impl View for CapacityGauge {
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
        let cx = self.x + self.width / 2.0;
        let cy = self.y + self.height / 2.0;
        let center = Point::new(px(cx), px(cy));
        let radii = Vec2::new(px(CAP_RING_R), px(CAP_RING_R));
        let mut stroke = Stroke::new(px(CAP_TRACK_W));
        stroke.start_cap = Cap::Round;
        stroke.end_cap = Cap::Round;

        // Gray background track, full circle.
        scene.stroke(
            &stroke,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.track)),
            None,
            &Arc::new(center, radii, CAP_START, CAP_SWEEP, 0.0),
        );
        // Value arc from the top, clockwise.
        let sweep = CAP_SWEEP * self.shown_fraction().clamp(0.0, 1.0) as f64;
        if sweep > 0.001 {
            scene.stroke(
                &stroke,
                Affine::IDENTITY,
                &Brush::Solid(self.eff(self.fill)),
                None,
                &Arc::new(center, radii, CAP_START, sweep, 0.0),
            );
        }

        // Center value text.
        if let Some(f) = self.value_text.as_ref() {
            let layout = fonts.layout_text_weighted(
                &f(self.value),
                CAP_VALUE_SIZE,
                self.eff(self.text_color),
                600.0,
                None,
            );
            let (tw, th) = FontSystem::layout_size(&layout);
            draw_layout(
                scene,
                &layout,
                cx - (tw / fonts.scale) / 2.0,
                cy - (th / fonts.scale) / 2.0,
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
        let mut gauge = CapacityGauge::new(65.0, 0.0, 100.0);
        let mut fonts = FontSystem::new();
        assert_eq!(gauge.measure(&mut fonts), (138.0, 138.0));
    }

    #[test]
    fn clamps_value_into_range() {
        let gauge = CapacityGauge::new(500.0, 0.0, 100.0);
        assert_eq!(gauge.value(), 100.0);
        assert_eq!(gauge.fraction(), 1.0);
    }

    #[test]
    fn set_value_tweens_arc() {
        let mut gauge = CapacityGauge::new(65.0, 0.0, 100.0).value_text(|v| format!("{v:.0}%"));
        gauge.set_value(20.0);
        assert_eq!(gauge.value(), 20.0);
        assert!(gauge.anim.is_some());
    }

    #[test]
    fn manual_fill_and_track_win() {
        let mut gauge = CapacityGauge::new(65.0, 0.0, 100.0)
            .fill(Color::from_rgb8(0x34, 0xc7, 0x59))
            .track_color(Color::from_rgb8(0x11, 0x11, 0x11));
        gauge.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        assert_eq!(gauge.fill, Color::from_rgb8(0x34, 0xc7, 0x59));
        assert_eq!(gauge.track, Color::from_rgb8(0x11, 0x11, 0x11));
        let mut plain = CapacityGauge::new(65.0, 0.0, 100.0);
        plain.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), false);
        assert_eq!(plain.fill, Color::BLACK);
        assert_eq!(plain.track, CAP_TRACK_LIGHT);
    }
}
