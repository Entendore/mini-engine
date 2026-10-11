use std::time::{SystemTime, UNIX_EPOCH};

/// xorshift64* — tiny, fast, dependency-free, deterministic from a seed.
///
/// Seeding with a fixed value makes runs reproducible — pair that with the
/// fixed timestep and [`ScriptedBackend`](crate::backend::ScriptedBackend)
/// for fully deterministic replays.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// A nonzero seed produces a distinct stream; zero is remapped.
    pub fn new(seed: u64) -> Self {
        // zero is a fixed point for xorshift; never allow it
        Self { state: if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed } }
    }

    /// Seed from the wall clock (good enough for a mini engine).
    pub fn from_entropy() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x1234_5678);
        Self::new(nanos)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform float in `[0, 1)`.
    pub fn next_f32
(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Uniform float in `[min, max)`.
    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_streams_are_identical() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn ranges_stay_in_bounds() {
        let mut rng = Rng::new(7);
        for _ in 0..1_000 {
            let v = rng.range_f32(-3.0, 5.0);
            assert!((-3.0..5.0).contains(&v));
        }
    }
}
