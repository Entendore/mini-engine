//! minifb implementation of the host contract: it measures frame time,
//! gathers input, and presents finished pixel buffers.

use std::time::{Duration, Instant};

use minifb::{
    Key, KeyRepeat, MouseButton as OsMouseButton, MouseMode, Window as OsWindow, WindowOptions,
};

use mini_core::backend::{FrameInput, WindowBackend};
use mini_core::config::EngineConfig;
use mini_core::error::{EngineError, Result};
use mini_core::input::KeyCode;
use mini_core::math::Vec2;

/// A real OS window that measures frame time and gathers input.
pub struct MinifbBackend {
    window: OsWindow,
    width: usize,
    height: usize,
    last_poll: Option<Instant>,
}

impl MinifbBackend {
    /// Create the OS window and apply the fps cap from `config`.
    pub fn new(config: &EngineConfig) -> Result<Self> {
        let window = OsWindow::new(
            config.title(),
            config.width(),
            config.height(),
            WindowOptions::default(),
        )
        .map_err(|e| EngineError::WindowInit(e.to_string()))?;
        window.limit_update_rate(Some(Duration::from_micros(
            1_000_000 / config.target_fps().max(1) as u64,
        )));
        Ok(Self { window, width: config.width(), height: config.height(), last_poll: None })
    }
}

impl WindowBackend for MinifbBackend {
    fn poll(&mut self) -> Option<FrameInput> {
        if !self.window.is_open() {
            return None;
        }
        // The host owns time: dt is measured here, not in the core.
        let now = Instant::now();
        let dt = self.last_poll.map_or(0.0, |last| (now - last).as_secs_f32());
        self.last_poll = Some(now);

        let mouse_pos =
            self.window.get_mouse_pos(MouseMode::Clamp).map(|(x, y)| Vec2::new(x, y));
        let mouse_buttons = [
            self.window.get_mouse_down(OsMouseButton::Left),
            self.window.get_mouse_down(OsMouseButton::Middle),
            self.window.get_mouse_down(OsMouseButton::Right),
        ];
        let (sx, sy) = self.window.get_scroll_wheel();

        Some(FrameInput {
            dt,
            keys: self.window.get_keys().into_iter().filter_map(map_key).collect(),
            mouse_pos,
            mouse_buttons,
            wheel: Vec2::new(sx, sy),
            chars: self
                .window
                .get_keys_pressed(KeyRepeat::No)
                .into_iter()
                .filter_map(key_to_char)
                .collect(),
            focused: self.window.is_active(),
            close_requested: false,
        })
    }

    fn present(&mut self, pixels: &[u32], width: usize, height: usize) -> Result<()> {
        self.window
            .update_with_buffer(pixels, width, height)
            .map_err(|e| EngineError::Present(e.to_string()))
    }
}

fn map_key(key: Key) -> Option<KeyCode> {
    Some(match key {
        Key::A => KeyCode::A,
        Key::B => KeyCode::B,
        Key::C => KeyCode::C,
        Key::D => KeyCode::D,
        Key::E => KeyCode::E,
        Key::F => KeyCode::F,
        Key::G => KeyCode::G,
        Key::H => KeyCode::H,
        Key::I => KeyCode::I,
        Key::J => KeyCode::J,
        Key::K => KeyCode::K,
        Key::L => KeyCode::L,
        Key::M => KeyCode::M,
        Key::N => KeyCode::N,
        Key::O => KeyCode::O,
        Key::P => KeyCode::P,
        Key::Q => KeyCode::Q,
        Key::R => KeyCode::R,
        Key::S => KeyCode::S,
        Key::T => KeyCode::T,
        Key::U => KeyCode::U,
        Key::V => KeyCode::V,
        Key::W => KeyCode::W,
        Key::X => KeyCode::X,
        Key::Y => KeyCode::Y,
        Key::Z => KeyCode::Z,
        Key::Left => KeyCode::Left,
        Key::Right => KeyCode::Right,
        Key::Up => KeyCode::Up,
        Key::Down => KeyCode::Down,
        Key::Space => KeyCode::Space,
        Key::Enter => KeyCode::Enter,
        Key::Escape => KeyCode::Escape,
        Key::LeftShift => KeyCode::LeftShift,
        Key::Backspace => KeyCode::Backspace,
        Key::Num0 => KeyCode::Num0,
        Key::Num1 => KeyCode::Num1,
        Key::Num2 => KeyCode::Num2,
        Key::Num3 => KeyCode::Num3,
        Key::Num4 => KeyCode::Num4,
        Key::Num5 => KeyCode::Num5,
        Key::Num6 => KeyCode::Num6,
        Key::Num7 => KeyCode::Num7,
        Key::Num8 => KeyCode::Num8,
        Key::Num9 => KeyCode::Num9,
        _ => return None,
    })
}

/// Best-effort text mapping for `Input::typed()` (no shift handling — this
/// is a mini engine; full IME belongs in a real host).
fn key_to_char(key: Key) -> Option<char> {
    Some(match key {
        Key::A => 'a',
        Key::B => 'b',
        Key::C => 'c',
        Key::D => 'd',
        Key::E => 'e',
        Key::F => 'f',
        Key::G => 'g',
        Key::H => 'h',
        Key::I => 'i',
        Key::J => 'j',
        Key::K => 'k',
        Key::L => 'l',
        Key::M => 'm',
        Key::N => 'n',
        Key::O => 'o',
        Key::P => 'p',
        Key::Q => 'q',
        Key::R => 'r',
        Key::S => 's',
        Key::T => 't',
        Key::U => 'u',
        Key::V => 'v',
        Key::W => 'w',
        Key::X => 'x',
        Key::Y => 'y',
        Key::Z => 'z',
        Key::Num0 => '0',
        Key::Num1 => '1',
        Key::Num2 => '2',
        Key::Num3 => '3',
        Key::Num4 => '4',
        Key::Num5 => '5',
        Key::Num6 => '6',
        Key::Num7 => '7',
        Key::Num8 => '8',
        Key::Num9 => '9',
        Key::Space => ' ',
        _ => return None,
    })
}
