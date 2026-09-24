use std::any::Any;
use std::time::Instant;

use vello::Scene;
use vello::kurbo::{Affine, BezPath, Cap, Join, Rect, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::layout::View;
use crate::animation::{Animatable, Easing, Repeat, Tween, TweenAnim};
use crate::renderer::backdrop::fill_backdrop;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::{FontSystem, draw_layout};
use crate::theme::desaturate;

use super::super::buttons::{
    BUTTON_BG_DARK, BUTTON_BG_LIGHT, BUTTON_FONT_SIZE, BUTTON_GAP, BUTTON_ICON_SIZE,
    BUTTON_PAD_X, BUTTON_PAD_Y, BUTTON_RADIUS,
};

/// Switch track width in logical px (stretched long and slim,
/// macOS style).
pub const TOGGLE_SWITCH_W: f32 = 64.0;
/// Switch track height in logical px (three quarters of the iOS
/// measure).
pub const TOGGLE_SWITCH_H: f32 = 22.32;
/// Switch knob padding inside the track in logical px (macOS measure:
/// the knob nearly fills the track height).
pub const TOGGLE_KNOB_PAD: f32 = 2.0;
/// Switch knob width relative to its height (macOS measure: the knob is
/// a wide capsule, wider than tall).
pub const TOGGLE_KNOB_W_RATIO: f32 = 1.35;
/// Switch knob growth per side while held: the white knob turns liquid
/// glass and overflows the track, like the slider knob.
pub const TOGGLE_KNOB_EXPAND: f32 = 3.0;
/// Knob slide animation time in seconds.
pub const TOGGLE_ANIM_SECONDS: f32 = 0.20;
/// Drag-release snap time in seconds.
pub const TOGGLE_SNAP_SECONDS: f32 = 0.15;
/// Checkbox box size in logical px.
pub const TOGGLE_BOX: f32 = 21.12;
/// Checkbox corner radius in logical px.
pub const TOGGLE_BOX_RADIUS: f32 = 5.76;
/// Leading settings-row icon badge size in logical px.
pub const TOGGLE_ICON_BOX: f32 = 26.88;
/// Leading icon badge corner radius in logical px.
pub const TOGGLE_ICON_RADIUS: f32 = 6.72;
/// Glyph box inside the leading icon badge in logical px.
pub const TOGGLE_ICON_GLYPH: f32 = 15.36;
/// Row label size in logical px.
pub const TOGGLE_LABEL_SIZE: f32 = 17.0;
/// Gap between badge, box/control and label in logical px.
pub const TOGGLE_GAP: f32 = 8.0;
/// Default on-color (theme accent blue, same as sliders).
pub const TOGGLE_ACCENT: Color = Color::from_rgb8(0x00, 0x7a, 0xff);
/// Off track/box fill for light mode.
pub const TOGGLE_OFF_LIGHT: Color = Color::from_rgb8(0xe5, 0xe5, 0xe5);
/// Off track/box fill for dark mode.
pub const TOGGLE_OFF_DARK: Color = Color::from_rgb8(0x3a, 0x3a, 0x3c);

/// Toggle presentation (SwiftUI `toggleStyle`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleStyle {
    /// iOS switch stretched long and slim: 64x22.32 track, wide
    /// capsule knob, generous stretch of track beside the knob.
    #[default]
    Switch,
    /// Rounded button: gray fill when off, accent fill with white text
    /// when on.
    Button,
    /// Rounded box with checkmark: gray when off, accent with white
    /// check when on.
    Checkbox,
}

