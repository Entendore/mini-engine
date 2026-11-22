//! A minimal entity–component system with **generational handles** and a
//! **component registry**.
//!
//! - [`Entity`] — `{ id, generation }`. IDs are recycled on despawn; the
//!   generation bump invalidates old handles, so stale handles can never
//!   alias a recycled entity (queries return `None`/`false` instead).
//! - [`World`] — entity lifetimes plus storage for *any* component type.
//!   `world.insert(e, MyThing)` just works (blanket [`Component`] impl);
//!   components live in per-type [`Pool`]s keyed by `TypeId`.
//! - [`join2`] / [`join2_mut`] / [`join3`] / [`join3_mut`] — pool-level
//!   joins; [`World`] adds ergonomic sugar (`query`, `each2_mut`, ...).
//!
//! The built-ins below (`Transform`, `Velocity`, ...) are ordinary
//! components kept in the prelude purely for convenience.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::math::Vec2;
use crate::render::Color;

/// Handle to a thing that exists in the [`World`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entity {
    pub(crate) id: u32,
    pub(crate) gen: u32,
}

impl Entity {
    /// Stable numeric id (recycled across despawns — always pair with the
    /// generation when storing handles long-term).
    pub fn id(self) -> u32 {
        self.id
    }

    /// Bumped every time the id is recycled; stale handles carry the old one.
    pub fn generation(self) -> u32 {
        self.gen
    }
}

/// Marker for data that can live in a [`World`]. Blanket-implemented for
/// every `'static` type — define a struct and start inserting.
pub trait Component: Any + 'static {}
impl<T: Any + 'static> Component for T {}

/// Where a thing is (screen space, y down), plus where it was *last* step —
/// the raw material for render interpolation at
/// [`Time::alpha`](crate::time::Time::alpha).
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    /// Position as of the current fixed step.
    pub pos: Vec2,
    /// Position as of the previous fixed step. Keep in sync via
    /// [`Transform::advance`] / [`Transform::snap`].
    pub prev: Vec2,
}

impl Transform {
    /// A transform whose history starts at `pos`.
    pub fn new(pos: Vec2) -> Self {
        Self { pos, prev: pos }
    }

    /// Convenience constructor.
    pub fn at(x: f32, y: f32) -> Self {
        Self::new(Vec2::new(x, y))
    }

    /// Move to `new_pos`, remembering the old position for interpolation.
    pub fn advance(&mut self, new_pos: Vec2) {
        self.prev = self.pos;
        self.pos = new_pos;
    }

    /// Teleport: reset history so no interpolation streak is drawn.
    pub fn snap(&mut self, pos: Vec2) {
        self.pos = pos;
        self.prev = pos;
    }

    /// Position blended between the previous and current step.
    /// `alpha` is [`Time::alpha`](crate::time::Time::alpha).
    pub fn interp(&self, alpha: f32) -> Vec2 {
        self.prev.lerp(self.pos, alpha.clamp(0.0, 1.0))
    }
}

/// How a thing moves.
#[derive(Debug, Clone, Copy)]
pub struct Velocity {
    pub v: Vec2,
}

/// How a thing looks (filled rect).
#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    pub w: f32,
    pub h: f32,
    pub color: Color,
}

/// What a thing can bump into (AABB centered on its [`Transform`]).
#[derive(Debug, Clone, Copy)]
pub struct Collider {
    pub w: f32,
    pub h: f32,
}

/// Countdown for things that remove themselves (particles, bullets...).
#[derive(Debug, Clone, Copy)]
pub struct Lifetime {
    pub remaining: f32,
}

#[derive(Debug, Clone, Copy)]
struct Slot<T> {
    gen: u32,
    value: Option<T>,
}

impl<T> Slot<T> {
    fn empty() -> Self {
        Self { gen: 0, value: None }
    }
}

/// Sparse component storage: slots aligned with `Entity` ids.
#[derive(Debug, Default)]
pub struct Pool<T> {
    slots: Vec<Slot<T>>,
}

impl<T> Pool<T> {
    /// Insert a component. Returns `false` (doing nothing) unless the slot's
    /// generation matches the handle exactly — a stale handle can never
    /// claim a slot, even a freshly retired (empty) one.
    pub fn insert(&mut self, entity: Entity, component: T) -> bool {
        let i = entity.id as usize;
        if i >= self.slots.len() {
            self.slots.resize_with(i + 1, Slot::empty);
        }
        let slot = &mut self.slots[i];
        if slot.gen != entity.gen {
            return false;
        }
        slot.value = Some(component);
        true
    }

