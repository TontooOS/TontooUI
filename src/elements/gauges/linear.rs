use std::any::Any;
use std::time::Instant;

use vello::Scene;
use vello::kurbo::{Affine, Circle, RoundedRect};
use vello::peniko::{Brush, Color, Fill};

use super::super::layout::View;
use super::gauge::{GAUGE_ANIM_SECONDS, GAUGE_TITLE_GAP, GAUGE_TITLE_SIZE};
use crate::animation::{Easing, Repeat, Tween, TweenAnim};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::desaturate;

/// Track line height in logical px.
pub const LINEAR_TRACK_H: f32 = 6.0;
/// Knob outer radius in logical px (fixed, independent of size).
pub const LINEAR_KNOB_R: f32 = 8.0;
/// Knob middle ring radius in logical px (fixed).
pub const LINEAR_RING_R: f32 = 5.5;
/// Knob center dot radius in logical px (fixed).
pub const LINEAR_DOT_R: f32 = 2.5;
/// Value label size in logical px.
pub const LINEAR_VALUE_SIZE: f32 = 15.0;
/// Gap between value label and track in logical px.
pub const LINEAR_VALUE_GAP: f32 = 10.0;

/// Linear gauge: value label left of a thin track line with a knob
/// marker at the value fraction, like the reference. Display-only
/// (no mouse handling); value changes tween the knob there. The
/// track and knob stay monochrome (label color on the mode
/// background) in every theme.
pub struct LinearGauge {
    min: f64,
    max: f64,
    value: f64,
    shown: f64,
    title: Option<String>,
    value_text: Option<Box<dyn Fn(f64) -> String>>,
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
    tx0: f32,
    tx1: f32,
    ty: f32,
}

