# Animation

Physics-based animation engine. Time comes from `FrameClock` (built for a
display refresh rate, deltas clamped), motion from tweens, springs and
decay flings. Physics integrates in fixed substeps, so results stay stable
and smooth from 30 Hz to 240 Hz+.

## Modules

| Module | Path | Description |
|---|---|---|
| `clock` | `src/animation/clock.rs` | Hz-aware frame clock with clamped deltas |
| `easing` | `src/animation/easing.rs` | Easing curves incl. CSS cubic-bezier |
| `animatable` | `src/animation/animatable.rs` | Lerp trait for tweenable values |
| `tween` | `src/animation/tween.rs` | Duration tweens with delay/repeat/reverse |
| `spring` | `src/animation/spring.rs` | Damped spring physics |
| `decay` | `src/animation/decay.rs` | Friction fling physics |

## Clock

```rust
pub fn new(refresh_hz: f32) -> Self
pub fn tick(&mut self, now: Instant) -> f32
pub fn elapsed(&self) -> f64
pub fn dt(&self) -> f32
pub fn refresh_hz(&self) -> f32
pub fn frame_budget(&self) -> f32
pub fn fps(&self) -> f32
```

`tick` returns the clamped delta (`MAX_DT` = 0.1 s), so hitches never
explode the simulation. `fps` is an exponentially smoothed measurement.

## Easing

```rust
pub enum Easing {
    Linear,
    SineIn, SineOut, SineInOut,
    QuadIn, QuadOut, QuadInOut,
    CubicIn, CubicOut, CubicInOut,
    ExpoOut,
    BackOut,
    Bezier(f32, f32, f32, f32),
}
```

```rust
pub fn apply(&self, progress: f32) -> f32
```

Default is `CubicOut`. `Bezier` solves CSS `cubic-bezier(x1, y1, x2, y2)`
with Newton-Raphson plus bisection fallback. Inputs clamp to [0, 1].

## Animatable

```rust
pub trait Animatable: Clone {
    fn lerp(&self, other: &Self, t: f32) -> Self;
}
```

Implemented for `f32`, `f64`, `(f32, f32)`, `bool` (steps at 0.5) and
Peniko `Color` (perceptual blend, shorter hue arc). Note: `Color` also has
an inherent 3-argument `lerp`; plain `color.lerp(&other, t)` calls bind to
that one, generic tween code uses this trait.

## Tween

```rust
pub fn new(from: T, to: T, duration: f32) -> Self
pub fn delay(self, seconds: f32) -> Self
pub fn easing(self, easing: Easing) -> Self
pub fn repeat(self, repeat: Repeat) -> Self
pub fn autoreverse(self, autoreverse: bool) -> Self
pub fn sample(&self, elapsed: f32) -> (T, bool)
```

```rust
pub enum Repeat {
    Never,
    Times(u32),
    Forever,
}
```

`sample` returns the value plus done (never done for `Forever`). With
`autoreverse`, odd cycles run backwards; an even `Times` count ends back at
`from`.

```rust
pub struct TweenAnim<T: Animatable>;
pub fn update(&mut self, elapsed: f32) -> bool
pub fn value(&self) -> &T
pub fn done(&self) -> bool
```

Running tween bound to elapsed seconds.

## Spring

```rust
pub fn new(stiffness: f32, damping: f32, mass: f32) -> Self
pub fn response(response: f32, damping_fraction: f32) -> Self
```

`response` is the approximate settle time in seconds, `damping_fraction`
1.0 for critical damping (no overshoot), lower for bounce.

```rust
pub struct SpringAnim;
pub fn new(spring: Spring, start: f32, target: f32) -> Self
pub fn with_velocity(self, velocity: f32) -> Self
pub fn retarget(&mut self, target: f32)
pub fn update(&mut self, dt: f32)
pub fn value(&self) -> f32
pub fn velocity(&self) -> f32
pub fn settled(&self) -> bool
pub fn snap(&mut self)
```

Damped harmonic oscillator, semi-implicit Euler in 1/240 s substeps.
`retarget` keeps velocity, so interruptions stay smooth. `settled` is true
below 0.05 units and 1.0 units/s; then snap and stop driving frames.

## Decay

```rust
pub fn new(friction: f32) -> Self
```

Friction in 1/s: 2.0 glides long, 8.0 stops fast.

```rust
pub struct DecayAnim;
pub fn new(decay: Decay, start: f32, velocity: f32) -> Self
pub fn update(&mut self, dt: f32)
pub fn value(&self) -> f32
pub fn velocity(&self) -> f32
pub fn settled(&self) -> bool
pub fn stop(&mut self)
```

