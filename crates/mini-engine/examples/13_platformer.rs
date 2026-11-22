//! Putting it together — a tiny platformer:
//! ActionMap bindings, gravity + jump, axis-by-axis AABB tile collision,
//! a camera that follows the player, coin pickups, and audio.
//! No assets: the level is an ASCII map.

use mini_engine::prelude::*;

const W: f32 = 480.0;
const H: f32 = 272.0;
const TS: f32 = 16.0; // tile size
const LEVEL_W: usize = 40;
const LEVEL_H: usize = 15;
const PW: f32 = 10.0; // player size
const PH: f32 = 14.0;
const MOVE: f32 = 120.0;
const GRAV: f32 = 780.0;
const JUMP: f32 = 295.0;

const SND_JUMP: Sound = Sound { freq_hz: 260.0, slide_hz: 180.0, ms: 120, ..Sound::BEEP };
const SND_COIN: Sound = Sound { freq_hz: 990.0, ms: 70, wave: Waveform::Square, ..Sound::BEEP };

const LEVEL: [&str; LEVEL_H] = [
    "########################################",
    "#......................................#",
    "#..........o.o.o.......................#",
    "#.........########.....................#",
    "#......................................#",
    "#....o.................................#",
    "#...####........o.o....................#",
    "#..............#####...................#",
    "#......................................#",
    "#..o.....................o.............#",
    "#.####..................###............#",
    "#......................................#",
    "#..........o...........................#",
    "#.........###..........................#",
    "########################################",
];

fn tile_rect(tx: usize, ty: usize) -> Rect {
    Rect::new(tx as f32 * TS, ty as f32 * TS, TS, TS)
}

struct Platformer {
    actions: ActionMap,
    solid: Vec<Vec<bool>>,
    coins: Vec<Vec2>,
    pos: Vec2, // top-left of the player
    vel: Vec2,
    grounded: bool,
    score: u32,
    total: usize,
    t: f32,
}

impl Platformer {
    fn new() -> Self {
        let mut actions = ActionMap::new();
        actions
            .bind("left", &[KeyCode::A, KeyCode::Left])
            .bind("right", &[KeyCode::D, KeyCode::Right])
            .bind("jump", &[KeyCode::Space, KeyCode::W, KeyCode::Up]);
        let mut game = Self {
            actions,
            solid: Vec::new(),
            coins: Vec::new(),
            pos: Vec2::new(28.0, 190.0),
            vel: Vec2::ZERO,
            grounded: false,
            score: 0,
            total: 0,
            t: 0.0,
        };
        game.reset();
        game
    }

    fn reset(&mut self) {
        self.solid = LEVEL
            .iter()
            .map(|row| (0..LEVEL_W).map(|x| row.chars().nth(x) == Some('#')).collect())
            .collect();
        self.coins = LEVEL
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                (0..LEVEL_W).filter_map(|x| {
                    (row.chars().nth(x) == Some('o'))
                        .then(|| Vec2::new(x as f32 * TS + TS / 2.0, y as f32 * TS + TS / 2.0))
                })
            })
            .collect();
        self.total = self.coins.len();
        self.pos = Vec2::new(28.0, 190.0);
        self.vel = Vec2::ZERO;
        self.score = 0;
    }

    fn player_rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, PW, PH)
    }

    /// Move by (dx, dy), then push out of any overlapping solid tile.
    /// Returns which axes were blocked.
    fn resolve(&mut self, dx: f32, dy: f32) -> (bool, bool) {
        self.pos.x += dx;
        self.pos.y += dy;
        let (mut hit_x, mut hit_y) = (false, false);
        for ty in 0..LEVEL_H {
            for tx in 0..LEVEL_W {
                if !self.solid[ty][tx] {
                    continue;
                }
                let tr = tile_rect(tx, ty);
                let pr = self.player_rect();
                if !pr.intersects(tr) {
                    continue;
                }
                if dx > 0.0 {
                    self.pos.x = tr.x - pr.w;
                    hit_x = true;
                }
                if dx < 0.0 {
                    self.pos.x = tr.max().x;
                    hit_x = true;
                }
                if dy > 0.0 {
                    self.pos.y = tr.y - pr.h;
                    hit_y = true;
                }
                if dy < 0.0 {
                    self.pos.y = tr.max().y;
                    hit_y = true;
                }
            }
        }
        (hit_x, hit_y)
    }
}

