/// Spring physics: damped harmonic oscillator integrated with
/// semi-implicit Euler in fixed substeps, so motion stays stable and smooth
/// on any refresh rate (30 Hz to 240 Hz+).
#[derive(Clone, Copy, Debug)]
pub struct Spring {
    stiffness: f32,
    damping: f32,
    mass: f32,
}

/// Max physics substep in seconds. Larger frame deltas split into steps of
/// at most this size.
pub const MAX_SUBSTEP: f32 = 1.0 / 240.0;

impl Spring {
    /// Raw spring with stiffness `k`, damping `c` and `mass`.
    pub fn new(stiffness: f32, damping: f32, mass: f32) -> Self {
        Self {
            stiffness: stiffness.max(0.0),
            damping: damping.max(0.0),
            mass: mass.max(f32::EPSILON),
        }
    }

    /// SwiftUI-style spring: `response` is the approximate settle time in
    /// seconds, `damping_fraction` is 1.0 for critical damping (no
    /// overshoot), lower for bouncy motion.
    pub fn response(response: f32, damping_fraction: f32) -> Self {
        let response = response.max(0.001);
        let omega = 2.0 * std::f32::consts::PI / response;
        Self {
            stiffness: omega * omega,
            damping: 2.0 * damping_fraction * omega,
            mass: 1.0,
        }
    }

    pub fn stiffness(&self) -> f32 {
        self.stiffness
    }

    pub fn damping(&self) -> f32 {
        self.damping
    }

    pub fn mass(&self) -> f32 {
        self.mass
    }
}

/// Running spring towards a target. Retarget any time; velocity carries
/// over, so interruptions stay smooth instead of snapping.
#[derive(Clone, Copy, Debug)]
pub struct SpringAnim {
    spring: Spring,
    target: f32,
    value: f32,
    velocity: f32,
}

/// Settle thresholds: position in units, velocity in units per second.
pub const SETTLE_POSITION: f32 = 0.05;
pub const SETTLE_VELOCITY: f32 = 1.0;

impl SpringAnim {
    pub fn new(spring: Spring, start: f32, target: f32) -> Self {
        Self {
            spring,
            target,
            value: start,
            velocity: 0.0,
        }
    }

    pub fn with_velocity(mut self, velocity: f32) -> Self {
        self.velocity = velocity;
        self
    }

    /// Move the target mid-flight. Velocity is kept.
    pub fn retarget(&mut self, target: f32) {
        self.target = target;
    }

    /// Advance by `dt` seconds (clamped to >= 0).
    pub fn update(&mut self, dt: f32) {
        let mut remaining = dt.max(0.0);
        while remaining > 0.0 {
            let h = remaining.min(MAX_SUBSTEP);
            let accel = (-self.spring.stiffness * (self.value - self.target)
                - self.spring.damping * self.velocity)
                / self.spring.mass;
            self.velocity += accel * h;
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

    pub fn target(&self) -> f32 {
        self.target
    }

    /// True when close enough to rest that callers can snap and stop.
    pub fn settled(&self) -> bool {
        (self.value - self.target).abs() < SETTLE_POSITION
            && self.velocity.abs() < SETTLE_VELOCITY
    }

    /// Snap to the target and kill velocity.
    pub fn snap(&mut self) {
        self.value = self.target;
        self.velocity = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simulate(anim: &mut SpringAnim, seconds: f32, hz: f32) {
        let steps = (seconds * hz).round() as u32;
        for _ in 0..steps {
            anim.update(1.0 / hz);
        }
    }

    #[test]
    fn converges_to_target() {
        for hz in [30.0, 60.0, 120.0, 240.0] {
            let mut anim = SpringAnim::new(Spring::response(0.4, 1.0), 0.0, 100.0);
            simulate(&mut anim, 3.0, hz);
            assert!(anim.settled(), "not settled at {hz} Hz");
            assert!((anim.value() - 100.0).abs() < 0.5, "off at {hz} Hz");
        }
    }

    #[test]
    fn underdamped_overshoots() {
        let mut anim = SpringAnim::new(Spring::response(0.4, 0.3), 0.0, 100.0);
        let mut peak = 0.0_f32;
        for _ in 0..240 {
            anim.update(1.0 / 120.0);
            peak = peak.max(anim.value());
        }
        assert!(peak > 100.0, "expected overshoot, peak={peak}");
    }

    #[test]
    fn retarget_keeps_motion_smooth() {
        let mut anim = SpringAnim::new(Spring::response(0.4, 1.0), 0.0, 100.0);
        simulate(&mut anim, 0.2, 120.0);
        let mid = anim.value();
        assert!(mid > 0.0 && mid < 100.0);
        anim.retarget(0.0);
        simulate(&mut anim, 3.0, 120.0);
        assert!(anim.settled());
        assert!(anim.value().abs() < 0.5);
    }

    #[test]
    fn big_dt_stays_stable() {
        let mut anim = SpringAnim::new(Spring::new(500.0, 20.0, 1.0), 0.0, 100.0);
        anim.update(0.1);
        assert!(anim.value().is_finite());
        assert!(anim.velocity().is_finite());
    }
}
