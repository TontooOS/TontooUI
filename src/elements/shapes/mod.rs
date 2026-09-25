pub mod capsule;
pub mod circle;
pub mod custom;
pub mod rectangle;
pub mod rounded;

pub use capsule::Capsule;
pub use circle::Circle;
pub use custom::CustomShape;
pub use rectangle::Rectangle;
pub use rounded::RoundedRectangle;

use vello::kurbo::Point;
use vello::peniko::{Brush, Color, ColorStop, Gradient};

use crate::theme::desaturate;

/// Default fill (system blue accent, like the reference rows).
pub const SHAPE_DEFAULT_FILL: Color = Color::from_rgb8(0x00, 0x7a, 0xff);
/// Default outline color for the stroked (unfilled) look.
pub const SHAPE_DEFAULT_STROKE: Color = Color::from_rgb8(0xff, 0x3b, 0x30);
/// Default outline width in logical px.
pub const SHAPE_STROKE_W: f32 = 4.0;
/// Default corner radius of `RoundedRectangle` in logical px.
pub const SHAPE_CORNER_RADIUS: f32 = 16.0;
/// Small drop shadow offset in logical px (behind the shape).
pub const SHAPE_SHADOW_DY: f32 = 2.0;
/// Small drop shadow color.
pub const SHAPE_SHADOW: Color = Color::from_rgba8(0, 0, 0, 64);

/// Paint of a shape: solid color, linear gradient (angle in degrees,
/// 0 = left to right, 90 = top to bottom) or radial gradient
/// (center to edge). Mirrors the reference rows: solid fill, outline
/// only, linear gradient fill.
#[derive(Clone, Debug, PartialEq)]
pub enum ShapeFill {
    Solid(Color),
    LinearGradient { colors: Vec<Color>, angle_deg: f32 },
    RadialGradient { colors: Vec<Color> },
}

impl Default for ShapeFill {
    fn default() -> Self {
        Self::Solid(SHAPE_DEFAULT_FILL)
    }
}

impl ShapeFill {
    /// Solid color fill.
    pub fn solid(color: Color) -> Self {
        Self::Solid(color)
    }

    /// Linear gradient fill across the shape bounds. At least two
    /// colors are expected; a single color behaves like a solid.
    pub fn linear(colors: Vec<Color>, angle_deg: f32) -> Self {
        Self::LinearGradient { colors, angle_deg }
    }

    /// Radial gradient fill from the shape center to its edge.
    pub fn radial(colors: Vec<Color>) -> Self {
        Self::RadialGradient { colors }
    }

    fn eff_colors(&self, focused: bool) -> Vec<Color> {
        let map = |c: &Color| {
            if focused {
                *c
            } else {
                desaturate(*c)
            }
        };
        match self {
            Self::Solid(color) => vec![map(color)],
            Self::LinearGradient { colors, .. } => colors.iter().map(map).collect(),
            Self::RadialGradient { colors } => colors.iter().map(map).collect(),
        }
    }
}

/// Evenly spread gradient stops across `colors` (offsets 0.0 to 1.0).
/// A single color becomes one stop at 0.0; empty input falls back to
/// `SHAPE_DEFAULT_FILL` so the brush is never empty.
pub(crate) fn gradient_stops(colors: &[Color]) -> Vec<ColorStop> {
    if colors.is_empty() {
        return vec![ColorStop {
            offset: 0.0,
            color: SHAPE_DEFAULT_FILL.into(),
        }];
    }
    let n = colors.len();
    colors
        .iter()
        .enumerate()
        .map(|(index, color)| ColorStop {
            offset: if n <= 1 {
                0.0
            } else {
                index as f32 / (n - 1) as f32
            },
            color: (*color).into(),
        })
        .collect()
}

/// Brush for `fill` spanning the logical bounds
/// (`x`, `y`, `width`, `height`). `scale` is `fonts.scale`.
/// Unfocused windows desaturate every stop like the rest of the palette.
pub(crate) fn fill_brush(
    fill: &ShapeFill,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    scale: f32,
    focused: bool,
) -> Brush {
    let s = scale as f64;
    let colors = fill.eff_colors(focused);
    match fill {
        ShapeFill::Solid(_) => {
            let color = colors.into_iter().next().unwrap_or(SHAPE_DEFAULT_FILL);
            Brush::Solid(color)
        }
        ShapeFill::LinearGradient { angle_deg, .. } => {
            let rad = angle_deg.to_radians() as f64;
            let (dx, dy) = (rad.cos(), rad.sin());
            let (cx, cy) = (x + width / 2.0, y + height / 2.0);
            let (hx, hy) = (dx * width as f64 / 2.0, dy * height as f64 / 2.0);
            Brush::Gradient(
                Gradient::new_linear(
                    Point::new((cx as f64 - hx) * s, (cy as f64 - hy) * s),
                    Point::new((cx as f64 + hx) * s, (cy as f64 + hy) * s),
                )
                .with_stops(gradient_stops(&colors).as_slice()),
            )
        }
        ShapeFill::RadialGradient { .. } => {
            let cx = (x + width / 2.0) as f64 * s;
            let cy = (y + height / 2.0) as f64 * s;
            let radius = (width.min(height) / 2.0).max(1.0) as f64 * s;
            Brush::Gradient(
                Gradient::new_radial(Point::new(cx, cy), radius as f32)
                    .with_stops(gradient_stops(&colors).as_slice()),
            )
        }
    }
}

/// Effective stroke color (desaturated when unfocused).
pub(crate) fn stroke_color(color: Color, focused: bool) -> Color {
    if focused {
        color
    } else {
        desaturate(color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stops_spread_evenly() {
        let stops = gradient_stops(&[Color::WHITE, Color::BLACK]);
        assert_eq!(stops.len(), 2);
        assert_eq!(stops[0].offset, 0.0);
        assert_eq!(stops[1].offset, 1.0);
    }

    #[test]
    fn stops_fall_back_on_empty() {
        let stops = gradient_stops(&[]);
        assert_eq!(stops.len(), 1);
    }

    #[test]
    fn solid_brush_stays_solid() {
        let brush = fill_brush(
            &ShapeFill::Solid(Color::from_rgb8(1, 2, 3)),
            0.0,
            0.0,
            100.0,
            50.0,
            1.0,
            true,
        );
        assert!(matches!(brush, Brush::Solid(_)));
    }

    #[test]
    fn unfocused_desaturates() {
        let brush = fill_brush(
            &ShapeFill::Solid(Color::from_rgb8(0x00, 0x7a, 0xff)),
            0.0,
            0.0,
            100.0,
            50.0,
            1.0,
            false,
        );
        match brush {
            Brush::Solid(color) => {
                let c = color.to_rgba8();
                assert_eq!(c.r, c.g);
                assert_eq!(c.g, c.b);
            }
            _ => panic!("expected solid"),
        }
    }
}
