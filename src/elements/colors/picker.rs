use std::any::Any;
use std::sync::OnceLock;

use vello::Scene;
use vello::kurbo::{Affine, Circle, Line, Point, Rect, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, ColorStop, Fill, Gradient};

use super::super::glass::{GlassContainer, GlassType};
use super::super::layout::View;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::{GlassAmount, ThemeMode, desaturate};

/// Panel padding in logical px.
pub const PICKER_PAD: f32 = 16.0;
/// Wheel diameter in logical px.
pub const PICKER_WHEEL: f32 = 232.0;
/// Gap between wheel and brightness bar in logical px.
pub const PICKER_GAP: f32 = 16.0;
/// Slider bar height in logical px.
pub const PICKER_BAR_H: f32 = 28.0;
/// Gap between brightness bar and opacity label in logical px.
pub const PICKER_LABEL_GAP: f32 = 8.0;
/// Gap between opacity label and row in logical px.
pub const PICKER_ROW_GAP: f32 = 6.0;
/// Panel corner radius in logical px.
pub const PICKER_RADIUS: f32 = 20.0;
/// Label size in logical px.
pub const PICKER_LABEL_SIZE: f32 = 13.0;
/// Percent pill width in logical px.
pub const PICKER_PILL_W: f32 = 64.0;
/// Label gray on the frosted panel.
pub const PICKER_LABEL_GRAY: Color = Color::from_rgba8(255, 255, 255, 160);
/// Percent pill fill.
pub const PICKER_PILL: Color = Color::from_rgb8(0x1e, 0x1e, 0x20);
/// Checker light/dark squares.
pub const PICKER_CHECK_A: Color = Color::from_rgb8(0xc0, 0xc0, 0xc0);
pub const PICKER_CHECK_B: Color = Color::from_rgb8(0x80, 0x80, 0x80);
/// Checker square size in logical px.
pub const PICKER_CHECK: f32 = 10.0;

/// Which slider the press is dragging.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DragTarget {
    Wheel,
    Brightness,
    Opacity,
}

/// HSV color: hue, saturation and value in 0..1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

/// RGB (0..1) to HSV.
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> Hsv {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d <= 0.0 {
        0.0
    } else if max == r {
        ((g - b) / d).rem_euclid(6.0) / 6.0
    } else if max == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };
    Hsv {
        h,
        s: if max <= 0.0 { 0.0 } else { d / max },
        v: max,
    }
}

/// HSV (0..1) to RGB (0..1).
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let h = h.rem_euclid(1.0) * 6.0;
    let i = h.floor() as i32;
    let f = h - h.floor();
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match i.rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

/// Peniko color to HSV plus alpha.
pub fn color_to_hsva(color: Color) -> (Hsv, f32) {
    let c = color.to_rgba8();
    (
        rgb_to_hsv(c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0),
        c.a as f32 / 255.0,
    )
}

