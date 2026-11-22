//! Tweens & easing.
//! Left: six boxes cross the screen, one per easing function (looping) —
//! the standalone `Tween` primitive, advanced by hand.
//! Click: a fire-and-forget `Tweens` "pop" — its closure receives the World.
//! T: tween the cyan box to a random spot (world-mutating tween).

use mini_engine::prelude::*;

const W: f32 = 480.0;
const H: f32 = 270.0;

const EASES: [EaseFn; 6] = [
    ease::linear,
    ease::smoothstep,
    ease::ease_in_quad,
    ease::ease_out_quad,
    ease::ease_in_out_cubic,
    ease::ease_out_back, // overshoots on purpose
];
const NAMES: [&str; 6] = ["LINEAR", "SMOOTHSTEP", "IN QUAD", "OUT QUAD", "IN-OUT CUBIC", "OUT BACK"];

struct TweenDemo {
    loops: [Tween; 6],
    tweens: Tweens,
    world: World,
    rng: Rng,
    hero: Entity,
}

impl TweenDemo {
    fn new() -> Self {
        let mut world = World::new();
        let hero = world.spawn();
        world.insert(hero, Transform::at(W / 2.0, H / 2.0));
        world.insert(hero, Sprite { w: 18.0, h: 18.0, color: Color::CYAN });

        Self {
            loops: [
                Tween::new(1.6, EASES[0]),
                Tween::new(1.6, EASES[1]),
                Tween::new(1.6, EASES[2]),
                Tween::new(1.6, EASES[3]),
                Tween::new(1.6, EASES[4]),
                Tween::new(1.6, EASES[5]),
            ],
            tweens: Tweens::new(),
            world,
            rng: Rng::from_entropy(),
            hero,
        }
    }
}

impl Game for TweenDemo {
    fn update(&mut self, ctx: &mut Ctx) {
        let dt = ctx.time.delta();

        // Looping primitives: advance, restart at the end.
        for tw in &mut self.loops {
            tw.advance(dt);
            if tw.finished() {
                tw.restart();
            }
        }

        // Fire-and-forget world tweens.
        self.tweens.update(dt, &mut self.world);

        if ctx.input.mouse_pressed(MouseButton::Left) {
            if let Some(p) = ctx.input.mouse_pos() {
                let e = self.world.spawn();
                self.world.insert(e, Transform::new(p));
                self.world.insert(e, Sprite { w: 26.0, h: 26.0, color: Color::YELLOW });
                // The closure captures the entity; it shrinks the sprite to
                // nothing and despawns on its final tick.
                self.tweens.add(0.45, ease::ease_out_quad, move |w, t| {
                    if t >= 1.0 {
                        w.despawn(e);
                        return;
                    }
                    if let Some(spr) = w.get_mut::<Sprite>(e) {
                        let s = 26.0 * (1.0 - t) + 2.0;
                        spr.w = s;
                        spr.h = s;
                        spr.color = Color::YELLOW.lerp(Color::RED, t);
                    }
                });
            }
        }

        if ctx.input.pressed(KeyCode::T) {
            let (e, from) = match self.world.get::<Transform>(self.hero) {
                Some(tr) => (self.hero, tr.pos),
                None => return,
            };
            let to = Vec2::new(
                self.rng.range_f32(60.0, W - 60.0),
                self.rng.range_f32(60.0, H - 60.0),
            );
            self.tweens.add(0.6, ease::ease_in_out_cubic, move |w, t| {
                if let Some(tr) = w.get_mut::<Transform>(e) {
                    tr.snap(from.lerp(to, t)); // snap: no interpolation history
                }
            });
        }
    }

    fn render(&mut self, ctx: &mut Ctx) {
        let r = &mut ctx.renderer;
        r.clear(Color::rgb(12, 14, 22));

        // Easing gallery: one row per function.
        for (i, tw) in self.loops.iter().enumerate() {
            let y = 30.0 + i as f32 * 32.0;
            let t = tw.fraction();
            r.text(16, y as i32 - 6, NAMES[i], 2, Color::GRAY);
            r.rect(Rect::new(120.0, y + 6.0, W - 180.0, 2.0), Color::rgb(40, 46, 66));
            let x = 120.0 + t * (W - 180.0);
            r.rect(Rect::from_center(Vec2::new(x, y + 7.0), 12.0, 12.0), Color::WHITE);
        }

        // World entities: the hero box + any live "pops".
        for (_, spr, tr) in self.world.query2::<Sprite, Transform>() {
            r.rect(Rect::from_center(tr.pos, spr.w, spr.h), spr.color);
        }

        r.text_right(W as i32 - 8, 8, "CLICK = POP   T = HOP", 2, Color::GRAY);
    }
}

fn main() -> Result<()> {
    let mut game = TweenDemo::new();
    run(EngineConfig::new().title("09 - tweens").size(480, 270), &mut game)
}