Exponential velocity decay for flings. Travels `velocity / friction`
units; settled below 1.0 units/s.

## Usage / Example

```rust
use std::time::Instant;
use tontooui::animation::{Easing, FrameClock, Repeat, Spring, SpringAnim, Tween, TweenAnim};

let mut clock = FrameClock::new(120.0);
let mut slide = TweenAnim::new(
    Tween::new(0.0_f32, 300.0, 0.4).easing(Easing::CubicOut),
);
let mut bounce = SpringAnim::new(Spring::response(0.5, 0.6), 0.0, 1.0);

// Per frame:
let dt = clock.tick(Instant::now());
slide.update(clock.elapsed() as f32);
bounce.update(dt);
let x = *slide.value();
let s = bounce.value();
```

## Tests

21 unit tests cover clock clamping, easing endpoints and known values,
bezier identity, lerp types, tween delay/repeat/reverse, spring
convergence at 30/60/120/240 Hz, overshoot, retargeting, big-dt stability
and decay distance (`cargo test --lib`).

## Element Animations

`src/elements/animation/`: `Animated<V>` wraps any child element
(`BasicText`, `BasicToolbar`, `SFSymbolImage`, shapes, ...) and plays
engine tweens against it. Default presets cover scale, rotate, move
and fade; `Phase` and `Keyframe` tracks build custom multi-step
motion. All motion samples through `Tween` (`AnimSpec` carries engine
delay, easing, repeat and autoreverse), so presets, phases and
keyframes share the exact timing model above.

## Transform

```rust
pub struct Transform { pub offset: (f32, f32), pub scale: f32, pub opacity: f32, pub rotation: f32 }
pub fn identity() -> Self
pub fn offset(x: f32, y: f32) -> Self
pub fn scale(factor: f32) -> Self
pub fn opacity(alpha: f32) -> Self
pub fn rotation(degrees: f32) -> Self
pub fn combine(&self, other: &Transform) -> Transform
```

- Sampled frame state. Combined timelines add offsets and rotations,
  multiply scales and opacities (clamped to 0..1).
- Implements the engine `Animatable`, so `Tween<Transform>` lerps
  every channel with the spec easing.

## Anchor

```rust
pub enum Anchor { TopLeading, Top, TopTrailing, Leading, Center, Trailing, BottomLeading, Bottom, BottomTrailing }
pub fn point(&self, x: f32, y: f32, w: f32, h: f32) -> (f32, f32)
```

Fixed point of scale and rotation inside the placed frame; default
is `Center`.

## AnimSpec

```rust
pub fn new(duration: f32) -> Self
pub fn seconds(duration: f32) -> Self
pub fn delay(self, seconds: f32) -> Self
pub fn easing(self, easing: Easing) -> Self
pub fn repeat(self, repeat: Repeat) -> Self
pub fn autoreverse(self, autoreverse: bool) -> Self
pub fn tween(&self, from: Transform, to: Transform) -> Tween<Transform>
```

- Timing of one motion. Default is 0.35 s with `CubicInOut`; delays
  clamp to >= 0.
- `tween` builds the engine tween the timeline samples.

## Timeline

```rust
pub fn single(from: Transform, to: Transform, spec: &AnimSpec) -> Self
pub fn phases(phases: &[Phase], easing: Easing) -> Self
pub fn keyframes(keys: &[Keyframe], total: f32) -> Self
pub fn repeat(self, repeat: Repeat) -> Self
pub fn set_repeat(&mut self, repeat: Repeat)
pub fn autoreverse(self, autoreverse: bool) -> Self
pub fn set_autoreverse(&mut self, autoreverse: bool)
pub fn sample(&self, elapsed: f32) -> (Transform, bool)
```

- Back-to-back engine tweens with a shared repeat. `sample` returns
  the transform plus done (never done for `Forever`).
- Repeating timelines wrap around the total; with autoreverse every
  pass ping-pongs back (a `Times(2)` autoreversed run ends at the
  start). Delays live in the first segment and re-apply each cycle.
- `phases` walks identity then each phase, every transition eased
  over that phase's seconds. `keyframes` spans consecutive keys with
  the arriving key's easing; a head gap eases out of identity and a
  tail gap holds the last key. Keys sort by `at`; zero spans hold
  without consuming time.

## Phase / Keyframe