    pub fn get(&self, entity: Entity) -> Option<&T> {
        let slot = self.slots.get(entity.id as usize)?;
        if slot.gen != entity.gen {
            return None;
        }
        slot.value.as_ref()
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let slot = self.slots.get_mut(entity.id as usize)?;
        if slot.gen != entity.gen {
            return None;
        }
        slot.value.as_mut()
    }

    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        let slot = self.slots.get_mut(entity.id as usize)?;
        if slot.gen != entity.gen {
            return None;
        }
        slot.value.take()
    }

    /// Invalidate every handle to this slot: clear the value, bump the gen.
    /// Called automatically by [`World::despawn`] for every registered pool.
    pub fn retire(&mut self, id: u32) {
        if let Some(slot) = self.slots.get_mut(id as usize) {
            slot.gen = slot.gen.wrapping_add(1);
            slot.value = None;
        }
    }

    /// Clear all values (generations are preserved, so stale handles stay
    /// invalid).
    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            slot.value = None;
        }
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.get(entity).is_some()
    }

    pub fn len(&self) -> usize {
        self.slots.iter().filter(|s| s.value.is_some()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.slots.iter().enumerate().filter_map(|(i, s)| {
            s.value.as_ref().map(|c| (Entity { id: i as u32, gen: s.gen }, c))
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
        self.slots.iter_mut().enumerate().filter_map(|(i, s)| {
            s.value.as_mut().map(|c| (Entity { id: i as u32, gen: s.gen }, c))
        })
    }
}

/// Type-erased operations every pool must offer the registry.
trait PoolErased: 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn retire(&mut self, id: u32);
    fn clear(&mut self);
}

impl<T: Component> PoolErased for Pool<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn retire(&mut self, id: u32) {
        Pool::retire(self, id);
    }
    fn clear(&mut self) {
        Pool::clear(self);
    }
}

/// `TypeId → pool` map. Lives inside [`World`].
#[derive(Default)]
struct Components {
    pools: HashMap<TypeId, Box<dyn PoolErased>>,
}

impl Components {
    fn pool_of<C: Component>(&mut self) -> &mut Pool<C> {
        self.pools
            .entry(TypeId::of::<C>())
            .or_insert_with(|| Box::new(Pool::<C>::default()))
            .as_any_mut()
            .downcast_mut::<Pool<C>>()
            .expect("pool type confusion")
    }

    fn pool<C: Component>(&self) -> Option<&Pool<C>> {
        self.pools.get(&TypeId::of::<C>())?.as_any().downcast_ref()
    }

    fn insert<C: Component>(&mut self, entity: Entity, component: C) -> bool {
        self.pool_of::<C>().insert(entity, component)
    }

    fn retire_all(&mut self, id: u32) {
        for pool in self.pools.values_mut() {
            pool.retire(id);
        }
    }

    fn clear_all(&mut self) {
        for pool in self.pools.values_mut() {
            pool.clear();
        }
    }
}

