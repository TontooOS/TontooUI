use std::any::Any;
use std::time::Instant;

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Brush, Color, Fill};

use super::super::layout::View;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::desaturate;

/// Bar height in logical px.
pub const PROGRESS_TRACK_H: f32 = 8.0;
/// Bar corner radius in logical px (fully rounded ends).
pub const PROGRESS_RADIUS: f32 = 4.0;
/// Centered title size in logical px.
pub const PROGRESS_TITLE_SIZE: f32 = 13.0;
/// Gap between title and bar in logical px.
pub const PROGRESS_TITLE_GAP: f32 = 8.0;
/// Default chase speed in fraction per second (full bar in ~8 s).
pub const PROGRESS_SPEED: f64 = 0.12;
/// Track fill for light mode.
pub const PROGRESS_TRACK_LIGHT: Color = Color::from_rgb8(0xe5, 0xe5, 0xe5);
/// Track fill for dark mode.
pub const PROGRESS_TRACK_DARK: Color = Color::from_rgb8(0x3a, 0x3a, 0x3c);
/// Default bar fill (theme accent blue).
pub const PROGRESS_FILL: Color = Color::from_rgb8(0x00, 0x7a, 0xff);

/// Linear progress view: thin rounded bar fed by the app. The app
/// (or dev) keeps sending progress numbers via `set_progress`, while
/// the shown fill chases the target at a limited speed: the bar
/// always moves slowly and keeps enough buffer to glide on when the
/// app stalls briefly, like the reference. Display-only (no mouse
/// handling).
pub struct LinearProgress {
    target: f64,
    shown: f64,
    speed: f64,
    title: Option<String>,
    fill: Color,
    fill_manual: bool,
    track: Color,
    track_manual: bool,
    text_color: Color,
    dark: bool,
    focused: bool,
    completed: bool,
    last_draw: Option<Instant>,
    on_complete: Option<Box<dyn FnMut()>>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl LinearProgress {
    pub fn new() -> Self {
        Self {
            target: 0.0,
            shown: 0.0,
            speed: PROGRESS_SPEED,
            title: None,
            fill: PROGRESS_FILL,
            fill_manual: false,
            track: PROGRESS_TRACK_DARK,
            track_manual: false,
            text_color: Color::WHITE,
            dark: true,
            focused: true,
            completed: false,
            last_draw: None,
            on_complete: None,
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

    /// Chase speed in fraction per second. Lower stays smoother and
    /// keeps more buffer; higher tracks the app tighter.
    pub fn speed(mut self, per_second: f64) -> Self {
        self.speed = per_second.max(0.001);
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

    pub fn on_complete(mut self, callback: impl FnMut() + 'static) -> Self {
        self.on_complete = Some(Box::new(callback));
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
                self.track = PROGRESS_TRACK_DARK;
            }
            self.text_color = Color::WHITE;
        } else {
            if !self.track_manual {
                self.track = PROGRESS_TRACK_LIGHT;
            }
            self.text_color = Color::BLACK;
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// App-reported progress target, 0.0 to 1.0 (clamped). Dropping
    /// below 1.0 re-arms the completion callback.
    pub fn set_progress(&mut self, value: f64) {
        self.target = value.clamp(0.0, 1.0);
        if self.target < 1.0 {
            self.completed = false;
        }
    }

    /// Chase speed in fraction per second.
    pub fn set_speed(&mut self, per_second: f64) {
        self.speed = per_second.max(0.001);
    }

    pub fn progress(&self) -> f64 {
        self.target
    }

    /// Currently shown fill (lags behind `progress`).
    pub fn displayed(&self) -> f64 {
        self.shown
    }

    fn advance(&mut self, now: Instant) {
        let dt = match self.last_draw {
            Some(last) => now.saturating_duration_since(last).as_secs_f64().min(0.5),
            None => 0.0,
        };
        self.last_draw = Some(now);
        let gap = self.target - self.shown;
        if gap.abs() > f64::EPSILON {
            let step = (self.speed * dt).min(gap.abs()) * gap.signum();
            self.shown = (self.shown + step).clamp(0.0, 1.0);
        }
        if !self.completed && self.target >= 1.0 && self.shown >= 1.0 {
            self.completed = true;
            if let Some(callback) = self.on_complete.as_mut() {
                callback();
            }
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

impl Default for LinearProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl View for LinearProgress {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        let header = if self.title.is_some() {
            PROGRESS_TITLE_SIZE + PROGRESS_TITLE_GAP
        } else {
            0.0
        };
        (160.0, header + PROGRESS_TRACK_H)
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
                PROGRESS_TITLE_SIZE,
                self.eff(self.text_color),
                400.0,
                None,
            );
            let (tw, th) = FontSystem::layout_size(&layout);
            draw_layout(
                scene,
                &layout,
                self.x + (self.width - tw / fonts.scale) / 2.0,
                self.y + (PROGRESS_TITLE_SIZE - th / fonts.scale) / 2.0,
                fonts.scale,
            );
        }

        let header = if self.title.is_some() {
            PROGRESS_TITLE_SIZE + PROGRESS_TITLE_GAP
        } else {
            0.0
        };
        let by = self.y + header;
        let track = RoundedRect::new(
            px(self.x),
            px(by),
            px(self.x + self.width),
            px(by + PROGRESS_TRACK_H),
            px(PROGRESS_RADIUS),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.track)),
            None,
            &track,
        );
        let fx = self.x + self.shown.clamp(0.0, 1.0) as f32 * self.width;
        if fx > self.x {
            let filled = RoundedRect::new(
                px(self.x),
                px(by),
                px(fx),
                px(by + PROGRESS_TRACK_H),
                px(PROGRESS_RADIUS),
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
    use std::cell::Cell;
    use std::rc::Rc;
    use std::time::Duration;

    /// Advance in frame-sized steps (dt caps per call, like real frames).
    fn pumped(target: f64, secs: f64) -> LinearProgress {
        pumped_speed(target, secs, PROGRESS_SPEED)
    }

    fn pumped_speed(target: f64, secs: f64, speed: f64) -> LinearProgress {
        let mut bar = LinearProgress::new().speed(speed);
        bar.set_progress(target);
        let t0 = Instant::now();
        let steps = (secs / 0.05).ceil().max(1.0) as u32;
        for i in 0..=steps {
            bar.advance(t0 + Duration::from_secs_f64(i as f64 * 0.05));
        }
        bar
    }

    #[test]
    fn clamps_progress_target() {
        let mut bar = LinearProgress::new();
        bar.set_progress(5.0);
        assert_eq!(bar.progress(), 1.0);
        bar.set_progress(-1.0);
        assert_eq!(bar.progress(), 0.0);
    }

    #[test]
    fn chases_slowly_and_keeps_buffer() {
        // App jumps to full: the bar only covered speed * time.
        let bar = pumped(1.0, 2.0);
        assert!((bar.displayed() - 0.24).abs() < 0.02);
        // App stalls there: the bar glides on by itself.
        let mut bar = bar;
        bar.set_progress(1.0);
        let t0 = Instant::now();
        bar.advance(t0);
        for i in 1..=40 {
            bar.advance(t0 + Duration::from_secs_f64(i as f64 * 0.05));
        }
        assert!(bar.displayed() > 0.4);
        // Long enough: arrives exactly.
        for i in 41..=400 {
            bar.advance(t0 + Duration::from_secs_f64(i as f64 * 0.05));
        }
        assert_eq!(bar.displayed(), 1.0);
    }

    #[test]
    fn speed_drives_arrival() {
        let bar = pumped_speed(1.0, 2.0, 1.0);
        assert_eq!(bar.displayed(), 1.0);
    }

    #[test]
    fn completes_once_and_rearms() {
        let fires = Rc::new(Cell::new(0));
        let count = fires.clone();
        let mut bar = LinearProgress::new().on_complete(move || count.set(count.get() + 1));
        bar.set_progress(1.0);
        let t0 = Instant::now();
        bar.advance(t0);
        let mut t = 0.0;
        while bar.displayed() < 1.0 {
            t += 0.05;
            bar.advance(t0 + Duration::from_secs_f64(t));
        }
        assert_eq!(fires.get(), 1);
        // Still complete: no second fire.
        bar.advance(t0 + Duration::from_secs_f64(t + 5.0));
        assert_eq!(fires.get(), 1);
        // Dropping below full re-arms: glide down first, then up.
        bar.set_progress(0.5);
        while bar.displayed() > 0.5 {
            t += 0.05;
            bar.advance(t0 + Duration::from_secs_f64(t));
        }
        bar.set_progress(1.0);
        while bar.displayed() < 1.0 {
            t += 0.05;
            bar.advance(t0 + Duration::from_secs_f64(t));
        }
        assert_eq!(fires.get(), 2);
    }

    #[test]
    fn manual_fill_wins_over_theme_accent() {
        let mut bar = LinearProgress::new().fill(Color::from_rgb8(0xff, 0x2d, 0x99));
        bar.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        assert_eq!(bar.fill, Color::from_rgb8(0xff, 0x2d, 0x99));
        let mut plain = LinearProgress::new();
        plain.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        assert_eq!(plain.fill, Color::from_rgb8(0xff, 0x2d, 0x55));
    }
}
