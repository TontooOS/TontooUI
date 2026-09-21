use super::animatable::Animatable;
use super::easing::Easing;

/// Repeat behavior of a tween.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Repeat {
    #[default]
    Never,
    Times(u32),
    Forever,
}

/// Time-based interpolation between two values.
#[derive(Clone, Debug)]
pub struct Tween<T: Animatable> {
    from: T,
    to: T,
    duration: f32,
    delay: f32,
    easing: Easing,
    repeat: Repeat,
    autoreverse: bool,
}

impl<T: Animatable> Tween<T> {
    pub fn new(from: T, to: T, duration: f32) -> Self {
        Self {
            from,
            to,
            duration: duration.max(0.0),
            delay: 0.0,
            easing: Easing::default(),
            repeat: Repeat::Never,
            autoreverse: false,
        }
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

    /// Sample at `elapsed` seconds since start. Returns the value plus
    /// whether the animation finished (never true for `Forever`).
    pub fn sample(&self, elapsed: f32) -> (T, bool) {
        let t = elapsed - self.delay;
        if t <= 0.0 {
            return (self.from.clone(), false);
        }
        if self.duration <= 0.0 {
            return (self.to.clone(), true);
        }
        let total = match self.repeat {
            Repeat::Never => 1,
            Repeat::Times(n) => n.max(1),
            Repeat::Forever => u32::MAX,
        };
        let cycle = (t / self.duration) as u32;
        if cycle >= total {
            let end_at_start = self.autoreverse && total % 2 == 0;
            let value = if end_at_start {
                self.from.clone()
            } else {
                self.to.clone()
            };
            return (value, true);
        }
        let mut p = (t % self.duration) / self.duration;
        if self.autoreverse && cycle % 2 == 1 {
            p = 1.0 - p;
        }
        (self.from.lerp(&self.to, self.easing.apply(p)), false)
    }
}

/// Running tween bound to a clock. Feed it elapsed seconds, read the value.
#[derive(Clone, Debug)]
pub struct TweenAnim<T: Animatable> {
    tween: Tween<T>,
    value: T,
    done: bool,
}

impl<T: Animatable> TweenAnim<T> {
    pub fn new(tween: Tween<T>) -> Self {
        let value = tween.from.clone();
        Self {
            tween,
            value,
            done: false,
        }
    }

    /// Advance to `elapsed` seconds since start. Returns true when finished.
    pub fn update(&mut self, elapsed: f32) -> bool {
        let (value, done) = self.tween.sample(elapsed);
        self.value = value;
        self.done = done;
        done
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn done(&self) -> bool {
        self.done
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delay_holds_start_value() {
        let tween = Tween::new(0.0_f32, 10.0, 1.0).delay(0.5);
        assert_eq!(tween.sample(0.25).0, 0.0);
        assert!(!tween.sample(0.25).1);
    }

    #[test]
    fn linear_midpoint() {
        let tween = Tween::new(0.0_f32, 10.0, 2.0).easing(Easing::Linear);
        let (value, done) = tween.sample(1.0);
        assert_eq!(value, 5.0);
        assert!(!done);
        let (end, done) = tween.sample(2.5);
        assert_eq!(end, 10.0);
        assert!(done);
    }

    #[test]
    fn autoreverse_returns_to_start() {
        let tween = Tween::new(0.0_f32, 10.0, 1.0)
            .easing(Easing::Linear)
            .repeat(Repeat::Times(2))
            .autoreverse(true);
        assert_eq!(tween.sample(0.5).0, 5.0);
        assert_eq!(tween.sample(1.5).0, 5.0);
        let (end, done) = tween.sample(2.0);
        assert_eq!(end, 0.0);
        assert!(done);
    }

    #[test]
    fn driver_tracks_value_and_done() {
        let mut anim = TweenAnim::new(Tween::new(0.0_f32, 4.0, 1.0).easing(Easing::Linear));
        assert!(!anim.update(0.5));
        assert_eq!(*anim.value(), 2.0);
        assert!(anim.update(1.0));
        assert!(anim.done());
    }
}
