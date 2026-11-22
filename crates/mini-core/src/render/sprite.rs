//! Sprite components: static images and atlas-driven animation, drawn
//! interpolated between simulation steps by [`draw_sprites`].

use std::rc::Rc;

use super::atlas::{Animator, Atlas};
use super::{Blit, Color, Image, Renderer};
use crate::ecs::{Entity, Transform, World};
use crate::math::Vec2;

/// A blittable image attached to an entity, drawn centered on its
/// [`Transform`] (plus `offset`). [`animate_sprites`] keeps this in sync with
/// [`AnimatedSprite`]; games may also set it directly for static sprites.
#[derive(Clone)]
pub struct SpriteImage {
    /// The frame to draw.
    pub image: Rc<Image>,
    /// Offset from the entity's transform center, in pixels.
    pub offset: Vec2,
    /// Per-channel multiply before blending.
    pub tint: Option<Color>,
    /// Extra opacity in `0.0..=1.0`.
    pub alpha: f32,
}

impl SpriteImage {
    /// Full-brightness, fully opaque, centered on the transform.
    pub fn new(image: Rc<Image>) -> Self {
        Self { image, offset: Vec2::ZERO, tint: None, alpha: 1.0 }
    }

    /// Builder-style tint.
    pub fn with_tint(mut self, tint: Color) -> Self {
        self.tint = Some(tint);
        self
    }
}

/// Atlas-driven frame animation; pairs with [`SpriteImage`].
pub struct AnimatedSprite {
    /// The frame source.
    pub atlas: Rc<Atlas>,
    /// Drives the frame index over time.
    pub animator: Animator,
}

impl AnimatedSprite {
    /// Plays every frame of `atlas` at `fps`; loop or play once.
    pub fn new(atlas: Rc<Atlas>, fps: f32, looping: bool) -> Self {
        Self { atlas, animator: Animator::new(fps, atlas.frame_count(), looping) }
    }
}

/// Advance every [`AnimatedSprite`] and sync its entity's [`SpriteImage`]
/// to the current frame (inserting one if absent). Call once per update.
pub fn animate_sprites(world: &mut World, dt: f32) {
    for (_, a) in world.query_mut::<AnimatedSprite>() {
        a.animator.advance(dt);
    }
    let updates: Vec<(Entity, Rc<Image>)> = world
        .query::<AnimatedSprite>()
        .filter_map(|(e, a)| a.atlas.frame(a.animator.index()).map(|img| (e, img)))
        .collect();
    for (e, img) in updates {
        let inserted = match world.get_mut::<SpriteImage>(e) {
            Some(si) => {
                si.image = img;
                false
            }
            None => true,
        };
        if inserted {
            world.insert(e, SpriteImage::new(img));
        }
    }
}

/// Draw every [`SpriteImage`] blended between the previous and current
/// simulation step. (Rect [`Sprite`](crate::ecs::Sprite)s are drawn by the
/// game as before — this covers image sprites.)
pub fn draw_sprites(world: &World, r: &mut Renderer, alpha: f32) {
    for (_, si, tr) in world.query2::<SpriteImage, Transform>() {
        let center = tr.interp(alpha) + si.offset;
        r.blit_centered_opts(
            &si.image,
            center,
            Blit { tint: si.tint, alpha: si.alpha, ..Blit::default() },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::World;

    #[test]
    fn animate_sprites_syncs_frames_and_inserts_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, Transform::at(0.0, 0.0));
        world.insert(
            e,
            AnimatedSprite::new(Rc::new(Atlas::new(Image::solid(2, 2, Color::WHITE), 2, 1).unwrap()), 1.0, false),
        );

        animate_sprites(&mut world, 0.0);
        assert!(world.has::<SpriteImage>(e), "component auto-inserted");

        animate_sprites(&mut world, 1.0);
        let atlas = Atlas::new(Image::solid(2, 2, Color::WHITE), 2, 1).unwrap();
        let expected = atlas.frame(1).unwrap();
        assert_eq!(world.get::<SpriteImage>(e).unwrap().image.pixels(), expected.pixels());
    }
}
