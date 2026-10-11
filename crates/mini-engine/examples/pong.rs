//! MINI PONG — the capstone example. Menu scene -> play scene -> menu:
//! immediate-mode UI (buttons + music-volume slider), ActionMap bindings,
//! audio buses (music / sfx), atlas-animated ball (AnimatedSprite +
//! SpriteImage), world-mutating Tweens (AI glides home on game over),
//! springy title via the Tween primitive, and persistence (best rally
//! saved through ctx.storage).
//!
//! Controls: mouse or W/S or Up/Down - Space/Enter confirms - Q quits.

use std::rc::Rc;

use mini_engine::prelude::*;

const W: f32 = 640.0;
const H: f32 = 360.0;

const PADDLE_W: f32 = 12.0;
const PADDLE_H: f32 = 80.0;
const PADDLE_SPEED: f32 = 340.0;
const AI_BASE_SPEED: f32 = 270.0;

const BALL_SIZE: f32 = 10.0;
const BALL_IMG: u32 = 18;
const BALL_FRAMES: u32 = 6;
const BALL_START_SPEED: f32 = 250.0;
const BALL_MAX_SPEED: f32 = 620.0;

const WIN_SCORE: u32 = 5;
const SERVE_COUNTDOWN: f32 = 3.0;
const TRAIL_LEN: usize = 14;

const SND_PADDLE: Sound = Sound { freq_hz: 520.0, wave: Waveform::Square, ..Sound::BEEP };
const SND_WALL: Sound = Sound { freq_hz: 300.0, ms: 25, volume: 0.55, ..Sound::BEEP };
const SND_SCORE: Sound =
    Sound { freq_hz: 220.0, slide_hz: -140.0, ms: 250, wave: Waveform::Square, ..Sound::BEEP };
const SND_WIN: Sound =
    Sound { freq_hz: 440.0, slide_hz: 240.0, ms: 450, wave: Waveform::Triangle, ..Sound::BEEP };
const MUSIC: Sound = Sound {
    freq_hz: 110.0,
    ms: 1_500,
    wave: Waveform::Triangle,
    volume: 0.5,
    bus: Bus::Music,
    loops: true,
    ..Sound::BEEP
};

fn build_actions() -> ActionMap {
    let mut m = ActionMap::new();
    m.bind("up", &[KeyCode::W, KeyCode::Up])
        .bind("down", &[KeyCode::S, KeyCode::Down])
        .bind("confirm", &[KeyCode::Space, KeyCode::Enter]);
    m
}

/// Static soft-edged ball (menu decoration).
fn make_ball(size: u32) -> Image {
    let r = size as f32 * 0.5;
    let mut pixels = Vec::with_capacity((size * size) as usize);
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - r;
            let dy = y as f32 + 0.5 - r;
            let d = (dx * dx + dy * dy).sqrt() / r;
            let alpha = ((1.0 - d) * 2.5).clamp(0.0, 1.0);
            let shade = (1.0 - d * 0.55).clamp(0.0, 1.0);
            let v = (200.0 * shade + 40.0) as u32;
            pixels.push(((alpha * 255.0) as u32) << 24 | (v << 16) | (v << 8) | v);
        }
    }
    Image::from_pixels(size, size, pixels).expect("generator produces the right count")
}

/// Pulsing-glow atlas for the in-game ball.
fn make_ball_atlas(size: u32, frames: u32) -> Atlas {
    let r = size as f32 * 0.5;
    let mut pixels = Vec::new();
    for f in 0..frames {
        let glow = 0.55 + 0.45 * (f as f32 / frames as f32 * std::f32::consts::TAU).sin().abs();
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 + 0.5 - r;
                let dy = y as f32 + 0.5 - r;
                let d = (dx * dx + dy * dy).sqrt() / r;
                let core = ((1.0 - d) * 2.2).clamp(0.0, 1.0);
                let alpha = (core * glow).clamp(0.0, 1.0);
                let shade = (1.0 - d * 0.5).clamp(0.0, 1.0);
                let v = (215.0 * shade + 30.0) as u32;
                pixels.push(((alpha * 255.0) as u32) << 24 | (v << 16) | (v << 8) | v);
            }
        }
    }
    let img = Image::from_pixels(size * frames, size, pixels).expect("generator");
    Atlas::new(img, frames, 1).expect("even grid")
}

// ---------------------------------------------------------------------------
// Menu
// ---------------------------------------------------------------------------

