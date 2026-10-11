//! Easing functions. All take `t` (clamped to `0.0..=1.0`) and return the
//! eased progress; use them for tweens, UI slides, hit flashes, ...
//! (`ease_out_back` deliberately overshoots `1.0` — that's the point.)

/// Constant speed.
pub fn linear(t: f32) -> f32 {
    t.clamp(0.0, 1.0)
}

/// Smooth acceleration + deceleration (the workhorse).
pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn ease_in_quad(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t
}

pub fn ease_out_quad(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t)
}

pub fn ease_in_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Overshoots slightly, then settles — springy UI.
pub fn ease_out_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_hold() {
        for f in [linear, smoothstep, ease_in_quad, ease_out_quad, ease_in_out_cubic] {
            assert_eq!(f(0.0), 0.0);
            assert_eq!(f(1.0), 1.0);
        }
    }

    #[test]
    fn out_back_overshoots() {
        assert!(ease_out_back(0.7) > 1.0);
        assert_eq!(ease_out_back(0.0), 0.0);
        assert_eq!(ease_out_back(1.0), 1.0);
    }
}
