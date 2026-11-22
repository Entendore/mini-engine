//! The game contract and the loop that enforces it. OS-free: the loop is
//! driven by any [`WindowBackend`](crate::backend::WindowBackend), which
//! makes it simulatable (and testable) anywhere.

use crate::audio::Audio;
use crate::backend::WindowBackend;
use crate::config::EngineConfig;
use crate::input::{Input, KeyCode, MouseButton};
use crate::render::Renderer;
use crate::storage::{MemoryStorage, Storage};
use crate::time::Time;

/// Never simulate more than this per frame (host stalled, tab backgrounded...).
const MAX_FRAME_DELTA: f32 = 0.25;
/// Cap catch-up steps per frame to avoid the "spiral of death".
const MAX_CATCH_UP_STEPS: u32 = 5;

/// Services the engine hands to the game on every callback.
///
/// Games only *read* this struct — the engine constructs it — so new fields
/// can be added in minor releases without breaking games.
pub struct Ctx<'a> {
    /// Fixed-rate timing (see [`Time::delta`], [`Time::alpha`]).
    pub time: &'a Time,
    /// Input edges for this frame.
    pub input: &'a Input,
    /// Draw here. Primitives respect the camera; reset it for HUDs.
    pub renderer: &'a mut Renderer,
    /// Fire-and-forget synth audio.
    pub audio: &'a Audio,
    /// Persistent key/value storage (memory by default; the host may install
    /// a file-backed one). Games encode their own formats.
    pub storage: &'a mut dyn Storage,
    quit: &'a mut bool,
}

impl Ctx<'_> {
    /// Ask the engine to exit after this frame (win screens, Q-to-quit, ...).
    pub fn quit(&mut self) {
        *self.quit = true;
    }
}

/// Implement this to make a game. The engine calls:
/// 1. `init` once, 2. `update` at a fixed rate, 3. `render` once per frame.
///
/// # API stability
/// New methods will always ship with default implementations, so your impl
/// never breaks on upgrade.
pub trait Game {
    /// Called once before the loop starts.
    fn init(&mut self, _ctx: &mut Ctx) {}
    /// Fixed-rate simulation step (keeps gameplay deterministic).
    fn update(&mut self, ctx: &mut Ctx);
    /// Draw the current state once per frame.
    fn render(&mut self, ctx: &mut Ctx);
    /// Called once after the loop ends.
    fn on_exit(&mut self) {}
}

/// Owns all subsystems and runs the main loop against any backend.
pub struct Engine {
    config: EngineConfig,
    renderer: Renderer,
    input: Input,
    time: Time,
    audio: Audio,
    storage: Box<dyn Storage>,
}

impl Engine {
    /// Validate `config` and build the engine. No OS resources are touched —
    /// that is the host's job.
    pub fn new(config: EngineConfig) -> crate::Result<Self> {
        if config.width() == 0 || config.height() == 0 {
            return Err(crate::EngineError::InvalidConfig(
                "width and height must be non-zero".into(),
            ));
        }
        if config.update_hz() <= 0.0 || config.target_fps() == 0 {
            return Err(crate::EngineError::InvalidConfig(
                "update_hz and target_fps must be positive".into(),
            ));
        }
        Ok(Self {
            renderer: Renderer::new(config.width(), config.height()),
            config,
            input: Input::default(),
            time: Time::new(),
            audio: Audio::new(),
            storage: Box::new(MemoryStorage::new()),
        })
    }

    /// The validated startup configuration.
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// The shared audio mixer handle (an output backend may pull from it).
    pub fn audio(&self) -> &Audio {
        &self.audio
    }

    /// Install a persistence backend (the `mini-engine` facade does this with
    /// its `FileStorage`). Call before [`Engine::run`].
    pub fn set_storage(&mut self, storage: Box<dyn Storage>) {
        self.storage = storage;
    }

    /// Read-only view of the current storage (tests, introspection).
    pub fn storage(&self) -> &dyn Storage {
        self.storage.as_ref()
    }

    /// Run `game` against `backend` until the backend ends the session
    /// (`poll` → `None` or `close_requested`), Escape is pressed (if
    /// enabled), or the game calls [`Ctx::quit`].
    pub fn run<G: Game>(
        &mut self,
        backend: &mut dyn WindowBackend,
        game: &mut G,
    ) -> crate::Result<()> {
        let step = 1.0 / self.config.update_hz().max(1.0);
        let mut accumulator = 0.0_f32;
        let mut quit = false;

        {
            let mut ctx = self.ctx(&mut quit);
            game.init(&mut ctx);
        }

        while let Some(frame) = backend.poll() {
            if frame.close_requested {
                break;
            }

            // 1. Input: edges (pressed/released) derived by diffing frames.
            self.input.begin_frame(
                frame.keys,
                frame.mouse_pos,
                mouse_buttons(frame.mouse_buttons),
                frame.wheel,
                frame.chars,
            );
            if self.config.quit_on_escape() && self.input.pressed(KeyCode::Escape) {
                quit = true;
            }

            // 2. Timing: clamp host dt defensively, accumulate leftovers.
            let frame_delta = frame.dt.clamp(0.0, MAX_FRAME_DELTA);
            self.time.begin_frame(frame_delta);

            // 3. Fixed-rate simulation (gameplay is stable at any fps).
            let paused = self.config.pause_when_unfocused() && !frame.focused;
            if !paused {
                accumulator += frame_delta;
                let mut steps = 0;
                while accumulator >= step && steps < MAX_CATCH_UP_STEPS {
                    accumulator -= step;
                    steps += 1;
                    self.time.begin_step(step);
                    let mut ctx = self.ctx(&mut quit);
                    game.update(&mut ctx);
                    if quit {
                        break;
                    }
                }
                if accumulator > step {
                    accumulator = 0.0; // fell too far behind: drop the backlog
                }
            }

            // 4. Interpolation fraction for this frame's render (0..=1).
            let alpha = if step > 0.0 { (accumulator / step).clamp(0.0, 1.0) } else { 0.0 };
            self.time.set_alpha(alpha);

            // 5. Render + present.
            {
                let mut ctx = self.ctx(&mut quit);
                game.render(&mut ctx);
            }
            backend.present(
                self.renderer.pixels(),
                self.renderer.width(),
                self.renderer.height(),
            )?;

            if quit {
                break;
            }
        }

        game.on_exit();
        Ok(())
    }

