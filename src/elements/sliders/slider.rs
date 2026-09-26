use std::any::Any;
use std::time::Instant;

use vello::Scene;
use vello::kurbo::{Affine, Circle, Point, Rect, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, ColorStop, Fill, Gradient};

use super::super::layout::View;
use super::super::glass::{GLASS_ZOOM, glass_edge_width};
use crate::animation::{Easing, Repeat, Tween, TweenAnim};
use crate::renderer::backdrop::fill_lens_glass;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::{GlassAmount, desaturate};

/// Track height in logical px.
pub const SLIDER_TRACK_H: f32 = 6.0;
/// Knob width in logical px (pill shape, wider than tall).
pub const SLIDER_KNOB_W: f32 = 24.0;
/// Knob height in logical px.
pub const SLIDER_KNOB_H: f32 = 19.0;
/// Extra width when pressed (glass expand): 20% of the knob width.
pub const SLIDER_KNOB_EXPAND_W: f32 = 4.8;
/// Extra height when pressed (glass expand): 20% of the knob height.
pub const SLIDER_KNOB_EXPAND_H: f32 = 3.8;
/// Label size for header text.
pub const SLIDER_HEADER_SIZE: f32 = 15.0;
/// Label size for min/max text.
pub const SLIDER_SMALL_SIZE: f32 = 11.0;
/// Click-to-point animation time in seconds.
pub const SLIDER_ANIM_SECONDS: f32 = 0.25;
/// Press-expand animation time in seconds (knob grows while held).
pub const SLIDER_EXPAND_SECONDS: f32 = 0.18;

/// Track fill for light mode.
pub const SLIDER_TRACK_LIGHT: Color = Color::from_rgb8(0xe5, 0xe5, 0xe5);
/// Track fill for dark mode.
pub const SLIDER_TRACK_DARK: Color = Color::from_rgb8(0x3a, 0x3a, 0x3c);
/// Default value fill (theme accent blue).
pub const SLIDER_FILL: Color = Color::from_rgb8(0x00, 0x7a, 0xff);

/// Horizontal slider: basic, stepped, labeled, ticked, colored and glass
/// variants. Clicking the track animates the knob there; pressing the knob
/// (or holding) follows the mouse directly. While pressed the knob turns
/// liquid glass.
pub struct Slider {
    min: f64,
    max: f64,
    step: f64,
    value: f64,
    shown: f64,
    title: Option<String>,
    min_label: Option<String>,
    max_label: Option<String>,
    value_text: Option<Box<dyn Fn(f64) -> String>>,
    show_ticks: bool,
    fill: Color,
    fill_manual: bool,
    track: Color,
    track_manual: bool,
    text_color: Color,
    glass: bool,
    glass_amount: GlassAmount,
    dark: bool,
    focused: bool,
    dragging: bool,
    knob_expand: f32,
    expand_anim: Option<TweenAnim<f32>>,
    expand_time: f32,
    expand_to: Option<f32>,
    anim: Option<TweenAnim<f64>>,
    anim_time: f32,
    last_draw: Option<Instant>,
    on_change: Option<Box<dyn FnMut(f64)>>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    tx0: f32,
    tcy: f32,
    tx1: f32,
}

impl Slider {
    pub fn new(value: f64, min: f64, max: f64) -> Self {
        let min = min.min(max);
        let max = max.max(min);
        let value = value.clamp(min, max);
        Self {
            min,
            max,
            step: 0.0,
            value,
            shown: value,
            title: None,
            min_label: None,
            max_label: None,
            value_text: None,
            show_ticks: false,
            fill: SLIDER_FILL,
            fill_manual: false,
            track: SLIDER_TRACK_DARK,
            track_manual: false,
            text_color: Color::WHITE,
            glass: false,
            glass_amount: GlassAmount::Glass,
            dark: true,
            focused: true,
            dragging: false,
            knob_expand: 0.0,
            expand_anim: None,
            expand_time: 0.0,
            expand_to: None,
            anim: None,
            anim_time: 0.0,
            last_draw: None,
            on_change: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            tx0: 0.0,
            tcy: 0.0,
            tx1: 0.0,
        }
    }

