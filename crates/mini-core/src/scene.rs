//! Optional scene management. [`SceneStack`] implements [`Game`], so adopting
//! it is one line in `main`: `run(config, &mut SceneStack::with_seed(...))`.
//!
//! Only the top scene updates and renders; scenes below stay parked (use
//! `on_pause`/`on_resume` to mute music etc.). A `Pop` that empties the stack
//! quits the session.

use crate::engine::{Ctx, Game};

/// One screen / game state.
///
/// # API stability
/// New methods always ship with default implementations, so your impl never
/// breaks on upgrade.
pub trait Scene {
    /// Called when the scene becomes the top of the stack.
    fn init(&mut self, _ctx: &mut Ctx) {}
    /// Fixed-rate simulation step; return the desired [`Transition`].
    fn update(&mut self, ctx: &mut Ctx) -> Transition;
    /// Draw the current state (only the top scene renders).
    fn render(&mut self, ctx: &mut Ctx);
    /// Called when the scene leaves the stack for good.
    fn on_exit(&mut self) {}
    /// Another scene was pushed on top of this one.
    fn on_pause(&mut self, _ctx: &mut Ctx) {}
    /// This scene became the top again.
    fn on_resume(&mut self, _ctx: &mut Ctx) {}
}

/// What the scene wants to happen after its update.
pub enum Transition {
    /// Keep running this scene.
    None,
    /// Remove this scene (revealing the one beneath; empty stack = quit).
    Pop,
    /// Park this scene and start a new one on top.
    Push(Box<dyn Scene>),
    /// Exit this scene and start a new one in its place.
    Replace(Box<dyn Scene>),
    /// End the whole session (with `on_exit` for every scene on the stack).
    Quit,
}

/// A LIFO stack of scenes driven as a single [`Game`].
pub struct SceneStack {
    scenes: Vec<Box<dyn Scene>>,
}

impl SceneStack {
    /// An empty stack (the session quits on the first update).
    pub fn new() -> Self {
        Self { scenes: Vec::new() }
    }

    /// A stack whose bottom scene is `scene`. Its `init` runs at engine init.
    pub fn with_seed(scene: Box<dyn Scene>) -> Self {
        Self { scenes: vec![scene] }
    }

    pub fn len(&self) -> usize {
        self.scenes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.scenes.is_empty()
    }

    fn handle(&mut self, transition: Transition, ctx: &mut Ctx) {
        match transition {
            Transition::None => {}
            Transition::Quit => ctx.quit(),
            Transition::Pop => {
                if let Some(mut popped) = self.scenes.pop() {
                    popped.on_exit();
                }
                if let Some(top) = self.scenes.last_mut() {
                    top.on_resume(ctx);
                } else {
                    ctx.quit(); // nothing left to show
                }
            }
            Transition::Push(mut scene) => {
                if let Some(top) = self.scenes.last_mut() {
                    top.on_pause(ctx);
                }
                scene.init(ctx);
                self.scenes.push(scene);
            }
            Transition::Replace(mut scene) => {
                if let Some(mut old) = self.scenes.pop() {
                    old.on_exit();
                }
                scene.init(ctx);
                self.scenes.push(scene);
            }
        }
    }
}

impl Default for SceneStack {
    fn default() -> Self {
        Self::new()
    }
}

impl Game for SceneStack {
    fn init(&mut self, ctx: &mut Ctx) {
        if let Some(top) = self.scenes.last_mut() {
            top.init(ctx);
        }
    }

    fn update(&mut self, ctx: &mut Ctx) {
        if self.scenes.is_empty() {
            ctx.quit();
            return;
        }
        let transition = self.scenes.last_mut().expect("checked above").update(ctx);
        self.handle(transition, ctx);
    }

    fn render(&mut self, ctx: &mut Ctx) {
        if let Some(top) = self.scenes.last_mut() {
            top.render(ctx);
        }
    }

