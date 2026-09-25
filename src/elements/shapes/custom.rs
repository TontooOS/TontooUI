use std::any::Any;
use std::f32::consts::PI;

use vello::Scene;
use vello::kurbo::{Affine, BezPath, Stroke};
use vello::peniko::{Brush, Color, Fill};

use super::super::layout::View;
use super::{
    SHAPE_DEFAULT_FILL, SHAPE_DEFAULT_STROKE, SHAPE_SHADOW, SHAPE_SHADOW_DY,
    SHAPE_STROKE_W, ShapeFill, fill_brush, stroke_color,
};
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;

/// Free-form polygon shape from normalized points (0.0 to 1.0 across
/// the shape bounds), like the reference rows: solid blue triangle
/// and solid purple hexagon. Built-ins cover `triangle`, `diamond`,
/// `pentagon`, `hexagon`, `star` and regular `polygon` outlines;
/// `points` takes any custom outline. Display-only (no mouse handling).
pub struct CustomShape {
    points: Vec<(f32, f32)>,
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

impl CustomShape {
    /// Custom outline from normalized points (0.0 to 1.0). At least
    /// three points are expected; fewer draw nothing.
    pub fn points(points: Vec<(f32, f32)>, width: f32, height: f32) -> Self {
        Self {
            points,
            width: width.max(0.0),
            height: height.max(0.0),
            fill: ShapeFill::Solid(SHAPE_DEFAULT_FILL),
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

    /// Up-pointing triangle, like the blue reference shape.
    pub fn triangle(width: f32, height: f32) -> Self {
        Self::points(vec![(0.5, 0.0), (1.0, 1.0), (0.0, 1.0)], width, height)
    }

    /// Four-point diamond (left/right/top/bottom tips).
    pub fn diamond(width: f32, height: f32) -> Self {
        Self::points(
            vec![(0.5, 0.0), (1.0, 0.5), (0.5, 1.0), (0.0, 0.5)],
            width,
            height,
        )
    }

    /// Regular polygon with `sides` corners (clamped to >= 3),
    /// starting at the top so even counts get a top tip, like the
    /// purple hexagon reference shape.
    pub fn polygon(sides: usize, width: f32, height: f32) -> Self {
        let sides = sides.max(3);
        let points = (0..sides)
            .map(|i| {
                let angle = -PI / 2.0 + 2.0 * PI * i as f32 / sides as f32;
                (
                    0.5 + 0.5 * angle.cos(),
                    0.5 + 0.5 * angle.sin(),
                )
            })
            .collect();
        Self::points(points, width, height)
    }

    /// Regular pentagon.
    pub fn pentagon(width: f32, height: f32) -> Self {
        Self::polygon(5, width, height)
    }

    /// Regular hexagon, like the purple reference shape.
    pub fn hexagon(width: f32, height: f32) -> Self {
        Self::polygon(6, width, height)
    }

    /// Five-point star.
    pub fn star(width: f32, height: f32) -> Self {
        let points = (0..10)
            .map(|i| {
                let outer = i % 2 == 0;
                let radius = if outer { 0.5 } else { 0.5 * 0.382 };
                let angle = -PI / 2.0 + PI * i as f32 / 5.0;
                (
                    0.5 + radius * angle.cos(),
                    0.5 + radius * angle.sin(),
                )
            })
            .collect();
        Self::points(points, width, height)
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

    /// Normalized outline points (0.0 to 1.0).
    pub fn points_value(&self) -> &[(f32, f32)] {
        &self.points
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }

    fn path(&self, x: f32, y: f32, scale: f32) -> Option<BezPath> {
        if self.points.len() < 3 {
            return None;
        }
        let s = scale as f64;
        let mut path = BezPath::new();
        for (index, (nx, ny)) in self.points.iter().enumerate() {
            let px = (x + nx.clamp(0.0, 1.0) * self.placed_w) as f64 * s;
            let py = (y + ny.clamp(0.0, 1.0) * self.placed_h) as f64 * s;
            if index == 0 {
                path.move_to((px, py));
            } else {
                path.line_to((px, py));
            }
        }
        path.close_path();
        Some(path)
    }
}

impl View for CustomShape {
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
        let Some(path) = self.path(self.x, self.y, fonts.scale) else {
            return;
        };
        if self.shadow {
            if let Some(ghost) =
                self.path(self.x, self.y + SHAPE_SHADOW_DY, fonts.scale)
            {
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(SHAPE_SHADOW),
                    None,
                    &ghost,
                );
            }
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
            scene.fill(Fill::NonZero, Affine::IDENTITY, &brush, None, &path);
        } else {
            let scale = fonts.scale as f64;
            scene.stroke(
                &Stroke::new(self.stroke_width.max(1.0) as f64 * scale),
                Affine::IDENTITY,
                &Brush::Solid(stroke_color(self.stroke, self.focused)),
                None,
                &path,
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
    fn triangle_has_three_points() {
        let shape = CustomShape::triangle(120.0, 110.0);
        assert_eq!(shape.points_value().len(), 3);
    }

    #[test]
    fn hexagon_has_six_points() {
        let shape = CustomShape::hexagon(120.0, 110.0);
        assert_eq!(shape.points_value().len(), 6);
        for (nx, ny) in shape.points_value() {
            assert!((0.0..=1.0).contains(nx));
            assert!((0.0..=1.0).contains(ny));
        }
    }

    #[test]
    fn polygon_clamps_to_triangle() {
        let shape = CustomShape::polygon(0, 100.0, 100.0);
        assert_eq!(shape.points_value().len(), 3);
    }

    #[test]
    fn keeps_intrinsic_size() {
        let mut fonts = FontSystem::new();
        let mut shape = CustomShape::hexagon(120.0, 110.0);
        assert_eq!(shape.measure(&mut fonts), (120.0, 110.0));
        shape.place(&mut fonts, 0.0, 0.0, 120.0, 110.0);
        assert_eq!(shape.rect(), (0.0, 0.0, 120.0, 110.0));
    }

    #[test]
    fn outline_mode_reports_unfilled() {
        let shape = CustomShape::triangle(120.0, 110.0).filled(false);
        assert!(!shape.is_filled());
    }
}