    /// Step size in value units. `0.0` (default) is continuous; otherwise
    /// the value snaps to `min + k * step`.
    pub fn step(mut self, step: f64) -> Self {
        self.step = step.max(0.0);
        self.value = self.snap(self.value);
        self.shown = self.value;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn min_label(mut self, label: impl Into<String>) -> Self {
        self.min_label = Some(label.into());
        self
    }

    pub fn max_label(mut self, label: impl Into<String>) -> Self {
        self.max_label = Some(label.into());
        self
    }

    /// Centered header above the track, rebuilt live (e.g. `|v| format!("Value: {v:.0}")`).
    pub fn value_text(mut self, f: impl Fn(f64) -> String + 'static) -> Self {
        self.value_text = Some(Box::new(f));
        self
    }

    pub fn show_ticks(mut self, show: bool) -> Self {
        self.show_ticks = show;
        self
    }

    /// Manual fill: wins over the system accent until cleared. The fill
    /// follows the system default/color unless the dev sets it by hand.
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

    /// Glass track: translucent frost per LiquidGlass stage instead of a
    /// solid track. Same size, same layout.
    pub fn glass(mut self, glass: bool) -> Self {
        self.glass = glass;
        self
    }

    pub fn on_change(mut self, callback: impl FnMut(f64) + 'static) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    /// Live theme: accent fill, mode grays, glass stage. A manually set
    /// fill/track color wins over the system one.
    pub fn set_theme(&mut self, accent: Color, dark: bool, glass: GlassAmount) {
        if !self.fill_manual {
            self.fill = accent;
        }
        self.dark = dark;
        self.glass_amount = glass;
        if dark {
            if !self.track_manual {
                self.track = SLIDER_TRACK_DARK;
            }
            self.text_color = Color::WHITE;
        } else {
            if !self.track_manual {
                self.track = SLIDER_TRACK_LIGHT;
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

    /// True while the knob is held and follows the pointer. Apps use this
    /// to return true from `App::wants_backdrop` so the shell runs the
    /// backdrop blur pass.
    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    pub fn set_value(&mut self, value: f64) {
        let snapped = self.snap(value.clamp(self.min, self.max));
        if snapped != self.value {
            self.value = snapped;
            self.shown = snapped;
            self.anim = None;
            self.notify();
        }
    }

    fn snap(&self, value: f64) -> f64 {
        if self.step <= 0.0 {
            return value;
        }
        let steps = ((value - self.min) / self.step).round();
        (self.min + steps * self.step).clamp(self.min, self.max)
    }

    fn fraction(&self, value: f64) -> f32 {
        if self.max <= self.min {
            return 0.0;
        }
        ((value - self.min) / (self.max - self.min)) as f32
    }

    fn notify(&mut self) {
        if let Some(callback) = self.on_change.as_mut() {
            callback(self.value);
        }
    }

    fn label_width(fonts: &mut FontSystem, text: &str) -> f32 {
        let layout = fonts.layout_text(text, SLIDER_SMALL_SIZE, Color::WHITE, None);
        FontSystem::layout_size(&layout).0 / fonts.scale
    }

    fn header_height(&self) -> f32 {
        if self.title.is_some() || self.value_text.is_some() {
            24.0
        } else {
            0.0
        }
    }

    fn value_at(&self, x: f32) -> f64 {
        if self.tx1 <= self.tx0 {
            return self.min;
        }
        let p = ((x - self.tx0) / (self.tx1 - self.tx0)).clamp(0.0, 1.0) as f64;
        self.snap(self.min + p * (self.max - self.min))
    }

    fn knob_x(&self, value: f64) -> f32 {
        self.tx0 + self.fraction(value) * (self.tx1 - self.tx0)
    }

    fn knob_hit(&self, x: f32, y: f32) -> bool {
        let kx = self.knob_x(self.shown);
        let hw = SLIDER_KNOB_W / 2.0 + 6.0;
        let hh = SLIDER_KNOB_H / 2.0 + 6.0;
        (x - kx).abs() <= hw && (y - self.tcy).abs() <= hh
    }

    fn track_hit(&self, x: f32, y: f32) -> bool {
        x >= self.tx0 - 8.0 && x <= self.tx1 + 8.0 && (y - self.tcy).abs() <= 18.0
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        let (x, y) = (x as f32, y as f32);
        if self.knob_hit(x, y) {
            // Pressed knob follows the mouse directly.
            self.dragging = true;
            self.anim = None;
            self.set_value(self.value_at(x));
        } else if self.track_hit(x, y) {
            // Track click animates the knob there.
            let target = self.value_at(x);
            self.anim = Some(TweenAnim::new(
                Tween::new(self.shown, target, SLIDER_ANIM_SECONDS)
                    .easing(Easing::CubicOut)
                    .repeat(Repeat::Never),
            ));
            self.anim_time = 0.0;
        }
    }

    pub fn mouse_move(&mut self, x: f64, _y: f64) {
        if self.dragging {
            self.set_value(self.value_at(x as f32));
        }
    }

    pub fn mouse_up(&mut self, _x: f64, _y: f64) {
        self.dragging = false;
    }

    fn advance(&mut self, now: Instant) {
        let dt = match self.last_draw {
            Some(last) => now.saturating_duration_since(last).as_secs_f32().min(0.1),
            None => 0.0,
        };
        self.last_draw = Some(now);
        // Press-expand eases toward held/released (restarts from the
        // live value, so mid-flight reversals stay smooth).
        let expand_target = if self.dragging { 1.0f32 } else { 0.0 };
        if self.expand_to != Some(expand_target)
            && (self.knob_expand - expand_target).abs() > 0.0005
        {
            self.expand_anim = Some(TweenAnim::new(
                Tween::new(self.knob_expand, expand_target, SLIDER_EXPAND_SECONDS)
                    .easing(Easing::CubicOut),
            ));
            self.expand_to = Some(expand_target);
            self.expand_time = 0.0;
        }
        if self.expand_anim.is_some() {
            self.expand_time += dt;
            let (done, value) = match self.expand_anim.as_mut() {
                Some(anim) => (anim.update(self.expand_time), *anim.value()),
                None => (true, self.knob_expand),
            };
            if done {
                self.expand_anim = None;
                self.expand_to = None;
            }
            self.knob_expand = value;
        }
        if self.anim.is_some() && !self.dragging {
            self.anim_time += dt;
            let done = self.anim.as_mut().expect("anim set").update(self.anim_time);
            let target = *self.anim.as_ref().expect("anim set").value();
            self.shown = target;
            if done {
                self.anim = None;
                self.set_value(target);
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

    fn glass_track(&self) -> Color {
        let (r, g, b, a) = match (self.glass_amount, self.dark) {
            (GlassAmount::Less, true) => (10, 10, 12, 150),
            (GlassAmount::Less, false) => (255, 255, 255, 150),
            (GlassAmount::Much, false) => (255, 255, 255, 14),
            _ => {
                return if self.dark {
                    Color::from_rgba8(255, 255, 255, 26)
                } else {
                    Color::from_rgba8(0, 0, 0, 20)
                };
            }
        };
        Color::from_rgba8(r, g, b, a)
    }

    fn render(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
        now: Instant,
    ) {
        self.advance(now);
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let (tx0, tcy, tx1) = (self.tx0, self.tcy, self.tx1);

        let header = match (&self.title, &self.value_text) {
            (Some(title), Some(f)) => Some(format!("{title}  {}", f(self.value))),
            (Some(title), None) => Some(title.clone()),
            (None, Some(f)) => Some(f(self.value)),
            (None, None) => None,
        };
        if let Some(text) = header {
            let layout = fonts.layout_text_weighted(
                &text,
                SLIDER_HEADER_SIZE,
                self.eff(self.text_color),
                600.0,
                None,
            );
            let (tw, th) = FontSystem::layout_size(&layout);
            draw_layout(
                scene,
                &layout,
                self.x + (self.width - tw / fonts.scale) / 2.0,
                self.y + (24.0 - th / fonts.scale) / 2.0,
                fonts.scale,
            );
        }

        if tx1 <= tx0 {
            return;
        }
        let track_color = if self.glass {
            self.glass_track()
        } else {
            self.track
        };
        let track_rect = |x0: f32, x1: f32| {
            RoundedRect::new(
                px(x0),
                px(tcy - SLIDER_TRACK_H / 2.0),
                px(x1),
                px(tcy + SLIDER_TRACK_H / 2.0),
                px(SLIDER_TRACK_H / 2.0),
            )
        };
        // Base track.
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(track_color)),
            None,
            &track_rect(tx0, tx1),
        );
        // Filled part.
        let kx = tx0 + self.fraction(self.shown) * (tx1 - tx0);
        if kx > tx0 {
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(self.eff(self.fill)),
                None,
                &track_rect(tx0, kx),
            );
        }

        // Step ticks under the track.
        if self.show_ticks && self.step > 0.0 {
            let steps = ((self.max - self.min) / self.step).round() as usize;
            for i in 0..=steps.min(64) {
                let v = self.min + i as f64 * self.step;
                let tx = tx0 + self.fraction(v) * (tx1 - tx0);
                let dot = Circle::new((px(tx), px(tcy + 12.0)), 1.5 * scale);
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(self.eff(self.text_color)),
                    None,
                    &dot,
                );
            }
        }

        // Knob shadow first (under the knob). Skipped on the capture pass
        // while dragging so the blur sees the track behind the glass.
        let kw = SLIDER_KNOB_W / 2.0 + self.knob_expand * SLIDER_KNOB_EXPAND_W / 2.0;
        let kh = SLIDER_KNOB_H / 2.0 + self.knob_expand * SLIDER_KNOB_EXPAND_H / 2.0;
        let kr = kh;
        let skip_knob = self.dragging && images.is_capture_pass();
        if !skip_knob {
            scene.draw_blurred_rounded_rect(
                Affine::IDENTITY,
                vello::kurbo::Rect::new(px(kx - kw), px(tcy - kh), px(kx + kw), px(tcy + kh)),
                Color::from_rgba8(0, 0, 0, 40),
                px(kr),
                6.0 * scale,
            );
        }
        let knob = RoundedRect::new(
            px(kx - kw),
            px(tcy - kh),
            px(kx + kw),
            px(tcy + kh),
            px(kr),
        );
        if skip_knob {
            // Capture pass: leave the knob area empty for the blur.
        } else if self.dragging {
            // Liquid glass knob: clear minified center, hairline blurred
            // rim (same adaptive edge as `GlassContainer`).
            fill_lens_glass(
                scene,
                images,
                &Rect::new(px(kx - kw), px(tcy - kh), px(kx + kw), px(tcy + kh)),
                px(kr),
                GLASS_ZOOM,
                glass_edge_width(SLIDER_KNOB_H) as f64 * scale,
            );
            let tint = if self.dark {
                Color::from_rgba8(255, 255, 255, 26)
            } else {
                Color::from_rgba8(0, 0, 0, 20)
            };
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(tint),
                None,
                &knob,
            );
            let bevel = RoundedRect::new(
                px(kx - kw) + 1.0 * scale,
                px(tcy - kh) + 1.0 * scale,
                px(kx + kw) - 1.0 * scale,
                px(tcy + kh) - 1.0 * scale,
                (px(kr) - 1.0 * scale).max(0.0),
            );
            let knob_y0 = px(tcy - kh);
            let knob_y1 = px(tcy + kh);
            let bevel_brush = Gradient::new_linear(
                Point::new(px(kx), knob_y0),
                Point::new(px(kx), knob_y1),
            )
            .with_stops([
                ColorStop {
                    offset: 0.0,
                    color: Color::from_rgba8(255, 255, 255, 115).into(),
                },
                ColorStop {
                    offset: 0.4,
                    color: Color::TRANSPARENT.into(),
                },
                ColorStop {
                    offset: 0.85,
                    color: Color::from_rgba8(255, 255, 255, 40).into(),
                },
                ColorStop {
                    offset: 1.0,
                    color: Color::from_rgba8(255, 255, 255, 80).into(),
                },
            ]);
            scene.stroke(
                &Stroke::new(2.0 * scale),
                Affine::IDENTITY,
                &Brush::Gradient(bevel_brush),
                None,
                &bevel,
            );
            let chroma_r = RoundedRect::new(
                px(kx - kw) - 0.75 * scale,
                px(tcy - kh) - 0.75 * scale,
                px(kx + kw) + 0.75 * scale,
                px(tcy + kh) + 0.75 * scale,
                px(kr) + 0.75 * scale,
            );
            scene.stroke(
                &Stroke::new(1.0 * scale),
                Affine::IDENTITY,
                &Brush::Solid(Color::from_rgba8(255, 90, 120, 30)),
                None,
                &chroma_r,
            );
            let chroma_c = RoundedRect::new(
                px(kx - kw) + 0.75 * scale,
                px(tcy - kh) + 0.75 * scale,
                px(kx + kw) - 0.75 * scale,
                px(tcy + kh) - 0.75 * scale,
                (px(kr) - 0.75 * scale).max(0.0),
            );
            scene.stroke(
                &Stroke::new(1.0 * scale),
                Affine::IDENTITY,
                &Brush::Solid(Color::from_rgba8(90, 200, 255, 30)),
                None,
                &chroma_c,
            );
        } else {
            // Minimal contact shadow under the white knob: small downward
            // offset, tight blur, low alpha.
            scene.draw_blurred_rounded_rect(
                Affine::IDENTITY,
                vello::kurbo::Rect::new(
                    px(kx - kw),
                    px(tcy - kh) + 1.5 * scale,
                    px(kx + kw),
                    px(tcy + kh) + 1.5 * scale,
                ),
                Color::from_rgba8(0, 0, 0, 35),
                px(kr),
                2.5 * scale,
            );
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(self.eff(Color::WHITE)),
                None,
                &knob,
            );
        }

        // Side labels.
        if let Some(title) = self.title.clone() {
            let layout =
                fonts.layout_text(&title, SLIDER_SMALL_SIZE, self.eff(self.text_color), None);
            let (_, th) = FontSystem::layout_size(&layout);
            draw_layout(scene, &layout, self.x, tcy - th / fonts.scale / 2.0, fonts.scale);
        }
        if let Some(min) = self.min_label.clone() {
            let layout =
                fonts.layout_text(&min, SLIDER_SMALL_SIZE, self.eff(self.text_color), None);
            let (tw, th) = FontSystem::layout_size(&layout);
            draw_layout(
                scene,
                &layout,
                tx0 - tw / fonts.scale - 8.0,
                tcy - th / fonts.scale / 2.0,
                fonts.scale,
            );
        }
        if let Some(max) = self.max_label.clone() {
            let layout =
                fonts.layout_text(&max, SLIDER_SMALL_SIZE, self.eff(self.text_color), None);
            let (_, th) = FontSystem::layout_size(&layout);
            draw_layout(
                scene,
                &layout,
                tx1 + 8.0,
                tcy - th / fonts.scale / 2.0,
                fonts.scale,
            );
        }
    }
}

impl View for Slider {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        let header = if self.title.is_some() || self.value_text.is_some() {
            24.0
        } else {
            0.0
        };
        (160.0, header + SLIDER_KNOB_H + 16.0)
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w;
        self.height = h;
        // Measure side labels once per layout; track fills the rest.
        let header_h = self.header_height();
        let mut left = x;
        let mut right = x + w;
        if let Some(title) = self.title.clone() {
            left += Self::label_width(fonts, &title) + 8.0;
        }
        if let Some(min) = self.min_label.clone() {
            left += Self::label_width(fonts, &min) + 8.0;
        }
        if let Some(max) = self.max_label.clone() {
            right -= Self::label_width(fonts, &max) + 8.0;
        }
        self.tx0 = left;
        self.tx1 = right.max(left);
        self.tcy = y + header_h + (h - header_h) / 2.0;
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        self.render(scene, fonts, images, Instant::now());
    }

    /// View event path (containers forward here): arm the knob or
    /// start the track animation, like the inherent `mouse_down`.
    fn mouse_down(&mut self, x: f64, y: f64) {
        self.mouse_down(x, y);
    }

    /// View event path: release the knob, like `mouse_up`.
    fn mouse_up(&mut self, x: f64, y: f64) {
        self.mouse_up(x, y);
    }

    /// View event path: held knob follows hover moves (apps forward
    /// pointer moves as hover while containers route them here).
    fn set_hover(&mut self, x: f32, y: f32) {
        self.mouse_move(x as f64, y as f64);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::text::FontSystem;

    fn slider() -> Slider {
        let mut slider = Slider::new(10.0, 0.0, 100.0);
        let mut fonts = FontSystem::new();
        let (_, h) = slider.measure(&mut fonts);
        slider.place(&mut fonts, 0.0, 0.0, 400.0, h);
        slider
    }

    /// The View event path (used inside VStack/HStack pages) must arm
    /// the knob exactly like the inherent handlers: the slider demo
    /// downcasts around this, containers cannot.
    #[test]
    fn view_path_drags_knob() {
        let mut slider = slider();
        let kx = slider.knob_x(10.0);
        let tcy = slider.tcy;
        // Press the knob through the View trait.
        View::mouse_down(&mut slider, kx as f64, tcy as f64);
        assert!(slider.is_dragging());
        // Hover moves follow while held.
        View::set_hover(&mut slider, 200.0, tcy);
        assert_eq!(slider.value(), 50.0);
        // Release ends the drag.
        View::mouse_up(&mut slider, 200.0, tcy as f64);
        assert!(!slider.is_dragging());
    }

    #[test]
    fn view_path_track_click_animates() {
        let mut slider = slider();
        let tcy = slider.tcy;
        // Far from the knob (value 10 sits near the left edge).
        View::mouse_down(&mut slider, 380.0, tcy as f64);
        assert!(!slider.is_dragging());
        assert!(slider.anim.is_some());
        View::mouse_up(&mut slider, 380.0, tcy as f64);
    }

    #[test]
    fn press_expand_eases_out_and_back() {
        use std::time::Duration;

        let mut slider = slider();
        let kx = slider.knob_x(10.0);
        let tcy = slider.tcy;
        View::mouse_down(&mut slider, kx as f64, tcy as f64);
        assert!(slider.is_dragging());
        let t0 = Instant::now();
        slider.advance(t0);
        assert_eq!(slider.knob_expand, 0.0);
        // Halfway through time rushes ahead (CubicOut).
        slider.advance(t0 + Duration::from_millis(90));
        let mid = slider.knob_expand;
        assert!(mid > 0.5 && mid < 1.0, "mid was {mid}");
        slider.advance(t0 + Duration::from_millis(90 + 200));
        assert_eq!(slider.knob_expand, 1.0);
        // Release eases back down.
        View::mouse_up(&mut slider, kx as f64, tcy as f64);
        let t1 = Instant::now();
        slider.advance(t1);
        slider.advance(t1 + Duration::from_millis(90));
        let back = slider.knob_expand;
        assert!(back > 0.0 && back < 1.0, "back was {back}");
        slider.advance(t1 + Duration::from_millis(90 + 200));
        assert_eq!(slider.knob_expand, 0.0);
    }

    #[test]
    fn view_path_reports_changes() {
        use std::cell::Cell;
        use std::rc::Rc;

        let seen: Rc<Cell<f64>> = Rc::new(Cell::new(-1.0));
        let capture = seen.clone();
        let mut slider = Slider::new(10.0, 0.0, 100.0).on_change(move |v| capture.set(v));
        let mut fonts = FontSystem::new();
        let (_, h) = slider.measure(&mut fonts);
        slider.place(&mut fonts, 0.0, 0.0, 400.0, h);
        let kx = slider.knob_x(10.0);
        let tcy = slider.tcy;
        View::mouse_down(&mut slider, kx as f64, tcy as f64);
        View::set_hover(&mut slider, 200.0, tcy);
        assert_eq!(seen.get(), 50.0);
    }
}
