use std::any::Any;
use std::time::Instant;

use vello::Scene;
use vello::kurbo::{Affine, RoundedRect};
use vello::peniko::{Brush, Color, Fill};

use super::layout::View;
use crate::animation::{Easing, Repeat, Tween, TweenAnim};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::theme::desaturate;

/// Idle thumb width in logical px. The thumb widens on hover.
pub const SCROLLBAR_W: f32 = 6.0;
/// Hover/drag thumb width in logical px.
pub const SCROLLBAR_W_HOVER: f32 = 10.0;
/// Minimum thumb length in logical px.
pub const SCROLLBAR_MIN_THUMB: f32 = 24.0;
/// Minimum track height in logical px (stack measure).
pub const SCROLLBAR_MIN_TRACK: f32 = 44.0;
/// Thumb base alpha (0-255); the fade multiplies it.
pub const SCROLLBAR_ALPHA: u8 = 180;
/// Fade in/out time in seconds.
pub const SCROLLBAR_FADE_SECONDS: f32 = 0.25;
/// Idle seconds after the last activity before fading out.
pub const SCROLLBAR_HIDE_DELAY: f32 = 1.0;
/// Track-click page jump time in seconds.
pub const SCROLLBAR_PAGE_ANIM_SECONDS: f32 = 0.25;
/// Hover lighten amount (0-1 toward white).
pub const SCROLLBAR_HOVER_LIGHTEN: f32 = 0.15;
/// Press darken amount (0-1 toward black).
pub const SCROLLBAR_PRESS_DARKEN: f32 = 0.12;
/// Default thumb gray.
pub const SCROLLBAR_GRAY: Color = Color::from_rgb8(0x8e, 0x8e, 0x93);
/// Track fill for dark mode (subtle, shown on hover).
pub const SCROLLBAR_TRACK_DARK: Color = Color::from_rgba8(255, 255, 255, 26);
/// Track fill for light mode (subtle, shown on hover).
pub const SCROLLBAR_TRACK_LIGHT: Color = Color::from_rgba8(0, 0, 0, 20);
/// Theme accent that maps to the gray default (Multicolor/Blue).
pub const SCROLLBAR_DEFAULT_ACCENT: Color = Color::from_rgb8(0x00, 0x7a, 0xff);
/// Thumb widen speed in logical px per second.
pub const SCROLLBAR_WIDEN_SPEED: f32 = 30.0;
/// Generous hit width for the thin thumb in logical px.
pub const SCROLLBAR_HIT_W: f32 = 14.0;

fn lighten(color: Color, amount: f32) -> Color {
    let c = color.to_rgba8();
    let t = amount.clamp(0.0, 1.0);
    let mix = |v: u8| (v as f32 + (255.0 - v as f32) * t).round() as u8;
    Color::from_rgba8(mix(c.r), mix(c.g), mix(c.b), c.a)
}

fn darken(color: Color, amount: f32) -> Color {
    let c = color.to_rgba8();
    let t = amount.clamp(0.0, 1.0);
    let mix = |v: u8| (v as f32 * (1.0 - t)).round() as u8;
    Color::from_rgba8(mix(c.r), mix(c.g), mix(c.b), c.a)
}

