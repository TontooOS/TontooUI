pub mod animated;

pub use animated::{Animated, Spin};

use crate::animation::{Animatable, Easing, Repeat, Tween};

/// Default preset duration in seconds (SwiftUI-like snappy default).
pub const ANIM_DEFAULT_SECONDS: f32 = 0.35;
/// Clip margin around the animated frame in logical px. Fading uses a
/// layer clipped to the placed bounds expanded by this, so offset
/// children never get cut mid-flight.
pub const ANIM_CLIP_MARGIN: f32 = 4096.0;

/// Sampled transform of one frame: pixel offset, uniform scale,
/// opacity and rotation in degrees. Every preset writes one channel;
/// phases and keyframes blend all four. Combined timelines add
/// offsets and rotations, multiply scales and opacities.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub offset: (f32, f32),
    pub scale: f32,
    pub opacity: f32,
    pub rotation: f32,
}

impl Transform {
    /// No offset, unit scale, full opacity, no rotation.
    pub fn identity() -> Self {
        Self {
            offset: (0.0, 0.0),
            scale: 1.0,
            opacity: 1.0,
            rotation: 0.0,
        }
    }

    /// Pixel offset from the placed origin.
    pub fn offset(x: f32, y: f32) -> Self {
        Self {
            offset: (x, y),
            ..Self::identity()
        }
    }

    /// Uniform scale factor around the anchor.
    pub fn scale(factor: f32) -> Self {
        Self {
            scale: factor,
            ..Self::identity()
        }
    }

    /// Opacity (clamped to 0..1 on blend).
    pub fn opacity(alpha: f32) -> Self {
        Self {
            opacity: alpha,
            ..Self::identity()
        }
    }

    /// Rotation in degrees around the anchor.
    pub fn rotation(degrees: f32) -> Self {
        Self {
            rotation: degrees,
            ..Self::identity()
        }
    }

    /// Fold another sampled transform into this one: offsets and
    /// rotations add, scales and opacities multiply.
    pub fn combine(&self, other: &Transform) -> Transform {
        Transform {
            offset: (self.offset.0 + other.offset.0, self.offset.1 + other.offset.1),
            scale: self.scale * other.scale,
            opacity: (self.opacity * other.opacity).clamp(0.0, 1.0),
            rotation: self.rotation + other.rotation,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Animatable for Transform {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Transform {
            offset: self.offset.lerp(&other.offset, t),
            scale: self.scale.lerp(&other.scale, t),
            opacity: self.opacity.lerp(&other.opacity, t).clamp(0.0, 1.0),
            rotation: self.rotation.lerp(&other.rotation, t),
        }
    }
}

/// Anchor for scale and rotation: the fixed point of the frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Anchor {
    TopLeading,
    Top,
    TopTrailing,
    Leading,
    #[default]
    Center,
    Trailing,
    BottomLeading,
    Bottom,
    BottomTrailing,
}

impl Anchor {
    /// Fixed point inside the (`x`, `y`, `w`, `h`) frame in logical px.
    pub fn point(&self, x: f32, y: f32, w: f32, h: f32) -> (f32, f32) {
        let px = match self {
            Self::TopLeading | Self::Leading | Self::BottomLeading => x,
            Self::Top | Self::Center | Self::Bottom => x + w / 2.0,
            Self::TopTrailing | Self::Trailing | Self::BottomTrailing => x + w,
        };
        let py = match self {
            Self::TopLeading | Self::Top | Self::TopTrailing => y,
            Self::Leading | Self::Center | Self::Trailing => y + h / 2.0,
            Self::BottomLeading | Self::Bottom | Self::BottomTrailing => y + h,
        };
        (px, py)
    }
}

/// Timing of one motion: duration plus engine delay, easing, repeat
/// and autoreverse (all sampled through `Tween`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimSpec {
    pub duration: f32,
    pub delay: f32,
    pub easing: Easing,
    pub repeat: Repeat,
    pub autoreverse: bool,
}

impl AnimSpec {
    pub fn new(duration: f32) -> Self {
        Self {
            duration: duration.max(0.0),
            delay: 0.0,
            easing: Easing::CubicInOut,
            repeat: Repeat::Never,
            autoreverse: false,
        }
    }