struct MenuScene {
    ui: Ui,
    actions: ActionMap,
    blink: Timer,
    title: Tween,
    ball: Image,
    t: f32,
    best: u32,
    music_vol: f32,
}

impl MenuScene {
    fn new() -> Self {
        Self {
            ui: Ui::new(),
            actions: build_actions(),
            blink: Timer::new(0.55),
            title: Tween::new(0.7, ease::ease_out_back),
            ball: make_ball(BALL_IMG),
            t: 0.0,
            best: 0,
            music_vol: 0.5,
        }
    }

    /// Runs on init AND on resume (returning from PlayScene) so the best
    /// rally and the music volume stay current.
    fn refresh(&mut self, ctx: &mut Ctx) {
        self.best = ctx.storage.load("best_rally").and_then(|s| s.parse().ok()).unwrap_or(0);
        ctx.audio.set_bus_volume(Bus::Music, self.music_vol);
    }
}

impl Scene for MenuScene {
    fn init(&mut self, ctx: &mut Ctx) {
        self.refresh(ctx);
    }

    fn on_resume(&mut self, ctx: &mut Ctx) {
        self.refresh(ctx);
    }

    fn update(&mut self, ctx: &mut Ctx) -> Transition {
        let dt = ctx.time.delta();
        self.blink.advance(dt);
        if self.blink.finished() {
            self.blink.restart();
        }
        self.title.advance(dt);
        self.t += dt;

        // Widgets were drawn (and hit-tested) during the previous render;
        // their events arrive here.
        for ev in self.ui.take_events() {
            match ev {
                UiEvent::Click(id) if id == WidgetId::from_name("play") => {
                    return Transition::Push(Box::new(PlayScene::new()));
                }
                UiEvent::Click(id) if id == WidgetId::from_name("quit") => ctx.quit(),
                UiEvent::Slid(id, v) if id == WidgetId::from_name("music") => {
                    self.music_vol = v;
                    ctx.audio.set_bus_volume(Bus::Music, v);
                }
                _ => {}
            }
        }

        if ctx.input.action_pressed(&self.actions, "confirm") {
            return Transition::Push(Box::new(PlayScene::new()));
        }
        if ctx.input.pressed(KeyCode::Q) {
            ctx.quit();
        }
        Transition::None
    }

    fn render(&mut self, ctx: &mut Ctx) {
        let r = &mut ctx.renderer;
        r.v_gradient(Color::rgb(13, 16, 26), Color::rgb(26, 21, 38));

        // Bobbing ball at the top.
        let bob = (self.t * 2.2).sin() * 10.0;
        r.blit(&self.ball, W as i32 / 2 - BALL_IMG as i32 / 2, (H * 0.10 + bob) as i32);

        // Springy title slide-in (Tween primitive).
        let k = self.title.fraction();
        let title_y = H as i32 / 2 - 92 - ((1.0 - k) * 40.0) as i32;
        r.text_centered(title_y, "MINI PONG", 8, Color::WHITE);

        // Panel + widgets (drawn here; events handled in update).
        let panel = Rect::new(W * 0.5 - 110.0, H * 0.5 - 20.0, 220.0, 136.0);
        self.ui.panel(r, panel);

        if self.best > 0 {
            r.text_centered(panel.y as i32 + 8, &format!("BEST RALLY {}", self.best), 2, Color::YELLOW);
        }
        let play_rect = Rect::new(W * 0.5 - 70.0, panel.y + 30.0, 140.0, 24.0);
        self.ui.button(ctx.input, r, WidgetId::from_name("play"), play_rect, "PLAY");
        let quit_rect = Rect::new(W * 0.5 - 70.0, panel.y + 62.0, 140.0, 24.0);
        self.ui.button(ctx.input, r, WidgetId::from_name("quit"), quit_rect, "QUIT");

        r.text_centered(panel.y as i32 + 92, "MUSIC VOLUME", 2, Color::GRAY);
        let slider_rect = Rect::new(W * 0.5 - 70.0, panel.y + 104.0, 140.0, 10.0);
        self.ui.slider(
            ctx.input,
            r,
            WidgetId::from_name("music"),
            slider_rect,
            self.music_vol,
        );

        r.text_centered(H as i32 - 14, "SPACE ALSO PLAYS  -  Q QUITS", 2, Color::GRAY);
        r.text(8, 8, &format!("{:.0} FPS", ctx.time.fps()), 2, Color::GRAY);
    }
}

