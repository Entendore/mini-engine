//! The smallest complete mini-engine program.
//!
//! Every game implements `Game` and hands itself to `run`. The engine:
//!   1. calls `init` once,
//!   2. calls `update` at a fixed rate (`update_hz`, default 60),
//!   3. calls `render` once per frame,
//!   4. closes on Escape (default).

use mini_engine::prelude::*;

struct Hello;

impl Game for Hello {
    fn update(&mut self, _ctx: &mut Ctx) {
        // Simulation goes here. Nothing to simulate yet.
    }

    fn render(&mut self, ctx: &mut Ctx) {
        let r = &mut ctx.renderer;
        r.clear(Color::BLACK);
        r.text_centered(90, "HELLO, MINI ENGINE", 4, Color::WHITE);
        r.text_centered(140, &format!("{:.0} FPS", ctx.time.fps()), 2, Color::GRAY);
        r.text_centered(170, "PRESS ESC TO QUIT", 2, Color::GRAY);
    }
}

fn main() -> Result<()> {
    let config = EngineConfig::new().title("01 - hello window").size(480, 270);
    run(config, &mut Hello)
}