    pub fn seconds(duration: f32) -> Self {
        Self::new(duration)
    }

    pub fn delay(mut self, seconds: f32) -> Self {
        self.delay = seconds.max(0.0);
        self
    }

    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    pub fn repeat(mut self, repeat: Repeat) -> Self {
        self.repeat = repeat;
        self
    }

    pub fn autoreverse(mut self, autoreverse: bool) -> Self {
        self.autoreverse = autoreverse;
        self
    }

    /// Engine tween between two transforms under this timing.
    pub fn tween(&self, from: Transform, to: Transform) -> Tween<Transform> {
        Tween::new(from, to, self.duration)
            .delay(self.delay)
            .easing(self.easing)
            .repeat(self.repeat)
            .autoreverse(self.autoreverse)
    }
}

impl Default for AnimSpec {
    fn default() -> Self {
        Self::new(ANIM_DEFAULT_SECONDS)
    }
}

/// One named phase of a phase animation: the transform holds while
/// the phase is active; transitions between phases animate over
/// `seconds` with the shared easing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Phase {
    pub transform: Transform,
    pub seconds: f32,
}

impl Phase {
    pub fn new(transform: Transform, seconds: f32) -> Self {
        Self {
            transform,
            seconds: seconds.max(0.0),
        }
    }
}

/// One keyframe of a keyframe track: transform at normalized time
/// `at` (0..1 of the total), reached with `easing` from the previous
/// key.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Keyframe {
    pub at: f32,
    pub transform: Transform,
    pub easing: Easing,
}

impl Keyframe {
    pub fn new(at: f32, transform: Transform, easing: Easing) -> Self {
        Self {
            at: at.clamp(0.0, 1.0),
            transform,
            easing,
        }
    }

    /// Keyframe at time `at` with the default UI easing.
    pub fn at(at: f32, transform: Transform) -> Self {
        Self::new(at, transform, Easing::CubicInOut)
    }
}

/// A motion timeline: back-to-back engine tweens (one per preset,
/// phase transition or keyframe span) with a shared repeat over the
/// whole timeline. Autoreverse ping-pongs the full timeline.
/// Sampled with elapsed seconds; combines with other timelines
/// through `Transform::combine`.
#[derive(Clone, Debug)]
pub struct Timeline {
    segments: Vec<(Tween<Transform>, f32)>,
    repeat: Repeat,
    autoreverse: bool,
}

impl Timeline {
    /// Single motion from `from` to `to` under `spec`.
    pub fn single(from: Transform, to: Transform, spec: &AnimSpec) -> Self {
        Self {
            segments: vec![(spec.tween(from, to), spec.duration)],
            repeat: spec.repeat,
            autoreverse: spec.autoreverse,
        }
    }

    /// Phase motion: identity, then each phase in order, every
    /// transition eased with `easing` over that phase's seconds.
    pub fn phases(phases: &[Phase], easing: Easing) -> Self {
        let mut segments = Vec::new();
        let mut from = Transform::identity();
        for phase in phases {
            let spec = AnimSpec::new(phase.seconds).easing(easing);
            segments.push((spec.tween(from, phase.transform), phase.seconds));
            from = phase.transform;
        }
        Self {
            segments,
            repeat: Repeat::Never,
            autoreverse: false,
        }
    }

    /// Keyframe motion over `total` seconds: spans between consecutive
    /// keys ease with the arriving key's easing. A head gap eases out
    /// of identity; a tail gap holds the last key.
    pub fn keyframes(keys: &[Keyframe], total: f32) -> Self {
        let total = total.max(0.0);
        let mut sorted = keys.to_vec();
        sorted.sort_by(|a, b| a.at.partial_cmp(&b.at).unwrap_or(std::cmp::Ordering::Equal));
        let mut segments = Vec::new();
        let mut from = Transform::identity();
        let mut prev_at = 0.0;
        for key in &sorted {
            let span = ((key.at - prev_at).max(0.0) * total).max(0.0);
            let spec = AnimSpec::new(span).easing(key.easing);
            segments.push((spec.tween(from, key.transform), span));
            from = key.transform;
            prev_at = key.at;
        }
        if prev_at < 1.0 {
            let span = ((1.0 - prev_at) * total).max(0.0);
            let spec = AnimSpec::new(span).easing(Easing::Linear);
            segments.push((spec.tween(from, from), span));
        }
        Self {
            segments,
            repeat: Repeat::Never,
            autoreverse: false,
        }
    }

