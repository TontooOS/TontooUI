use vello::peniko::Color;
use vello::peniko::color::HueDirection;

/// Values a tween can interpolate. Linear blend from `self` (t=0) to
/// `other` (t=1).
pub trait Animatable: Clone {
    fn lerp(&self, other: &Self, t: f32) -> Self;
}

impl Animatable for f32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Animatable for f64 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t as f64
    }
}

impl Animatable for (f32, f32) {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        (self.0.lerp(&other.0, t), self.1.lerp(&other.1, t))
    }
}

/// Perceptual blend through the color crate (shorter hue arc). Note:
/// `Color` also has an inherent 3-argument `lerp`, so plain
/// `color.lerp(&other, t)` calls bind to that one; generic tween code uses
/// this trait impl.
impl Animatable for Color {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        (*self).lerp(*other, t, HueDirection::Shorter)
    }
}

/// Discrete values switch halfway instead of blending.
impl Animatable for bool {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        if t < 0.5 { *self } else { *other }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_lerp() {
        assert_eq!(0.0_f32.lerp(&10.0, 0.25), 2.5);
        assert_eq!(0.0_f64.lerp(&10.0, 0.25), 2.5);
    }

    #[test]
    fn tuple_lerp() {
        assert_eq!((0.0_f32, 10.0_f32).lerp(&(10.0, 0.0), 0.5), (5.0, 5.0));
    }

    #[test]
    fn color_endpoints() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        assert_eq!(Animatable::lerp(&a, &b, 0.0), a);
        assert_eq!(Animatable::lerp(&a, &b, 1.0), b);
        let mid = Animatable::lerp(&a, &b, 0.5);
        let rgba = mid.to_rgba8();
        assert_eq!((rgba.r, rgba.g, rgba.b, rgba.a), (128, 128, 128, 255));
    }

    #[test]
    fn bool_steps() {
        assert!(!false.lerp(&true, 0.49));
        assert!(false.lerp(&true, 0.5));
    }
}