impl LinearGauge {
    pub fn new(value: f64, min: f64, max: f64) -> Self {
        let min = min.min(max);
        let max = max.max(min);
        let value = value.clamp(min, max);
        Self {
            min,
            max,
            value,
            shown: value,
            title: None,
            value_text: None,
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
            tx0: 0.0,
            tx1: 0.0,
            ty: 0.0,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Value label left of the track, rebuilt live (e.g. `|v| format!("{v:.0}%")`).
    pub fn value_text(mut self, f: impl Fn(f64) -> String + 'static) -> Self {
        self.value_text = Some(Box::new(f));
        self
    }

    /// Live theme: label/track/knob grays plus the background the
    /// knob ring samples. The gauge stays monochrome, so the accent
    /// is accepted for API parity but not drawn.
    pub fn set_theme(&mut self, _accent: Color, dark: bool) {
        self.dark = dark;
        if dark {
            self.text_color = Color::WHITE;
            self.bg = Color::from_rgb8(0x1d, 0x1d, 0x1d);
        } else {
            self.text_color = Color::BLACK;
            self.bg = Color::from_rgb8(0xec, 0xec, 0xec);
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

    /// Set the value (clamped); the knob tweens there.
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

    fn knob_x(&self) -> f32 {
        self.tx0 + self.shown_fraction().clamp(0.0, 1.0) * (self.tx1 - self.tx0)
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

    fn header_h(&self) -> f32 {
        if self.title.is_some() {
            GAUGE_TITLE_SIZE + GAUGE_TITLE_GAP
        } else {
            0.0
        }
    }

    fn body_h(&self) -> f32 {
        LINEAR_KNOB_R * 2.0
    }

    fn label_w(&self, fonts: &mut FontSystem) -> f32 {
        match self.value_text.as_ref() {
            Some(f) => {
                let layout = fonts.layout_text(&f(self.value), LINEAR_VALUE_SIZE, Color::WHITE, None);
                FontSystem::layout_size(&layout).0 / fonts.scale + LINEAR_VALUE_GAP
            }
            None => 0.0,
        }
    }
}

impl View for LinearGauge {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        (self.label_w(fonts) + 160.0, self.header_h() + self.body_h())
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w;
        self.height = h;
        self.tx0 = x + self.label_w(fonts);
        self.tx1 = (x + w).max(self.tx0);
        self.ty = y + self.header_h();
    }

    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, _images: &mut ImageLoader<'_>) {
        self.advance(Instant::now());
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;

        if let Some(title) = self.title.clone() {
            let layout = fonts.layout_text_weighted(
                &title,
                GAUGE_TITLE_SIZE,
                self.eff(self.text_color),
                400.0,
                None,
            );
            let (tw, th) = FontSystem::layout_size(&layout);
            draw_layout(
                scene,
                &layout,
                self.x + (self.width - tw / fonts.scale) / 2.0,
                self.y + (GAUGE_TITLE_SIZE - th / fonts.scale) / 2.0,
                fonts.scale,
            );
        }

        let cy = self.ty + self.body_h() / 2.0;
        // Value label left of the track.
        if let Some(f) = self.value_text.as_ref() {
            let layout = fonts.layout_text_weighted(
                &f(self.value),
                LINEAR_VALUE_SIZE,
                self.eff(self.text_color),
                600.0,
                None,
            );
            let (_, th) = FontSystem::layout_size(&layout);
            draw_layout(scene, &layout, self.x, cy - (th / fonts.scale) / 2.0, fonts.scale);
        }

        // Thin track line.
        if self.tx1 > self.tx0 {
            let track = RoundedRect::new(
                px(self.tx0),
                px(cy - LINEAR_TRACK_H / 2.0),
                px(self.tx1),
                px(cy + LINEAR_TRACK_H / 2.0),
                px(LINEAR_TRACK_H / 2.0),
            );
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(self.eff(self.text_color)),
                None,
                &track,
            );
        }

        // Knob: label-color disc, background ring, label-color dot.
        // Fixed sizes, independent of the track length.
        let kx = self.knob_x();
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.text_color)),
            None,
            &Circle::new((px(kx), px(cy)), px(LINEAR_KNOB_R)),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.bg)),
            None,
            &Circle::new((px(kx), px(cy)), px(LINEAR_RING_R)),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.text_color)),
            None,
            &Circle::new((px(kx), px(cy)), px(LINEAR_DOT_R)),
        );
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_value_into_range() {
        let gauge = LinearGauge::new(99.0, 0.0, 100.0);
        assert_eq!(gauge.value(), 99.0);
        assert_eq!(gauge.fraction(), 0.99);
    }

    #[test]
    fn set_value_tweens_knob() {
        let mut gauge = LinearGauge::new(60.0, 0.0, 100.0).value_text(|v| format!("{v:.0}%"));
        gauge.set_value(80.0);
        assert_eq!(gauge.value(), 80.0);
        assert!(gauge.anim.is_some());
    }

    #[test]
    fn knob_tracks_fraction() {
        let mut gauge = LinearGauge::new(60.0, 0.0, 100.0);
        let mut fonts = FontSystem::new();
        gauge.place(&mut fonts, 0.0, 0.0, 200.0, 20.0);
        assert!((gauge.knob_x() - (gauge.tx0 + 0.6 * (gauge.tx1 - gauge.tx0))).abs() < 0.001);
    }

    #[test]
    fn value_label_widens_measure() {
        let mut plain = LinearGauge::new(60.0, 0.0, 100.0);
        let mut labeled = LinearGauge::new(60.0, 0.0, 100.0).value_text(|v| format!("{v:.0}%"));
        let mut fonts = FontSystem::new();
        let (plain_w, _) = plain.measure(&mut fonts);
        let (labeled_w, _) = labeled.measure(&mut fonts);
        assert_eq!(plain_w, 160.0);
        assert!(labeled_w > plain_w);
    }

    #[test]
    fn theme_inverts_monochrome() {
        let mut gauge = LinearGauge::new(60.0, 0.0, 100.0);
        gauge.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), false);
        assert_eq!(gauge.text_color, Color::BLACK);
        assert_eq!(gauge.bg, Color::from_rgb8(0xec, 0xec, 0xec));
    }
}
