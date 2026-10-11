//! The ECS — spawn, custom components, systems, despawn, collision queries.
//!
//! Hold LMB to emit particles. Each one carries:
//!   Transform / Velocity / Sprite / Lifetime  — built-ins
//!   Pulse                                     — a CUSTOM component:
//!     any 'static struct becomes a component with one `world.insert`.

use mini_engine::prelude::*;

struct Pulse {
    phase: f32,
}

struct Particles {
    world: World,
    rng: Rng,
}

impl Particles {
    fn emit(&mut self, at: Vec2) {
        for _ in 0..8 {
            let e = self.world.spawn();
            let angle = self.rng.range_f32(0.0, std::f32::consts::TAU);
            let speed = self.rng.range_f32(30.0, 190.0);
            self.world.insert(e, Transform::new(at));
            self.world.insert(e, Velocity { v: Vec2::from_angle(angle) * speed });
            self.world.insert(e, Sprite { w: 4.0, h: 4.0, color: Color::WHITE });
            self.world.insert(e, Lifetime { remaining: self.rng.range_f32(0.4, 1.2) });
            self.world.insert(e, Pulse { phase: self.rng.range_f32(0.0, std::f32::consts::TAU) });
        }
    }
}

impl Game for Particles {
    fn update(&mut self, ctx: &mut Ctx) {
        let dt = ctx.time.delta();

        if ctx.input.mouse_down(MouseButton::Left) {
            if let Some(p) = ctx.input.mouse_pos() {
                self.emit(p);
            }
        }
        if ctx.input.pressed(KeyCode::C) {
            self.world.clear();
        }

        // System 1: movement — every Transform+Velocity pair, mutably.
        self.world.each2_mut(|_, tr: &mut Transform, vel: &mut Velocity| {
            tr.advance(tr.pos + vel.v * dt);
            vel.v *= 1.0 - 1.6 * dt; // drag
        });

        // System 2: the custom component drives a color cycle.
        self.world.each2_mut(|_, spr: &mut Sprite, pu: &mut Pulse| {
            pu.phase += dt * 4.0;
            spr.color = Color::CYAN.lerp(Color::MAGENTA, 0.5 + 0.5 * pu.phase.sin());
        });

        // System 3: lifetimes — tick, then despawn the expired.
        for (_, life) in self.world.query_mut::<Lifetime>() {
            life.remaining -= dt;
        }
        let dead: Vec<Entity> = self
            .world
            .query::<Lifetime>()
            .filter(|(_, l)| l.remaining <= 0.0)
            .map(|(e, _)| e)
            .collect();
        for e in dead {
            self.world.despawn(e);
        }
    }

    fn render(&mut self, ctx: &mut Ctx) {
        let alpha = ctx.time.alpha();
        let r = &mut ctx.renderer;
        r.clear(Color::rgb(10, 12, 20));

        // One query draws every particle.
        for (_, spr, tr) in self.world.query2::<Sprite, Transform>() {
            r.rect(Rect::from_center(tr.interp(alpha), spr.w, spr.h), spr.color);
        }

        // physics module: entity-vs-entity AABB overlap over the whole world.
        let pairs = collision_pairs(&self.world).len();
        r.text(
            8,
            8,
            &format!("ENTITIES {}   OVERLAPPING PAIRS {}", self.world.entity_count(), pairs),
            2,
            Color::GRAY,
        );
        r.text_right(472, 8, "HOLD LMB TO EMIT - C CLEARS", 2, Color::GRAY);
    }
}

fn main() -> Result<()> {
    let mut game = Particles { world: World::new(), rng: Rng::from_entropy() };
    run(EngineConfig::new().title("05 - ecs").size(480, 270), &mut game)
}
