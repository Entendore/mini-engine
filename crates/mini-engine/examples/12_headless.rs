//! Headless simulation — NO WINDOW. Runs anywhere, including CI.
//!
//! The core loop is backend-driven: feed it a `ScriptedBackend` and you can
//! test whole gameplay sessions with plain asserts. With a seeded `Rng` and
//! identical `FrameInput`s, runs are bit-identical — that's what makes
//! `RecordingBackend` -> replay a real testing tool.

use mini_engine::prelude::*;

/// A tiny "game": paddle physics plus optional seeded jitter.
struct AutoPilot {
    x: f32,
    steps: u32,
    rng: Rng,
    jitter: bool,
}

impl Game for AutoPilot {
    fn update(&mut self, ctx: &mut Ctx) {
        self.steps += 1;
        self.x += ctx.input.axis(KeyCode::Left, KeyCode::Right) * 100.0 * ctx.time.delta();
        if self.jitter && self.rng.next_f32() < 0.1 {
            self.x += self.rng.range_f32(-5.0, 5.0); // chaos, but SEEDED chaos
        }
    }
    fn render(&mut self, _ctx: &mut Ctx) {}
}

fn main() -> Result<()> {
    // ---- Part 1: scripted input, asserted outcomes ------------------------
    let mut script = ScriptedBackend::new();
    script.repeat(
        60,
        FrameInput { dt: 1.0 / 60.0, keys: vec![KeyCode::Right], ..Default::default() },
    );
    script.repeat(
        60,
        FrameInput { dt: 1.0 / 60.0, keys: vec![KeyCode::Left], ..Default::default() },
    );

    let mut engine = Engine::new(EngineConfig::new())?;
    let mut game = AutoPilot { x: 0.0, steps: 0, rng: Rng::new(1), jitter: false };
    engine.run(&mut script, &mut game)?;

    assert_eq!(game.steps, 120, "120 frames at 1/60s = 120 fixed steps");
    assert!(game.x.abs() < 0.01, "1s right + 1s left returns to start (x = {})", game.x);
    println!("part 1 ok: 120 steps, x = {:.4}", game.x);

    // ---- Part 2: record a run, replay it, demand identical results --------
    let mut source = ScriptedBackend::new();
    source.repeat(90, FrameInput { dt: 1.0 / 60.0, ..Default::default() });

    let mut recorder = RecordingBackend::new(source);
    let mut first = AutoPilot { x: 0.0, steps: 0, rng: Rng::new(42), jitter: true };
    Engine::new(EngineConfig::new())?.run(&mut recorder, &mut first)?;
    let frames = recorder.into_frames();

    let mut replay = ScriptedBackend::new();
    for frame in frames {
        replay.push(frame);
    }
    let mut second = AutoPilot { x: 0.0, steps: 0, rng: Rng::new(42), jitter: true };
    Engine::new(EngineConfig::new())?.run(&mut replay, &mut second)?;

    assert_eq!(first.x, second.x, "replay must be bit-identical");
    assert_eq!(first.steps, second.steps);
    println!("part 2 ok: recorded 90 frames, replay matched exactly (x = {:.4})", second.x);
    Ok(())
}