/// Boolean toggle in switch, button or checkbox style: optional label,
/// optional leading SF Symbol (CoreIcon) badge for switch/checkbox rows
/// (inside the button for button style), accent on-color from the system
/// theme unless set manually, and an `on_toggle` callback.
///
/// Clicks toggle on release inside the row (`mouse_down` arms,
/// `View::mouse_up` fires); the shell forwards both. Missing icons draw
/// the row without badge.
pub struct Toggle {
    label: String,
    icon: Option<String>,
    style: ToggleStyle,
    on: bool,
    shown: f32,
    on_color: Color,
    on_manual: bool,
    dark: bool,
    text_color: Color,
    badge_bg: Color,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    sx: f32,
    sy: f32,
    bx: f32,
    by: f32,
    badge_x: f32,
    badge_y: f32,
    label_x: f32,
    label_y: f32,
    armed: bool,
    hovered: bool,
    dragging: bool,
    down_x: f32,
    dragged: bool,
    disabled: bool,
    focused: bool,
    anim: Option<TweenAnim<f32>>,
    anim_time: f32,
    last_draw: Option<Instant>,
    on_toggle: Option<Box<dyn FnMut(bool)>>,
}

impl Toggle {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            style: ToggleStyle::Switch,
            on: false,
            shown: 0.0,
            on_color: TOGGLE_ACCENT,
            on_manual: false,
            dark: true,
            text_color: Color::WHITE,
            badge_bg: BUTTON_BG_DARK,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            sx: 0.0,
            sy: 0.0,
            bx: 0.0,
            by: 0.0,
            badge_x: 0.0,
            badge_y: 0.0,
            label_x: 0.0,
            label_y: 0.0,
            armed: false,
            hovered: false,
            dragging: false,
            down_x: 0.0,
            dragged: false,
            disabled: false,
            focused: true,
            anim: None,
            anim_time: 0.0,
            last_draw: None,
            on_toggle: None,
        }
    }

    pub fn style(mut self, style: ToggleStyle) -> Self {
        self.style = style;
        self
    }

    /// Initial state without animation and without firing `on_toggle`.
    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self.shown = if on { 1.0 } else { 0.0 };
        self.anim = None;
        self
    }

    /// SF Symbol name for the leading badge (switch/checkbox) or the
    /// in-button glyph (button style), e.g. `"airplane"`, `"wifi"`.
    pub fn icon(mut self, name: impl Into<String>) -> Self {
        self.icon = Some(name.into());
        self
    }

    /// Manual on-color: wins over the system accent until cleared. The
    /// on-color follows the system accent unless the dev sets it by hand.
    pub fn fill(mut self, color: Color) -> Self {
        self.on_color = color;
        self.on_manual = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_toggle(mut self, callback: impl FnMut(bool) + 'static) -> Self {
        self.on_toggle = Some(Box::new(callback));
        self
    }

    /// Live theme: accent on-color, mode grays, badge background. A
    /// manually set fill wins over the system accent.
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        if !self.on_manual {
            self.on_color = accent;
        }
        self.dark = dark;
        if dark {
            self.text_color = Color::WHITE;
            self.badge_bg = BUTTON_BG_DARK;
        } else {
            self.text_color = Color::BLACK;
            self.badge_bg = BUTTON_BG_LIGHT;
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    pub fn is_on(&self) -> bool {
        self.on
    }

    /// True while the switch knob is held and follows the pointer.
    /// Apps use this to return true from `App::wants_backdrop`.
    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    /// Set the state immediately (no animation). Fires `on_toggle` when
    /// the state changed.
    pub fn set_on(&mut self, on: bool) {
        if on != self.on {
            self.on = on;
            self.shown = if on { 1.0 } else { 0.0 };
            self.anim = None;
            self.notify();
        }
    }

    /// Flip the state with knob animation. Fires `on_toggle`.
    pub fn toggle(&mut self) {
        let on = !self.on;
        self.on = on;
        let target = if on { 1.0 } else { 0.0 };
        self.anim = Some(TweenAnim::new(
            Tween::new(self.shown, target, TOGGLE_ANIM_SECONDS)
                .easing(Easing::CubicOut)
                .repeat(Repeat::Never),
        ));
        self.anim_time = 0.0;
        self.notify();
    }

    fn notify(&mut self) {
        if let Some(callback) = self.on_toggle.as_mut() {
            callback(self.on);
        }
    }

    fn hit(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    fn switch_hit(&self, x: f32, y: f32) -> bool {
        x >= self.sx
            && x <= self.sx + TOGGLE_SWITCH_W
            && y >= self.sy
            && y <= self.sy + TOGGLE_SWITCH_H
    }

    fn knob_w() -> f32 {
        (TOGGLE_SWITCH_H - TOGGLE_KNOB_PAD * 2.0) * TOGGLE_KNOB_W_RATIO
    }

    fn travel() -> f32 {
        TOGGLE_SWITCH_W - TOGGLE_KNOB_PAD * 2.0 - Self::knob_w()
    }

    /// Knob position for a pointer x over the track, 0.0 (off) to 1.0
    /// (on). The knob center maps to the pointer so a grabbed knob does
    /// not jump.
    fn shown_at(&self, x: f32) -> f32 {
        let travel = Self::travel();
        if travel <= 0.0 {
            return if self.on { 1.0 } else { 0.0 };
        }
        ((x - (self.sx + TOGGLE_KNOB_PAD + Self::knob_w() / 2.0)) / travel).clamp(0.0, 1.0)
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        let (x, y) = (x as f32, y as f32);
        // Pressing the switch track starts a drag: the knob follows the
        // pointer until release. Everything else arms a plain click.
        if self.style == ToggleStyle::Switch && self.switch_hit(x, y) {
            self.dragging = true;
            self.down_x = x;
            self.dragged = false;
            self.armed = true;
            self.anim = None;
            self.shown = self.shown_at(x);
        } else if self.hit(x, y) {
            self.armed = true;
        }
    }

    pub fn mouse_move(&mut self, x: f64, y: f64) {
        let (x, y) = (x as f32, y as f32);
        self.hovered = self.hit(x, y);
        if self.dragging {
            if (x - self.down_x).abs() > 4.0 {
                self.dragged = true;
            }
            self.shown = self.shown_at(x);
        }
    }

    pub fn mouse_up(&mut self, x: f64, y: f64) {
        self.finish_up(x, y);
    }

    fn finish_up(&mut self, x: f64, y: f64) {
        if self.dragging {
            self.dragging = false;
            self.armed = false;
            if self.disabled {
                return;
            }
            if !self.dragged {
                // Tap on the track flips like a click.
                self.shown = if self.on { 1.0 } else { 0.0 };
                if self.switch_hit(x as f32, y as f32) {
                    self.toggle();
                }
                return;
            }
            // Dragged: snap to the nearer stop, firing when the state
            // changed.
            let target = self.shown > 0.5;
            if target != self.on {
                self.on = target;
                self.notify();
            }
            let end = if target { 1.0 } else { 0.0 };
            self.anim = Some(TweenAnim::new(
                Tween::new(self.shown, end, TOGGLE_SNAP_SECONDS)
                    .easing(Easing::CubicOut)
                    .repeat(Repeat::Never),
            ));
            self.anim_time = 0.0;
            return;
        }
        self.finish_click(x, y);
    }

    fn finish_click(&mut self, x: f64, y: f64) {
        let was_armed = self.armed;
        self.armed = false;
        if was_armed && !self.disabled && self.hit(x as f32, y as f32) {
            self.toggle();
        }
    }

    pub fn set_hover(&mut self, x: f32, y: f32) {
        self.hovered = self.hit(x, y);
    }

    fn advance(&mut self, now: Instant) {
        let dt = match self.last_draw {
            Some(last) => now.saturating_duration_since(last).as_secs_f32().min(0.1),
            None => 0.0,
        };
        self.last_draw = Some(now);
        if self.dragging {
            // While held the pointer owns `shown`; never snap it back to
            // the on/off endpoint or the knob would freeze mid-drag.
            return;
        }
        if let Some(anim) = self.anim.as_mut() {
            self.anim_time += dt;
            let done = anim.update(self.anim_time);
            self.shown = *anim.value();
            if done {
                self.anim = None;
                self.shown = if self.on { 1.0 } else { 0.0 };
            }
        } else {
            self.shown = if self.on { 1.0 } else { 0.0 };
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

    fn off_color(&self) -> Color {
        if self.dark {
            TOGGLE_OFF_DARK
        } else {
            TOGGLE_OFF_LIGHT
        }
    }

    /// Current track/box fill, crossfading gray to on-color with the knob.
    fn track_color(&self) -> Color {
        Animatable::lerp(&self.off_color(), &self.on_color, self.shown.clamp(0.0, 1.0))
    }

    fn label_size(&self, fonts: &mut FontSystem) -> (f32, f32) {
        if self.label.is_empty() {
            return (0.0, 0.0);
        }
        let size = match self.style {
            ToggleStyle::Button => BUTTON_FONT_SIZE,
            _ => TOGGLE_LABEL_SIZE,
        };
        let layout = fonts.layout_text(&self.label, size, Color::WHITE, None);
        let (tw, th) = FontSystem::layout_size(&layout);
        (tw / fonts.scale, th / fonts.scale)
    }

    fn render_switch(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        self.draw_badge(scene, images, fonts.scale);
        self.draw_label(scene, fonts, TOGGLE_LABEL_SIZE, 600.0);

        // Track crossfades gray to on-color while the knob slides.
        let track = RoundedRect::new(
            px(self.sx),
            px(self.sy),
            px(self.sx + TOGGLE_SWITCH_W),
            px(self.sy + TOGGLE_SWITCH_H),
            px(TOGGLE_SWITCH_H / 2.0),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.track_color())),
            None,
            &track,
        );

        // Capsule knob (macOS measure): nearly full track height and
        // wider than tall, sliding from the left stop to the right stop.
        // While held the knob turns liquid glass and grows past the track.
        let knob_h = TOGGLE_SWITCH_H - TOGGLE_KNOB_PAD * 2.0;
        let knob_w = Self::knob_w();
        let travel = Self::travel();
        let kx = self.sx + TOGGLE_KNOB_PAD + self.shown.clamp(0.0, 1.0) * travel;
        let ky = self.sy + TOGGLE_KNOB_PAD;
        let held = self.dragging && !self.disabled;
        let expand = if held { TOGGLE_KNOB_EXPAND } else { 0.0 };
        let kr = knob_h / 2.0 + expand;
        // Capture pass while held: leave the knob empty so the blur sees
        // the track behind the glass.
        let skip_knob = held && images.is_capture_pass();
        if !skip_knob {
            scene.draw_blurred_rounded_rect(
                Affine::IDENTITY,
                Rect::new(
                    px(kx - expand),
                    px(ky - expand),
                    px(kx + knob_w + expand),
                    px(ky + knob_h + expand),
                ),
                Color::from_rgba8(0, 0, 0, 40),
                px(kr),
                6.0 * scale,
            );
        }
        let knob = RoundedRect::new(
            px(kx - expand),
            px(ky - expand),
            px(kx + knob_w + expand),
            px(ky + knob_h + expand),
            px(kr),
        );
        if skip_knob {
            // Capture pass: knob body omitted for the backdrop blur.
        } else if held {
            fill_backdrop(scene, images, &knob);
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(Color::from_rgba8(255, 255, 255, 64)),
                None,
                &knob,
            );
            scene.stroke(
                &Stroke::new(1.5 * scale),
                Affine::IDENTITY,
                &Brush::Solid(Color::from_rgba8(255, 255, 255, 200)),
                None,
                &knob,
            );
        } else {
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(self.eff(Color::WHITE)),
                None,
                &knob,
            );
        }
    }

    fn render_checkbox(&mut self, scene: &mut Scene, fonts: &mut FontSystem) {
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let box_rect = RoundedRect::new(
            px(self.bx),
            px(self.by),
            px(self.bx + TOGGLE_BOX),
            px(self.by + TOGGLE_BOX),
            px(TOGGLE_BOX_RADIUS),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.track_color())),
            None,
            &box_rect,
        );
        if self.shown > 0.5 {
            // White checkmark, drawn with round caps.
            let mut path = BezPath::new();
            path.move_to((px(self.bx + 5.76), px(self.by + 11.04)));
            path.line_to((px(self.bx + 9.408), px(self.by + 14.4)));
            path.line_to((px(self.bx + 15.36), px(self.by + 7.68)));
            let mut stroke = Stroke::new(2.4 * scale);
            stroke.start_cap = Cap::Round;
            stroke.end_cap = Cap::Round;
            stroke.join = Join::Round;
            scene.stroke(
                &stroke,
                Affine::IDENTITY,
                &Brush::Solid(self.eff(Color::WHITE)),
                None,
                &path,
            );
        }
        self.draw_label(scene, fonts, TOGGLE_LABEL_SIZE, 400.0);
    }

    fn render_button(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        images: &mut ImageLoader<'_>,
    ) {
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let bg = if self.on {
            self.on_color
        } else {
            self.badge_bg
        };
        let text = if self.on {
            Color::WHITE
        } else {
            self.text_color
        };
        let shape = RoundedRect::new(
            px(self.x),
            px(self.y),
            px(self.x + self.width),
            px(self.y + self.height),
            px(BUTTON_RADIUS),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(bg)),
            None,
            &shape,
        );
        // Press lightens in dark mode, darkens in light mode.
        let (press, hover) = if self.dark {
            (
                Color::from_rgba8(255, 255, 255, 36),
                Color::from_rgba8(255, 255, 255, 20),
            )
        } else {
            (
                Color::from_rgba8(0, 0, 0, 36),
                Color::from_rgba8(0, 0, 0, 15),
            )
        };
        if self.armed && !self.disabled {
            scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(press), None, &shape);
        } else if self.hovered && !self.disabled {
            scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(hover), None, &shape);
        }

        let layout = fonts.layout_text(&self.label, BUTTON_FONT_SIZE, self.eff(text), None);
        let (tw, th) = FontSystem::layout_size(&layout);
        let tw = tw / fonts.scale;
        let th = th / fonts.scale;
        let has_icon = self.icon.is_some();
        let icon_box = if has_icon { BUTTON_ICON_SIZE } else { 0.0 };
        let gap = if has_icon { BUTTON_GAP } else { 0.0 };
        let content_w = icon_box + gap + tw;
        let mut cx = self.x + (self.width - content_w) / 2.0;
        let cy = self.y + (self.height - th.max(BUTTON_ICON_SIZE)) / 2.0;
        if let Some(name) = self.icon.clone() {
            let target = (BUTTON_ICON_SIZE * fonts.scale * 2.0).ceil().max(1.0) as u32;
            if let Some((image, iw, ih)) = images.get(&name, self.eff(text), target) {
                let s = (BUTTON_ICON_SIZE / iw as f32).min(BUTTON_ICON_SIZE / ih as f32);
                let ix = cx + (BUTTON_ICON_SIZE - iw as f32 * s) / 2.0;
                let iy = cy + (th.max(BUTTON_ICON_SIZE) - ih as f32 * s) / 2.0;
                let transform = Affine::translate((ix as f64 * scale, iy as f64 * scale))
                    * Affine::scale(s as f64 * scale);
                scene.draw_image(&image, transform);
            }
            cx += icon_box + gap;
        }
        draw_layout(scene, &layout, cx, self.y + (self.height - th) / 2.0, fonts.scale);
    }

    /// Leading settings-row badge (switch/checkbox rows only): gray
    /// rounded square with the SF Symbol tinted to the label color.
    fn draw_badge(&mut self, scene: &mut Scene, images: &mut ImageLoader<'_>, glyph_scale: f32) {
        let Some(name) = self.icon.clone() else {
            return;
        };
        if self.style == ToggleStyle::Button {
            return;
        }
        let scale = glyph_scale as f64;
        let px = |v: f32| v as f64 * scale;
        let badge = RoundedRect::new(
            px(self.badge_x),
            px(self.badge_y),
            px(self.badge_x + TOGGLE_ICON_BOX),
            px(self.badge_y + TOGGLE_ICON_BOX),
            px(TOGGLE_ICON_RADIUS),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.eff(self.badge_bg)),
            None,
            &badge,
        );
        let target = (TOGGLE_ICON_GLYPH * glyph_scale * 2.0).ceil().max(1.0) as u32;
        if let Some((image, iw, ih)) = images.get(&name, self.eff(self.text_color), target) {
            let s = (TOGGLE_ICON_GLYPH / iw as f32).min(TOGGLE_ICON_GLYPH / ih as f32);
            let ix = self.badge_x + (TOGGLE_ICON_BOX - iw as f32 * s) / 2.0;
            let iy = self.badge_y + (TOGGLE_ICON_BOX - ih as f32 * s) / 2.0;
            let transform =
                Affine::translate((ix as f64 * scale, iy as f64 * scale)) * Affine::scale(s as f64 * scale);
            scene.draw_image(&image, transform);
        }
    }

    fn draw_label(&self, scene: &mut Scene, fonts: &mut FontSystem, size: f32, weight: f32) {
        if self.label.is_empty() {
            return;
        }
        let layout = fonts.layout_text_weighted(&self.label, size, self.eff(self.text_color), weight, None);
        draw_layout(scene, &layout, self.label_x, self.label_y, fonts.scale);
    }
}

