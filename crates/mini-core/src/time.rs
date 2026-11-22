/// Read-only view of timing, handed to the game every callback.
#[derive(Debug, Clone, Copy)]
pub struct Time {
    delta: f32,       // simulation step length (constant with fixed timestep)
    frame_delta: f32, // real measured frame time
    elapsed: f32,
    frame: u64,
    fps: f32,
    alpha: f32, // leftover-accumulator fraction for render interpolation
}

impl Time {
    pub(crate) fn new() -> Self {
        Self { delta: 0.0, frame_delta: 0.0, elapsed: 0.0, frame: 0, fps: 0.0, alpha: 0.0 }
    }

    pub(crate) fn begin_frame(&mut self, frame_delta: f32) {
        self.frame_delta = frame_delta;
        self.elapsed += frame_delta;
        self.frame += 1;
        if frame_delta > 0.0 {
            let instant = 1.0 / frame_delta;
            self.fps = if self.fps <= 0.0 { instant } else { self.fps * 0.9 + instant * 0.1 };
        }
    }

    pub(crate) fn begin_step(&mut self, delta: f32) {
        self.delta = delta;
    }

    pub(crate) fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha;
    }

    /// Seconds since the previous `update` (constant with a fixed step).
    pub fn delta(&self) -> f32 {
        self.delta
    }

    /// Real measured time of the last rendered frame.
    pub fn frame_delta(&self) -> f32 {
        self.frame_delta
    }

    /// Interpolation fraction in `0.0..=1.0`: how far between the previous
    /// and current simulation step this rendered frame sits. Feed it to
    /// [`crate::ecs::Transform::interp`] to de-jitter rendering.
    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    pub fn frame(&self) -> u64 {
        self.frame
    }

    /// Smoothed frames-per-second estimate.
    pub fn fps(&self) -> f32 {
        self.fps
    }
}

/// A simple countdown / stopwatch. Clamps at its duration; restart at will.
///
/// ```
/// use mini_core::time::Timer;
/// let mut t = Timer::new(0.5);
/// t.advance(0.25);
/// assert!(!t.finished());
/// t.advance(0.25);
/// assert!(t.finished() && t.fraction() == 1.0);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Timer {
    duration: f32,
    elapsed: f32,
}

impl Timer {
    /// A timer lasting `duration` seconds (negative durations are clamped to 0).
    pub fn new(duration: f32) -> Self {
        Self { duration: duration.max(0.0), elapsed: 0.0 }
    }

    /// Advance by `dt` seconds (clamped at the duration).
    pub fn advance(&mut self, dt: f32) {
        self.elapsed = (self.elapsed + dt.max(0.0)).min(self.duration);
    }

    pub fn finished(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Progress in `0.0..=1.0`.
    pub fn fraction(&self) -> f32 {
        if self.duration <= 0.0 {
            1.0
        } else {
            self.elapsed / self.duration
        }
    }

    pub fn remaining(&self) -> f32 {
        self.duration - self.elapsed
    }

    pub fn restart(&mut self) {
        self.elapsed = 0.0;
    }

    /// Restart with a new duration.
    pub fn restart_with(&mut self, duration: f32) {
        *self = Timer::new(duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_progresses_and_clamps() {
        let mut t = Timer::new(1.0);
        t.advance(0.4);
        assert!((t.fraction() - 0.4).abs() < 1e-6);
        t.advance(2.0);
        assert!(t.finished());
        assert_eq!(t.fraction(), 1.0);
        t.restart();
        assert!(!t.finished());
        t.restart_with(0.1);
        assert_eq!(t.remaining(), 0.1);
    }
}