    pub fn repeat(mut self, repeat: Repeat) -> Self {
        self.repeat = repeat;
        self
    }

    pub fn set_repeat(&mut self, repeat: Repeat) {
        self.repeat = repeat;
    }

    pub fn autoreverse(mut self, autoreverse: bool) -> Self {
        self.autoreverse = autoreverse;
        self
    }

    pub fn set_autoreverse(&mut self, autoreverse: bool) {
        self.autoreverse = autoreverse;
    }

    fn total(&self) -> f32 {
        self.segments.iter().map(|(_, d)| *d).sum()
    }

    /// Sample at `elapsed` seconds. Returns the transform plus done
    /// (never done for `Forever`, like the engine). Repeating
    /// timelines wrap around the total; with autoreverse every pass
    /// ping-pongs back. Delays live inside the first segment's tween
    /// and re-apply each cycle.
    pub fn sample(&self, elapsed: f32) -> (Transform, bool) {
        if self.segments.is_empty() {
            return (Transform::identity(), true);
        }
        let total = self.total();
        if total <= 0.0 {
            let last = self.segments.last().map(|(t, _)| t.sample(0.0).0);
            return (last.unwrap_or_else(Transform::identity), true);
        }
        // One pass length: forward only, or forward plus back.
        let unit = if self.autoreverse { total * 2.0 } else { total };
        let local = match self.repeat {
            Repeat::Never => elapsed,
            Repeat::Times(n) => {
                if elapsed >= unit * n.max(1) as f32 {
                    let back_at_start = self.autoreverse;
                    return (
                        if back_at_start {
                            self.start_state()
                        } else {
                            self.end_state()
                        },
                        true,
                    );
                }
                ping_pong(elapsed % unit, total, self.autoreverse)
            }
            Repeat::Forever => ping_pong(elapsed % unit, total, self.autoreverse),
        };
        if self.repeat == Repeat::Never && local >= total {
            return (self.end_state(), true);
        }
        (self.sample_forward(local), false)
    }

    fn sample_forward(&self, local: f32) -> Transform {
        let mut cursor = 0.0;
        for (tween, duration) in &self.segments {
            // Zero spans hold their start without consuming time.
            if *duration <= 0.0 {
                if local <= cursor {
                    return tween.sample(0.0).0;
                }
                continue;
            }
            if local < cursor + duration {
                return tween.sample(local - cursor).0;
            }
            cursor += duration;
        }
        self.end_state()
    }

    fn start_state(&self) -> Transform {
        self.segments
            .first()
            .map(|(tween, _)| tween.sample(0.0).0)
            .unwrap_or_else(Transform::identity)
    }

    fn end_state(&self) -> Transform {
        self.segments
            .last()
            .map(|(tween, duration)| tween.sample(*duration).0)
            .unwrap_or_else(Transform::identity)
    }
}

