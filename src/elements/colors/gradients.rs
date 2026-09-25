use vello::kurbo::Point;
use vello::peniko::{Brush, Color, ColorStop, Gradient};

use super::system::SystemColor;

/// General gradient paint: linear, vertical, radial and angular
/// (conic sweep), like the reference bars. Built from plain colors
/// or system colors; `brush` spans the paint over a logical rect at
/// the display scale.
#[derive(Clone, Debug, PartialEq)]
pub enum GradientPaint {
    /// Straight blend at `angle_deg` (0 = left to right).
    Linear { colors: Vec<Color>, angle_deg: f32 },
    /// Straight blend from top to bottom.
    Vertical { colors: Vec<Color> },
    /// Blend from the center glow out to the edge.
    Radial { colors: Vec<Color> },
    /// Conic sweep around the center (rainbow wheel).
    Angular { colors: Vec<Color> },
}

impl GradientPaint {
    /// Linear blend at an angle in degrees.
    pub fn linear(colors: Vec<Color>, angle_deg: f32) -> Self {
        Self::Linear { colors, angle_deg }
    }

    /// Linear blend from system colors at an angle.
    pub fn linear_system(colors: Vec<SystemColor>, angle_deg: f32) -> Self {
        Self::linear(colors.iter().map(|c| c.color()).collect(), angle_deg)
    }

    /// Top-to-bottom blend.
    pub fn vertical(colors: Vec<Color>) -> Self {
        Self::Vertical { colors }
    }

    /// Top-to-bottom blend from system colors.
    pub fn vertical_system(colors: Vec<SystemColor>) -> Self {
        Self::vertical(colors.iter().map(|c| c.color()).collect())
    }

    /// Center glow out to the edge.
    pub fn radial(colors: Vec<Color>) -> Self {
        Self::Radial { colors }
    }

    /// Center glow from system colors.
    pub fn radial_system(colors: Vec<SystemColor>) -> Self {
        Self::radial(colors.iter().map(|c| c.color()).collect())
    }

    /// Conic sweep around the center.
    pub fn angular(colors: Vec<Color>) -> Self {
        Self::Angular { colors }
    }

    /// Conic sweep from system colors.
    pub fn angular_system(colors: Vec<SystemColor>) -> Self {
        Self::angular(colors.iter().map(|c| c.color()).collect())
    }

    /// Reference preset: blue to purple, left to right.
    pub fn preset_linear() -> Self {
        Self::linear_system(vec![SystemColor::Blue, SystemColor::Purple], 0.0)
    }

    /// Reference preset: red, orange and yellow, top to bottom.
    pub fn preset_vertical() -> Self {
        Self::vertical_system(vec![
            SystemColor::Red,
            SystemColor::Orange,
            SystemColor::Yellow,
        ])
    }

    /// Reference preset: bright center glow into purple.
    pub fn preset_radial() -> Self {
        Self::Radial {
            colors: vec![
                Color::from_rgb8(0xd6, 0xe6, 0xff),
                SystemColor::Blue.color(),
                SystemColor::Purple.color(),
            ],
        }
    }

    /// Reference preset: conic rainbow wheel (green, yellow, red,
    /// purple, blue, back to green).
    pub fn preset_angular() -> Self {
        Self::angular_system(vec![
            SystemColor::Green,
            SystemColor::Yellow,
            SystemColor::Red,
            SystemColor::Purple,
            SystemColor::Blue,
            SystemColor::Green,
        ])
    }

    /// Paint colors in stop order (empty falls back to blue).
    pub fn colors(&self) -> Vec<Color> {
        let colors = match self {
            Self::Linear { colors, .. } => colors.clone(),
            Self::Vertical { colors } => colors.clone(),
            Self::Radial { colors } => colors.clone(),
            Self::Angular { colors } => colors.clone(),
        };
        if colors.is_empty() {
            vec![SystemColor::Blue.color()]
        } else {
            colors
        }
    }

    /// Brush spanning the logical (`x`, `y`, `width`, `height`) rect
    /// at `scale` (`fonts.scale`).
    pub fn brush(&self, x: f32, y: f32, width: f32, height: f32, scale: f32) -> Brush {
        let s = scale as f64;
        let stops = even_stops(&self.colors());
        match self {
            Self::Linear { angle_deg, .. } => {
                let rad = angle_deg.to_radians() as f64;
                let (dx, dy) = (rad.cos(), rad.sin());
                let (cx, cy) = (x + width / 2.0, y + height / 2.0);
                let (hx, hy) = (dx * width as f64 / 2.0, dy * height as f64 / 2.0);
                Brush::Gradient(
                    Gradient::new_linear(
                        Point::new((cx as f64 - hx) * s, (cy as f64 - hy) * s),
                        Point::new((cx as f64 + hx) * s, (cy as f64 + hy) * s),
                    )
                    .with_stops(stops.as_slice()),
                )
            }
            Self::Vertical { .. } => {
                Brush::Gradient(
                    Gradient::new_linear(
                        Point::new((x + width / 2.0) as f64 * s, y as f64 * s),
                        Point::new((x + width / 2.0) as f64 * s, (y + height) as f64 * s),
                    )
                    .with_stops(stops.as_slice()),
                )
            }
            Self::Radial { .. } => {
                let radius = (width.min(height) / 2.0).max(1.0) as f64 * s;
                Brush::Gradient(
                    Gradient::new_radial(
                        Point::new((x + width / 2.0) as f64 * s, (y + height / 2.0) as f64 * s),
                        radius as f32,
                    )
                    .with_stops(stops.as_slice()),
                )
            }
            Self::Angular { .. } => Brush::Gradient(
                Gradient::new_sweep(
                    Point::new((x + width / 2.0) as f64 * s, (y + height / 2.0) as f64 * s),
                    0.0,
                    std::f32::consts::TAU,
                )
                .with_stops(stops.as_slice()),
            ),
        }
    }
}

/// Evenly spread stops across `colors` (offsets 0.0 to 1.0).
fn even_stops(colors: &[Color]) -> Vec<ColorStop> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_builds_a_gradient_brush() {
        let paints = [
            GradientPaint::linear_system(vec![SystemColor::Blue, SystemColor::Purple], 0.0),
            GradientPaint::vertical_system(vec![SystemColor::Red, SystemColor::Yellow]),
            GradientPaint::radial_system(vec![SystemColor::Blue, SystemColor::Purple]),
            GradientPaint::angular_system(vec![SystemColor::Green, SystemColor::Red]),
        ];
        for paint in paints {
            assert!(paint.colors().len() >= 2);
            assert!(
                matches!(paint.brush(0.0, 0.0, 200.0, 100.0, 1.0), Brush::Gradient(_)),
                "{paint:?}"
            );
        }
    }

    #[test]
    fn presets_match_reference_bars() {
        assert_eq!(
            GradientPaint::preset_linear().colors(),
            vec![SystemColor::Blue.color(), SystemColor::Purple.color()]
        );
        assert_eq!(GradientPaint::preset_vertical().colors().len(), 3);
        assert_eq!(GradientPaint::preset_radial().colors().len(), 3);
        // Rainbow wheel closes the loop.
        let wheel = GradientPaint::preset_angular().colors();
        assert_eq!(wheel.first(), wheel.last());
    }

    #[test]
    fn empty_falls_back_to_blue() {
        assert_eq!(
            GradientPaint::linear(vec![], 0.0).colors(),
            vec![SystemColor::Blue.color()]
        );
    }
}