impl View for Toggle {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        let (label_w, label_h) = self.label_size(fonts);
        let badge_w = if self.icon.is_some() && self.style != ToggleStyle::Button {
            TOGGLE_ICON_BOX + TOGGLE_GAP
        } else {
            0.0
        };
        match self.style {
            ToggleStyle::Switch => {
                let label_part = if label_w > 0.0 { label_w + TOGGLE_GAP } else { 0.0 };
                (
                    badge_w + label_part + TOGGLE_SWITCH_W,
                    TOGGLE_SWITCH_H.max(label_h).max(if self.icon.is_some() {
                        TOGGLE_ICON_BOX
                    } else {
                        0.0
                    }),
                )
            }
            ToggleStyle::Checkbox => {
                let label_part = if label_w > 0.0 { TOGGLE_GAP + label_w } else { 0.0 };
                (
                    badge_w + TOGGLE_BOX + label_part,
                    TOGGLE_BOX.max(label_h).max(if self.icon.is_some() {
                        TOGGLE_ICON_BOX
                    } else {
                        0.0
                    }),
                )
            }
            ToggleStyle::Button => {
                let has_icon = self.icon.is_some();
                let icon_box = if has_icon { BUTTON_ICON_SIZE } else { 0.0 };
                let gap = if has_icon { BUTTON_GAP } else { 0.0 };
                (
                    label_w + icon_box + gap + BUTTON_PAD_X * 2.0,
                    label_h.max(BUTTON_ICON_SIZE) + BUTTON_PAD_Y * 2.0,
                )
            }
        }
    }

    fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w;
        self.height = h;
        let (_, label_h) = self.label_size(fonts);
        match self.style {
            ToggleStyle::Switch => {
                self.sx = x + w - TOGGLE_SWITCH_W;
                self.sy = y + (h - TOGGLE_SWITCH_H) / 2.0;
                let mut cx = x;
                if self.icon.is_some() {
                    self.badge_x = cx;
                    self.badge_y = y + (h - TOGGLE_ICON_BOX) / 2.0;
                    cx += TOGGLE_ICON_BOX + TOGGLE_GAP;
                }
                self.label_x = cx;
                self.label_y = y + (h - label_h) / 2.0;
            }
            ToggleStyle::Checkbox => {
                let mut cx = x;
                if self.icon.is_some() {
                    self.badge_x = cx;
                    self.badge_y = y + (h - TOGGLE_ICON_BOX) / 2.0;
                    cx += TOGGLE_ICON_BOX + TOGGLE_GAP;
                }
                self.bx = cx;
                self.by = y + (h - TOGGLE_BOX) / 2.0;
                self.label_x = cx + TOGGLE_BOX + TOGGLE_GAP;
                self.label_y = y + (h - label_h) / 2.0;
            }
            ToggleStyle::Button => {}
        }
    }

    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, images: &mut ImageLoader<'_>) {
        self.advance(Instant::now());
        match self.style {
            ToggleStyle::Switch => self.render_switch(scene, fonts, images),
            ToggleStyle::Checkbox => self.render_checkbox(scene, fonts),
            ToggleStyle::Button => self.render_button(scene, fonts, images),
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

    #[test]
    fn switch_measures_scaled_ios_size_without_label() {
        let mut toggle = Toggle::new("");
        let mut fonts = FontSystem::new();
        let (w, h) = toggle.measure(&mut fonts);
        assert_eq!((w, h), (TOGGLE_SWITCH_W, TOGGLE_SWITCH_H));
        assert_eq!((TOGGLE_SWITCH_W, TOGGLE_SWITCH_H), (64.0, 22.32));
    }

    #[test]
    fn knob_is_macos_capsule() {
        let knob_h = TOGGLE_SWITCH_H - TOGGLE_KNOB_PAD * 2.0;
        let knob_w = knob_h * TOGGLE_KNOB_W_RATIO;
        // Capsule: wider than tall, nearly full track height.
        assert!(knob_w > knob_h);
        assert!(knob_h / TOGGLE_SWITCH_H > 0.8);
        // Positive travel with symmetric stops.
        let travel = TOGGLE_SWITCH_W - TOGGLE_KNOB_PAD * 2.0 - knob_w;
        assert!(travel > 0.0);
    }

    #[test]
    fn dragging_survives_frame_advance() {
        // Regression: draw() must not reset the held knob to the on/off
        // endpoint, or dragging left would do nothing and snap back on.
        let mut toggle = Toggle::new("Wi-Fi").on(true);
        let mut fonts = FontSystem::new();
        let (w, _) = toggle.measure(&mut fonts);
        toggle.place(&mut fonts, 0.0, 0.0, w, TOGGLE_SWITCH_H);
        let left = (w - TOGGLE_SWITCH_W + 3.0) as f64;
        let right = (w - 3.0) as f64;
        toggle.mouse_down(right, 14.0);
        toggle.mouse_move(left, 14.0);
        let held = toggle.shown;
        assert!(held < 0.5);
        toggle.advance(Instant::now());
        assert_eq!(toggle.shown, held);
        toggle.mouse_up(left, 14.0);
        assert!(!toggle.is_on());
    }

    #[test]
    fn track_press_holds_dragging_state() {
        let mut toggle = Toggle::new("Wi-Fi");
        let mut fonts = FontSystem::new();
        let (w, _) = toggle.measure(&mut fonts);
        toggle.place(&mut fonts, 0.0, 0.0, w, TOGGLE_SWITCH_H);
        assert!(!toggle.dragging);
        toggle.mouse_down((w - 10.0) as f64, 14.0);
        assert!(toggle.dragging);
        toggle.mouse_up((w - 10.0) as f64, 14.0);
        assert!(!toggle.dragging);
    }

    #[test]
    fn track_tap_flips_without_drag() {
        let mut toggle = Toggle::new("Wi-Fi");
        let mut fonts = FontSystem::new();
        let (w, h) = toggle.measure(&mut fonts);
        toggle.place(&mut fonts, 0.0, 0.0, w, h);
        // Press and release on the right end of the track without moving.
        let x = (w - 3.0) as f64;
        toggle.mouse_down(x, 14.0);
        toggle.mouse_up(x, 14.0);
        assert!(toggle.is_on());
    }

    #[test]
    fn drag_slides_knob_and_snaps_on_release() {
        let mut toggle = Toggle::new("Wi-Fi");
        let mut fonts = FontSystem::new();
        let (w, _) = toggle.measure(&mut fonts);
        toggle.place(&mut fonts, 0.0, 0.0, w, TOGGLE_SWITCH_H);
        let left = (w - TOGGLE_SWITCH_W + 3.0) as f64;
        let right = (w - 3.0) as f64;
        toggle.mouse_down(left, 14.0);
        toggle.mouse_move(right, 14.0);
        // Knob followed the pointer past halfway.
        assert!(toggle.shown > 0.5);
        toggle.mouse_up(right, 14.0);
        assert!(toggle.is_on());
        // Snap animation runs to the on stop.
        assert!(toggle.anim.is_some());
    }

    #[test]
    fn drag_below_half_snaps_back_off() {
        let mut toggle = Toggle::new("Wi-Fi").on(true);
        let mut fonts = FontSystem::new();
        let (w, _) = toggle.measure(&mut fonts);
        toggle.place(&mut fonts, 0.0, 0.0, w, TOGGLE_SWITCH_H);
        let left = (w - TOGGLE_SWITCH_W + 3.0) as f64;
        let right = (w - 3.0) as f64;
        toggle.mouse_down(right, 14.0);
        toggle.mouse_move(left, 14.0);
        assert!(toggle.shown < 0.5);
        toggle.mouse_up(left, 14.0);
        assert!(!toggle.is_on());
    }

    #[test]
    fn click_arms_and_toggles_on_release_inside() {
        let mut toggle = Toggle::new("Switch Style");
        let mut fonts = FontSystem::new();
        let (w, h) = toggle.measure(&mut fonts);
        toggle.place(&mut fonts, 0.0, 0.0, w, h);
        assert!(!toggle.is_on());
        // Release without press does nothing.
        toggle.mouse_up(10.0, 10.0);
        assert!(!toggle.is_on());
        // Press and release inside toggles on with animation running.
        toggle.mouse_down(10.0, 10.0);
        toggle.mouse_up(10.0, 10.0);
        assert!(toggle.is_on());
        assert!(toggle.anim.is_some());
        // Press inside, release outside keeps the state.
        toggle.mouse_down(10.0, 10.0);
        toggle.mouse_up(1000.0, 1000.0);
        assert!(toggle.is_on());
    }

    #[test]
    fn set_on_is_immediate_and_fires_callback() {
        let fires = Rc::new(Cell::new(0));
        let count = fires.clone();
        let mut toggle = Toggle::new("Wi-Fi").on_toggle(move |_| count.set(count.get() + 1));
        toggle.set_on(true);
        assert!(toggle.is_on());
        assert_eq!(toggle.shown, 1.0);
        assert!(toggle.anim.is_none());
        assert_eq!(fires.get(), 1);
        // Same value does not fire again.
        toggle.set_on(true);
        assert_eq!(fires.get(), 1);
    }

    #[test]
    fn manual_fill_wins_over_theme_accent() {
        let mut toggle = Toggle::new("Fixed").fill(Color::from_rgb8(0x34, 0xc7, 0x59));
        toggle.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        assert_eq!(toggle.on_color, Color::from_rgb8(0x34, 0xc7, 0x59));
        let mut plain = Toggle::new("Auto");
        plain.set_theme(Color::from_rgb8(0xff, 0x2d, 0x55), true);
        assert_eq!(plain.on_color, Color::from_rgb8(0xff, 0x2d, 0x55));
    }

    #[test]
    fn checkbox_row_is_box_plus_label() {
        let mut toggle = Toggle::new("Checkbox Style").style(ToggleStyle::Checkbox);
        let mut fonts = FontSystem::new();
        let (w, h) = toggle.measure(&mut fonts);
        assert!(w > TOGGLE_BOX);
        assert_eq!(h.max(TOGGLE_BOX), h);
    }
}