impl Game for Platformer {
    fn update(&mut self, ctx: &mut Ctx) {
        let dt = ctx.time.delta();
        self.t += dt;

        if ctx.input.pressed(KeyCode::R) {
            self.reset();
        }

        // -- horizontal: input + resolve -----------------------------------
        let dir = ctx.input.action_axis(&self.actions, "left", "right");
        self.vel.x = dir * MOVE;
        let (hit_x, _) = self.resolve(self.vel.x * dt, 0.0);
        if hit_x {
            self.vel.x = 0.0;
        }

        // -- vertical: gravity + jump + resolve ------------------------------
        self.vel.y = (self.vel.y + GRAV * dt).min(400.0);
        if self.grounded && ctx.input.action_pressed(&self.actions, "jump") {
            self.vel.y = -JUMP;
            ctx.audio.play(SND_JUMP);
        }
        let (_, hit_y) = self.resolve(0.0, self.vel.y * dt);
        self.grounded = hit_y && self.vel.y > 0.0;
        if hit_y {
            self.vel.y = 0.0;
        }

        // -- pickups: plain rect overlap --------------------------------------
        let pr = self.player_rect();
        if let Some(i) = self
            .coins
            .iter()
            .position(|c| Rect::from_center(*c, 12.0, 12.0).intersects(pr))
        {
            self.coins.remove(i);
            self.score += 1;
            ctx.audio.play(SND_COIN);
        }
    }

    fn render(&mut self, ctx: &mut Ctx) {
        let r = &mut ctx.renderer;
        r.v_gradient(Color::rgb(18, 22, 40), Color::rgb(34, 28, 52));

        // Camera follows the player horizontally; the 32px top strip is HUD
        // (camera y = -32 shifts the world DOWN, leaving screen top free).
        let cam_x = (self.pos.x + PW / 2.0 - W / 2.0).clamp(0.0, LEVEL_W as f32 * TS - W);
        r.set_camera(Vec2::new(cam_x, -32.0));

        // Tiles (checkerboard so edges read clearly).
        for ty in 0..LEVEL_H {
            for tx in 0..LEVEL_W {
                if self.solid[ty][tx] {
                    let c = if (tx + ty) % 2 == 0 {
                        Color::rgb(70, 84, 120)
                    } else {
                        Color::rgb(60, 72, 104)
                    };
                    r.rect(tile_rect(tx, ty), c);
                }
            }
        }

        // Coins (bobbing).
        for c in &self.coins {
            let y = c.y + (self.t * 4.0 + c.x * 0.1).sin() * 2.0;
            r.circle(c.x as i32, y as i32, 3, Color::YELLOW);
        }

        // Player.
        let pr = self.player_rect();
        r.rect(pr, Color::CYAN);
        r.rect_outline(pr, Color::WHITE);
        r.set_camera(Vec2::ZERO); // HUD below is screen-space

        r.text(8, 10, &format!("COINS {}/{}", self.score, self.total), 2, Color::WHITE);
        r.text_right(W as i32 - 8, 10, "A/D MOVE  SPACE JUMP  R RESET", 2, Color::GRAY);
        if self.coins.is_empty() {
            r.text_centered(H as i32 / 2, "ALL COINS COLLECTED!", 5, Color::YELLOW);
        }
    }
}

fn main() -> Result<()> {
    let mut game = Platformer::new();
    run(EngineConfig::new().title("13 - platformer").size(480, 272), &mut game)
}