    fn on_exit(&mut self) {
        for scene in self.scenes.iter_mut().rev() {
            scene.on_exit();
        }
        self.scenes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{FrameInput, ScriptedBackend};
    use crate::config::EngineConfig;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone)]
    struct Log(Rc<RefCell<Vec<String>>>);

    /// Records lifecycle calls. Plans fire "after N updates" (0 = never),
    /// in priority order: replace, then push, then pop.
    struct Rec {
        name: &'static str,
        log: Log,
        push_after: u32,
        pop_after: u32,
        replace_after: u32,
        updates: u32,
    }

    impl Rec {
        fn new(
            name: &'static str,
            log: &Log,
            push_after: u32,
            pop_after: u32,
            replace_after: u32,
        ) -> Box<Self> {
            Box::new(Self {
                name,
                log: log.clone(),
                push_after,
                pop_after,
                replace_after,
                updates: 0,
            })
        }
    }

    impl Scene for Rec {
        fn init(&mut self, _ctx: &mut Ctx) {
            self.log.0.borrow_mut().push(format!("{}.init", self.name));
        }
        fn update(&mut self, _ctx: &mut Ctx) -> Transition {
            self.updates += 1;
            let n = self.updates;
            self.log.0.borrow_mut().push(format!("{}.update", self.name));
            if self.replace_after > 0 && n == self.replace_after {
                return Transition::Replace(Self::new("C", &self.log, 0, 0, 0));
            }
            if self.push_after > 0 && n == self.push_after {
                return Transition::Push(Self::new("B", &self.log, 0, 1, 0));
            }
            if self.pop_after > 0 && n == self.pop_after {
                return Transition::Pop;
            }
            Transition::None
        }
        fn render(&mut self, _ctx: &mut Ctx) {}
        fn on_pause(&mut self, _ctx: &mut Ctx) {
            self.log.0.borrow_mut().push(format!("{}.pause", self.name));
        }
        fn on_resume(&mut self, _ctx: &mut Ctx) {
            self.log.0.borrow_mut().push(format!("{}.resume", self.name));
        }
        fn on_exit(&mut self) {
            self.log.0.borrow_mut().push(format!("{}.exit", self.name));
        }
    }

    /// Drive the stack through `frames` real engine frames (headless).
    fn run_frames(stack: &mut SceneStack, frames: usize) {
        let mut backend = ScriptedBackend::new();
        for _ in 0..frames {
            backend.push(FrameInput { dt: 1.0 / 60.0, ..FrameInput::default() });
        }
        let mut engine = Engine::new(EngineConfig::new()).unwrap();
        engine.run(&mut backend, stack).unwrap();
    }

    #[test]
    fn push_pop_lifecycle_ordering() {
        let log = Log(Rc::new(RefCell::new(Vec::new())));
        let mut stack = SceneStack::with_seed(Rec::new("A", &log, 1, 0, 0));
        run_frames(&mut stack, 3);

        let events = log.0.borrow().join(",");
        assert_eq!(
            events,
            "A.init,A.update,A.pause,B.init,B.update,B.exit,A.resume,A.update,A.exit",
            "got: {events}"
        );
    }

    #[test]
    fn popping_the_last_scene_quits() {
        let log = Log(Rc::new(RefCell::new(Vec::new())));
        let mut stack = SceneStack::with_seed(Rec::new("A", &log, 0, 1, 0));
        run_frames(&mut stack, 3);

        let events = log.0.borrow().join(",");
        assert_eq!(events, "A.init,A.update,A.exit", "got: {events}");
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn replace_exits_old_and_takes_over() {
        // A replaces itself with C on its first update; C idles; at run end
        // the remaining stack exits top-down.
        let log = Log(Rc::new(RefCell::new(Vec::new())));
        let mut stack = SceneStack::with_seed(Rec::new("A", &log, 0, 0, 1));
        run_frames(&mut stack, 3);

        let events = log.0.borrow().join(",");
        assert_eq!(
            events,
            "A.init,A.update,A.exit,C.init,C.update,C.update,C.exit",
            "got: {events}"
        );
    }
}
