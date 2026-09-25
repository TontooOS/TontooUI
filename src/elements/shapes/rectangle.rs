use std::any::Any;

use vello::Scene;
use vello::kurbo::{Affine, Rect, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::layout::View;
use super::{
    SHAPE_DEFAULT_STROKE, SHAPE_SHADOW, SHAPE_SHADOW_DY, SHAPE_STROKE_W, ShapeFill,
    fill_brush, stroke_color,
};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;

/// Solid rectangle with optional gradient fill and outline look, like
/// the reference rows: filled blue, red outline (unfilled) and a
/// blue-to-purple linear gradient. Display-only (no mouse handling).
pub struct Rectangle {
    width: f32,
    height: f32,
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

impl Rectangle {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width: width.max(0.0),
            height: height.max(0.0),
            fill: ShapeFill::default(),
            filled: true,
            stroke: SHAPE_DEFAULT_STROKE,
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

    /// Outline color used when unfilled (`filled(false)`). Filled
    /// shapes draw borderless like the reference rows; the stroke
    /// color applies to the outline look.
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

impl Default for Rectangle {
    fn default() -> Self {
        Self::new(200.0, 120.0)
    }
}

impl View for Rectangle {
    fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
        (self.width, self.height)
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
        let px = |v: f32| v as f64 * scale;
        let shape = Rect::new(
            px(self.x),
            px(self.y),
            px(self.x + self.placed_w),
            px(self.y + self.placed_h),
        );
        if self.shadow {
            let ghost = Rect::new(
                px(self.x),
                px(self.y + SHAPE_SHADOW_DY),
                px(self.x + self.placed_w),
                px(self.y + self.placed_h + SHAPE_SHADOW_DY),
            );
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(SHAPE_SHADOW),
                None,
                &ghost,
            );
        }
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
                &Stroke::new(px(self.stroke_width.max(1.0))),
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
        let mut rect = Rectangle::new(200.0, 120.0);
        assert_eq!(rect.measure(&mut fonts), (200.0, 120.0));
        rect.place(&mut fonts, 0.0, 0.0, 200.0, 120.0);
        assert_eq!(rect.rect(), (0.0, 0.0, 200.0, 120.0));
    }

    #[test]
    fn outline_mode_reports_unfilled() {
        let rect = Rectangle::new(200.0, 120.0)
            .stroke(Color::from_rgb8(0xff, 0x3b, 0x30))
            .filled(false);
        assert!(!rect.is_filled());
        assert_eq!(
            rect.stroke_color(),
            Color::from_rgb8(0xff, 0x3b, 0x30)
        );
    }

    #[test]
    fn clamps_negative_values() {
        let rect = Rectangle::new(-10.0, -5.0).stroke_width(-2.0);
        assert_eq!(rect.stroke_width_value(), 0.0);
        let mut fonts = FontSystem::new();
        let mut rect = rect;
        assert_eq!(rect.measure(&mut fonts), (0.0, 0.0));
    }

    #[test]
    fn gradient_builders_fill() {
        let rect = Rectangle::new(200.0, 120.0).linear_gradient(
            vec![
                Color::from_rgb8(0x00, 0x7a, 0xff),
                Color::from_rgb8(0xaf, 0x52, 0xde),
            ],
            0.0,
        );
        assert!(rect.is_filled());
        assert!(matches!(
            rect.fill_value(),
            ShapeFill::LinearGradient { .. }
        ));
    }
}
