use std::any::Any;
use std::time::Instant;

use vello::Scene;
use vello::kurbo::{Affine, Rect};
use vello::peniko::{BlendMode, Fill};

use super::super::layout::View;
use super::{
    ANIM_CLIP_MARGIN, Anchor, AnimSpec, Keyframe, Phase, Timeline, Transform,
};
use crate::animation::Repeat;
use crate::renderer::images::ImageLoader;
use crate::renderer::text::FontSystem;

/// Opt-in rotation for raster children. The wrapper samples rotation
/// degrees from the engine and hands them here; glyph and layout
/// children cannot rotate their baked output, so only the image
/// elements (which paint through a `draw_image` transform) implement
/// it. Anything else keeps rotation at rest.
pub trait Spin: View {
    fn set_spin(&mut self, degrees: f32);
}

/// Animated wrapper around any child element (`BasicText`,
/// `BasicToolbar`, `SFSymbolImage`, shapes, ...). Presets write one
/// transform channel each through engine tweens; phases and keyframes
/// blend all four. Motions combine: offsets and rotations add, scales
/// and opacities multiply.
///
/// Channel application (documented, no surprises):
/// - offset shifts the child's placed origin: exact for every child.
/// - opacity wraps the child draw in an alpha layer: exact for every
///   child (skipped above 0.999, so idle wrappers cost nothing).
/// - scale sizes the placed rect around the anchor: exact for
///   rect-filling children (shapes, images); glyph children keep
///   their baked size and only their box scales.
/// - rotation turns `Spin` children around the anchor; other children
///   sample it but cannot apply it.
///
/// Time runs from the first draw (`Instant`, like gauges and
/// sliders); the shell redraws continuously, so wrappers just work.
/// `restart` replays, `repeat` loops every motion.
pub struct Animated<V> {
    child: V,
    motions: Vec<Timeline>,
    anchor: Anchor,
    spin: Option<fn(&mut V, f32)>,
    t0: Option<Instant>,
    done: bool,
    x: f32,
    y: f32,
    placed_w: f32,
    placed_h: f32,
}

impl<V: View> Animated<V> {
    pub fn new(child: V) -> Self {
        Self {
            child,
            motions: Vec::new(),
            anchor: Anchor::Center,
            spin: None,
            t0: None,
            done: true,
            x: 0.0,
            y: 0.0,
            placed_w: 0.0,
            placed_h: 0.0,
        }
    }

    /// Fixed point for scale and rotation.
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Fade preset: opacity from `from` to `to` under `spec`.
    pub fn fade(mut self, from: f32, to: f32, spec: AnimSpec) -> Self {
        self.motions.push(Timeline::single(
            Transform::opacity(from),
            Transform::opacity(to),
            &spec,
        ));
        self.done = false;
        self
    }

    /// Fade preset from transparent to opaque.
    pub fn fade_in(self, spec: AnimSpec) -> Self {
        self.fade(0.0, 1.0, spec)
    }

    /// Fade preset from opaque to transparent.
    pub fn fade_out(self, spec: AnimSpec) -> Self {
        self.fade(1.0, 0.0, spec)
    }

    /// Move preset: offset from (0, 0) to (`dx`, `dy`) under `spec`.
    pub fn move_by(mut self, dx: f32, dy: f32, spec: AnimSpec) -> Self {
        self.motions.push(Timeline::single(
            Transform::identity(),
            Transform::offset(dx, dy),
            &spec,
        ));
        self.done = false;
        self
    }

    /// Move preset: offset from (`dx`, `dy`) back to rest under `spec`
    /// (slide-in from an offset).
    pub fn move_from(mut self, dx: f32, dy: f32, spec: AnimSpec) -> Self {
        self.motions.push(Timeline::single(
            Transform::offset(dx, dy),
            Transform::identity(),
            &spec,
        ));
        self.done = false;
        self
    }

    /// Scale preset: uniform factor from `from` to `to` around the
    /// anchor under `spec`.
    pub fn scale(mut self, from: f32, to: f32, spec: AnimSpec) -> Self {
        self.motions.push(Timeline::single(
            Transform::scale(from),
            Transform::scale(to),
            &spec,
        ));
        self.done = false;
        self
    }

