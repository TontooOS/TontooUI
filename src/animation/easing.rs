/// Easing curves. Maps linear progress `p` in [0, 1] to eased progress.
/// Inputs outside [0, 1] are clamped, so overshooting drivers stay safe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    SineIn,
    SineOut,
    SineInOut,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    ExpoOut,
    BackOut,
    /// CSS-style cubic-bezier(x1, y1, x2, y2).
    Bezier(f32, f32, f32, f32),
}

impl Default for Easing {
    fn default() -> Self {
        Easing::CubicOut
    }
}

impl Easing {
    pub fn apply(&self, progress: f32) -> f32 {
        let p = progress.clamp(0.0, 1.0);
        match *self {
            Easing::Linear => p,
            Easing::SineIn => 1.0 - (p * std::f32::consts::FRAC_PI_2).cos(),
            Easing::SineOut => (p * std::f32::consts::FRAC_PI_2).sin(),
            Easing::SineInOut => 0.5 * (1.0 - (p * std::f32::consts::PI).cos()),
            Easing::QuadIn => p * p,
            Easing::QuadOut => 1.0 - (1.0 - p) * (1.0 - p),
            Easing::QuadInOut => {
                if p < 0.5 {
                    2.0 * p * p
                } else {
                    1.0 - (-2.0 * p + 2.0).powi(2) / 2.0
                }
            }
            Easing::CubicIn => p * p * p,
            Easing::CubicOut => 1.0 - (1.0 - p).powi(3),
            Easing::CubicInOut => {
                if p < 0.5 {
                    4.0 * p * p * p
                } else {
                    1.0 - (-2.0 * p + 2.0).powi(3) / 2.0
                }
            }
            Easing::ExpoOut => {
                if p >= 1.0 {
                    1.0
                } else {
                    1.0 - 2.0_f32.powf(-10.0 * p)
                }
            }
            Easing::BackOut => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (p - 1.0).powi(3) + c1 * (p - 1.0).powi(2)
            }
            Easing::Bezier(x1, y1, x2, y2) => cubic_bezier(x1, y1, x2, y2, p),
        }
    }
}

/// Solve CSS cubic-bezier(x1, y1, x2, y2) for input progress `x`.
/// Newton-Raphson with bisection fallback (same approach as WebCore).
fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    fn sample(ax: f32, bx: f32, cx: f32, t: f32) -> f32 {
        ((ax * t + bx) * t + cx) * t
    }
    // Coefficients for x(t) and y(t).
    let cx = 3.0 * x1;
    let bx = 3.0 * (x2 - x1) - cx;
    let ax = 1.0 - cx - bx;
    let cy = 3.0 * y1;
    let by = 3.0 * (y2 - y1) - cy;
    let ay = 1.0 - cy - by;

    // Newton-Raphson: solve x(t) = x.
    let mut t = x;
    for _ in 0..8 {
        let fx = sample(ax, bx, cx, t) - x;
        if fx.abs() < 1e-6 {
            return sample(ay, by, cy, t);
        }
        let dfx = 3.0 * ax * t * t + 2.0 * bx * t + cx;
        if dfx.abs() < 1e-6 {
            break;
        }
        t -= fx / dfx;
    }
    // Bisection fallback.
    let mut lo = 0.0;
    let mut hi = 1.0;
    t = x;
    while hi - lo > 1e-6 {
        let fx = sample(ax, bx, cx, t);
        if fx < x {
            lo = t;
        } else {
            hi = t;
        }
        t = (lo + hi) * 0.5;
    }
    sample(ay, by, cy, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_hold_for_all_curves() {
        let curves = [
            Easing::Linear,
            Easing::SineIn,
            Easing::SineOut,
            Easing::SineInOut,
            Easing::QuadIn,
            Easing::QuadOut,
            Easing::QuadInOut,
            Easing::CubicIn,
            Easing::CubicOut,
            Easing::CubicInOut,
            Easing::ExpoOut,
            Easing::BackOut,
            Easing::Bezier(0.25, 0.1, 0.25, 1.0),
        ];
        for curve in curves {
            assert_eq!(curve.apply(0.0), 0.0, "{curve:?} start");
            assert!((curve.apply(1.0) - 1.0).abs() < 1e-5, "{curve:?} end");
        }
    }

    #[test]
    fn known_values() {
        assert!((Easing::Linear.apply(0.5) - 0.5).abs() < 1e-6);
        assert!((Easing::CubicOut.apply(0.5) - 0.875).abs() < 1e-6);
        assert!((Easing::QuadIn.apply(0.5) - 0.25).abs() < 1e-6);
        // ease-out leads linear early in the curve.
        assert!(Easing::CubicOut.apply(0.25) > 0.25);
        assert!(Easing::CubicIn.apply(0.25) < 0.25);
    }

    #[test]
    fn linear_bezier_matches_linear() {
        let bezier = Easing::Bezier(0.0, 0.0, 1.0, 1.0);
        for i in 0..=10 {
            let p = i as f32 / 10.0;
            assert!((bezier.apply(p) - p).abs() < 1e-4, "p={p}");
        }
    }

    #[test]
    fn css_ease_shape() {
        // CSS `ease` starts slow, moves fast, lands soft.
        let ease = Easing::Bezier(0.25, 0.1, 0.25, 1.0);
        assert!(ease.apply(0.2) > 0.2);
        assert!((ease.apply(1.0) - 1.0).abs() < 1e-4);
    }
}
