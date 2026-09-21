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

## Cross References

- [Renderer.md](Renderer.md) – window loop and frame pipeline the engine drives
- [Layout.md](Layout.md) – views whose properties get animated