/// Map a wrapped pass offset into a forward sampling position:
/// plain wraps run forward, ping-pong runs back after the midpoint.
fn ping_pong(offset: f32, total: f32, autoreverse: bool) -> f32 {
    if autoreverse && offset >= total {
        (total * 2.0 - offset).max(0.0)
    } else {
        offset
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            segments: Vec::new(),
            repeat: Repeat::Never,
            autoreverse: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_combines_channels() {
        let a = Transform {
            offset: (10.0, 0.0),
            scale: 2.0,
            opacity: 0.5,
            rotation: 30.0,
        };
        let b = Transform {
            offset: (0.0, 5.0),
            scale: 0.5,
            opacity: 0.5,
            rotation: 15.0,
        };
        let c = a.combine(&b);
        assert_eq!(c.offset, (10.0, 5.0));
        assert_eq!(c.scale, 1.0);
        assert_eq!(c.opacity, 0.25);
        assert_eq!(c.rotation, 45.0);
    }

    #[test]
    fn opacity_clamps_on_blend() {
        let a = Transform::opacity(2.0);
        let b = Transform::opacity(2.0);
        assert_eq!(a.combine(&b).opacity, 1.0);
        let mid = Animatable::lerp(&Transform::opacity(0.0), &Transform::opacity(1.0), 0.5);
        assert_eq!(mid.opacity, 0.5);
    }

    #[test]
    fn anchor_points_match_corners() {
        assert_eq!(Anchor::Center.point(0.0, 0.0, 100.0, 40.0), (50.0, 20.0));
        assert_eq!(Anchor::TopLeading.point(10.0, 5.0, 100.0, 40.0), (10.0, 5.0));
        assert_eq!(
            Anchor::BottomTrailing.point(10.0, 5.0, 100.0, 40.0),
            (110.0, 45.0)
        );
    }

    #[test]
    fn single_timeline_runs_and_finishes() {
        let spec = AnimSpec::new(1.0).easing(Easing::Linear);
        let line = Timeline::single(Transform::identity(), Transform::scale(2.0), &spec);
        let (start, done) = line.sample(0.0);
        assert_eq!(start.scale, 1.0);
        assert!(!done);
        assert_eq!(line.sample(0.5).0.scale, 1.5);
        let (end, done) = line.sample(2.0);
        assert_eq!(end.scale, 2.0);
        assert!(done);
    }

    #[test]
    fn forever_loops_without_finishing() {
        let spec = AnimSpec::new(1.0)
            .easing(Easing::Linear)
            .repeat(Repeat::Forever);
        let line = Timeline::single(Transform::identity(), Transform::opacity(0.0), &spec);
        let (mid, done) = line.sample(2.5);
        assert!(!done);
        assert!((mid.opacity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn autoreverse_ping_pongs() {
        let spec = AnimSpec::new(1.0)
            .easing(Easing::Linear)
            .repeat(Repeat::Forever)
            .autoreverse(true);
        let line = Timeline::single(Transform::identity(), Transform::scale(2.0), &spec);
        assert_eq!(line.sample(0.5).0.scale, 1.5);
        // Second half of the period runs back to the start.
        assert_eq!(line.sample(1.5).0.scale, 1.5);
        assert!((line.sample(2.0).0.scale - 1.0).abs() < 1e-6);
        assert!(!line.sample(2.0).1);
    }

    #[test]
    fn autoreverse_times_ends_at_start() {
        let spec = AnimSpec::new(1.0)
            .easing(Easing::Linear)
            .repeat(Repeat::Times(2))
            .autoreverse(true);
        let line = Timeline::single(Transform::identity(), Transform::scale(2.0), &spec);
        let (end, done) = line.sample(10.0);
        assert!(done);
        assert!((end.scale - 1.0).abs() < 1e-6);
    }

    #[test]
    fn phases_walk_each_state() {
        let line = Timeline::phases(
            &[
                Phase::new(Transform::offset(20.0, 0.0), 1.0),
                Phase::new(Transform::offset(-20.0, 0.0), 1.0),
            ],
            Easing::Linear,
        );
        assert_eq!(line.sample(0.0).0.offset, (0.0, 0.0));
        assert_eq!(line.sample(1.0).0.offset, (20.0, 0.0));
        assert_eq!(line.sample(1.5).0.offset, (0.0, 0.0));
        let (end, done) = line.sample(2.0);
        assert_eq!(end.offset, (-20.0, 0.0));
        assert!(done);
    }

    #[test]
    fn keyframes_hit_every_key() {
        let line = Timeline::keyframes(
            &[
                Keyframe::at(0.0, Transform::identity()),
                Keyframe::at(0.5, Transform::offset(0.0, -40.0)),
                Keyframe::at(1.0, Transform::identity()),
            ],
            2.0,
        );
        assert_eq!(line.sample(0.0).0.offset, (0.0, 0.0));
        assert_eq!(line.sample(1.0).0.offset, (0.0, -40.0));
        let (end, done) = line.sample(2.0);
        assert_eq!(end.offset, (0.0, 0.0));
        assert!(done);
    }

    #[test]
    fn keyframe_head_gap_eases_out_of_identity() {
        let line = Timeline::keyframes(&[Keyframe::at(1.0, Transform::scale(3.0))], 1.0);
        assert_eq!(line.sample(0.0).0.scale, 1.0);
    }
}
