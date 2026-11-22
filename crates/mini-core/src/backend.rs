//! The host contract. The engine core never touches the OS or the clock: a
//! *host* — the `mini-engine` facade, a test, a future web shell — implements
//! [`WindowBackend`] and feeds the engine [`FrameInput`]s.
//!
//! Because the host owns time, the core contains no `Instant`, no threads,
//! and no platform code — which is why [`ScriptedBackend`] can replay a game
//! deterministically inside `cargo test`.

use std::collections::VecDeque;

use crate::input::KeyCode;
use crate::math::Vec2;

/// Everything the engine needs to know about the outside world for one frame.
#[derive(Debug, Clone)]
pub struct FrameInput {
    /// Real seconds elapsed since the previous poll, measured by the host.
    pub dt: f32,
    /// Keys currently held.
    pub keys: Vec<KeyCode>,
    /// Cursor position in pixels, if the pointer is over the surface.
    pub mouse_pos: Option<Vec2>,
    /// `[left, middle, right]` buttons currently down.
    pub mouse_buttons: [bool; 3],
    /// Scroll wheel delta this frame.
    pub wheel: Vec2,
    /// Characters typed this frame (text entry).
    pub chars: Vec<char>,
    /// Whether the surface has input focus.
    pub focused: bool,
    /// Whether the user asked to close the session.
    pub close_requested: bool,
}

impl Default for FrameInput {
    fn default() -> Self {
        Self {
            dt: 0.0,
            keys: Vec::new(),
            mouse_pos: None,
            mouse_buttons: [false; 3],
            wheel: Vec2::ZERO,
            chars: Vec::new(),
            focused: true,
            close_requested: false,
        }
    }
}

/// Source of frames + destination for finished pixel buffers.
///
/// Implement this to port the engine to a new platform (winit, SDL, web...).
/// `poll` returning `None` ends the session cleanly (the game still gets
/// `Game::on_exit`).
pub trait WindowBackend {
    /// Produce the next frame's input, or `None` when the session is over.
    fn poll(&mut self) -> Option<FrameInput>;
    /// Present a finished framebuffer (`pixels.len() == width * height`,
    /// row-major 0xAARRGGBB).
    fn present(&mut self, pixels: &[u32], width: usize, height: usize) -> crate::Result<()>;
}

/// A [`WindowBackend`] that plays back scripted [`FrameInput`]s. The engine
/// runs fully headless — this is how gameplay gets tested in CI.
#[derive(Debug, Default)]
pub struct ScriptedBackend {
    frames: VecDeque<FrameInput>,
}

impl ScriptedBackend {
    /// An empty script: the session ends immediately after `init`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append one frame.
    pub fn push(&mut self, frame: FrameInput) -> &mut Self {
        self.frames.push_back(frame);
        self
    }

    /// Append `n` copies of the same frame (idle time, held keys, ...).
    pub fn repeat(&mut self, n: usize, frame: FrameInput) -> &mut Self {
        for _ in 0..n {
            self.frames.push_back(frame.clone());
        }
        self
    }
}

impl WindowBackend for ScriptedBackend {
    fn poll(&mut self) -> Option<FrameInput> {
        self.frames.pop_front()
    }

    fn present(&mut self, _pixels: &[u32], _width: usize, _height: usize) -> crate::Result<()> {
        Ok(())
    }
}

/// Wraps any [`WindowBackend`] and records the frames it produces. Feed the
/// recording to [`ScriptedBackend`] to replay a session — with a seeded
/// [`Rng`](crate::rng::Rng) and the fixed step, replays are deterministic.
pub struct RecordingBackend<B: WindowBackend> {
    inner: B,
    recorded: Vec<FrameInput>,
}

impl<B: WindowBackend> RecordingBackend<B> {
    /// Start recording everything `inner` produces.
    pub fn new(inner: B) -> Self {
        Self { inner, recorded: Vec::new() }
    }

    /// End the session and take the recording.
    pub fn into_frames(self) -> Vec<FrameInput> {
        self.recorded
    }
}

impl<B: WindowBackend> WindowBackend for RecordingBackend<B> {
    fn poll(&mut self) -> Option<FrameInput> {
        let frame = self.inner.poll()?;
        self.recorded.push(frame.clone());
        Some(frame)
    }

    fn present(&mut self, pixels: &[u32], width: usize, height: usize) -> crate::Result<()> {
        self.inner.present(pixels, width, height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_captures_frames_in_order() {
        let mut inner = ScriptedBackend::new();
        for i in 0..3 {
            let mut frame = FrameInput { dt: 1.0 / 60.0, ..Default::default() };
            if i == 1 {
                frame.keys.push(KeyCode::Space);
            }
            inner.push(frame);
        }
        let mut recorder = RecordingBackend::new(inner);
        let mut polls = 0;
        while recorder.poll().is_some() {
            polls += 1;
        }
        assert_eq!(polls, 3);

        let frames = recorder.into_frames();
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[1].keys, vec![KeyCode::Space]);
    }

    #[test]
    fn scripted_repeat_pushes_n_copies() {
        let mut script = ScriptedBackend::new();
        script.repeat(5, FrameInput { dt: 1.0 / 60.0, ..Default::default() });
        let mut polls = 0;
        while script.poll().is_some() {
            polls += 1;
        }
        assert_eq!(polls, 5);
    }
}