    /// Scale preset from zero to full size (pop-in).
    pub fn pop(self, spec: AnimSpec) -> Self {
        self.scale(0.0, 1.0, spec)
    }

    /// Phase animation: identity, then each phase in order, every
    /// transition eased with `easing` over that phase's seconds.
    /// Loop with `repeat`.
    pub fn phases(mut self, phases: Vec<Phase>, easing: crate::animation::Easing) -> Self {
        self.motions.push(Timeline::phases(&phases, easing));
        self.done = false;
        self
    }

    /// Keyframe animation: custom spans over `total` seconds, each key
    /// reached with its own easing.
    pub fn keyframes(mut self, keys: Vec<Keyframe>, total: f32) -> Self {
        self.motions.push(Timeline::keyframes(&keys, total));
        self.done = false;
        self
    }

    /// Loop every motion (presets, phases, keyframes). Looping
    /// ping-pongs when the motion's spec set `autoreverse`, otherwise
    /// it jumps back to the start.
    pub fn repeat(mut self, repeat: Repeat) -> Self {
        for motion in &mut self.motions {
            motion.set_repeat(repeat);
        }
        self.done = false;
        self
    }

    /// Replay from the start on the next draw.
    pub fn restart(&mut self) {
        self.t0 = None;
        self.done = self.motions.is_empty();
    }

    /// Wrapped child for state updates (typing, toggles, ...).
    pub fn child_mut(&mut self) -> &mut V {
        &mut self.child
    }

    /// True when every motion finished (never for `Forever`).
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// Combined transform at `elapsed` seconds (pure sampling, used
    /// by draw and tests).
    pub fn sample(&mut self, elapsed: f32) -> Transform {
        let mut out = Transform::identity();
        let mut done = true;
        for motion in &self.motions {
            let (t, d) = motion.sample(elapsed);
            out = out.combine(&t);
            done &= d;
        }
        self.done = done;
        out
    }

    /// Placed rect (x, y, width, height) in logical px.
    pub fn rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.placed_w, self.placed_h)
    }
}

impl<V: View + Spin> Animated<V> {
    /// Rotate preset: `from` to `to` degrees around the anchor under
    /// `spec`. Only available for `Spin` children (raster images);
    /// the sampled degrees reach the child through `Spin::set_spin`.
    pub fn rotation(mut self, from_deg: f32, to_deg: f32, spec: AnimSpec) -> Self {
        self.motions.push(Timeline::single(
            Transform::rotation(from_deg),
            Transform::rotation(to_deg),
            &spec,
        ));
        self.spin = Some(Spin::set_spin);
        self.done = false;
        self
    }

    /// Phase animation with rotation support for `Spin` children.
    pub fn spin_phases(
        mut self,
        phases: Vec<Phase>,
        easing: crate::animation::Easing,
    ) -> Self {
        self.motions.push(Timeline::phases(&phases, easing));
        self.spin = Some(Spin::set_spin);
        self.done = false;
        self
    }

    /// Keyframe animation with rotation support for `Spin` children.
    pub fn spin_keyframes(mut self, keys: Vec<Keyframe>, total: f32) -> Self {
        self.motions.push(Timeline::keyframes(&keys, total));
        self.spin = Some(Spin::set_spin);
        self.done = false;
        self
    }
}

