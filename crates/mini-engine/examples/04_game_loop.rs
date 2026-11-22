//! The game-loop pattern: fixed-rate simulation + interpolated rendering.
//!
//! `update` runs exactly `update_hz` times per second regardless of display
//! refresh. Rendering happens more or less often; to avoid stutter, positions
//! are blended between the previous and current step with `time.alpha()`.

use mini_engine::prelude::*;

const W: f32 = 480.0;
const H: f32 = 270.0;
const SPEED: f32 = 220.0;

struct Bounce {
    tr: Transform, // has `prev` built in for interpolation
    vel: Vec2,
    trail: Vec<Vec2>,
}

impl Game for Bounce {
    fn update(&mut self, ctx: &mut Ctx) {
        let dt = ctx.time.delta(); // constant 1/60 with the default config

        // `advance` (not `pos =`) records the previous position for `interp`.
        let mut next = self.tr.pos + self.vel * dt;
        if next.x < 12.0 || next.x > W - 12.0 {
            self.vel.x = -self.vel.x;
            next.x = next.x.clamp(12.0, W - 12.0);
        }
        if next.y < 12.0 || next.y > H - 12.0 {
            self.vel.y = -self.vel.y;
            next.y = next.y.clamp(12.0, H - 12.0);
        }
        self.tr.advance(next);

        self.trail.push(self.tr.pos);
        if self.trail.len() > 20 {
            self.trail.remove(0);
        }
    }

    fn render(&mut self, ctx: &mut Ctx) {
        let alpha = ctx.time.alpha(); // 0..=1 between the two sim steps
        let r = &mut ctx.renderer;
        r.clear(Color::rgb(12, 14, 22));

        // Trail at raw positions — fading hides any step jitter.
        for (i, p) in self.trail.iter().enumerate() {
            let t = (i + 1) as f32 / self.trail.len() as f32;
            let c = Color::rgb(60, 70, 110).lerp(Color::CYAN, t * 0.5);
            r.circle(p.x as i32, p.y as i32, 3, c);
        }

        // The ball: interpolated — smooth at any refresh rate.
        let p = self.tr.interp(alpha);
        r.circle(p.x as i32, p.y as i32, 10, Color::CYAN);
        r.circle_outline(p.x as i32, p.y as i32, 12, Color::WHITE);

        r.text(8, 8, &format!("ALPHA {:.2}   FPS {:.0}", alpha, ctx.time.fps()), 2, Color::GRAY);
        r.text_right(W as i32 - 8, 8, "FIXED 60 HZ SIM - INTERPOLATED DRAW", 2, Color::GRAY);
    }
}

fn main() -> Result<()> {
    // Try `.update_hz(30.0)` — the sim runs at 30 but still *looks* smooth,
    // because rendering blends between steps.
    let config = EngineConfig::new().title("04 - game loop").size(480, 270);
    let mut game = Bounce {
        tr: Transform::at(W / 2.0, H / 2.0),
        vel: Vec2::from_angle(0.7) * SPEED,
        trail: Vec::new(),
    };
    run(config, &mut game)
}
