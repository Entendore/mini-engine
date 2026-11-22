//! Input — the full surface:
//! * keys: `held` (every frame) vs `pressed`/`released` (edge frames only)
//! * `axis` for paired keys
//! * mouse: position, per-frame delta, buttons, scroll wheel
//! * text: `typed()` chars + `KeyCode::Backspace`
//!
//! Move with WASD/arrows, drag with LMB, scroll to recolor, type below.

use mini_engine::prelude::*;

struct InputDemo {
    pos: Vec2,
    hue: f32,
    text: String,
    caret: Timer,
}

impl Game for InputDemo {
    fn update(&mut self, ctx: &mut Ctx) {
        let dt = ctx.time.delta();

        // -- keyboard -------------------------------------------------------
        // `held` is true every frame the key is down — right for movement.
        let mut dir = Vec2::ZERO;
        if ctx.input.held(KeyCode::W) || ctx.input.held(KeyCode::Up) {
            dir.y -= 1.0;
        }
        if ctx.input.held(KeyCode::S) || ctx.input.held(KeyCode::Down) {
            dir.y += 1.0;
        }
        if ctx.input.held(KeyCode::A) || ctx.input.held(KeyCode::Left) {
            dir.x -= 1.0;
        }
        if ctx.input.held(KeyCode::D) || ctx.input.held(KeyCode::Right) {
            dir.x += 1.0;
        }
        self.pos += dir.normalized() * 160.0 * dt;

        // `pressed` fires on exactly one frame — right for jumps and menus.
        if ctx.input.pressed(KeyCode::Enter) {
            println!("Enter pressed (fires once per press)");
        }

        // -- mouse ----------------------------------------------------------
        if ctx.input.mouse_down(MouseButton::Left) {
            if let Some(p) = ctx.input.mouse_pos() {
                self.pos = p; // drag the square
            }
        }
        // Wheel: positive `y` = scroll up. mouse_delta() is per-frame motion.
        self.hue = (self.hue + ctx.input.wheel().y * 0.08).rem_euclid(1.0);

        // -- text entry ------------------------------------------------------
        for ch in ctx.input.typed() {
            if self.text.chars().count() < 24 {
                self.text.push(*ch);
            }
        }
        if ctx.input.pressed(KeyCode::Backspace) {
            self.text.pop();
        }

        self.caret.advance(dt);
        if self.caret.finished() {
            self.caret.restart();
        }
    }

    fn render(&mut self, ctx: &mut Ctx) {
        let r = &mut ctx.renderer;
        r.v_gradient(Color::rgb(16, 18, 28), Color::rgb(30, 26, 44));

        if let Some(p) = ctx.input.mouse_pos() {
            let d = ctx.input.mouse_delta();
            r.text(
                8,
                8,
                &format!("MOUSE {:4.0},{:4.0}   DELTA {:5.1},{:5.1}", p.x, p.y, d.x, d.y),
                2,
                Color::GRAY,
            );
        }

        let color = Color::CYAN.lerp(Color::MAGENTA, self.hue);
        r.rect(Rect::from_center(self.pos, 24.0, 24.0), color);
        r.rect_outline(Rect::from_center(self.pos, 28.0, 28.0), Color::WHITE);

        // Text box with blinking caret.
        let box_r = Rect::new(90.0, 190.0, 300.0, 30.0);
        r.rect(box_r, Color::rgb(20, 24, 38));
        r.rect_outline(box_r, Color::rgb(70, 78, 100));
        r.text(box_r.x as i32 + 8, box_r.y as i32 + 12, &self.text, 2, Color::WHITE);
        if self.caret.fraction() < 0.5 {
            let x = box_r.x as i32 + 8 + r.text_width(&self.text, 2);
            r.text(x, box_r.y as i32 + 12, "_", 2, Color::GRAY);
        }
        r.text(8, 250, "WASD MOVE  -  LMB DRAG  -  WHEEL COLOR  -  TYPE BELOW", 2, Color::GRAY);
    }
}

fn main() -> Result<()> {
    let mut game =
        InputDemo { pos: Vec2::new(240.0, 135.0), hue: 0.0, text: String::new(), caret: Timer::new(0.5) };
    run(EngineConfig::new().title("02 - input").size(480, 270), &mut game)
}