// ---------------------------------------------------------------------------
// Play
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Phase {
    /// Ball centered, counting down to the serve.
    Serving { timer: f32, dir: f32 },
    Rally,
    GameOver { player_won: bool },
}

struct PlayScene {
    world: World,
    rng: Rng,
    actions: ActionMap,
    tweens: Tweens,
    ball: Entity,
    player: Entity,
    ai: Entity,
    score: (u32, u32),
    rally: u32,
    ai_error: f32,
    shake: f32,
    trail: Vec<Vec2>,
    phase: Phase,
    music: Option<VoiceHandle>,
}

fn spawn_paddle(world: &mut World, x: f32, color: Color) -> Entity {
    let e = world.spawn();
    world.insert(e, Transform::at(x, H * 0.5));
    world.insert(e, Velocity { v: Vec2::ZERO });
    world.insert(e, Sprite { w: PADDLE_W, h: PADDLE_H, color });
    world.insert(e, Collider { w: PADDLE_W, h: PADDLE_H });
    e
}

impl PlayScene {
    fn new() -> Self {
        let mut world = World::new();
        let player = spawn_paddle(&mut world, 28.0, Color::CYAN);
        let ai = spawn_paddle(&mut world, W - 28.0, Color::ORANGE);

        let ball = world.spawn();
        world.insert(ball, Transform::at(W * 0.5, H * 0.5));
        world.insert(ball, Velocity { v: Vec2::ZERO });
        world.insert(ball, Collider { w: BALL_SIZE, h: BALL_SIZE });
        world.insert(
            ball,
            AnimatedSprite::new(Rc::new(make_ball_atlas(BALL_IMG, BALL_FRAMES)), 8.0, true),
        );

        let mut scene = Self {
            world,
            rng: Rng::from_entropy(),
            actions: build_actions(),
            tweens: Tweens::new(),
            ball,
            player,
            ai,
            score: (0, 0),
            rally: 0,
            ai_error: 0.0,
            shake: 0.0,
            trail: Vec::new(),
            phase: Phase::Rally,
            music: None,
        };
        scene.start_serve(1.0);
        scene
    }

    fn random_dir(&mut self) -> f32 {
        if self.rng.next_f32() < 0.5 {
            -1.0
        } else {
            1.0
        }
    }

    fn start_serve(&mut self, dir: f32) {
        self.trail.clear();
        self.rally = 0;
        self.ai_error = self.rng.range_f32(-30.0, 30.0);
        if let Some(tr) = self.world.get_mut::<Transform>(self.ball) {
            tr.snap(Vec2::new(W * 0.5, H * 0.5)); // snap: no interpolation streak
        }
        if let Some(vel) = self.world.get_mut::<Velocity>(self.ball) {
            vel.v = Vec2::ZERO;
        }
        self.phase = Phase::Serving { timer: SERVE_COUNTDOWN, dir };
    }

    fn serve(&mut self, dir: f32) {
        let d = Vec2::new(dir, self.rng.range_f32(-0.55, 0.55)).normalized();
        if let Some(vel) = self.world.get_mut::<Velocity>(self.ball) {
            vel.v = d * BALL_START_SPEED;
        }
    }

    /// White -> red as the rally speeds up.
    fn ball_color(&self) -> Color {
        let speed = self.world.get::<Velocity>(self.ball).map_or(0.0, |v| v.v.length());
        let t = ((speed - BALL_START_SPEED) / (BALL_MAX_SPEED - BALL_START_SPEED)).clamp(0.0, 1.0);
        Color::WHITE.lerp(Color::RED, t)
    }

    fn spawn_burst(&mut self, at: Vec2, color: Color) {
        for _ in 0..24 {
            let e = self.world.spawn();
            let angle = self.rng.range_f32(0.0, std::f32::consts::TAU);
            let speed = self.rng.range_f32(40.0, 240.0);
            let size = self.rng.range_f32(2.0, 5.0);
            self.world.insert(e, Transform::new(at));
            self.world.insert(e, Velocity { v: Vec2::from_angle(angle) * speed });
            self.world.insert(e, Sprite { w: size, h: size, color });
            self.world.insert(e, Lifetime { remaining: self.rng.range_f32(0.3, 0.9) });
        }
    }