/// HSV plus alpha to peniko color.
pub fn hsva_to_color(hsv: Hsv, alpha: f32) -> Color {
    let (r, g, b) = hsv_to_rgb(hsv.h, hsv.s, hsv.v);
    Color::from_rgba8(
        (r.clamp(0.0, 1.0) * 255.0).round() as u8,
        (g.clamp(0.0, 1.0) * 255.0).round() as u8,
        (b.clamp(0.0, 1.0) * 255.0).round() as u8,
        (alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

/// Wheel PNG bytes (256 px RGBA), generated once: conic hue ring
/// fading to white in the center, like the reference wheel.
fn wheel_png() -> &'static [u8] {
    static WHEEL: OnceLock<Vec<u8>> = OnceLock::new();
    WHEEL.get_or_init(|| {
        use image::codecs::png::PngEncoder;
        use image::ImageEncoder;

        const N: u32 = 256;
        let mut raw = Vec::with_capacity((N * N * 4) as usize);
        for py in 0..N {
            for px in 0..N {
                let dx = (px as f32 + 0.5 - N as f32 / 2.0) / (N as f32 / 2.0);
                let dy = (py as f32 + 0.5 - N as f32 / 2.0) / (N as f32 / 2.0);
                let r = dx.hypot(dy);
                if r > 1.0 {
                    raw.extend_from_slice(&[0, 0, 0, 0]);
                    continue;
                }
                let hue = dy.atan2(dx) / std::f32::consts::TAU;
                let t = r.clamp(0.0, 1.0);
                let (hr, hg, hb) = hsv_to_rgb(hue, t, 1.0);
                // White center glow.
                let w = (1.0 - t).powi(2);
                raw.extend_from_slice(&[
                    ((hr * (1.0 - w) + w) * 255.0).round() as u8,
                    ((hg * (1.0 - w) + w) * 255.0).round() as u8,
                    ((hb * (1.0 - w) + w) * 255.0).round() as u8,
                    255,
                ]);
            }
        }
        let mut png = Vec::new();
        PngEncoder::new(&mut png)
            .write_image(&raw, N, N, image::ExtendedColorType::Rgba8)
            .expect("wheel PNG encodes");
        png
    })
}

/// Color picker popup in a frosted glass panel: hue/saturation wheel
/// with a crosshair, brightness slider, opacity label plus checker
/// transparency slider with a percent pill (like the reference).
/// Menu-like: triggered with `show` by a button or the app, reports
/// every change through `on_change`, reads back via `selected`, and
/// dismisses on outside clicks.
pub struct ColorPicker {
    hsv: Hsv,
    opacity: f32,
    on_change: Option<Box<dyn FnMut(Color)>>,
    glass: GlassContainer,
    dark: bool,
    focused: bool,
    visible: bool,
    drag: Option<DragTarget>,
    vx: f32,
    vy: f32,
    vw: f32,
    vh: f32,
    x: f32,
    y: f32,
    pct_text: String,
    pct_layout: Option<parley::Layout<crate::renderer::text::SolidBrush>>,
    label_layout: Option<parley::Layout<crate::renderer::text::SolidBrush>>,
    layout_scale: f32,
    dirty: bool,
}

impl ColorPicker {
    pub fn new() -> Self {
        Self {
            hsv: Hsv { h: 0.5, s: 1.0, v: 1.0 },
            opacity: 1.0,
            on_change: None,
            glass: GlassContainer::new().glass_type(GlassType::Frosted),
            dark: true,
            focused: true,
            visible: false,
            drag: None,
            vx: 0.0,
            vy: 0.0,
            vw: 0.0,
            vh: 0.0,
            x: 0.0,
            y: 0.0,
            pct_text: "100 %".to_string(),
            pct_layout: None,
            label_layout: None,
            layout_scale: 0.0,
            dirty: true,
        }
    }

    /// Fired with the new color on every change (wheel, sliders,
    /// `set_color`).
    pub fn on_change(mut self, callback: impl FnMut(Color) + 'static) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    /// Start from a color (converts to HSV plus alpha).
    pub fn color(mut self, color: Color) -> Self {
        self.set_color(color);
        self
    }

    /// Start from a color (converts to HSV plus alpha). Fires
    /// `on_change` when it changed anything.
    pub fn set_color(&mut self, color: Color) {
        let (hsv, alpha) = color_to_hsva(color);
        if (hsv.h - self.hsv.h).abs() > f32::EPSILON
            || (hsv.s - self.hsv.s).abs() > f32::EPSILON
            || (hsv.v - self.hsv.v).abs() > f32::EPSILON
            || (alpha - self.opacity).abs() > f32::EPSILON
        {
            self.hsv = hsv;
            self.opacity = alpha;
            self.fire();
        }
    }

    /// Currently selected color.
    pub fn selected(&self) -> Color {
        hsva_to_color(self.hsv, self.opacity)
    }

    /// Currently selected HSV.
    pub fn hsv_value(&self) -> Hsv {
        self.hsv
    }

    /// Live theme for the frost (labels stay panel-fixed).
    pub fn set_theme(&mut self, mode: ThemeMode, glass: GlassAmount) {
        self.dark = mode == ThemeMode::Dark;
        self.glass.set_theme(mode, glass);
        self.dirty = true;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.glass.set_focused(focused);
    }

    /// Viewport the card centers in.
    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.vx = x;
        self.vy = y;
        self.vw = w;
        self.vh = h;
    }

    /// Trigger the popup (from a button or the app).
    pub fn show(&mut self) {
        self.visible = true;
        self.drag = None;
    }

    /// Close the popup, keeping the selection.
    pub fn dismiss(&mut self) {
        self.visible = false;
        self.drag = None;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Panel width in logical px (fixed geometry).
    pub fn panel_width() -> f32 {
        PICKER_PAD * 2.0 + PICKER_WHEEL
    }

    /// Panel height in logical px (fixed geometry).
    pub fn panel_height() -> f32 {
        PICKER_PAD + PICKER_WHEEL + PICKER_GAP + PICKER_BAR_H + PICKER_LABEL_GAP
            + PICKER_LABEL_SIZE * 1.25
            + PICKER_ROW_GAP
            + PICKER_BAR_H
            + PICKER_PAD
    }

    /// Last drawn panel origin plus fixed size (informational).
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, Self::panel_width(), Self::panel_height())
    }

    /// Panel rect (x, y, width, height), centered in the viewport.
    /// Pure helper for hit testing and tests.
    pub fn card_rect(&self) -> (f32, f32, f32, f32) {
        let (w, h) = (Self::panel_width(), Self::panel_height());
        (
            self.vx + (self.vw - w) / 2.0,
            self.vy + (self.vh - h) / 2.0,
            w,
            h,
        )
    }

    /// Wheel center and radius in logical px.
    pub fn wheel_rect(&self) -> (f32, f32, f32) {
        let (x, y, _, _) = self.card_rect();
        (
            x + PICKER_PAD + PICKER_WHEEL / 2.0,
            y + PICKER_PAD + PICKER_WHEEL / 2.0,
            PICKER_WHEEL / 2.0,
        )
    }

    /// Brightness bar rect in logical px.
    pub fn brightness_rect(&self) -> (f32, f32, f32, f32) {
        let (x, y, w, _) = self.card_rect();
        (
            x + PICKER_PAD,
            y + PICKER_PAD + PICKER_WHEEL + PICKER_GAP,
            w - PICKER_PAD * 2.0,
            PICKER_BAR_H,
        )
    }

    /// Opacity bar rect in logical px.
    pub fn opacity_rect(&self) -> (f32, f32, f32, f32) {
        let (x, y, w, _) = self.card_rect();
        let oy = y + PICKER_PAD + PICKER_WHEEL + PICKER_GAP + PICKER_BAR_H
            + PICKER_LABEL_GAP
            + PICKER_LABEL_SIZE * 1.25
            + PICKER_ROW_GAP;
        (
            x + PICKER_PAD,
            oy,
            w - PICKER_PAD * 2.0 - PICKER_PILL_W - 8.0,
            PICKER_BAR_H,
        )
    }

    /// Map a wheel point to hue/saturation (clamped inside).
    pub fn point_to_hs(&self, x: f32, y: f32) -> (f32, f32) {
        let (cx, cy, r) = self.wheel_rect();
        let (dx, dy) = ((x - cx) / r, (y - cy) / r);
        let dist = dx.hypot(dy).min(1.0);
        let hue = (dy.atan2(dx) / std::f32::consts::TAU).rem_euclid(1.0);
        (hue, dist)
    }

    /// Selected point on the wheel in logical px.
    pub fn hs_point(&self) -> (f32, f32) {
        let (cx, cy, r) = self.wheel_rect();
        let angle = self.hsv.h * std::f32::consts::TAU;
        (cx + angle.cos() * self.hsv.s * r, cy + angle.sin() * self.hsv.s * r)
    }

    fn fire(&mut self) {
        let color = self.selected();
        self.pct_text = format!("{} %", (self.opacity * 100.0).round() as u32);
        self.dirty = true;
        if let Some(callback) = self.on_change.as_mut() {
            callback(color);
        }
    }

    fn in_rect(x: f32, y: f32, rect: (f32, f32, f32, f32)) -> bool {
        let (rx, ry, rw, rh) = rect;
        x >= rx && x <= rx + rw && y >= ry && y <= ry + rh
    }

    fn in_wheel(&self, x: f32, y: f32) -> bool {
        let (cx, cy, r) = self.wheel_rect();
        (x - cx).hypot(y - cy) <= r
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if !self.visible {
            return;
        }
        let (x, y) = (x as f32, y as f32);
        if self.in_wheel(x, y) {
            self.drag = Some(DragTarget::Wheel);
            let (h, s) = self.point_to_hs(x, y);
            self.hsv.h = h;
            self.hsv.s = s;
            self.fire();
        } else if Self::in_rect(x, y, self.brightness_rect()) {
            self.drag = Some(DragTarget::Brightness);
            let (bx, _, bw, _) = self.brightness_rect();
            self.hsv.v = ((x - bx) / bw).clamp(0.0, 1.0);
            self.fire();
        } else if Self::in_rect(x, y, self.opacity_rect()) {
            self.drag = Some(DragTarget::Opacity);
            let (ox, _, ow, _) = self.opacity_rect();
            self.opacity = ((x - ox) / ow).clamp(0.0, 1.0);
            self.fire();
        } else if !Self::in_rect(x, y, self.card_rect()) {
            // Outside click dismisses, keeping the selection.
            self.dismiss();
        }
    }

    pub fn mouse_move(&mut self, x: f64, y: f64) {
        let (x, y) = (x as f32, y as f32);
        match self.drag {
            Some(DragTarget::Wheel) => {
                let (h, s) = self.point_to_hs(x, y);
                self.hsv.h = h;
                self.hsv.s = s;
                self.fire();
            }
            Some(DragTarget::Brightness) => {
                let (bx, _, bw, _) = self.brightness_rect();
                self.hsv.v = ((x - bx) / bw).clamp(0.0, 1.0);
                self.fire();
            }
            Some(DragTarget::Opacity) => {
                let (ox, _, ow, _) = self.opacity_rect();
                self.opacity = ((x - ox) / ow).clamp(0.0, 1.0);
                self.fire();
            }
            None => {}
        }
    }

    pub fn mouse_up(&mut self, _x: f64, _y: f64) {
        self.drag = None;
    }

    fn ensure_labels(&mut self, fonts: &mut FontSystem) {
        if !self.dirty
            && self.pct_layout.is_some()
            && self.label_layout.is_some()
            && self.layout_scale == fonts.scale
        {
            return;
        }
        let label = if self.dark {
            PICKER_LABEL_GRAY
        } else {
            Color::from_rgba8(0x27, 0x27, 0x27, 160)
        };
        self.label_layout = Some(fonts.layout_text(
            "Opacity",
            PICKER_LABEL_SIZE,
            label,
            None,
        ));
        self.pct_layout = Some(fonts.layout_text(
            &self.pct_text,
            PICKER_LABEL_SIZE,
            Color::WHITE,
            None,
        ));
        self.layout_scale = fonts.scale;
        self.dirty = false;
    }

    fn eff(&self, color: Color) -> Color {
        if self.focused {
            color
        } else {
            desaturate(color)
        }
    }
}

impl Default for ColorPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl View for ColorPicker {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        (Self::panel_width(), Self::panel_height())
    }

    fn place(&mut self, _fonts: &mut FontSystem, x: f32, y: f32, _w: f32, _h: f32) {
        // Placement is informational: the panel centers in the
        // viewport set via `set_viewport` on every draw.
        self.x = x;
        self.y = y;
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        if !self.visible || self.vw <= 0.0 || self.vh <= 0.0 {
            return;
        }
        if images.is_capture_pass() {
            // Backdrop capture: skip the whole popup so the blur sees
            // only what sits behind the frost.
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let (x, y, w, h) = self.card_rect();
        self.x = x;
        self.y = y;
        self.glass.set_bounds(x, y, w, h);
        self.glass.set_radius(PICKER_RADIUS);
        self.glass.draw(scene, fonts, images);

        // Hue wheel: baked texture clipped to the disc.
        let (cx, cy, r) = self.wheel_rect();
        let target = (PICKER_WHEEL * fonts.scale * 2.0).ceil().max(1.0) as u32;
        if let Some((image, iw, _)) = images.raster("tontooui.huewheel", wheel_png(), target) {
            let disc = Circle::new((px(cx), px(cy)), px(r));
            scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &disc);
            let s = (PICKER_WHEEL / iw as f32) as f64 * scale;
            let transform = Affine::translate((
                (cx - PICKER_WHEEL / 2.0) as f64 * scale,
                (cy - PICKER_WHEEL / 2.0) as f64 * scale,
            )) * Affine::scale(s);
            scene.draw_image(&image, transform);
            scene.pop_layer();
        }
        // Crosshair at the selection.
        let (hx, hy) = self.hs_point();
        let ring = Circle::new((px(hx), px(hy)), px(11.0));
        scene.stroke(
            &Stroke::new(px(2.0)),
            Affine::IDENTITY,
            &Brush::Solid(Color::from_rgba8(0, 0, 0, 170)),
            None,
            &ring,
        );
        for horizontal in [true, false] {
            let line = if horizontal {
                Line::new((px(hx - 15.0), px(hy)), (px(hx + 15.0), px(hy)))
            } else {
                Line::new((px(hx), px(hy - 15.0)), (px(hx), px(hy + 15.0)))
            };
            scene.stroke(
                &Stroke::new(px(2.0)),
                Affine::IDENTITY,
                &Brush::Solid(Color::from_rgba8(0, 0, 0, 170)),
                None,
                &line,
            );
        }

        // Brightness bar: full color into black, ring knob.
        let (bx, by, bw, bh) = self.brightness_rect();
        let full = hsva_to_color(
            Hsv { h: self.hsv.h, s: self.hsv.s, v: 1.0 },
            1.0,
        );
        let bar = RoundedRect::new(px(bx), px(by), px(bx + bw), px(by + bh), px(bh / 2.0));
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Gradient(
                Gradient::new_linear(Point::new(px(bx), px(by)), Point::new(px(bx + bw), px(by)))
                    .with_stops([
                        ColorStop { offset: 0.0, color: full.into() },
                        ColorStop { offset: 1.0, color: Color::BLACK.into() },
                    ]),
            ),
            None,
            &bar,
        );
        knob(scene, scale, bx + self.hsv.v * bw, by + bh / 2.0, bh / 2.0 - 2.0);

        // Opacity label and checker transparency bar with percent pill.
        self.ensure_labels(fonts);
        if let Some(layout) = self.label_layout.as_ref() {
            let (_, th) = FontSystem::layout_size(layout);
            draw_layout(scene, layout, x + PICKER_PAD, by + bh + PICKER_LABEL_GAP, fonts.scale);
            let _ = th;
        }
        let (ox, oy, ow, oh) = self.opacity_rect();
        let obar = RoundedRect::new(px(ox), px(oy), px(ox + ow), px(oy + oh), px(oh / 2.0));
        // Checkerboard base.
        scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &obar);
        let mut row = 0;
        let mut yy = oy;
        while yy < oy + oh {
            let mut xx = ox;
            let mut col = row;
            while xx < ox + ow {
                let fill = if col % 2 == 0 { PICKER_CHECK_A } else { PICKER_CHECK_B };
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(fill),
                    None,
                    &Rect::new(
                        px(xx),
                        px(yy),
                        px((xx + PICKER_CHECK).min(ox + ow)),
                        px((yy + PICKER_CHECK).min(oy + oh)),
                    ),
                );
                xx += PICKER_CHECK;
                col += 1;
            }
            yy += PICKER_CHECK;
            row += 1;
        }
        // Selected color veil: transparent left, opaque right.
        let solid = hsva_to_color(self.hsv, 1.0).to_rgba8();
        let veil = self.eff(Color::from_rgba8(solid.r, solid.g, solid.b, 255));
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Gradient(
                Gradient::new_linear(Point::new(px(ox), px(oy)), Point::new(px(ox + ow), px(oy)))
                    .with_stops([
                        ColorStop { offset: 0.0, color: Color::TRANSPARENT.into() },
                        ColorStop { offset: 1.0, color: veil.into() },
                    ]),
            ),
            None,
            &obar,
        );
        scene.pop_layer();
        knob(scene, scale, ox + self.opacity * ow, oy + oh / 2.0, oh / 2.0 - 2.0);

        // Percent pill on the row's right end.
        let pill = RoundedRect::new(
            px(x + w - PICKER_PAD - PICKER_PILL_W),
            px(oy),
            px(x + w - PICKER_PAD),
            px(oy + oh),
            px(8.0),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(PICKER_PILL)),
            None,
            &pill,
        );
        if let Some(layout) = self.pct_layout.as_ref() {
            let (tw, th) = FontSystem::layout_size(layout);
            draw_layout(
                scene,
                layout,
                x + w - PICKER_PAD - PICKER_PILL_W + (PICKER_PILL_W - tw / fonts.scale) / 2.0,
                oy + (oh - th / fonts.scale) / 2.0,
                fonts.scale,
            );
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Ring knob: dark outline plus white core.
fn knob(scene: &mut Scene, scale: f64, cx: f32, cy: f32, r: f32) {    let px = |v: f32| v as f64 * scale;
    scene.stroke(
        &Stroke::new(px(2.0)),
        Affine::IDENTITY,
        &Brush::Solid(Color::from_rgba8(0, 0, 0, 150)),
        None,
        &Circle::new((px(cx), px(cy)), px(r + 1.5)),
    );
    scene.stroke(
        &Stroke::new(px(2.5)),
        Affine::IDENTITY,
        &Brush::Solid(Color::WHITE),
        None,
        &Circle::new((px(cx), px(cy)), px(r)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_round_trips_primaries() {
        for (rgb, expected_h) in [
            ((1.0, 0.0, 0.0), 0.0),
            ((0.0, 1.0, 0.0), 1.0 / 3.0),
            ((0.0, 0.0, 1.0), 2.0 / 3.0),
        ] {
            let hsv = rgb_to_hsv(rgb.0, rgb.1, rgb.2);
            assert!((hsv.h - expected_h).abs() < 1e-5, "{rgb:?}");
            assert!((hsv.s - 1.0).abs() < 1e-6);
            assert!((hsv.v - 1.0).abs() < 1e-6);
            let back = hsv_to_rgb(hsv.h, hsv.s, hsv.v);
            assert!((back.0 - rgb.0).abs() < 1e-5);
            assert!((back.1 - rgb.1).abs() < 1e-5);
            assert!((back.2 - rgb.2).abs() < 1e-5);
        }
    }

    #[test]
    fn gray_has_no_saturation() {
        let hsv = rgb_to_hsv(0.5, 0.5, 0.5);
        assert_eq!(hsv.s, 0.0);
        assert!((hsv.v - 0.5).abs() < 1e-6);
    }

    #[test]
    fn color_round_trips_with_alpha() {
        let color = Color::from_rgba8(0x30, 0xb0, 0xc7, 200);
        let (hsv, alpha) = color_to_hsva(color);
        let back = hsva_to_color(hsv, alpha);
        assert_eq!(back.to_rgba8(), color.to_rgba8());
    }

    #[test]
    fn wheel_point_maps_hue_and_saturation() {
        let mut picker = ColorPicker::new();
        picker.set_viewport(0.0, 0.0, 800.0, 600.0);
        let (cx, cy, r) = picker.wheel_rect();
        // Right edge: hue 0 (red), full saturation.
        let (h, s) = picker.point_to_hs(cx + r, cy);
        assert!(h.abs() < 1e-5, "{h}");
        assert!((s - 1.0).abs() < 1e-6);
        // Center: zero saturation.
        assert_eq!(picker.point_to_hs(cx, cy).1, 0.0);
    }

    #[test]
    fn wheel_press_selects_and_reports() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let seen: Rc<RefCell<Option<Color>>> = Rc::new(RefCell::new(None));
        let out = seen.clone();
        let mut picker = ColorPicker::new().on_change(move |c| {
            *out.borrow_mut() = Some(c);
        });
        picker.set_viewport(0.0, 0.0, 800.0, 600.0);
        picker.show();
        let (cx, cy, r) = picker.wheel_rect();
        picker.mouse_down((cx + r) as f64, cy as f64);
        // Right edge at full brightness: pure red, opaque.
        let picked = seen.borrow().expect("reported").to_rgba8();
        assert_eq!(picked, Color::from_rgb8(255, 0, 0).to_rgba8());
        assert_eq!(picker.hsv_value().s, 1.0);
    }

    #[test]
    fn sliders_map_linearly() {
        let mut picker = ColorPicker::new();
        picker.set_viewport(0.0, 0.0, 800.0, 600.0);
        picker.show();
        let (bx, by, bw, bh) = picker.brightness_rect();
        picker.mouse_down((bx + bw / 2.0) as f64, (by + bh / 2.0) as f64);
        assert!((picker.hsv_value().v - 0.5).abs() < 1e-6);
        let (ox, oy, ow, oh) = picker.opacity_rect();
        picker.mouse_down((ox + ow / 4.0) as f64, (oy + oh / 2.0) as f64);
        assert!((picker.selected().to_rgba8().a as f32 - 255.0 * 0.25).abs() < 2.0);
    }

    #[test]
    fn outside_click_dismisses_keeping_selection() {
        let mut picker = ColorPicker::new();
        picker.set_viewport(0.0, 0.0, 800.0, 600.0);
        picker.show();
        assert!(picker.is_visible());
        picker.mouse_down(790.0, 590.0);
        assert!(!picker.is_visible());
    }

    #[test]
    fn hidden_picker_ignores_input() {
        let mut picker = ColorPicker::new();
        picker.set_viewport(0.0, 0.0, 800.0, 600.0);
        let before = picker.selected();
        let (cx, cy, r) = picker.wheel_rect();
        picker.mouse_down((cx + r) as f64, cy as f64);
        assert_eq!(picker.selected().to_rgba8(), before.to_rgba8());
    }
}
