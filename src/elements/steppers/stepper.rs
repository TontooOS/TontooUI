use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, BezPath, Cap, Join, Rect, RoundedRect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::layout::View;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;
use crate::theme::desaturate;

/// Intrinsic width in logical px (vertical stepper).
pub const STEPPER_W: f32 = 34.0;
/// Intrinsic height in logical px (vertical stepper).
pub const STEPPER_H: f32 = 56.0;
/// Corner radius in logical px.
pub const STEPPER_RADIUS: f32 = 8.0;
/// Divider inset from the control edges in logical px.
pub const STEPPER_DIV_INSET: f32 = 6.0;
/// Chevron stroke width in logical px.
pub const STEPPER_CHEV_STROKE: f32 = 2.0;
/// Alpha multiplier for a chevron that cannot step further.
pub const STEPPER_DISABLED_ALPHA: f32 = 0.35;

/// Dark mode control fill.
pub const STEPPER_BG_DARK: Color = Color::from_rgb8(0x2c, 0x2c, 0x2e);
/// Light mode control fill.
pub const STEPPER_BG_LIGHT: Color = Color::from_rgb8(0xe9, 0xe9, 0xeb);
/// Divider in dark mode.
pub const STEPPER_DIVIDER_DARK: Color = Color::from_rgba8(255, 255, 255, 36);
/// Divider in light mode.
pub const STEPPER_DIVIDER_LIGHT: Color = Color::from_rgba8(0, 0, 0, 31);
/// Default accent (pressed chevron).
pub const STEPPER_ACCENT: Color = Color::from_rgb8(0x00, 0x7a, 0xff);

/// Stepper layout direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepperOrientation {
    /// Up chevron on top, down chevron at the bottom (reference image).
    #[default]
    Vertical,
    /// Left chevron on the left, right chevron on the right.
    Horizontal,
}

/// Which half of the control was pressed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepperSide {
    Increment,
    Decrement,
}

