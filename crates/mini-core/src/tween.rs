//! Animation primitives: a standalone [`Tween`] you advance yourself, and a
//! [`Tweens`] manager for fire-and-forget world mutations.
//!
//! The manager's callbacks receive `(&mut World, eased_t)` — capturing the
//! entity and endpoints is all a caller needs, and the borrow checker is
//! satisfied by construction.

use crate::ecs::World;

/// Signature of easing functions — see [`crate::math::ease`].
pub type EaseFn = fn(f32) -> f32;

/// A progress tracker with easing. Advance it, read the eased fraction.
#[derive(Debug, Clone, Copy)]
pub struct Tween {
    duration: f32,
    elapsed: f32,
    ease: EaseFn,
}

impl Tween {
    /// A tween lasting `duration` seconds with easing function `ease`.
    pub fn new(duration: f32, ease: EaseFn) -> Self {
        Self { duration: duration.max(f32::EPSILON), elapsed: 0.0, ease }
    }

    /// Advance by `dt` and return the eased fraction. Overshooting eases
    /// (e.g. `ease_out_back`) may exceed 1.0. Sticks at the end.
    pub fn advance(&mut self, dt: f32) -> f32 {
        self.elapsed = (self.elapsed + dt.max(0.0)).min(self.duration);
        self.fraction()
    }

    /// The eased fraction at the current position (without advancing).
    pub fn fraction(&self) -> f32 {
        (self.ease)(self.elapsed / self.duration)
    }

    pub fn finished(&self) -> bool {
        self.elapsed >= self.duration
    }

    pub fn restart(&mut self) {
        self.elapsed = 0.0;
    }
}

/// Id returned by [`Tweens::add`]; use it to [`Tweens::cancel`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TweenId(u64);

type Tick = Box<dyn FnMut(&mut World, f32)>;

struct Active {
    id: TweenId,
    tween: Tween,
    tick: Tick,
}

/// Fire-and-forget tweens that mutate the [`World`]:
///
/// ```ignore
/// let (paddle, start_y) = (self.ai, self.world.get::<Transform>(self.ai).unwrap().pos.y);
/// self.tweens.add(0.9, ease::ease_in_out_cubic, move |w, t| {
///     if let Some(tr) = w.get_mut::<Transform>(paddle) {
///         tr.pos.y = start_y.lerp(H * 0.5, t);
///     }
/// });
/// // once per update:
/// self.tweens.update(dt, &mut self.world);
/// ```
#[derive(Default)]
pub struct Tweens {
    active: Vec<Active>,
    next_id: u64,
}

impl Tweens {
    /// An empty manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a tween lasting `duration` seconds; `tick` runs every update
    /// with the eased fraction, and once more at the end.
    pub fn add(
        &mut self,
        duration: f32,
        ease: EaseFn,
        tick: impl FnMut(&mut World, f32) + 'static,
    ) -> TweenId {
        let id = TweenId(self.next_id);
        self.next_id += 1;
        self.active.push(Active { id, tween: Tween::new(duration, ease), tick });
        id
    }

    /// Advance all tweens, ticking them against `world`. Finished tweens get
    /// their final tick and are removed.
    pub fn update(&mut self, dt: f32, world: &mut World) {
        let mut i = 0;
        while i < self.active.len() {
            let t = self.active[i].tween.advance(dt);
            if self.active[i].tween.finished() {
                let mut done = self.active.swap_remove(i);
                (done.tick)(world, t);
            } else {
                let running = &mut self.active[i];
                (running.tick)(world, t);
                i += 1;
            }
        }
    }

    /// Cancel a tween (its final tick is not run).
    pub fn cancel(&mut self, id: TweenId) {
        self.active.retain(|a| a.id != id);
    }

    pub fn is_active(&self, id: TweenId) -> bool {
        self.active.iter().any(|a| a.id == id)
    }

    pub fn len(&self) -> usize {
        self.active.len()
    }

    pub fn is_empty(&self) -> bool {
        self.active.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::Transform;
    use crate::math::ease;
    use crate::math::Vec2;

    #[test]
    fn tweens_drive_the_world_and_finish() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Transform::new(Vec2::ZERO));

        let mut tweens = Tweens::new();
        let id = tweens.add(1.0, ease::linear, move |w, t| {
            if let Some(tr) = w.get_mut::<Transform>(e) {
                tr.pos = Vec2::new(10.0 * t, 0.0);
            }
        });

        for _ in 0..10 {
            tweens.update(0.1, &mut world);
        }
        assert_eq!(world.get::<Transform>(e).unwrap().pos.x, 10.0);
        assert!(!tweens.is_active(id));

        tweens.update(1.0, &mut world); // no-op after completion
        assert_eq!(world.get::<Transform>(e).unwrap().pos.x, 10.0);
    }

    #[test]
    fn cancel_removes_without_final_tick() {
        let mut world = World::new();
        let mut tweens = Tweens::new();
        let id = tweens.add(1.0, ease::linear, |w, _t| {
            w.spawn(); // would be visible if the final tick ran
        });
        tweens.update(0.5, &mut world);
        let before = world.entity_count();
        tweens.cancel(id);
        tweens.update(1.0, &mut world);
        assert_eq!(world.entity_count(), before);
        assert!(tweens.is_empty());
    }

    #[test]
    fn tween_primitive_eases_and_sticks() {
        let mut tw = Tween::new(0.5, ease::ease_in_quad);
        assert!((tw.advance(0.25) - 0.25).abs() < 1e-6); // (0.5)^2
        assert!(!tw.finished());
        assert_eq!(tw.advance(10.0), 1.0);
        assert!(tw.finished());
        tw.restart();
        assert_eq!(tw.fraction(), 0.0);
    }
}
