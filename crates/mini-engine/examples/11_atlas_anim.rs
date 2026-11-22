//! Atlas animation — two routes:
//!   1. Declarative: an `AnimatedSprite` component + the `animate_sprites`
//!      system + `draw_sprites` in render (the comet, left).
//!   2. Manual: `atlas.frame(animator.index())` whenever you want, with any
//!      blit options (the pulsing ring, right).
//!
//! Both atlases are generated in code. With `--features png` you'd instead
//! write:
//!   let img = mini_engine::assets::load_png("sprites.png")?;
//!   let atlas = Atlas::new(img, 8, 2).ok_or("grid must divide evenly")?;

use std::rc::Rc;

use mini_engine::prelude::*;

const W: f32 = 480.0;
const H: f32 = 270.0;

/// Frames of a comet orbiting a center, laid out in one long row.
fn spinner_atlas(frames: u32, size: u32) -> Atlas {
    let c = size as f32 * 0.5;
    let orbit = size as f32 * 0.32;
    let mut px = Vec::new();
    for f in 0..frames {
        let a = f as f32 / frames as f32 * std::f32::consts::TAU;
        let hx = c + a.cos() * orbit;
        let hy = c + a.sin() * orbit;
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 + 0.5 - hx;
                let dy = y as f32 + 0.5 - hy;
                let d = (dx * dx + dy * dy).sqrt();
                let alpha = (1.0 - d / 4.5).clamp(0.0, 1.0);
                px.push(((alpha * 255.0) as u32) << 24 | (255 << 16) | (200 << 8) | 90);
            }
        }
    }
    let img = Image::from_pixels(size * frames, size, px).expect("generator");
    Atlas::new(img, frames, 1).expect("even grid")
}

/// Frames of an expanding, fading ring.
fn ring_atlas(frames: u32, size: u32) -> Atlas {
    let c = size as f32 * 0.5;
    let mut px = Vec::new();
    for f in 0..frames {
        let t = f as f32 / frames as f32;
        let radius = 4.0 + t * (size as f32 * 0.45);
        let alpha = 1.0 - t;
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 + 0.5 - c;
                let dy = y as f32 + 0.5 - c;
                let d = (dx * dx + dy * dy).sqrt();
                let edge = (1.0 - (d - radius).abs() / 1.6).clamp(0.0, 1.0);
                px.push(((edge * alpha * 255.0) as u32) << 24 | (90 << 16) | (220 << 8) | 255);
            }
        }
    }
    let img = Image::from_pixels(size * frames, size, px).expect("generator");
    Atlas::new(img, frames, 1).expect("even grid")
}

struct AtlasDemo {
    world: World,
    ring_atlas: Rc<Atlas>,
    ring_anim: Animator,
}

impl Game for AtlasDemo {
    fn update(&mut self, ctx: &mut Ctx) {
        // Route 1: one system call advances every AnimatedSprite and keeps
        // its SpriteImage frame in sync.
        animate_sprites(&mut self.world, ctx.time.delta());

        // Route 2: advance the manual animator yourself.
        self.ring_anim.advance(ctx.time.delta());
    }

    fn render(&mut self, ctx: &mut Ctx) {
        let r = &mut ctx.renderer;
        r.clear(Color::rgb(12, 14, 22));

        // Route 1: draw every SpriteImage (interpolated between sim steps).
        draw_sprites(&self.world, r, ctx.time.alpha());

        // Route 2: grab the frame and blit it with full control.
        if let Some(frame) = self.ring_atlas.frame(self.ring_anim.index()) {
            r.blit_centered_opts(&frame, Vec2::new(W - 110.0, H / 2.0), Blit::tinted(Color::CYAN));
        }

        r.text_centered(40, "ATLAS ANIMATION", 4, Color::WHITE);
        r.text(16, H as i32 - 30, "LEFT: ANIMATEDSPRITE SYSTEM", 2, Color::GRAY);
        r.text_right(W as i32 - 16, H as i32 - 30, "RIGHT: MANUAL ATLAS.FRAME()", 2, Color::GRAY);
    }
}

fn main() -> Result<()> {
    let mut world = World::new();
    let spinner = world.spawn();
    world.insert(spinner, Transform::at(110.0, H / 2.0));
    // Playing the whole atlas, looping, at 12 fps:
    world.insert(
        spinner,
        AnimatedSprite::new(Rc::new(spinner_atlas(12, 48)), 12.0, true),
    );

    let mut game = AtlasDemo {
        world,
        ring_atlas: Rc::new(ring_atlas(10, 80)),
        ring_anim: Animator::new(10.0, 10, true),
    };
    run(EngineConfig::new().title("11 - atlas animation").size(480, 270), &mut game)
}
