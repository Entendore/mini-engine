//! Scenes — the SceneStack pattern for title/game/pause flows.
//!
//! `SceneStack` implements `Game`, so you pass it to `run` directly. Only
//! the TOP scene updates and renders — pausing the simulation below is free.
//! Scenes steer the flow with `Transition`: Push / Pop / Replace / Quit.
//!
//! Shared data lives in an `Rc<RefCell<..>>` every scene holds a clone of.
//! Engine-level ESC quitting is disabled; the scenes own that key now.

use std::cell::RefCell;
use std::rc::Rc;

use mini_engine::prelude::*;

const W: f32 = 480.0;
const H: f32 = 270.0;

type Shared = Rc<RefCell<u32>>; // a score shared across scenes

struct TitleScene {
    shared: Shared,
}

impl Scene for TitleScene {
    fn update(&mut self, ctx: &mut Ctx) -> Transition {
        if ctx.input.pressed(KeyCode::Space) {
            *self.shared.borrow_mut() = 0;
            return Transition::Push(Box::new(GameScene {
                shared: self.shared.clone(),
                pos: Vec2::new(W / 2.0, H / 2.0),
                tick: Timer::new(1.0),
            }));
        }
        if ctx.input.pressed(KeyCode::Q) {
            return Transition::Quit; // ends the session, runs every on_exit
        }
        Transition::None
    }

    fn render(&mut self, ctx: &mut Ctx) {
        let r = &mut ctx.renderer;
        r.clear(Color::rgb(18, 20, 32));
        r.text_centered(90, "SCENE DEMO", 5, Color::WHITE);
        r.text_centered(140, "SPACE - PLAY      Q - QUIT", 2, Color::GRAY);
    }
}

struct GameScene {
    shared: Shared,
    pos: Vec2,
    tick: Timer,
}

impl Scene for GameScene {
    fn update(&mut self, ctx: &mut Ctx) -> Transition {
        // ESC pops back to the title (engine ESC-quit is disabled in config).
        if ctx.input.pressed(KeyCode::Escape) {
            return Transition::Pop;
        }
        // P pushes a pause scene ON TOP; this scene stops updating entirely.
        if ctx.input.pressed(KeyCode::P) {
            return Transition::Push(Box::new(PauseScene { shared: self.shared.clone() }));
        }

        let dt = ctx.time.delta();
        let move = Vec2::new(
            ctx.input.axis(KeyCode::Left, KeyCode::Right),
            ctx.input.axis(KeyCode::Up, KeyCode::Down),
        );
        self.pos += move * 180.0 * dt;

        self.tick.advance(dt);
        if self.tick.finished() {
            self.tick.restart();
            *self.shared.borrow_mut() += 1; // one point per second
        }
        Transition::None
    }

    fn render(&mut self, ctx: &mut Ctx) {
        let r = &mut ctx.renderer;
        r.clear(Color::rgb(16, 18, 30));
        r.text_centered(16, &format!("SCORE {}", *self.shared.borrow()), 3, Color::WHITE);
        r.rect(Rect::from_center(self.pos, 20.0, 20.0), Color::GREEN);
        r.text_centered(H as i32 - 20, "P - PAUSE    ESC - TITLE", 2, Color::GRAY);
    }
}

struct PauseScene {
    shared: Shared,
}

impl Scene for PauseScene {
    fn update(&mut self, ctx: &mut Ctx) -> Transition {
        // While this scene is on top, GameScene neither updates nor renders.
        if ctx.input.pressed(KeyCode::P) || ctx.input.pressed(KeyCode::Escape) {
            return Transition::Pop; // resume the game
        }
        Transition::None
    }

    fn render(&mut self, ctx: &mut Ctx) {
        // Only the top scene draws — repaint a "frozen game" look here.
        let r = &mut ctx.renderer;
        r.clear(Color::rgb(16, 18, 30));
        r.text_centered(110, "PAUSED", 6, Color::YELLOW);
        r.text_centered(160, &format!("SCORE SO FAR {}", *self.shared.borrow()), 2, Color::GRAY);
        r.text_centered(190, "P - RESUME", 2, Color::GRAY);
    }

    // Lifecycle hooks — handy for muting music, saving, etc.
    fn on_pause(&mut self, _ctx: &mut Ctx) {
        println!("[game] paused");
    }
    fn on_resume(&mut self, _ctx: &mut Ctx) {
        println!("[game] resumed");
    }
    fn on_exit(&mut self) {
        println!("[game] exited");
    }
}

fn main() -> Result<()> {
    let config = EngineConfig::new().title("07 - scenes").size(480, 270).quit_on_escape(false);

    let shared: Shared = Rc::new(RefCell::new(0));
    let mut stack = SceneStack::with_seed(Box::new(TitleScene { shared }));
    run(config, &mut stack)
}
