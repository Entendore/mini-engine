//! Collision queries over `Transform` + `Collider` (AABB, y-down).
//! O(n²) — exactly right for the entity counts a mini engine targets. For
//! tiles and static level geometry, resolve against rects directly (see the
//! platformer example) — these helpers are for entity-vs-entity.

use crate::ecs::{Collider, Entity, Transform, World};
use crate::math::Rect;

fn rect_of(world: &World, e: Entity) -> Option<Rect> {
    let tr = world.get::<Transform>(e)?;
    let c = world.get::<Collider>(e)?;
    Some(Rect::from_center(tr.pos, c.w, c.h))
}

/// All overlapping pairs; each unordered pair is reported once.
pub fn collision_pairs(world: &World) -> Vec<(Entity, Entity)> {
    let entities: Vec<Entity> = world.entities().collect();
    let mut pairs = Vec::new();
    for i in 0..entities.len() {
        for j in i + 1..entities.len() {
            if let (Some(a), Some(b)) = (rect_of(world, entities[i]), rect_of(world, entities[j])) {
                if a.intersects(b) {
                    pairs.push((entities[i], entities[j]));
                }
            }
        }
    }
    pairs
}

/// Every entity whose collider overlaps `e`'s (excluding `e` itself).
pub fn overlaps(world: &World, e: Entity) -> Vec<Entity> {
    let Some(me) = rect_of(world, e) else { return Vec::new() };
    world
        .entities()
        .filter(|other| *other != e)
        .filter(|other| rect_of(world, *other).is_some_and(|r| me.intersects(r)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec2;

    fn body(world: &mut World, x: f32, y: f32, size: f32) -> Entity {
        let e = world.spawn();
        world.insert(e, Transform::new(Vec2::new(x, y)));
        world.insert(e, Collider { w: size, h: size });
        e
    }

    #[test]
    fn pairs_are_reported_once_and_respect_despawn() {
        let mut world = World::new();
        let a = body(&mut world, 0.0, 0.0, 10.0);
        let b = body(&mut world, 5.0, 0.0, 10.0);
        let far = body(&mut world, 100.0, 100.0, 10.0);

        let pairs = collision_pairs(&world);
        assert_eq!(pairs.len(), 1);
        let (p, q) = pairs[0];
        assert!((p == a && q == b) || (p == b && q == a));
        assert_eq!(overlaps(&world, far), Vec::new());
        assert_eq!(overlaps(&world, a), vec![b]);

        world.despawn(b);
        assert!(collision_pairs(&world).is_empty());
        assert!(overlaps(&world, a).is_empty());
    }

    #[test]
    fn entities_without_collider_or_transform_are_skipped() {
        let mut world = World::new();
        let bare = world.spawn(); // neither component
        let solid = body(&mut world, 0.0, 0.0, 10.0);
        assert!(collision_pairs(&world).is_empty());
        assert_eq!(overlaps(&world, solid), Vec::new());
        assert_eq!(overlaps(&world, bare), Vec::new());
        let _ = bare;
    }
}