impl<V: View + 'static> View for Animated<V> {
    fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
        // Intrinsic size stays untransformed so stacks never jitter
        // while a motion runs.
        self.child.measure(fonts)
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
        images: &mut ImageLoader<'_>,
    ) {
        if self.placed_w <= 0.0 || self.placed_h <= 0.0 {
            return;
        }
        let now = Instant::now();
        let t0 = *self.t0.get_or_insert(now);
        let elapsed = now.saturating_duration_since(t0).as_secs_f32();
        let t = self.sample(elapsed);
        if let Some(apply) = self.spin.as_mut() {
            apply(&mut self.child, t.rotation);
        }
        // Scale the placed rect around the anchor, then shift.
        let s = t.scale.max(0.0);
        let (ax, ay) = self.anchor.point(self.x, self.y, self.placed_w, self.placed_h);
        let (w, h) = (self.placed_w * s, self.placed_h * s);
        let (ox, oy) = (
            ax + (self.x - ax) * s + t.offset.0,
            ay + (self.y - ay) * s + t.offset.1,
        );
        self.child.place(fonts, ox, oy, w, h);
        if t.opacity < 0.999 {
            let scale = fonts.scale as f64;
            let m = ANIM_CLIP_MARGIN as f64 * scale;
            let clip = Rect::new(
                ox as f64 * scale - m,
                oy as f64 * scale - m,
                (ox + w) as f64 * scale + m,
                (oy + h) as f64 * scale + m,
            );
            scene.push_layer(
                Fill::NonZero,
                BlendMode::default(),
                t.opacity,
                Affine::IDENTITY,
                &clip,
            );
            self.child.draw(scene, fonts, images);
            scene.pop_layer();
        } else {
            self.child.draw(scene, fonts, images);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::Easing;

    struct Stub {
        w: f32,
        h: f32,
        spin_deg: f32,
    }

    impl View for Stub {
        fn measure(&mut self, _fonts: &mut FontSystem) -> (f32, f32) {
            (self.w, self.h)
        }

        fn place(
            &mut self,
            _fonts: &mut FontSystem,
            _x: f32,
            _y: f32,
            _w: f32,
            _h: f32,
        ) {
        }

        fn draw(
            &mut self,
            _scene: &mut Scene,
            _fonts: &mut FontSystem,
            _images: &mut ImageLoader<'_>,
        ) {
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    impl Spin for Stub {
        fn set_spin(&mut self, degrees: f32) {
            self.spin_deg = degrees;
        }
    }

    fn stub() -> Stub {
        Stub {
            w: 100.0,
            h: 40.0,
            spin_deg: 0.0,
        }
    }

    #[test]
    fn presets_combine() {
        let mut anim = Animated::new(stub())
            .move_by(10.0, 0.0, AnimSpec::new(1.0).easing(Easing::Linear))
            .fade(1.0, 0.0, AnimSpec::new(1.0).easing(Easing::Linear));
        let mid = anim.sample(0.5);
        assert_eq!(mid.offset, (5.0, 0.0));
        assert!((mid.opacity - 0.5).abs() < 1e-6);
        assert!(!anim.is_done());
        let end = anim.sample(1.0);
        assert_eq!(end.offset, (10.0, 0.0));
        assert_eq!(end.opacity, 0.0);
        assert!(anim.is_done());
    }

    #[test]
    fn rotation_reaches_spin_child() {
        let mut anim = Animated::new(stub()).rotation(
            0.0,
            90.0,
            AnimSpec::new(1.0).easing(Easing::Linear),
        );
        assert_eq!(anim.sample(0.5).rotation, 45.0);
        // The fn stored by `rotation` forwards degrees.
        let mut child = stub();
        let apply = anim.spin.expect("spin fn");
        apply(&mut child, 45.0);
        assert_eq!(child.spin_deg, 45.0);
    }

    #[test]
    fn repeat_loops_phases() {
        let mut anim = Animated::new(stub())
            .phases(
                vec![Phase::new(Transform::scale(2.0), 1.0)],
                Easing::Linear,
            )
            .repeat(Repeat::Forever);
        assert_eq!(anim.sample(0.5).scale, 1.5);
        assert_eq!(anim.sample(1.5).scale, 1.5);
        assert!(!anim.is_done());
    }

    #[test]
    fn restart_clears_done() {
        let mut anim =
            Animated::new(stub()).fade_out(AnimSpec::new(1.0).easing(Easing::Linear));
        anim.sample(5.0);
        assert!(anim.is_done());
        anim.restart();
        assert!(!anim.is_done());
    }

    #[test]
    fn empty_wrapper_is_identity_done() {
        let mut anim = Animated::new(stub());
        assert_eq!(anim.sample(3.0), Transform::identity());
        assert!(anim.is_done());
    }
}
