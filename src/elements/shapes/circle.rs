use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, Ellipse, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::layout::View;
use super::{
    SHAPE_DEFAULT_STROKE, SHAPE_SHADOW, SHAPE_SHADOW_DY, SHAPE_STROKE_W, ShapeFill,
    fill_brush, stroke_color,
};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;

/// Filled circle with optional gradient fill and outline look, like
/// the reference rows: solid green, orange outline (unfilled) and a
/// radial blue highlight. Display-only (no mouse handling).
pub struct Circle {
    diameter: f32,
    fill: ShapeFill,
    filled: bool,
    stroke: Color,
    stroke_width: f32,
    shadow: bool,
    focused: bool,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl Circle {
    pub fn new(diameter: f32) -> Self {
        Self {
            diameter: diameter.max(0.0),
            fill: ShapeFill::Solid(Color::from_rgb8(0x34, 0xc7, 0x59)),
            filled: true,
            stroke: Color::from_rgb8(0xff, 0x95, 0x00),
            stroke_width: SHAPE_STROKE_W,
            shadow: false,
            focused: true,
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        }
    }

    /// Solid color fill (implies `filled(true)`).
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = ShapeFill::Solid(color);
        self.filled = true;
        self
    }

    /// Linear gradient fill at `angle_deg` (0 = left to right).
    pub fn linear_gradient(mut self, colors: Vec<Color>, angle_deg: f32) -> Self {
        self.fill = ShapeFill::linear(colors, angle_deg);
        self.filled = true;
        self
    }

    /// Radial gradient fill from the center to the edge.
    pub fn radial_gradient(mut self, colors: Vec<Color>) -> Self {
        self.fill = ShapeFill::radial(colors);
        self.filled = true;
        self
    }

    /// `true` paints the interior; `false` draws the outline only
    /// (transparent fill, `stroke` border).
    pub fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    /// Outline color used when unfilled (`filled(false)`).
    pub fn stroke(mut self, color: Color) -> Self {
        self.stroke = color;
        self
    }

    /// Outline width in logical px (clamped to >= 0).
    pub fn stroke_width(mut self, px: f32) -> Self {
        self.stroke_width = px.max(0.0);
        self
    }

    /// Small drop shadow behind the shape.
    pub fn shadow(mut self, shadow: bool) -> Self {
        self.shadow = shadow;
        self
    }

    /// Default outline color for the stroked look.
    pub fn default_stroke() -> Color {
        SHAPE_DEFAULT_STROKE
    }

    pub fn set_fill(&mut self, fill: ShapeFill) {
        self.fill = fill;
    }

    pub fn set_filled(&mut self, filled: bool) {
        self.filled = filled;
    }

    pub fn set_stroke(&mut self, color: Color) {
        self.stroke = color;
    }

    pub fn set_stroke_width(&mut self, px: f32) {
        self.stroke_width = px.max(0.0);
    }

    pub fn set_shadow(&mut self, shadow: bool) {
        self.shadow = shadow;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn fill_value(&self) -> &ShapeFill {
        &self.fill
    }

    pub fn is_filled(&self) -> bool {
        self.filled
    }

    pub fn stroke_color(&self) -> Color {
        self.stroke
    }

    pub fn stroke_width_value(&self) -> f32 {
        self.stroke_width
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }
}

impl Default for Circle {
    fn default() -> Self {
        Self::new(120.0)
    }
}

impl View for Circle {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        (self.diameter, self.diameter)
    }

    fn place(&mut self, _fonts: &mut FontSystem, x: f32, y: f32, w: f32, h: f32) {
        self.x = x;
        self.y = y;
        self.placed_w = w;
        self.placed_h = h;
    }

    fn draw(
        &mut self,
        scene: &mut Scene,
        fonts: &mut FontSystem,
        _images: &mut ImageLoader<'_>,
    ) {
        if self.placed_w <= 0.0 || self.placed_h <= 0.0 {
            return;
        }
        let scale = fonts.scale as f64;
        let cx = (self.x + self.placed_w / 2.0) as f64 * scale;
        let cy = (self.y + self.placed_h / 2.0) as f64 * scale;
        let (rx, ry) = (
            self.placed_w as f64 * scale / 2.0,
            self.placed_h as f64 * scale / 2.0,
        );
        if self.shadow {
            let ghost = Ellipse::new(
                (cx, cy + SHAPE_SHADOW_DY as f64 * scale),
                (rx, ry),
                0.0,
            );
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(SHAPE_SHADOW),
                None,
                &ghost,
            );
        }
        let shape = Ellipse::new((cx, cy), (rx, ry), 0.0);
        if self.filled {
            let brush = fill_brush(
                &self.fill,
                self.x,
                self.y,
                self.placed_w,
                self.placed_h,
                fonts.scale,
                self.focused,
            );
            scene.fill(Fill::NonZero, Affine::IDENTITY, &brush, None, &shape);
        } else {
            scene.stroke(
                &Stroke::new(self.stroke_width.max(1.0) as f64 * scale),
                Affine::IDENTITY,
                &Brush::Solid(stroke_color(self.stroke, self.focused)),
                None,
                &shape,
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
    use crate::renderer::text::FontSystem;

    #[test]
    fn keeps_intrinsic_size() {
        let mut fonts = FontSystem::new();
        let mut circle = Circle::new(120.0);
        assert_eq!(circle.measure(&mut fonts), (120.0, 120.0));
        circle.place(&mut fonts, 0.0, 0.0, 120.0, 120.0);
        assert_eq!(circle.rect(), (0.0, 0.0, 120.0, 120.0));
    }

    #[test]
    fn outline_mode_reports_unfilled() {
        let circle = Circle::new(120.0).filled(false);
        assert!(!circle.is_filled());
    }

    #[test]
    fn clamps_negative_values() {
        let circle = Circle::new(-8.0).stroke_width(-2.0);
        assert_eq!(circle.stroke_width_value(), 0.0);
        let mut fonts = FontSystem::new();
        let mut circle = circle;
        assert_eq!(circle.measure(&mut fonts), (0.0, 0.0));
    }

    #[test]
    fn radial_builder_fills() {
        let circle = Circle::new(120.0).radial_gradient(vec![
            Color::WHITE,
            Color::from_rgb8(0x00, 0x7a, 0xff),
        ]);
        assert!(circle.is_filled());
        assert!(matches!(
            circle.fill_value(),
            ShapeFill::RadialGradient { .. }
        ));
    }
}
