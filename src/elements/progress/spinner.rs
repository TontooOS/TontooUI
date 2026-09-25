use std::any::Any;
use std::time::Instant;

use vello::Scene;
use vello::kurbo::{Affine, Line, Stroke};
use vello::peniko::{Brush, Color};

use super::super::layout::View;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};

/// Spoke count around the dial.
pub const SPINNER_SPOKES: usize = 12;
/// Outer spoke radius in logical px.
pub const SPINNER_R_OUT: f32 = 16.0;
/// Inner spoke radius in logical px.
pub const SPINNER_R_IN: f32 = 9.0;
/// Spoke line width in logical px.
pub const SPINNER_SPOKE_W: f32 = 3.5;
/// Seconds per spoke step (one revolution per second).
pub const SPINNER_STEP_SECONDS: f64 = 1.0 / SPINNER_SPOKES as f64;
/// Lightest trail alpha (head spoke is fully opaque).
pub const SPINNER_TAIL_ALPHA: f32 = 0.15;
/// Caption size in logical px.
pub const SPINNER_TEXT_SIZE: f32 = 13.0;
/// Gap between dial and caption in logical px.
pub const SPINNER_TEXT_GAP: f32 = 8.0;
/// Default spoke color (always gray, never the accent).
pub const SPINNER_GRAY: Color = Color::from_rgb8(0x8e, 0x8e, 0x93);

/// Indeterminate spinner: twelve spokes rotating with a fade trail
/// behind the head spoke, like the reference, plus an optional
/// caption (`"Loading..."`) below. Display-only (no mouse handling,
/// no accent on purpose): the spokes stay gray unless the dev sets
/// a color by hand.
pub struct Spinner {
    color: Color,
    text: Option<String>,
    dark: bool,
    text_color: Color,
    t0: Instant,
    head: usize,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            color: SPINNER_GRAY,
            text: None,
            dark: true,
            text_color: Color::from_rgb8(0x9a, 0x9a, 0x9e),
            t0: Instant::now(),
            head: 0,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    /// Spoke and caption color. There is intentionally no accent
    /// following: the spinner stays gray unless set here.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Optional caption under the dial (e.g. `"Loading..."`).
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Mode for the caption gray.
    pub fn set_dark(&mut self, dark: bool) {
        self.dark = dark;
        self.text_color = if dark {
            Color::from_rgb8(0x9a, 0x9a, 0x9e)
        } else {
            Color::from_rgb8(0x6e, 0x6e, 0x72)
        };
    }

    /// Head spoke index (0-11) for the animation phase.
    pub fn head(&self) -> usize {
        self.head
    }

    /// Trail alpha for a spoke age (0 = head, 11 = oldest).
    pub fn trail_alpha(age: usize) -> f32 {
        let t = (age.min(SPINNER_SPOKES - 1) as f32) / (SPINNER_SPOKES - 1) as f32;
        1.0 - t * (1.0 - SPINNER_TAIL_ALPHA)
    }

    fn advance(&mut self, now: Instant) {
        let elapsed = now.saturating_duration_since(self.t0).as_secs_f64();
        self.head = (elapsed / SPINNER_STEP_SECONDS).floor() as usize % SPINNER_SPOKES;
    }

    fn with_alpha(color: Color, alpha: f32) -> Color {
        let c = color.to_rgba8();
        Color::from_rgba8(c.r, c.g, c.b, (c.a as f32 * alpha.clamp(0.0, 1.0)).round() as u8)
    }

    fn dial() -> f32 {
        (SPINNER_R_OUT + SPINNER_SPOKE_W / 2.0 + 2.0) * 2.0
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl View for Spinner {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        let caption = if self.text.is_some() {
            SPINNER_TEXT_SIZE + SPINNER_TEXT_GAP
        } else {
            0.0
        };
        (Self::dial(), Self::dial() + caption)
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
        let cy = self.y + Self::dial() / 2.0;

        for i in 0..SPINNER_SPOKES {
            let age = (self.head + SPINNER_SPOKES - i) % SPINNER_SPOKES;
            let angle = -std::f64::consts::FRAC_PI_2 + i as f64 * 2.0 * std::f64::consts::PI / SPINNER_SPOKES as f64;
            let (sin, cos) = angle.sin_cos();
            let line = Line::new(
                (
                    px(cx + SPINNER_R_IN * cos as f32),
                    px(cy + SPINNER_R_IN * sin as f32),
                ),
                (
                    px(cx + SPINNER_R_OUT * cos as f32),
                    px(cy + SPINNER_R_OUT * sin as f32),
                ),
            );
            let mut stroke = Stroke::new(px(SPINNER_SPOKE_W));
            stroke.start_cap = vello::kurbo::Cap::Round;
            stroke.end_cap = vello::kurbo::Cap::Round;
            scene.stroke(
                &stroke,
                Affine::IDENTITY,
                &Brush::Solid(Self::with_alpha(self.color, Self::trail_alpha(age))),
                None,
                &line,
            );
        }
        if let Some(text) = self.text.clone() {
            let layout = fonts.layout_text_weighted(
                &text,
                SPINNER_TEXT_SIZE,
                self.text_color,
                400.0,
                None,
            );
            let (tw, th) = FontSystem::layout_size(&layout);
            draw_layout(
                scene,
                &layout,
                cx - (tw / fonts.scale) / 2.0,
                self.y + Self::dial() + SPINNER_TEXT_GAP
                    + (SPINNER_TEXT_SIZE - th / fonts.scale) / 2.0,
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
    use std::time::Duration;

    #[test]
    fn defaults_to_gray() {
        let spinner = Spinner::new();
        assert_eq!(spinner.color, SPINNER_GRAY);
        assert_eq!(spinner.head(), 0);
    }

    #[test]
    fn trail_fades_behind_head() {
        assert_eq!(Spinner::trail_alpha(0), 1.0);
        assert!((Spinner::trail_alpha(11) - SPINNER_TAIL_ALPHA).abs() < 1e-6);
        let mut last = 2.0;
        for age in 0..SPINNER_SPOKES {
            let a = Spinner::trail_alpha(age);
            assert!(a < last || age == 0);
            last = a;
        }
    }

    #[test]
    fn head_steps_with_time() {
        let mut spinner = Spinner::new();
        let t0 = spinner.t0;
        spinner.advance(t0 + Duration::from_secs_f64(SPINNER_STEP_SECONDS * 3.0));
        assert_eq!(spinner.head(), 3);
        // Full revolution wraps around.
        spinner.advance(t0 + Duration::from_secs_f64(SPINNER_STEP_SECONDS * 12.0));
        assert_eq!(spinner.head(), 0);
    }

    #[test]
    fn caption_adds_height() {
        let mut plain = Spinner::new();
        let mut captioned = Spinner::new().text("Loading...");
        let mut fonts = FontSystem::new();
        let (_, plain_h) = plain.measure(&mut fonts);
        let (_, cap_h) = captioned.measure(&mut fonts);
        assert!(cap_h > plain_h);
        assert_eq!(cap_h - plain_h, SPINNER_TEXT_SIZE + SPINNER_TEXT_GAP);
    }

    #[test]
    fn dev_color_wins() {
        let spinner = Spinner::new().color(Color::from_rgb8(0xff, 0x2d, 0x55));
        assert_eq!(spinner.color, Color::from_rgb8(0xff, 0x2d, 0x55));
    }
}