```rust
pub struct Phase { pub transform: Transform, pub seconds: f32 }
pub fn new(transform: Transform, seconds: f32) -> Self
pub struct Keyframe { pub at: f32, pub transform: Transform, pub easing: Easing }
pub fn new(at: f32, transform: Transform, easing: Easing) -> Self
pub fn at(at: f32, transform: Transform) -> Self
```

- `Phase` holds one step of a phase animation (`seconds` clamped to
  >= 0). `Keyframe` pins a transform at normalized time `at`
  (clamped to 0..1); `at` uses the default UI easing.

## Animated

```rust
pub fn new(child: V) -> Self
pub fn anchor(self, anchor: Anchor) -> Self
pub fn fade(self, from: f32, to: f32, spec: AnimSpec) -> Self
pub fn fade_in(self, spec: AnimSpec) -> Self
pub fn fade_out(self, spec: AnimSpec) -> Self
pub fn move_by(self, dx: f32, dy: f32, spec: AnimSpec) -> Self
pub fn move_from(self, dx: f32, dy: f32, spec: AnimSpec) -> Self
pub fn scale(self, from: f32, to: f32, spec: AnimSpec) -> Self
pub fn pop(self, spec: AnimSpec) -> Self
pub fn phases(self, phases: Vec<Phase>, easing: Easing) -> Self
pub fn keyframes(self, keys: Vec<Keyframe>, total: f32) -> Self
pub fn rotation(self, from_deg: f32, to_deg: f32, spec: AnimSpec) -> Self
pub fn spin_phases(self, phases: Vec<Phase>, easing: Easing) -> Self
pub fn spin_keyframes(self, keys: Vec<Keyframe>, total: f32) -> Self
pub fn repeat(self, repeat: Repeat) -> Self
pub fn restart(&mut self)
pub fn child_mut(&mut self) -> &mut V
pub fn is_done(&self) -> bool
pub fn sample(&mut self, elapsed: f32) -> Transform
pub fn rect(&self) -> (f32, f32, f32, f32)
```

- Time runs from the first draw (`Instant`, like gauges and
  sliders); the shell redraws continuously, so wrappers just play.
  `measure` stays untransformed so stacks never jitter mid-motion.
- Channel application: offset shifts the placed origin (exact for
  every child); opacity wraps the draw in an alpha layer clipped to
  the expanded frame (skipped above 0.999, so idle wrappers cost
  nothing); scale sizes the placed rect around the anchor (exact for
  rect-filling children, box-only for baked glyphs); rotation turns
  `Spin` children around the anchor.
- `rotation`, `spin_phases` and `spin_keyframes` only exist for
  `Spin` children (raster images: `SFSymbolImage`, `AppImage`,
  `UrlImage`, `ImageOverlay`), forwarding sampled degrees through
  `Spin::set_spin`. Other children sample rotation but cannot apply
  it.
- `repeat` loops every motion; `restart` replays from the start.

## Spin

```rust
pub trait Spin: View {
    fn set_spin(&mut self, degrees: f32);
}
```

Opt-in rotation for raster children, applied to their `draw_image`
transform around the frame center (zero degrees cost nothing; only
the photo of an overlay card spins, gradient, caption and badge stay
put).

## Usage / Example

```rust
use tontooui::animation::{Easing, Repeat};
use tontooui::elements::{Animated, AnimSpec, BasicText, Keyframe, Phase, SFSymbolImage, Transform};

let slide = AnimSpec::new(0.9).easing(Easing::CubicInOut).autoreverse(true);
let text = Animated::new(BasicText::new("Hello Tontoo"))
  .fade_in(slide)
  .move_from(0.0, 28.0, slide)
  .repeat(Repeat::Forever);

let spin = Animated::new(SFSymbolImage::new("star.fill").size(64.0))
  .rotation(0.0, 360.0, AnimSpec::new(2.5).easing(Easing::Linear))
  .repeat(Repeat::Forever);

let bounce = Animated::new(Circle::new(90.0))
  .keyframes(vec![
    Keyframe::at(0.0, Transform::identity()),
    Keyframe::new(0.45, Transform::offset(0.0, -48.0), Easing::CubicOut),
    Keyframe::new(0.7, Transform::offset(0.0, 8.0), Easing::CubicIn),
    Keyframe::at(1.0, Transform::identity()),
  ], 1.4)
  .repeat(Repeat::Forever);
```

See `examples/animation.rs` for the running demo (text slide, symbol
spin, capsule pop, circle bounce, toolbar patrol).

## Cross References

- [Renderer.md](Renderer.md) – window loop and frame pipeline the engine drives
- [Layout.md](Layout.md) – views whose properties get animated