    /// Reflect the ball off `paddle` when it overlaps AND is moving into it.
    fn bounce_off_paddle(&mut self, paddle: Entity, facing_right: bool) -> bool {
        let (Some(ball_tr), Some(ball_vel), Some(paddle_tr)) = (
            self.world.get::<Transform>(self.ball).copied(),
            self.world.get::<Velocity>(self.ball).copied(),
            self.world.get::<Transform>(paddle).copied(),
        ) else {
            return false;
        };
        let (bw, bh) = self
            .world
            .get::<Collider>(self.ball)
            .map_or((BALL_SIZE, BALL_SIZE), |c| (c.w, c.h));
        let (pw, ph) = self
            .world
            .get::<Collider>(paddle)
            .map_or((PADDLE_W, PADDLE_H), |c| (c.w, c.h));

        let ball_rect = Rect::from_center(ball_tr.pos, bw, bh);
        let paddle_rect = Rect::from_center(paddle_tr.pos, pw, ph);
        if !ball_rect.intersects(paddle_rect) {
            return false;
        }
        let moving_in = if facing_right { ball_vel.v.x < 0.0 } else { ball_vel.v.x > 0.0 };
        if !moving_in {
            return false;
        }

        // Hit point decides the outgoing angle: -1 (top edge) .. 1 (bottom).
        let offset = ((ball_tr.pos.y - paddle_rect.center().y) / (ph * 0.5)).clamp(-0.85, 0.85);
        let speed = (ball_vel.v.length() * 1.06).clamp(BALL_START_SPEED, BALL_MAX_SPEED);
        let dir = Vec2::new(if facing_right { 1.0 } else { -1.0 }, offset).normalized();

        if let (Some(tr), Some(vel)) = (
            self.world.get_mut::<Transform>(self.ball),
            self.world.get_mut::<Velocity>(self.ball),
        ) {
            // Push the ball clear of the paddle so it can't get stuck inside.
            tr.pos.x = if facing_right {
                paddle_rect.max().x + bw * 0.5 + 1.0
            } else {
                paddle_rect.min().x - bw * 0.5 - 1.0
            };
            vel.v = dir * speed;
        }
        true
    }

    /// Keyboard wins if pressed; otherwise follow the mouse cursor.
    fn control_player(&mut self, ctx: &Ctx) {
        let key_dir = ctx.input.action_axis(&self.actions, "up", "down");
        let paddle_y = self.world.get::<Transform>(self.player).map_or(H * 0.5, |t| t.pos.y);
        let target = if key_dir != 0.0 {
            key_dir.clamp(-1.0, 1.0) * PADDLE_SPEED
        } else if let Some(mouse) = ctx.input.mouse_pos() {
            ((mouse.y - paddle_y) * 10.0).clamp(-PADDLE_SPEED, PADDLE_SPEED)
        } else {
            0.0
        };
        if let Some(vel) = self.world.get_mut::<Velocity>(self.player) {
            vel.v.y = target;
        }
    }

    fn control_ai(&mut self) {
        let (ball_pos, ball_vel) = match (
            self.world.get::<Transform>(self.ball),
            self.world.get::<Velocity>(self.ball),
        ) {
            (Some(t), Some(v)) => (t.pos, v.v),
            _ => (Vec2::new(W * 0.5, H * 0.5), Vec2::ZERO),
        };
        let target_y = if ball_vel.x > 0.0 { ball_pos.y + self.ai_error } else { H * 0.5 };
        let speed = AI_BASE_SPEED + (self.rally as f32 * 6.0).min(160.0);
        if let (Some(tr), Some(vel)) = (
            self.world.get_mut::<Transform>(self.ai),
            self.world.get_mut::<Velocity>(self.ai),
        ) {
            let dy = target_y - tr.pos.y;
            vel.v.y = if dy.abs() > 10.0 { dy.signum() * speed } else { 0.0 };
        }
    }

    /// End-of-match bookkeeping: persist the best rally, park the paddles,
    /// and glide the AI home with a tween.
    fn finish_match(&mut self, ctx: &mut Ctx, player_won: bool) {
        let best =
            ctx.storage.load("best_rally").and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        if self.rally > best {
            let _ = ctx.storage.save("best_rally", &self.rally.to_string());
        }

        self.trail.clear();
        for p in [self.player, self.ai] {
            if let Some(v) = self.world.get_mut::<Velocity>(p) {
                v.v = Vec2::ZERO;
            }
        }
        if let Some(vel) = self.world.get_mut::<Velocity>(self.ball) {
            vel.v = Vec2::ZERO;
        }

        let ai = self.ai;
        let start_y = self.world.get::<Transform>(ai).map_or(H * 0.5, |t| t.pos.y);
        self.tweens.add(0.9, ease::ease_in_out_cubic, move |w, t| {
            if let Some(tr) = w.get_mut::<Transform>(ai) {
                tr.pos.y = start_y.lerp(H * 0.5, t);
                tr.prev.y = tr.pos.y;
            }
        });

        self.phase = Phase::GameOver { player_won };
        ctx.audio.play(SND_WIN);
    }
}