/// Entity lifetimes + component storage for arbitrary types.
#[derive(Default)]
pub struct World {
    alive: Vec<bool>,
    gen: Vec<u32>,
    free: Vec<u32>,
    next_id: u32,
    comps: Components,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self) -> Entity {
        if let Some(id) = self.free.pop() {
            self.alive[id as usize] = true;
            Entity { id, gen: self.gen[id as usize] }
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.alive.push(true);
            self.gen.push(0);
            Entity { id, gen: 0 }
        }
    }

    /// Despawns the entity and invalidates all old handles across *every*
    /// registered component type. Returns `false` if the handle was stale.
    pub fn despawn(&mut self, entity: Entity) -> bool {
        let i = entity.id as usize;
        if i >= self.alive.len() || !self.alive[i] || self.gen[i] != entity.gen {
            return false;
        }
        self.alive[i] = false;
        self.gen[i] = self.gen[i].wrapping_add(1);
        self.free.push(entity.id);
        self.comps.retire_all(entity.id);
        true
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        let i = entity.id as usize;
        i < self.alive.len() && self.alive[i] && self.gen[i] == entity.gen
    }

    pub fn entity_count(&self) -> usize {
        self.alive.iter().filter(|&&a| a).count()
    }

    /// All currently living entities.
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.alive
            .iter()
            .zip(self.gen.iter())
            .enumerate()
            .filter(|&(_, (alive, _))| *alive)
            .map(|(i, (_, gen))| Entity { id: i as u32, gen: *gen })
    }

    /// Despawn everything.
    pub fn clear(&mut self) {
        for i in 0..self.alive.len() {
            if self.alive[i] {
                self.alive[i] = false;
                self.gen[i] = self.gen[i].wrapping_add(1);
                self.free.push(i as u32);
            }
        }
        self.comps.clear_all();
    }

    // -- component access ---------------------------------------------------

    /// Attach `component` to `entity`. Returns `false` if the handle is stale.
    pub fn insert<C: Component>(&mut self, entity: Entity, component: C) -> bool {
        self.comps.insert(entity, component)
    }

    pub fn get<C: Component>(&self, entity: Entity) -> Option<&C> {
        self.comps.pool::<C>().and_then(|p| p.get(entity))
    }

    pub fn get_mut<C: Component>(&mut self, entity: Entity) -> Option<&mut C> {
        self.comps.pool_of::<C>().get_mut(entity)
    }

    pub fn remove<C: Component>(&mut self, entity: Entity) -> Option<C> {
        self.comps.pool_of::<C>().remove(entity)
    }

    pub fn has<C: Component>(&self, entity: Entity) -> bool {
        self.comps.pool::<C>().is_some_and(|p| p.contains(entity))
    }

    /// Direct pool access for advanced use (see the `join*` functions).
    pub fn pool<C: Component>(&self) -> Option<&Pool<C>> {
        self.comps.pool::<C>()
    }

    /// Direct mutable pool access (creates the pool if missing).
    pub fn pool_mut<C: Component>(&mut self) -> &mut Pool<C> {
        self.comps.pool_of::<C>()
    }

    // -- queries --------------------------------------------------------------

    /// Every entity with a `C`.
    pub fn query<C: Component>(&self) -> impl Iterator<Item = (Entity, &C)> + '_ {
        self.comps.pool::<C>().into_iter().flat_map(|p| p.iter())
    }

    /// Every entity with both `A` and `B`.
    pub fn query2<A: Component, B: Component>(&self) -> impl Iterator<Item = (Entity, &A, &B)> + '_ {
        self.comps
            .pool::<A>()
            .zip(self.comps.pool::<B>())
            .into_iter()
            .flat_map(|(a, b)| join2(a, b))
    }

    /// Mutable iteration over every `C`.
    pub fn query_mut<C: Component>(&mut self) -> impl Iterator<Item = (Entity, &mut C)> + '_ {
        self.comps.pool_of::<C>().iter_mut()
    }

    /// Call `f` for every entity with a `C` (mutable).
    pub fn each_mut<C: Component>(&mut self, mut f: impl FnMut(Entity, &mut C)) {
        for (e, c) in self.comps.pool_of::<C>().iter_mut() {
            f(e, c);
        }
    }

    /// Call `f` for every entity with **both** `A` and `B` (both mutable).
    /// `A` and `B` must be different types.
    pub fn each2_mut<A: Component, B: Component>(
        &mut self,
        mut f: impl FnMut(Entity, &mut A, &mut B),
    ) {
        debug_assert_ne!(TypeId::of::<A>(), TypeId::of::<B>());
        // Temporarily lift both pools out of the map: clean mutable aliasing.
        let mut pa = self
            .comps
            .pools
            .remove(&TypeId::of::<A>())
            .unwrap_or_else(|| Box::new(Pool::<A>::default()));
        let mut pb = self
            .comps
            .pools
            .remove(&TypeId::of::<B>())
            .unwrap_or_else(|| Box::new(Pool::<B>::default()));
        {
            let a = pa.as_any_mut().downcast_mut::<Pool<A>>().expect("pool type confusion");
            let b = pb.as_any_mut().downcast_mut::<Pool<B>>().expect("pool type confusion");
            for (e, x, y) in join2_mut(a, b) {
                f(e, x, y);
            }
        }
        self.comps.pools.insert(TypeId::of::<A>(), pa);
        self.comps.pools.insert(TypeId::of::<B>(), pb);
    }
}

/// Iterate entities that have **both** components `T` and `U`.
pub fn join2<'a, T, U>(
    a: &'a Pool<T>,
    b: &'a Pool<U>,
) -> impl Iterator<Item = (Entity, &'a T, &'a U)> {
    a.slots.iter().zip(b.slots.iter()).enumerate().filter_map(|(i, (a, b))| {
        if a.gen != b.gen {
            return None; // generations disagree: one side is stale
        }
        Some((Entity { id: i as u32, gen: a.gen }, a.value.as_ref()?, b.value.as_ref()?))
    })
}

/// Mutable version of [`join2`].
pub fn join2_mut<'a, T, U>(
    a: &'a mut Pool<T>,
    b: &'a mut Pool<U>,
) -> impl Iterator<Item = (Entity, &'a mut T, &'a mut U)> {
    a.slots.iter_mut().zip(b.slots.iter_mut()).enumerate().filter_map(|(i, (a, b))| {
        if a.gen != b.gen {
            return None;
        }
        Some((Entity { id: i as u32, gen: a.gen }, a.value.as_mut()?, b.value.as_mut()?))
    })
}

