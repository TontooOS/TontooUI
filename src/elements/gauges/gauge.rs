use std::any::Any;
use std::time::Instant;

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Brush, Color, Fill};

use super::super::layout::View;
use crate::animation::{Easing, Repeat, Tween, TweenAnim};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::desaturate;

/// Bar height in logical px.
pub const GAUGE_TRACK_H: f32 = 12.0;
/// Bar corner radius in logical px (fully rounded ends).
pub const GAUGE_RADIUS: f32 = 6.0;
/// Centered title size in logical px.
pub const GAUGE_TITLE_SIZE: f32 = 13.0;
/// Gap between title and bar in logical px.
pub const GAUGE_TITLE_GAP: f32 = 8.0;
/// Fill change animation time in seconds.
pub const GAUGE_ANIM_SECONDS: f32 = 0.25;
/// Track fill for light mode.
pub const GAUGE_TRACK_LIGHT: Color = Color::from_rgb8(0xe5, 0xe5, 0xe5);
/// Track fill for dark mode.
pub const GAUGE_TRACK_DARK: Color = Color::from_rgb8(0x3a, 0x3a, 0x3c);
/// Default value fill (theme accent blue).
pub const GAUGE_FILL: Color = Color::from_rgb8(0x00, 0x7a, 0xff);

/// Basic gauge: centered title above a rounded bar with an accent
/// fill for the value fraction, like the reference. Display-only (no
/// mouse handling); value changes tween to the new fill width.
pub struct Gauge {
    min: f64,
    max: f64,
    value: f64,
    shown: f64,
    title: Option<String>,
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

impl Gauge {
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
            fill: GAUGE_FILL,
            fill_manual: false,
            track: GAUGE_TRACK_DARK,
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

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Manual fill: wins over the system accent until cleared. The
    /// fill follows the system accent unless the dev sets it by hand.
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

    /// Live theme: accent fill plus mode track gray and title color.
    /// A manually set fill/track wins over the system one.
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        if !self.fill_manual {
            self.fill = accent;
        }
        self.dark = dark;
        if dark {
            if !self.track_manual {
                self.track = GAUGE_TRACK_DARK;
            }
            self.text_color = Color::WHITE;
        } else {
            if !self.track_manual {
                self.track = GAUGE_TRACK_LIGHT;
            }
            self.text_color = Color::BLACK;
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

    /// Set the value (clamped); the fill tweens there.
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
}

impl View for Gauge {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        let header = if self.title.is_some() {
            GAUGE_TITLE_SIZE + GAUGE_TITLE_GAP
        } else {
            0.0
        };
        (160.0, header + GAUGE_TRACK_H)
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

        let header = if self.title.is_some() {
            GAUGE_TITLE_SIZE + GAUGE_TITLE_GAP
        } else {
            0.0
        };
        let by = self.y + header;
        let track = RoundedRect::new(
            px(self.x),
            px(by),
            px(self.x + self.width),
            px(by + GAUGE_TRACK_H),
            px(GAUGE_RADIUS),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.track)),
            None,
            &track,
        );
        let fx = self.x + self.shown_fraction().clamp(0.0, 1.0) * self.width;
        if fx > self.x {
            let filled = RoundedRect::new(
                px(self.x),
                px(by),
                px(fx),
                px(by + GAUGE_TRACK_H),
                px(GAUGE_RADIUS),
            );
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(self.eff(self.fill)),
                None,
                &filled,
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
    fn clamps_value_into_range() {
        let gauge = Gauge::new(99.0, 0.0, 1.0);
        assert_eq!(gauge.value(), 1.0);
        assert_eq!(gauge.fraction(), 1.0);
        let empty = Gauge::new(0.0, 5.0, 5.0);
        assert_eq!(empty.fraction(), 0.0);
    }

    #[test]
    fn set_value_tweens_fill() {
        let mut gauge = Gauge::new(0.2, 0.0, 1.0).title("Progress");
        gauge.set_value(0.8);
        assert_eq!(gauge.value(), 0.8);
        assert!(gauge.anim.is_some());
        // Same value restarts nothing.
        gauge.set_value(0.8);
    }

    #[test]
    fn set_value_clamps() {
        let mut gauge = Gauge::new(0.5, 0.0, 100.0);
        gauge.set_value(500.0);
        assert_eq!(gauge.value(), 100.0);
        gauge.set_value(-10.0);
        assert_eq!(gauge.value(), 0.0);
    }

    #[test]
    fn manual_fill_wins_over_theme_accent() {
        let mut gauge = Gauge::new(0.6, 0.0, 1.0).fill(Color::from_rgb8(0x34, 0xc7, 0x59));
        gauge.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        assert_eq!(gauge.fill, Color::from_rgb8(0x34, 0xc7, 0x59));
        let mut plain = Gauge::new(0.6, 0.0, 1.0);
        plain.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        assert_eq!(plain.fill, Color::from_rgb8(0xff, 0x2d, 0x55));
    }

    #[test]
    fn title_adds_header_height() {
        let mut plain = Gauge::new(0.5, 0.0, 1.0);
        let mut titled = Gauge::new(0.5, 0.0, 1.0).title("Progress");
        let mut fonts = FontSystem::new();
        let (_, plain_h) = plain.measure(&mut fonts);
        let (_, titled_h) = titled.measure(&mut fonts);
        assert_eq!(plain_h, GAUGE_TRACK_H);
        assert_eq!(titled_h, GAUGE_TRACK_H + GAUGE_TITLE_SIZE + GAUGE_TITLE_GAP);
    }
}