impl Scene for PlayScene {
    fn init(&mut self, ctx: &mut Ctx) {
        // Slow looping bassline; stopped when the scene exits.
        self.music = Some(ctx.audio.play(MUSIC));
    }

    fn on_exit(&mut self) {
        if let Some(m) = self.music.take() {
            m.stop();
        }
    }

    fn update(&mut self, ctx: &mut Ctx) -> Transition {
        let dt = ctx.time.delta();

        if ctx.input.pressed(KeyCode::Q) {
            ctx.quit();
            return Transition::None;
        }

        if !matches!(self.phase, Phase::GameOver { .. }) {
            self.control_player(ctx);
            self.control_ai();
        }

        // World-bound tweens (fire-and-forget animations).
        self.tweens.update(dt, &mut self.world);

        // Atlas animation + speed-based tint for the ball.
        animate_sprites(&mut self.world, dt);
        let tint = self.ball_color();
        if let Some(si) = self.world.get_mut::<SpriteImage>(self.ball) {
            si.tint = Some(tint);
        }

        match self.phase {
            Phase::Serving { timer, dir } => {
                let timer = timer - dt;
                if timer <= 0.0 {
                    self.serve(dir);
                    self.phase = Phase::Rally;
                } else {
                    self.phase = Phase::Serving { timer, dir };
                }
            }
            Phase::Rally => {}
            Phase::GameOver { .. } => {
                if ctx.input.action_pressed(&self.actions, "confirm") {
                    return Transition::Pop; // back to the menu
                }
            }
        }

        // -- Movement (records prev for interpolation) -------------------------
        self.world
            .each2_mut(|_, tr: &mut Transform, vel: &mut Velocity| tr.advance(tr.pos + vel.v * dt));

        // -- Keep paddles on screen ---------------------------------------------
        for paddle in [self.player, self.ai] {
            if let Some(tr) = self.world.get_mut::<Transform>(paddle) {
                tr.pos.y = tr.pos.y.clamp(PADDLE_H * 0.5, H - PADDLE_H * 0.5);
            }
        }

        // -- Rally systems ---------------------------------------------------------
        if matches!(self.phase, Phase::Rally) {
            if let Some(tr) = self.world.get::<Transform>(self.ball) {
                self.trail.push(tr.pos);
                if self.trail.len() > TRAIL_LEN {
                    self.trail.remove(0);
                }
            }

            let half =
                self.world.get::<Collider>(self.ball).map_or(BALL_SIZE * 0.5, |c| c.w * 0.5);
            if let (Some(tr), Some(vel)) = (
                self.world.get_mut::<Transform>(self.ball),
                self.world.get_mut::<Velocity>(self.ball),
            ) {
                if tr.pos.y < half {
                    tr.pos.y = half;
                    vel.v.y = vel.v.y.abs();
                    ctx.audio.play(SND_WALL);
                    self.shake = 2.0;
                } else if tr.pos.y > H - half {
                    tr.pos.y = H - half;
                    vel.v.y = -vel.v.y.abs();
                    ctx.audio.play(SND_WALL);
                    self.shake = 2.0;
                }
            }

            for (paddle, facing_right, color) in
                [(self.player, true, Color::CYAN), (self.ai, false, Color::ORANGE)]
            {
                if self.bounce_off_paddle(paddle, facing_right) {
                    self.rally += 1;
                    self.ai_error = self
                        .rng
                        .range_f32(-30.0, 30.0)
                        * (1.0 - (self.rally as f32 * 0.03).min(0.6));
                    ctx.audio.play(Sound {
                        freq_hz: 480.0 + self.rally.min(40) as f32 * 10.0,
                        ..SND_PADDLE
                    });
                    self.shake = 3.5;
                    if let Some(at) = self.world.get::<Transform>(self.ball).map(|t| t.pos) {
                        self.spawn_burst(at, color);
                    }
                }
            }

            // Scoring.
            let ball_x = self.world.get::<Transform>(self.ball).map_or(W * 0.5, |t| t.pos.x);
            let ball_y = self.world.get::<Transform>(self.ball).map_or(H * 0.5, |t| t.pos.y);
            let ai_scored = ball_x < -40.0;
            let player_scored = ball_x > W + 40.0;
            if ai_scored || player_scored {
                if player_scored {
                    self.score.0 += 1;
                } else {
                    self.score.1 += 1;
                }
                let x = if ai_scored { 8.0 } else { W - 8.0 };
                let color = if player_scored { Color::CYAN } else { Color::ORANGE };
                self.spawn_burst(Vec2::new(x, ball_y.clamp(8.0, H - 8.0)), color);
                self.shake = 10.0;
                ctx.audio.play(SND_SCORE);

                if self.score.0 >= WIN_SCORE || self.score.1 >= WIN_SCORE {
                    self.finish_match(ctx, self.score.0 >= WIN_SCORE);
                } else {
                    self.start_serve(self.random_dir());
                }
            }
        }

        // -- Lifetime system: expire particles ----------------------------------------
        for (_, life) in self.world.query_mut::<Lifetime>() {
            life.remaining -= dt;
        }
        let expired: Vec<Entity> = self
            .world
            .query::<Lifetime>()
            .filter(|(_, life)| life.remaining <= 0.0)
            .map(|(e, _)| e)
            .collect();
        for e in expired {
            self.world.despawn(e);
        }

        self.shake = (self.shake - 60.0 * dt).max(0.0);
        Transition::None
    }