/// Basic stepper: two chevron halves sharing one rounded control.
/// Each press moves `value` by `step` inside `min..=max`. A side that
/// cannot move further draws its chevron dimmed and ignores clicks.
pub struct Stepper {
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    orientation: StepperOrientation,
    accent: Color,
    dark: bool,
    fg: Color,
    disabled: bool,
    focused: bool,
    pressed: Option<StepperSide>,
    hovered: Option<StepperSide>,
    on_change: Option<Box<dyn FnMut(f64)>>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Stepper {
    pub fn new(value: f64, min: f64, max: f64) -> Self {
        let lo = min.min(max);
        let hi = min.max(max);
        Self {
            value: value.clamp(lo, hi),
            min: lo,
            max: hi,
            step: 1.0,
            orientation: StepperOrientation::Vertical,
            accent: STEPPER_ACCENT,
            dark: true,
            fg: Color::WHITE,
            disabled: false,
            focused: true,
            pressed: None,
            hovered: None,
            on_change: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    /// Step size in value units. Non-positive values keep the old step.
    /// `1.0` counts 1, 2, 3; `10.0` counts 10, 20, 30.
    pub fn step(mut self, step: f64) -> Self {
        if step > 0.0 && step.is_finite() {
            self.step = step;
        }
        self
    }

    /// Value range (`min..=max`). The value is clamped into it.
    pub fn range(mut self, min: f64, max: f64) -> Self {
        let lo = min.min(max);
        let hi = min.max(max);
        self.min = lo;
        self.max = hi;
        self.value = self.value.clamp(lo, hi);
        self
    }

    pub fn orientation(mut self, orientation: StepperOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, callback: impl FnMut(f64) + 'static) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    /// Live theme: pressed chevron uses the accent, mode drives the
    /// control fill, chevron and divider grays.
    pub fn set_theme(&mut self, accent: Color, dark: bool) {
        self.accent = accent;
        self.dark = dark;
        self.fg = if dark {
            Color::WHITE
        } else {
            Color::from_rgb8(0x3a, 0x3a, 0x3c)
        };
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
        if disabled {
            self.pressed = None;
            self.hovered = None;
        }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn set_value(&mut self, value: f64) {
        let clamped = value.clamp(self.min, self.max);
        if clamped != self.value {
            self.value = clamped;
            self.notify();
        }
    }

    pub fn set_step(&mut self, step: f64) {
        if step > 0.0 && step.is_finite() {
            self.step = step;
        }
    }

    pub fn set_range(&mut self, min: f64, max: f64) {
        let lo = min.min(max);
        let hi = min.max(max);
        self.min = lo;
        self.max = hi;
        self.set_value(self.value);
    }

    /// True when pressing increment would still move the value.
    pub fn increment_enabled(&self) -> bool {
        !self.disabled && self.value < self.max
    }

    /// True when pressing decrement would still move the value.
    pub fn decrement_enabled(&self) -> bool {
        !self.disabled && self.value > self.min
    }

    pub fn increment(&mut self) {
        if self.increment_enabled() {
            self.set_value(self.value + self.step);
        }
    }

    pub fn decrement(&mut self) {
        if self.decrement_enabled() {
            self.set_value(self.value - self.step);
        }
    }

    fn notify(&mut self) {
        if let Some(callback) = self.on_change.as_mut() {
            callback(self.value);
        }
    }

    fn hit(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    fn side_at(&self, x: f32, y: f32) -> Option<StepperSide> {
        if !self.hit(x, y) {
            return None;
        }
        match self.orientation {
            StepperOrientation::Vertical => {
                if y < self.y + self.height / 2.0 {
                    Some(StepperSide::Increment)
                } else {
                    Some(StepperSide::Decrement)
                }
            }
            StepperOrientation::Horizontal => {
                if x < self.x + self.width / 2.0 {
                    Some(StepperSide::Decrement)
                } else {
                    Some(StepperSide::Increment)
                }
            }
        }
    }

    fn side_enabled(&self, side: StepperSide) -> bool {
        match side {
            StepperSide::Increment => self.increment_enabled(),
            StepperSide::Decrement => self.decrement_enabled(),
        }
    }

    pub fn mouse_down(&mut self, x: f64, y: f64) {
        let side = self.side_at(x as f32, y as f32);
        match side {
            Some(StepperSide::Increment) if self.increment_enabled() => {
                self.pressed = Some(StepperSide::Increment);
                self.increment();
            }
            Some(StepperSide::Decrement) if self.decrement_enabled() => {
                self.pressed = Some(StepperSide::Decrement);
                self.decrement();
            }
            _ => {
                self.pressed = None;
            }
        }
    }

    pub fn mouse_up(&mut self, _x: f64, _y: f64) {
        self.pressed = None;
    }

    fn eff(&self, color: Color) -> Color {
        if self.focused {
            color
        } else {
            desaturate(color)
        }
    }

    fn with_alpha(color: Color, alpha: f32) -> Color {
        let c = color.to_rgba8();
        Color::from_rgba8(c.r, c.g, c.b, (c.a as f32 * alpha).round() as u8)
    }

    fn bg(&self) -> Color {
        let base = if self.dark {
            STEPPER_BG_DARK
        } else {
            STEPPER_BG_LIGHT
        };
        let mut bg = self.eff(base);
        if self.disabled {
            bg = Self::with_alpha(bg, 0.4);
        }
        bg
    }

    fn divider_color(&self) -> Color {
        let base = if self.dark {
            STEPPER_DIVIDER_DARK
        } else {
            STEPPER_DIVIDER_LIGHT
        };
        self.eff(base)
    }

    fn chevron_color(&self, side: StepperSide) -> Color {
        let mut color = if self.pressed == Some(side) && self.side_enabled(side) {
            self.accent
        } else {
            self.fg
        };
        color = self.eff(color);
        if self.disabled || !self.side_enabled(side) {
            color = Self::with_alpha(color, STEPPER_DISABLED_ALPHA);
        }
        color
    }

    fn chevron_path(x: f32, y: f32, w: f32, h: f32, dir: StepperSide, vertical: bool) -> BezPath {
        let mut path = BezPath::new();
        if vertical {
            if dir == StepperSide::Increment {
                // Up chevron: v shape pointing up.
                path.move_to((x - w / 2.0, y + h / 2.0));
                path.line_to((x, y - h / 2.0));
                path.line_to((x + w / 2.0, y + h / 2.0));
            } else {
                // Down chevron.
                path.move_to((x - w / 2.0, y - h / 2.0));
                path.line_to((x, y + h / 2.0));
                path.line_to((x + w / 2.0, y - h / 2.0));
            }
        } else if dir == StepperSide::Increment {
            // Right chevron.
            path.move_to((x - h / 2.0, y - w / 2.0));
            path.line_to((x + h / 2.0, y));
            path.line_to((x - h / 2.0, y + w / 2.0));
        } else {
            // Left chevron.
            path.move_to((x + h / 2.0, y - w / 2.0));
            path.line_to((x - h / 2.0, y));
            path.line_to((x + h / 2.0, y + w / 2.0));
        }
        path
    }

    #[allow(clippy::too_many_arguments)]
    fn stroke_chevron(
        scene: &mut Scene,
        cx: f32,
        cy: f32,
        side: StepperSide,
        vertical: bool,
        scale: f32,
        color: Color,
    ) {
        let w = 12.0;
        let h = 7.0;
        let px = |v: f32| v as f64 * scale as f64;
        let mut path = BezPath::new();
        let raw = Self::chevron_path(cx, cy, w, h, side, vertical);
        for el in raw.elements() {
            match el {
                vello::kurbo::PathEl::MoveTo(p) => {
                    path.move_to((px(p.x as f32), px(p.y as f32)));
                }
                vello::kurbo::PathEl::LineTo(p) => {
                    path.line_to((px(p.x as f32), px(p.y as f32)));
                }
                _ => {}
            }
        }
        let mut stroke = Stroke::new(STEPPER_CHEV_STROKE as f64 * scale as f64);
        stroke.start_cap = Cap::Round;
        stroke.end_cap = Cap::Round;
        stroke.join = Join::Round;
        scene.stroke(&stroke, Affine::IDENTITY, &Brush::Solid(color), None, &path);
    }

    fn render(&mut self, scene: &mut Scene, fonts: &FontSystem) {
        let scale = fonts.scale as f64;
        let px = |v: f32| v as f64 * scale;
        let vertical = self.orientation == StepperOrientation::Vertical;

        // Small contact shadow under the control.
        scene.draw_blurred_rounded_rect(
            Affine::IDENTITY,
            Rect::new(
                px(self.x),
                px(self.y),
                px(self.x + self.width),
                px(self.y + self.height),
            ),
            Color::from_rgba8(0, 0, 0, 35),
            px(STEPPER_RADIUS),
            6.0 * scale,
        );

        let bg_shape = RoundedRect::new(
            px(self.x),
            px(self.y),
            px(self.x + self.width),
            px(self.y + self.height),
            px(STEPPER_RADIUS),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.bg()),
            None,
            &bg_shape,
        );

        // Pressed / hovered half highlight, clipped to the rounded body so
        // the square half rect never spills over the outer corners.
        let highlight = |scene: &mut Scene, side: StepperSide, alpha: u8| {
            let color = if self.dark {
                Color::from_rgba8(255, 255, 255, alpha)
            } else {
                Color::from_rgba8(0, 0, 0, alpha)
            };
            let rect = match (vertical, side) {
                (true, StepperSide::Increment) => Rect::new(
                    px(self.x),
                    px(self.y),
                    px(self.x + self.width),
                    px(self.y + self.height / 2.0),
                ),
                (true, StepperSide::Decrement) => Rect::new(
                    px(self.x),
                    px(self.y + self.height / 2.0),
                    px(self.x + self.width),
                    px(self.y + self.height),
                ),
                (false, StepperSide::Decrement) => Rect::new(
                    px(self.x),
                    px(self.y),
                    px(self.x + self.width / 2.0),
                    px(self.y + self.height),
                ),
                (false, StepperSide::Increment) => Rect::new(
                    px(self.x + self.width / 2.0),
                    px(self.y),
                    px(self.x + self.width),
                    px(self.y + self.height),
                ),
            };
            scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &bg_shape);
            scene.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(color), None, &rect);
            scene.pop_layer();
        };
        if let Some(side) = self.pressed {
            if self.side_enabled(side) {
                highlight(scene, side, 36);
            }
        } else if let Some(side) = self.hovered {
            if self.side_enabled(side) {
                highlight(scene, side, 16);
            }
        }

        // Divider between the halves.
        let divider = self.divider_color();
        let stroke = Stroke::new(1.0 * scale);
        if vertical {
            let mid = self.y + self.height / 2.0;
            let line = vello::kurbo::Line::new(
                (px(self.x + STEPPER_DIV_INSET), px(mid)),
                (px(self.x + self.width - STEPPER_DIV_INSET), px(mid)),
            );
            scene.stroke(&stroke, Affine::IDENTITY, &Brush::Solid(divider), None, &line);
        } else {
            let mid = self.x + self.width / 2.0;
            let line = vello::kurbo::Line::new(
                (px(mid), px(self.y + STEPPER_DIV_INSET)),
                (px(mid), px(self.y + self.height - STEPPER_DIV_INSET)),
            );
            scene.stroke(&stroke, Affine::IDENTITY, &Brush::Solid(divider), None, &line);
        }

        // Chevrons, one per half.
        let (inc_pos, dec_pos) = if vertical {
            (
                (self.x + self.width / 2.0, self.y + self.height / 4.0),
                (self.x + self.width / 2.0, self.y + self.height * 3.0 / 4.0),
            )
        } else {
            (
                (self.x + self.width * 3.0 / 4.0, self.y + self.height / 2.0),
                (self.x + self.width / 4.0, self.y + self.height / 2.0),
            )
        };
        Self::stroke_chevron(
            scene,
            inc_pos.0,
            inc_pos.1,
            StepperSide::Increment,
            vertical,
            fonts.scale,
            self.chevron_color(StepperSide::Increment),
        );
        Self::stroke_chevron(
            scene,
            dec_pos.0,
            dec_pos.1,
            StepperSide::Decrement,
            vertical,
            fonts.scale,
            self.chevron_color(StepperSide::Decrement),
        );
    }
}

impl View for Stepper {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        match self.orientation {
            StepperOrientation::Vertical => (STEPPER_W, STEPPER_H),
            StepperOrientation::Horizontal => (STEPPER_H, STEPPER_W),
        }
    }

    fn place(&mut self, _fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.width = w;
        self.height = h;
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        _images: &mut ImageLoader<'_>,
    ) {
        self.render(scene, fonts);
    }

    fn mouse_down(&mut self, x: f64, y: f64) {
        self.mouse_down(x, y);
    }

    fn mouse_up(&mut self, x: f64, y: f64) {
        self.mouse_up(x, y);
    }

    fn set_hover(&mut self, x: f32, y: f32) {
        self.hovered = self.side_at(x, y);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stepper() -> Stepper {
        let mut stepper = Stepper::new(50.0, 0.0, 100.0);
        stepper.place(&mut crate::renderer::text::FontSystem::new(), 0.0, 0.0, STEPPER_W, STEPPER_H);
        stepper
    }

    #[test]
    fn steps_by_one_by_default() {
        let mut stepper = stepper();
        stepper.increment();
        assert_eq!(stepper.value(), 51.0);
        stepper.decrement();
        stepper.decrement();
        assert_eq!(stepper.value(), 49.0);
    }

    #[test]
    fn custom_step_counts_tens() {
        let mut stepper = Stepper::new(0.0, 0.0, 100.0).step(10.0);
        stepper.place(&mut crate::renderer::text::FontSystem::new(), 0.0, 0.0, STEPPER_W, STEPPER_H);
        stepper.increment();
        stepper.increment();
        stepper.increment();
        assert_eq!(stepper.value(), 30.0);
    }

    #[test]
    fn clamps_at_range_and_reports_sides() {
        let mut stepper = Stepper::new(100.0, 0.0, 100.0).step(10.0);
        stepper.place(&mut crate::renderer::text::FontSystem::new(), 0.0, 0.0, STEPPER_W, STEPPER_H);
        assert!(!stepper.increment_enabled());
        assert!(stepper.decrement_enabled());
        stepper.increment();
        assert_eq!(stepper.value(), 100.0);
        stepper.set_value(-50.0);
        assert_eq!(stepper.value(), 0.0);
        assert!(!stepper.decrement_enabled());
    }

    #[test]
    fn clicks_hit_top_and_bottom_halves() {
        let mut stepper = Stepper::new(50.0, 0.0, 100.0).step(10.0);
        stepper.place(&mut crate::renderer::text::FontSystem::new(), 0.0, 0.0, STEPPER_W, STEPPER_H);
        // Top half increments.
        stepper.mouse_down(17.0, 10.0);
        assert_eq!(stepper.value(), 60.0);
        stepper.mouse_up(17.0, 10.0);
        // Bottom half decrements.
        stepper.mouse_down(17.0, 46.0);
        assert_eq!(stepper.value(), 50.0);
    }

    #[test]
    fn limit_click_is_ignored() {
        let mut stepper = Stepper::new(0.0, 0.0, 100.0).step(10.0);
        stepper.place(&mut crate::renderer::text::FontSystem::new(), 0.0, 0.0, STEPPER_W, STEPPER_H);
        stepper.mouse_down(17.0, 46.0);
        assert_eq!(stepper.value(), 0.0);
        assert_eq!(stepper.pressed, None);
    }

    #[test]
    fn horizontal_sides_are_left_right() {
        let mut stepper = Stepper::new(50.0, 0.0, 100.0)
            .step(5.0)
            .orientation(StepperOrientation::Horizontal);
        stepper.place(&mut crate::renderer::text::FontSystem::new(), 0.0, 0.0, STEPPER_H, STEPPER_W);
        stepper.mouse_down(10.0, 17.0);
        assert_eq!(stepper.value(), 45.0);
        stepper.mouse_up(10.0, 17.0);
        stepper.mouse_down(46.0, 17.0);
        assert_eq!(stepper.value(), 50.0);
    }

    #[test]
    fn on_change_fires_per_step() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let seen = Rc::new(RefCell::new(Vec::new()));
        let mut stepper = Stepper::new(0.0, 0.0, 100.0).step(10.0).on_change({
            let seen = Rc::clone(&seen);
            move |v| seen.borrow_mut().push(v)
        });
        stepper.place(&mut crate::renderer::text::FontSystem::new(), 0.0, 0.0, STEPPER_W, STEPPER_H);
        stepper.increment();
        stepper.increment();
        drop(stepper);
        assert_eq!(*seen.borrow(), vec![10.0, 20.0]);
    }
}