/// Iterate entities that have all three components.
pub fn join3<'a, T, U, V>(
    a: &'a Pool<T>,
    b: &'a Pool<U>,
    c: &'a Pool<V>,
) -> impl Iterator<Item = (Entity, &'a T, &'a U, &'a V)> {
    a.slots
        .iter()
        .zip(b.slots.iter())
        .zip(c.slots.iter())
        .enumerate()
        .filter_map(|(i, ((a, b), c))| {
            if a.gen != b.gen || b.gen != c.gen {
                return None;
            }
            Some((
                Entity { id: i as u32, gen: a.gen },
                a.value.as_ref()?,
                b.value.as_ref()?,
                c.value.as_ref()?,
            ))
        })
}

/// Mutable triple join. All three component types must differ.
pub fn join3_mut<'a, T, U, V>(
    a: &'a mut Pool<T>,
    b: &'a mut Pool<U>,
    c: &'a mut Pool<V>,
) -> impl Iterator<Item = (Entity, &'a mut T, &'a mut U, &'a mut V)> {
    a.slots
        .iter_mut()
        .zip(b.slots.iter_mut())
        .zip(c.slots.iter_mut())
        .enumerate()
        .filter_map(|(i, ((a, b), c))| {
            if a.gen != b.gen || b.gen != c.gen {
                return None;
            }
            Some((
                Entity { id: i as u32, gen: a.gen },
                a.value.as_mut()?,
                b.value.as_mut()?,
                c.value.as_mut()?,
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Mana {
        amount: u8,
    }

    #[test]
    fn recycled_ids_get_new_generations() {
        let mut world = World::new();
        let a = world.spawn();
        world.insert(a, Mana { amount: 10 });
        assert!(world.despawn(a));
        assert!(!world.is_alive(a));
        assert_eq!(world.get::<Mana>(a), None);

        let b = world.spawn();
        assert_eq!(b.id(), a.id()); // id recycled...
        assert_eq!(b.generation(), a.generation() + 1); // ...generation bumped

        assert!(world.insert(b, Mana { amount: 20 }));
        assert_eq!(world.get::<Mana>(b).unwrap().amount, 20);
        // The stale handle sees nothing...
        assert_eq!(world.get::<Mana>(a), None);
        // ...and can't clobber the new occupant's slot.
        assert!(!world.insert(a, Mana { amount: 99 }));
        assert_eq!(world.get::<Mana>(b).unwrap().amount, 20);
    }

    #[test]
    fn stale_insert_cannot_claim_retired_slot() {
        let mut world = World::new();
        let a = world.spawn();
        world.insert(a, Mana { amount: 1 });
        world.despawn(a); // gen bumped, value cleared

        // Even though the slot is empty, the stale handle must be rejected.
        assert!(!world.insert(a, Mana { amount: 2 }));

        let b = world.spawn();
        assert_eq!(b.generation(), a.generation() + 1);
        assert!(world.insert(b, Mana { amount: 3 }));
        assert_eq!(world.get::<Mana>(b).unwrap().amount, 3);
        assert_eq!(world.get::<Mana>(a), None);
    }

    #[test]
    fn arbitrary_components_work_and_retire_on_despawn() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Mana { amount: 7 });
        assert!(world.has::<Mana>(e));
        assert_eq!(world.query::<Mana>().count(), 1);

        world.despawn(e);
        let e2 = world.spawn(); // same id, gen+1
        assert_eq!(world.get::<Mana>(e2), None, "custom pool retired with entity");
    }

    #[test]
    fn each2_mut_integrates_movement() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Transform::new(Vec2::ZERO));
        world.insert(e, Velocity { v: Vec2::new(10.0, 0.0) });

        world.each2_mut(|_, tr: &mut Transform, vel: &mut Velocity| {
            tr.advance(tr.pos + vel.v * 0.5);
        });

        assert_eq!(world.get::<Transform>(e).unwrap().pos, Vec2::new(5.0, 0.0));
        assert_eq!(world.get::<Transform>(e).unwrap().prev, Vec2::ZERO);
    }

    #[test]
    fn query2_yields_only_entities_with_both() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Transform::new(Vec2::ZERO));
        assert_eq!(world.query2::<Transform, Velocity>().count(), 0);
        world.insert(e, Velocity { v: Vec2::ZERO });
        assert_eq!(world.query2::<Transform, Velocity>().count(), 1);
    }

    #[test]
    fn interpolation_snap_and_interp() {
        let mut tr = Transform::at(0.0, 0.0);
        tr.advance(Vec2::new(10.0, 0.0));
        assert_eq!(tr.interp(0.5), Vec2::new(5.0, 0.0));
        tr.snap(Vec2::new(50.0, 50.0));
        assert_eq!(tr.interp(0.0), Vec2::new(50.0, 50.0), "snap clears history");
    }

    #[test]
    fn clear_despawns_everything() {
        let mut world = World::new();
        for _ in 0..3 {
            world.spawn();
        }
        world.clear();
        assert_eq!(world.entity_count(), 0);
        assert!(world.entities().next().is_none());
    }
}