    fn render(&mut self, ctx: &mut Ctx) {
        let alpha = ctx.time.alpha();
        let r = &mut ctx.renderer;
        r.v_gradient(Color::rgb(13, 16, 26), Color::rgb(26, 21, 38));

        if self.shake > 0.05 {
            r.set_camera(Vec2::new(
                self.rng.range_f32(-1.0, 1.0) * self.shake,
                self.rng.range_f32(-1.0, 1.0) * self.shake,
            ));
        }

        let mut y = 8.0;
        while y < H {
            r.rect(Rect::new(W * 0.5 - 2.0, y, 4.0, 12.0), Color::DARK_GRAY);
            y += 24.0;
        }

        let tint = self.ball_color();
        for (i, p) in self.trail.iter().enumerate() {
            let t = (i + 1) as f32 / self.trail.len().max(1) as f32;
            let c = Color::rgb(44, 52, 82).lerp(tint, t * 0.7);
            r.rect(Rect::from_center(*p, BALL_SIZE * 0.55, BALL_SIZE * 0.55), c);
        }

        // Image sprites (the animated ball), interpolated.
        draw_sprites(&self.world, r, alpha);

        // Rect sprites (paddles + particles), interpolated.
        for (_, spr, tr) in self.world.query2::<Sprite, Transform>() {
            r.rect(Rect::from_center(tr.interp(alpha), spr.w, spr.h), spr.color);
        }
        r.set_camera(Vec2::ZERO);

        let score = format!("{}   {}", self.score.0, self.score.1);
        r.text_centered(14, &score, 6, Color::WHITE);

        match self.phase {
            Phase::Serving { timer, .. } => {
                let n = timer.ceil().max(1.0) as i32;
                r.text_centered(H as i32 / 2 - 20, &format!("{n}"), 8, Color::GRAY);
                r.text_centered(H as i32 - 40, "FIRST TO 5", 2, Color::GRAY);
            }
            Phase::Rally => {
                if self.score.0 == WIN_SCORE - 1 || self.score.1 == WIN_SCORE - 1 {
                    r.text_centered(H as i32 - 40, "MATCH POINT", 2, Color::YELLOW);
                }
            }
            Phase::GameOver { player_won } => {
                let msg = if player_won { "YOU WIN" } else { "AI WINS" };
                let c = if player_won { Color::CYAN } else { Color::ORANGE };
                r.text_centered(H as i32 / 2 - 30, msg, 8, c);
                r.text_centered(H as i32 / 2 + 10, "SPACE - MENU      Q - QUIT", 2, Color::GRAY);
                r.text_centered(H as i32 / 2 + 30, &format!("RALLY {}", self.rally), 2, Color::YELLOW);
            }
        }

        r.text(8, 8, &format!("{:.0} FPS", ctx.time.fps()), 2, Color::GRAY);
        r.text_right(
            W as i32 - 8,
            8,
            &format!("ENTITIES {}", self.world.entity_count()),
            2,
            Color::GRAY,
        );
    }
}

fn main() -> Result<()> {
    let config = EngineConfig::new().title("MINI PONG").size(640, 360);
    let mut stack = SceneStack::with_seed(Box::new(MenuScene::new()));
    run(config, &mut stack)
}
