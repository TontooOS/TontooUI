use std::time::Instant;

/// Frame clock. Built for a display refresh rate (`refresh_hz`), so physics
/// and tweens advance in real time on any Hz. Delta times are clamped, so a
/// hitch (window move, breakpoint, tab switch) never explodes the
/// simulation; the animation just resumes smoothly.
pub struct FrameClock {
    refresh_hz: f32,
    start: Option<Instant>,
    last: Option<Instant>,
    elapsed: f64,
    dt: f32,
    fps: f32,
}

/// Largest accepted delta in seconds. Bigger gaps clamp to this.
pub const MAX_DT: f32 = 0.1;

impl FrameClock {
    pub fn new(refresh_hz: f32) -> Self {
        Self {
            refresh_hz: refresh_hz.max(1.0),
            start: None,
            last: None,
            elapsed: 0.0,
            dt: 0.0,
            fps: refresh_hz.max(1.0),
        }
    }

    /// Advance the clock to `now`. Returns the clamped delta in seconds.
    pub fn tick(&mut self, now: Instant) -> f32 {
        match self.last {
            None => {
                self.start = Some(now);
                self.last = Some(now);
                self.dt = 0.0;
            }
            Some(last) => {
                let raw = now.saturating_duration_since(last).as_secs_f32();
                self.dt = raw.min(MAX_DT);
                self.elapsed += self.dt as f64;
                self.last = Some(now);
                if self.dt > 0.0 {
                    let instant_fps = 1.0 / self.dt;
                    self.fps += (instant_fps - self.fps) * 0.05;
                }
            }
        }
        self.dt
    }

    /// Seconds since the first tick.
    pub fn elapsed(&self) -> f64 {
        self.elapsed
    }

    /// Last clamped delta in seconds.
    pub fn dt(&self) -> f32 {
        self.dt
    }

    /// Display refresh rate this clock was built for.
    pub fn refresh_hz(&self) -> f32 {
        self.refresh_hz
    }

    /// Target frame time (1 / refresh rate) in seconds.
    pub fn frame_budget(&self) -> f32 {
        1.0 / self.refresh_hz
    }

    /// Smoothed measured frames per second.
    pub fn fps(&self) -> f32 {
        self.fps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn first_tick_has_zero_dt() {
        let mut clock = FrameClock::new(60.0);
        let now = Instant::now();
        assert_eq!(clock.tick(now), 0.0);
        assert_eq!(clock.elapsed(), 0.0);
    }

    #[test]
    fn dt_clamps_huge_gaps() {
        let mut clock = FrameClock::new(120.0);
        let start = Instant::now();
        clock.tick(start);
        let dt = clock.tick(start + Duration::from_secs(5));
        assert_eq!(dt, MAX_DT);
        assert!((clock.elapsed() - MAX_DT as f64).abs() < 1e-6);
    }

    #[test]
    fn frame_budget_matches_hz() {
        assert!((FrameClock::new(60.0).frame_budget() - 1.0 / 60.0).abs() < 1e-6);
        assert!((FrameClock::new(144.0).frame_budget() - 1.0 / 144.0).abs() < 1e-6);
    }
}
