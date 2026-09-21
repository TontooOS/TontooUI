use super::spring::MAX_SUBSTEP;

/// Friction-based fling: a value with initial velocity that decays
/// exponentially. Used for swipe momentum, scroll glide and anything thrown
/// by the pointer.
#[derive(Clone, Copy, Debug)]
pub struct Decay {
    friction: f32,
}

impl Decay {
    /// `friction` in 1/s. Higher stops faster: 2.0 glides long, 8.0 stops
    /// almost immediately.
    pub fn new(friction: f32) -> Self {
        Self {
            friction: friction.max(0.0),
        }
    }

    pub fn friction(&self) -> f32 {
        self.friction
    }
}

/// Running decay. Stops when velocity drops below the settle threshold.
#[derive(Clone, Copy, Debug)]
pub struct DecayAnim {
    decay: Decay,
    value: f32,
    velocity: f32,
}

/// Velocity in units per second below which the fling counts as stopped.
pub const SETTLE_DECAY_VELOCITY: f32 = 1.0;

impl DecayAnim {
    pub fn new(decay: Decay, start: f32, velocity: f32) -> Self {
        Self {
            decay,
            value: start,
            velocity,
        }
    }

    /// Advance by `dt` seconds (clamped to >= 0).
    pub fn update(&mut self, dt: f32) {
        let mut remaining = dt.max(0.0);
        while remaining > 0.0 {
            let h = remaining.min(MAX_SUBSTEP);
            self.velocity *= (-self.decay.friction * h).exp();
            self.value += self.velocity * h;
            remaining -= h;
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    pub fn settled(&self) -> bool {
        self.velocity.abs() < SETTLE_DECAY_VELOCITY
    }

    pub fn stop(&mut self) {
        self.velocity = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flings_in_throw_direction_then_stops() {
        let mut anim = DecayAnim::new(Decay::new(4.0), 0.0, 1000.0);
        for _ in 0..600 {
            anim.update(1.0 / 120.0);
        }
        assert!(anim.value() > 0.0);
        assert!(anim.settled());
        // Analytical distance: v0 / friction = 250.
        assert!((anim.value() - 250.0).abs() < 5.0, "value={}", anim.value());
    }

    #[test]
    fn higher_friction_travels_less() {
        let mut soft = DecayAnim::new(Decay::new(2.0), 0.0, 1000.0);
        let mut hard = DecayAnim::new(Decay::new(8.0), 0.0, 1000.0);
        for _ in 0..1200 {
            soft.update(1.0 / 120.0);
            hard.update(1.0 / 120.0);
        }
        assert!(soft.value() > hard.value() * 2.0);
    }
}