/// Overlay scrollbar for the side of scrollable content. The thumb is
/// hidden until needed: it fades in while scrolling, hovering or
/// dragging and fades out after `SCROLLBAR_HIDE_DELAY` idle seconds.
/// Hovering widens the thumb from `SCROLLBAR_W` to
/// `SCROLLBAR_W_HOVER` and reveals the full track from top to bottom,
/// so the whole travel range is visible while aiming.
///
/// Dragging the thumb follows the mouse directly; clicking the track
/// jumps one page (the visible amount) toward the click with a short
/// tween. The offset model is logical px like the date popup lists:
/// `0.0` is the top, `max_offset()` the bottom.
///
/// Color: the thumb is gray while the theme accent is the default
/// (`Multicolor`/`Blue` resolve to `SCROLLBAR_DEFAULT_ACCENT`) and
/// follows any other theme accent. A manual `accent` wins over both.
/// Hover lightens the thumb, pressing darkens it.
pub struct Scrollbar {
    total: f32,
    visible: f32,
    offset: f32,
    shown: f32,
    base: Color,
    base_manual: bool,
    dark: bool,
    focused: bool,
    disabled: bool,
    hovered: bool,
    dragging: bool,
    grab: f32,
    width_cur: f32,
    opacity: f32,
    track_op: f32,
    idle: f32,
    anim: Option<TweenAnim<f32>>,
    anim_time: f32,
    last_draw: Option<Instant>,
    on_scroll: Option<Box<dyn FnMut(f32)>>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Scrollbar {
    pub fn new() -> Self {
        Self {
            total: 0.0,
            visible: 0.0,
            offset: 0.0,
            shown: 0.0,
            base: SCROLLBAR_GRAY,
            base_manual: false,
            dark: true,
            focused: true,
            disabled: false,
            hovered: false,
            dragging: false,
            grab: 0.0,
            width_cur: SCROLLBAR_W,
            opacity: 0.0,
            track_op: 0.0,
            idle: SCROLLBAR_HIDE_DELAY,
            anim: None,
            anim_time: 0.0,
            last_draw: None,
            on_scroll: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    /// Content model: `total` content length and `visible` viewport
    /// length in logical px. The offset clamps into range and the bar
    /// flashes visible when the content is scrollable.
    pub fn content(mut self, total: f32, visible: f32) -> Self {
        self.total = total.max(0.0);
        self.visible = visible.max(0.0);
        self.offset = self.offset.clamp(0.0, self.max_offset());
        self.shown = self.offset;
        self
    }

    /// Manual thumb color: wins over the theme accent until cleared.
    pub fn accent(mut self, color: Color) -> Self {
        self.base = color;
        self.base_manual = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_scroll(mut self, callback: impl FnMut(f32) + 'static) -> Self {
        self.on_scroll = Some(Box::new(callback));
        self
    }

    /// Live theme: gray thumb on the default accent, theme accent
    /// otherwise. A manually set accent wins over the system accent.
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        if !self.base_manual {
            self.base = if accent == SCROLLBAR_DEFAULT_ACCENT {
                SCROLLBAR_GRAY
            } else {
                accent
            };
        }
        self.dark = dark;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Content model without firing `on_scroll` unless the clamped
    /// offset changed. Flashes the bar when scrollable.
    pub fn set_content(&mut self, total: f32, visible: f32) {
        self.total = total.max(0.0);
        self.visible = visible.max(0.0);
        let clamped = self.offset.clamp(0.0, self.max_offset());
        if clamped != self.offset {
            self.offset = clamped;
            self.shown = clamped;
            self.anim = None;
            self.notify();
        }
        self.wake();
    }

    /// Manual placement for the side of scrollable content. The track
    /// spans the full rect; the thumb floats right-aligned inside it.
    pub fn set_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w.max(0.0);
        self.height = h.max(0.0);
    }

    pub fn offset(&self) -> f32 {
        self.offset
    }

    pub fn max_offset(&self) -> f32 {
        (self.total - self.visible).max(0.0)
    }

    pub fn scrollable(&self) -> bool {
        self.total > self.visible && self.visible > 0.0 && self.total > 0.0
    }

    /// Whether the thumb is currently painted (faded in).
    pub fn is_visible(&self) -> bool {
        self.opacity > 0.02
    }

    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    /// Set the offset immediately (clamped). Fires `on_scroll` when
    /// the offset changed and flashes the bar.
    pub fn set_offset(&mut self, offset: f32) {
        self.anim = None;
        self.commit(offset);
        self.wake();
    }

    /// Move by `delta` in logical px (clamped). Fires `on_scroll`
    /// when the offset changed and flashes the bar.
    pub fn scroll_by(&mut self, delta: f32) {
        self.set_offset(self.offset + delta);
    }

    /// Animated jump to `offset` (clamped), like a track click. The
    /// content follows the thumb through `on_scroll`.
    pub fn scroll_to(&mut self, offset: f32) {
        if self.disabled {
            return;
        }
        let target = offset.clamp(0.0, self.max_offset());
        if target == self.offset {
            self.wake();
            return;
        }
        self.anim = Some(TweenAnim::new(
            Tween::new(self.shown, target, SCROLLBAR_PAGE_ANIM_SECONDS)
                .easing(Easing::CubicOut)
                .repeat(Repeat::Never),
        ));
        self.anim_time = 0.0;
        self.wake();
    }

    /// Flash the bar visible (e.g. after the content changed).
    pub fn flash(&mut self) {
        self.wake();
    }

    fn wake(&mut self) {
        self.idle = 0.0;
    }

    fn notify(&mut self) {
        if let Some(callback) = self.on_scroll.as_mut() {
            callback(self.offset);
        }
    }

    fn commit(&mut self, value: f32) {
        let clamped = value.clamp(0.0, self.max_offset());
        if clamped != self.offset {
            self.offset = clamped;
            self.shown = clamped;
            self.notify();
        }
    }

    fn eff(&self, color: Color) -> Color {
        if self.focused {
            color
        } else {
            desaturate(color)
        }
    }

    fn thumb_color(&self) -> Color {
        if self.dragging {
            darken(self.base, SCROLLBAR_PRESS_DARKEN)
        } else if self.hovered {
            lighten(self.base, SCROLLBAR_HOVER_LIGHTEN)
        } else {
            self.base
        }
    }

    fn track_color(&self) -> Color {
        if self.dark {
            SCROLLBAR_TRACK_DARK
        } else {
            SCROLLBAR_TRACK_LIGHT
        }
    }

    fn thumb_h(&self) -> f32 {
        if !self.scrollable() || self.height <= 0.0 {
            return 0.0;
        }
        (self.height * self.visible / self.total)
            .clamp(SCROLLBAR_MIN_THUMB.min(self.height), self.height)
    }

    fn thumb_y(&self) -> f32 {
        let max = self.max_offset();
        if max <= 0.0 {
            return self.y;
        }
        self.y + (self.height - self.thumb_h()) * (self.shown / max)
    }

    fn thumb_w(&self) -> f32 {
        self.width_cur.min(self.width)
    }

    fn thumb_hit(&self, x: f32, y: f32) -> bool {
        let w = self.thumb_w();
        let tx = self.x + self.width - w;
        // Generous grab area for the thin thumb.
        let ex = ((SCROLLBAR_HIT_W - w) / 2.0).max(0.0);
        let ty = self.thumb_y();
        x >= tx - ex && x <= tx + w + ex && y >= ty - 2.0 && y <= ty + self.thumb_h() + 2.0
    }

    fn track_hit(&self, x: f32, y: f32) -> bool {
        x >= self.x - 2.0
            && x <= self.x + self.width + 2.0
            && y >= self.y
            && y <= self.y + self.height
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        if self.disabled || !self.scrollable() {
            return;
        }
        let (x, y) = (x as f32, y as f32);
        if self.thumb_hit(x, y) {
            // Pressed thumb follows the mouse directly.
            self.dragging = true;
            self.anim = None;
            self.grab = y - self.thumb_y();
        } else if self.track_hit(x, y) {
            // Track click jumps one page toward the click.
            let ty = self.thumb_y();
            if y < ty {
                self.scroll_to(self.offset - self.visible);
            } else {
                self.scroll_to(self.offset + self.visible);
            }
        }
        self.wake();
    }

    pub fn mouse_move(&mut self, x: f64, y: f64) {
        if self.disabled {
            return;
        }
        let (x, y) = (x as f32, y as f32);
        self.hovered = self.scrollable() && (self.thumb_hit(x, y) || self.track_hit(x, y));
        if self.dragging {
            let travel = (self.height - self.thumb_h()).max(0.0);
            let value = if travel > 0.0 {
                (y - self.grab - self.y) / travel * self.max_offset()
            } else {
                0.0
            };
            self.anim = None;
            self.commit(value);
        }
    }

    pub fn mouse_up(&mut self, _x: f64, _y: f64) {
        self.dragging = false;
    }

    /// Scroll the content (`dy` in logical px, down positive, like
    /// the date popup lists). Flashes the bar.
    pub fn mouse_wheel(&mut self, _dx: f64, dy: f64) {
        if self.disabled || !self.scrollable() {
            return;
        }
        self.set_offset(self.offset - dy as f32);
    }

    fn step(&mut self, dt: f32) {
        let dt = dt.max(0.0);
        // Widen toward the hover width.
        let w_target = if self.hovered || self.dragging {
            SCROLLBAR_W_HOVER
        } else {
            SCROLLBAR_W
        };
        let w_step = SCROLLBAR_WIDEN_SPEED * dt;
        self.width_cur += (w_target - self.width_cur).clamp(-w_step, w_step);
        if (self.width_cur - w_target).abs() < 0.05 {
            self.width_cur = w_target;
        }
        // Fade: visible while interacting or recently active.
        if self.dragging || self.hovered {
            self.idle = 0.0;
        } else {
            self.idle += dt;
        }
        let o_target = if self.scrollable()
            && (self.dragging || self.hovered || self.idle < SCROLLBAR_HIDE_DELAY)
        {
            1.0
        } else {
            0.0
        };
        let o_step = dt / SCROLLBAR_FADE_SECONDS.max(0.001);
        self.opacity += (o_target - self.opacity).clamp(-o_step, o_step);
        self.opacity = self.opacity.clamp(0.0, 1.0);
        // Track: visible while aiming (hover or drag), same fade speed.
        let t_target = if self.hovered || self.dragging {
            1.0
        } else {
            0.0
        };
        self.track_op += (t_target - self.track_op).clamp(-o_step, o_step);
        self.track_op = self.track_op.clamp(0.0, 1.0);
        // Page-jump animation; the offset follows the thumb so the
        // content glides with it.
        if self.anim.is_some() && !self.dragging {
            self.anim_time += dt;
            let done = self.anim.as_mut().expect("anim set").update(self.anim_time);
            let target = *self.anim.as_ref().expect("anim set").value();
            self.shown = target.clamp(0.0, self.max_offset());
            if self.shown != self.offset {
                self.offset = self.shown;
                self.notify();
            }
            if done {
                self.anim = None;
            }
        } else {
            self.shown = self.offset;
        }
    }

    fn advance(&mut self, now: Instant) {
        let dt = match self.last_draw {
            Some(last) => now.saturating_duration_since(last).as_secs_f32().min(0.1),
            None => 0.0,
        };
        self.last_draw = Some(now);
        self.step(dt);
    }
}

impl Default for Scrollbar {
    fn default() -> Self {
        Self::new()
    }
}

impl View for Scrollbar {
    /// Intrinsic size: hover width by a short minimum track. The bar
    /// spans the placed height, so prefer manual `set_rect` (or a
    /// `Frame`) for full-height side placement.
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        (SCROLLBAR_W_HOVER, SCROLLBAR_MIN_TRACK)
    }

    fn place(&mut self, _fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.set_rect(x, y, w, h);
    }

    fn draw(&mut self, scene: &mut Scene, fonts: &mut FontSystem, images: &mut ImageLoader<'_>) {
        if images.is_capture_pass() {
            return;
        }
        self.advance(Instant::now());
        if self.opacity <= 0.01 || !self.scrollable() {
            return;
        }
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let w = self.thumb_w();
        let h = self.thumb_h();
        let x = self.x + self.width - w;
        let y = self.thumb_y();
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        // Full-height track behind the thumb while aiming, so the
        // whole travel range is visible from top to bottom.
        if self.track_op > 0.01 {
            let tw = SCROLLBAR_W_HOVER.min(self.width);
            let tx = self.x + self.width - tw;
            if tw > 0.0 {
                let track = RoundedRect::new(
                    px(tx),
                    px(self.y),
                    px(tx + tw),
                    px(self.y + self.height),
                    px(tw / 2.0),
                );
                let t = self.eff(self.track_color()).to_rgba8();
                let mut t_alpha = (t.a as f32 * self.track_op).round() as u8;
                if self.disabled {
                    t_alpha = (t_alpha as f32 * 0.4).round() as u8;
                }
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(Color::from_rgba8(t.r, t.g, t.b, t_alpha)),
                    None,
                    &track,
                );
            }
        }
        let thumb = RoundedRect::new(px(x), px(y), px(x + w), px(y + h), px(w / 2.0));
        let c = self.eff(self.thumb_color()).to_rgba8();
        let mut alpha = (SCROLLBAR_ALPHA as f32 * self.opacity).round() as u8;
        if self.disabled {
            alpha = (alpha as f32 * 0.4).round() as u8;
        }
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(Color::from_rgba8(c.r, c.g, c.b, alpha)),
            None,
            &thumb,
        );
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        let _ = (x, y);
        self.dragging = false;
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar() -> Scrollbar {
        Scrollbar::new().content(1000.0, 200.0)
    }

    fn placed() -> Scrollbar {
        let mut b = bar();
        b.set_rect(0.0, 0.0, 10.0, 200.0);
        b
    }

    fn brightness(color: Color) -> u32 {
        let c = color.to_rgba8();
        c.r as u32 + c.g as u32 + c.b as u32
    }

    #[test]
    fn hidden_when_content_fits() {
        let mut b = Scrollbar::new().content(100.0, 200.0);
        assert!(!b.scrollable());
        assert_eq!(b.max_offset(), 0.0);
        b.step(10.0);
        assert_eq!(b.opacity, 0.0);
        assert!(!b.is_visible());
        // Wheel and presses do nothing without overflow.
        b.mouse_wheel(0.0, -50.0);
        assert_eq!(b.offset(), 0.0);
        b.mouse_down(5.0, 100.0);
        assert!(!b.dragging);
    }

    #[test]
    fn wheel_shows_and_scrolls_with_clamp() {
        let mut b = placed();
        b.mouse_wheel(0.0, -50.0);
        assert_eq!(b.offset(), 50.0);
        b.step(0.1);
        assert!(b.is_visible());
        // Past the end clamps.
        b.mouse_wheel(0.0, -100000.0);
        assert_eq!(b.offset(), b.max_offset());
        b.mouse_wheel(0.0, 100000.0);
        assert_eq!(b.offset(), 0.0);
    }

    #[test]
    fn fades_out_after_idle() {
        let mut b = placed();
        b.mouse_wheel(0.0, -50.0);
        b.step(0.1);
        assert!(b.is_visible());
        // Past delay plus fade time the bar is gone.
        b.step(SCROLLBAR_HIDE_DELAY + SCROLLBAR_FADE_SECONDS + 0.1);
        assert_eq!(b.opacity, 0.0);
        assert!(!b.is_visible());
    }

    #[test]
    fn thumb_geometry_is_proportional() {
        let b = placed();
        // 200 of 1000 over a 200 track: 40 thumb at the top.
        assert_eq!(b.thumb_h(), 40.0);
        assert_eq!(b.thumb_y(), 0.0);
        let mut b = placed();
        b.set_offset(400.0);
        assert_eq!(b.thumb_y(), 80.0);
    }

    #[test]
    fn drag_follows_mouse_with_grab() {
        let mut b = placed();
        // Thumb spans 0..40: press in the middle.
        b.mouse_down(5.0, 20.0);
        assert!(b.dragging);
        // Drag down 40: grab kept, offset maps the travel.
        b.mouse_move(5.0, 60.0);
        let travel = 200.0 - 40.0;
        assert_eq!(b.offset(), 40.0 / travel * 800.0);
        b.mouse_up(5.0, 60.0);
        assert!(!b.dragging);
    }

    #[test]
    fn track_click_pages_toward_click() {
        let mut b = placed();
        // Click below the thumb (0..40): one page down, animated.
        b.mouse_down(5.0, 150.0);
        assert!(!b.dragging);
        b.step(SCROLLBAR_PAGE_ANIM_SECONDS + 0.1);
        assert_eq!(b.offset(), 200.0);
        // Click above the thumb: one page up.
        b.mouse_down(5.0, 10.0);
        b.step(SCROLLBAR_PAGE_ANIM_SECONDS + 0.1);
        assert_eq!(b.offset(), 0.0);
    }

    #[test]
    fn track_click_clamps_at_ends() {
        let mut b = placed();
        b.set_offset(700.0);
        // Thumb spans 140..180: click at 190 hits the track below it.
        b.mouse_down(5.0, 190.0);
        assert!(!b.dragging);
        b.step(SCROLLBAR_PAGE_ANIM_SECONDS + 0.1);
        assert_eq!(b.offset(), 800.0);
    }

    #[test]
    fn hover_widens_and_lightens() {
        let mut b = placed();
        let plain = brightness(b.thumb_color());
        b.mouse_move(5.0, 100.0);
        assert!(b.hovered);
        b.step(1.0);
        assert_eq!(b.width_cur, SCROLLBAR_W_HOVER);
        assert!(brightness(b.thumb_color()) > plain);
        // Leaving returns to the idle width.
        b.mouse_move(500.0, 500.0);
        assert!(!b.hovered);
        b.step(1.0);
        assert_eq!(b.width_cur, SCROLLBAR_W);
    }

    #[test]
    fn hover_reveals_full_track_and_leave_hides_it() {
        let mut b = placed();
        assert_eq!(b.track_op, 0.0);
        b.mouse_move(5.0, 100.0);
        b.step(1.0);
        assert_eq!(b.track_op, 1.0);
        b.mouse_move(500.0, 500.0);
        b.step(1.0);
        assert_eq!(b.track_op, 0.0);
    }

    #[test]
    fn track_color_follows_mode() {
        let mut b = placed();
        b.set_theme(Color::from_rgb8(0x00, 0x7a, 0xff), true);
        assert_eq!(b.track_color(), SCROLLBAR_TRACK_DARK);
        b.set_theme(Color::from_rgb8(0x00, 0x7a, 0xff), false);
        assert_eq!(b.track_color(), SCROLLBAR_TRACK_LIGHT);
    }

    #[test]
    fn pressed_darkens_in_dark_mode() {
        let mut b = placed();
        b.set_theme(Color::from_rgb8(0x00, 0x7a, 0xff), true);
        let plain = brightness(b.thumb_color());
        b.mouse_down(5.0, 20.0);
        assert!(b.dragging);
        assert!(brightness(b.thumb_color()) < plain);
    }

    #[test]
    fn default_accent_is_gray_other_accents_follow() {
        let mut b = Scrollbar::new();
        // Multicolor/Blue default: gray thumb.
        b.set_theme(Color::from_rgb8(0x00, 0x7a, 0xff), true);
        assert_eq!(b.base, SCROLLBAR_GRAY);
        // Any other accent: thumb follows it.
        let red = Color::from_rgb8(0xff, 0x3b, 0x30);
        b.set_theme(red, true);
        assert_eq!(b.base, red);
        // Manual accent wins over the theme.
        let mut m = Scrollbar::new().accent(Color::from_rgb8(0x34, 0xc7, 0x59));
        m.set_theme(red, true);
        assert_eq!(m.base, Color::from_rgb8(0x34, 0xc7, 0x59));
    }

    #[test]
    fn callback_fires_only_on_change() {
        use std::cell::Cell;
        use std::rc::Rc;
        let fires = Rc::new(Cell::new(0));
        let count = fires.clone();
        let mut b = Scrollbar::new()
            .content(1000.0, 200.0)
            .on_scroll(move |_| count.set(count.get() + 1));
        b.set_rect(0.0, 0.0, 10.0, 200.0);
        b.set_offset(0.0);
        assert_eq!(fires.get(), 0);
        b.set_offset(100.0);
        assert_eq!(b.offset(), 100.0);
        assert_eq!(fires.get(), 1);
        // Drag to the same spot fires nothing.
        let y = b.thumb_y() + 5.0;
        b.mouse_down(5.0, y as f64);
        b.mouse_move(5.0, y as f64);
        b.mouse_up(5.0, y as f64);
        assert_eq!(fires.get(), 1);
    }

    #[test]
    fn set_content_reclamps_and_flashes() {
        let mut b = placed();
        b.set_offset(800.0);
        b.step(SCROLLBAR_HIDE_DELAY + SCROLLBAR_FADE_SECONDS + 0.1);
        assert!(!b.is_visible());
        // Shorter content clamps the offset and flashes the bar.
        b.set_content(400.0, 200.0);
        assert_eq!(b.offset(), 200.0);
        b.step(0.1);
        assert!(b.is_visible());
    }
}