    fn ctx<'a>(&'a mut self, quit: &'a mut bool) -> Ctx<'a> {
        Ctx {
            time: &self.time,
            input: &self.input,
            renderer: &mut self.renderer,
            audio: &self.audio,
            storage: &mut *self.storage,
            quit,
        }
    }
}

fn mouse_buttons(bits: [bool; 3]) -> impl Iterator<Item = MouseButton> {
    [MouseButton::Left, MouseButton::Middle, MouseButton::Right]
        .into_iter()
        .zip(bits)
        .filter_map(|(button, down)| down.then_some(button))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{FrameInput, ScriptedBackend};

    #[derive(Default)]
    struct Probe {
        updates: u32,
        renders: u32,
        spaces: u32,
        chars: usize,
        last_alpha: f32,
    }

    impl Game for Probe {
        fn update(&mut self, ctx: &mut Ctx) {
            self.updates += 1;
            if ctx.input.pressed(KeyCode::Space) {
                self.spaces += 1;
            }
            self.chars += ctx.input.typed().len();
            if ctx.input.pressed(KeyCode::Q) {
                ctx.quit();
            }
        }
        fn render(&mut self, ctx: &mut Ctx) {
            self.renders += 1;
            self.last_alpha = ctx.time.alpha();
        }
    }

    fn script(n: usize, dt: f32, mut edit: impl FnMut(usize, &mut FrameInput)) -> ScriptedBackend {
        let mut backend = ScriptedBackend::new();
        for i in 0..n {
            let mut frame = FrameInput { dt, ..FrameInput::default() };
            edit(i, &mut frame);
            backend.push(frame);
        }
        backend
    }

    #[test]
    fn fixed_steps_and_input_edges_headless() {
        let backend = script(120, 1.0 / 60.0, |i, f| {
            if i < 2 {
                f.keys.push(KeyCode::Space);
            }
            if i == 5 {
                f.chars = vec!['h', 'i'];
            }
        });
        let mut engine = Engine::new(EngineConfig::new().update_hz(60.0)).unwrap();
        let mut probe = Probe::default();
        engine.run(&mut backend, &mut probe).unwrap();

        assert_eq!(probe.updates, 120, "exactly one step per 1/60s frame");
        assert_eq!(probe.spaces, 2, "pressed fires only on the two edge frames");
        assert_eq!(probe.chars, 2);
        assert_eq!(probe.renders, 120);
    }

    #[test]
    fn render_alpha_is_the_leftover_fraction() {
        // 50 Hz frames vs 60 Hz steps: after each step a fraction remains.
        let backend = script(60, 1.0 / 50.0, |_, _| {});
        let mut engine = Engine::new(EngineConfig::new().update_hz(60.0)).unwrap();
        let mut probe = Probe::default();
        engine.run(&mut backend, &mut probe).unwrap();
        assert!(probe.last_alpha > 0.0 && probe.last_alpha < 1.0);
    }

    #[test]
    fn quit_flag_ends_the_session() {
        let backend = script(600, 1.0 / 60.0, |i, f| {
            if i == 10 {
                f.keys.push(KeyCode::Q);
            }
        });
        let mut engine = Engine::new(EngineConfig::new()).unwrap();
        let mut probe = Probe::default();
        engine.run(&mut backend, &mut probe).unwrap();

        assert!(probe.updates < 600, "loop stopped early via Ctx::quit");
        assert!(probe.renders <= probe.updates + 1);
    }

    #[test]
    fn invalid_configs_are_rejected() {
        assert!(Engine::new(EngineConfig::new().size(0, 100)).is_err());
        assert!(Engine::new(EngineConfig::new().update_hz(0.0)).is_err());
        assert!(Engine::new(EngineConfig::new().target_fps(0)).is_err());
    }

    #[derive(Default)]
    struct Saver {
        count: u32,
    }

    impl Game for Saver {
        fn update(&mut self, ctx: &mut Ctx) {
            self.count += 1;
            let _ = ctx.storage.save("count", &self.count.to_string());
        }
        fn render(&mut self, _ctx: &mut Ctx) {}
    }

    #[test]
    fn storage_round_trips_through_ctx() {
        let backend = script(5, 1.0 / 60.0, |_, _| {});
        let mut engine = Engine::new(EngineConfig::new()).unwrap();
        let mut game = Saver::default();
        engine.run(&mut backend, &mut game).unwrap();
        assert_eq!(engine.storage().load("count").as_deref(), Some("5"));
    }
}
