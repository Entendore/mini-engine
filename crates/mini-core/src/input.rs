use std::collections::{HashMap, HashSet};

use crate::math::Vec2;

/// Backend-agnostic key codes (the window backend maps its keys onto these).
/// `#[non_exhaustive]`: new keys may appear — match with a wildcard arm.
#[allow(missing_docs)] // the variant list is self-describing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum KeyCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Left, Right, Up, Down,
    Space, Enter, Escape, LeftShift, Backspace,
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
}

/// Mouse buttons. `#[non_exhaustive]`.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

impl MouseButton {
    fn index(self) -> usize {
        self as usize
    }
}

const MOUSE_BUTTONS: usize = 3;

/// Named actions bound to sets of keys (`"jump"` → `[Space, W]`). Gameplay
/// queries actions, not hardware — rebinding never touches game logic.
#[derive(Debug, Clone, Default)]
pub struct ActionMap {
    bindings: HashMap<Box<str>, Vec<KeyCode>>,
}

impl ActionMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind `action` to `keys`, replacing any previous bindings.
    pub fn bind(&mut self, action: impl Into<String>, keys: &[KeyCode]) -> &mut Self {
        self.bindings.insert(action.into().into_boxed_str(), keys.to_vec());
        self
    }

    /// Append one more key to an action.
    pub fn add_binding(&mut self, action: impl Into<String>, key: KeyCode) -> &mut Self {
        self.bindings.entry(action.into().into_boxed_str()).or_default().push(key);
        self
    }

    pub fn unbind(&mut self, action: &str) {
        self.bindings.remove(action);
    }

    pub fn bindings(&self, action: &str) -> &[KeyCode] {
        self.bindings.get(action).map(Vec::as_slice).unwrap_or(&[])
    }
}

/// Keyboard + mouse state with edge detection. The engine refreshes it once
/// per frame; games only read.
pub struct Input {
    held: HashSet<KeyCode>,
    pressed: HashSet<KeyCode>,
    released: HashSet<KeyCode>,
    mouse_pos: Option<Vec2>,
    mouse_delta: Vec2,
    mouse_held: [bool; MOUSE_BUTTONS],
    mouse_pressed: [bool; MOUSE_BUTTONS],
    mouse_released: [bool; MOUSE_BUTTONS],
    wheel: Vec2,
    typed: Vec<char>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            held: HashSet::new(),
            pressed: HashSet::new(),
            released: HashSet::new(),
            mouse_pos: None,
            mouse_delta: Vec2::ZERO,
            mouse_held: [false; MOUSE_BUTTONS],
            mouse_pressed: [false; MOUSE_BUTTONS],
            mouse_released: [false; MOUSE_BUTTONS],
            wheel: Vec2::ZERO,
            typed: Vec::new(),
        }
    }
}

impl Input {
    /// Called once per frame by the engine; edges (pressed/released) are
    /// derived by diffing against the previous frame.
    pub(crate) fn begin_frame(
        &mut self,
        now_held: impl IntoIterator<Item = KeyCode>,
        mouse_pos: Option<Vec2>,
        now_buttons: impl IntoIterator<Item = MouseButton>,
        wheel: Vec2,
        chars: Vec<char>,
    ) {
        let now: HashSet<KeyCode> = now_held.into_iter().collect();
        self.pressed = now.difference(&self.held).copied().collect();
        self.released = self.held.difference(&now).copied().collect();
        self.held = now;

        let mut buttons = [false; MOUSE_BUTTONS];
        for b in now_buttons {
            buttons[b.index()] = true;
        }
        for i in 0..MOUSE_BUTTONS {
            self.mouse_pressed[i] = buttons[i] && !self.mouse_held[i];
            self.mouse_released[i] = !buttons[i] && self.mouse_held[i];
        }
        self.mouse_held = buttons;

        // Delta must be computed before overwriting the previous position.
        self.mouse_delta = match (self.mouse_pos, mouse_pos) {
            (Some(prev), Some(cur)) => cur - prev,
            _ => Vec2::ZERO,
        };
        self.mouse_pos = mouse_pos;
        self.wheel = wheel;
        self.typed = chars;
    }

    // -- keyboard -----------------------------------------------------------

    pub fn held(&self, key: KeyCode) -> bool {
        self.held.contains(&key)
    }

    /// True only on the frame the key went down.
    pub fn pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    /// True only on the frame the key came up.
    pub fn released(&self, key: KeyCode) -> bool {
        self.released.contains(&key)
    }

    /// -1.0 when `negative` is held, +1.0 when `positive` is held.
    pub fn axis(&self, negative: KeyCode, positive: KeyCode) -> f32 {
        let mut v = 0.0;
        if self.held(negative) {
            v -= 1.0;
        }
        if self.held(positive) {
            v += 1.0;
        }
        v
    }

    /// Characters typed this frame (for text entry screens).
    pub fn typed(&self) -> &[char] {
        &self.typed
    }

    // -- mouse ----------------------------------------------------------------

    /// Cursor position in pixels (top-left origin), `None` when outside.
    pub fn mouse_pos(&self) -> Option<Vec2> {
        self.mouse_pos
    }

    /// Pointer movement since the previous frame (zero on the first frame
    /// and whenever the cursor leaves the surface).
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    pub fn mouse_down(&self, button: MouseButton) -> bool {
        self.mouse_held[button.index()]
    }

    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_pressed[button.index()]
    }

    pub fn mouse_released(&self, button: MouseButton) -> bool {
        self.mouse_released[button.index()]
    }

    /// Scroll delta accumulated this frame (usually just `y`).
    pub fn wheel(&self) -> Vec2 {
        self.wheel
    }

    // -- actions ---------------------------------------------------------------

    /// Any binding for `action` is currently held.
    pub fn action_held(&self, map: &ActionMap, action: &str) -> bool {
        map.bindings(action).iter().any(|&k| self.held(k))
    }

    /// Any binding for `action` went down this frame.
    pub fn action_pressed(&self, map: &ActionMap, action: &str) -> bool {
        map.bindings(action).iter().any(|&k| self.pressed(k))
    }

    /// Any binding for `action` came up this frame.
    pub fn action_released(&self, map: &ActionMap, action: &str) -> bool {
        map.bindings(action).iter().any(|&k| self.released(k))
    }

    /// -1.0 while `negative` is held, +1.0 while `positive` is held.
    pub fn action_axis(&self, map: &ActionMap, negative: &str, positive: &str) -> f32 {
        let mut v = 0.0;
        if self.action_held(map, negative) {
            v -= 1.0;
        }
        if self.action_held(map, positive) {
            v += 1.0;
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(input: &mut Input, keys: &[KeyCode], buttons: &[MouseButton], chars: Vec<char>) {
        input.begin_frame(
            keys.iter().copied(),
            Some(Vec2::new(10.0, 20.0)),
            buttons.iter().copied(),
            Vec2::ZERO,
            chars,
        );
    }

    #[test]
    fn key_edges_fire_once() {
        let mut input = Input::default();
        feed(&mut input, &[KeyCode::Space], &[], Vec::new());
        assert!(input.pressed(KeyCode::Space) && input.held(KeyCode::Space));
        feed(&mut input, &[KeyCode::Space], &[], Vec::new());
        assert!(!input.pressed(KeyCode::Space) && input.held(KeyCode::Space));
        feed(&mut input, &[], &[], Vec::new());
        assert!(input.released(KeyCode::Space) && !input.held(KeyCode::Space));
    }

    #[test]
    fn mouse_edges_fire_once() {
        let mut input = Input::default();
        feed(&mut input, &[], &[MouseButton::Left], Vec::new());
        assert!(input.mouse_pressed(MouseButton::Left));
        feed(&mut input, &[], &[MouseButton::Left], Vec::new());
        assert!(!input.mouse_pressed(MouseButton::Left) && input.mouse_down(MouseButton::Left));
        feed(&mut input, &[], &[], Vec::new());
        assert!(input.mouse_released(MouseButton::Left));
        assert_eq!(input.mouse_pos(), Some(Vec2::new(10.0, 20.0)));
    }

    #[test]
    fn typed_chars_land_in_the_buffer() {
        let mut input = Input::default();
        feed(&mut input, &[], &[], vec!['h', 'i']);
        assert_eq!(input.typed(), &['h', 'i']);
        feed(&mut input, &[], &[], Vec::new());
        assert!(input.typed().is_empty(), "chars are per-frame");
    }

    #[test]
    fn actions_resolve_through_the_map() {
        let mut input = Input::default();
        let mut map = ActionMap::new();
        map.bind("jump", &[KeyCode::Space]).add_binding("jump", KeyCode::W);

        feed(&mut input, &[KeyCode::Space], &[], Vec::new());
        assert!(input.action_pressed(&map, "jump") && input.action_held(&map, "jump"));

        feed(&mut input, &[KeyCode::W], &[], Vec::new());
        assert!(input.action_held(&map, "jump") && !input.action_pressed(&map, "jump"));

        map.bind("jump", &[KeyCode::Up]); // rebind replaces
        feed(&mut input, &[KeyCode::W], &[], Vec::new());
        assert!(!input.action_held(&map, "jump"));
        feed(&mut input, &[KeyCode::Up], &[], Vec::new());
        assert!(input.action_pressed(&map, "jump"));
    }

    #[test]
    fn action_axis_sums_directions() {
        let mut input = Input::default();
        let mut map = ActionMap::new();
        map.bind("left", &[KeyCode::A]).bind("right", &[KeyCode::D]);
        feed(&mut input, &[KeyCode::A, KeyCode::D], &[], Vec::new());
        assert_eq!(input.action_axis(&map, "left", "right"), 0.0);
        feed(&mut input, &[KeyCode::D], &[], Vec::new());
        assert_eq!(input.action_axis(&map, "left", "right"), 1.0);
    }

    #[test]
    fn mouse_delta_tracks_motion() {
        let mut input = Input::default();
        input.begin_frame([], Some(Vec2::new(10.0, 10.0)), [], Vec2::ZERO, Vec::new());
        assert_eq!(input.mouse_delta(), Vec2::ZERO, "no previous frame yet");
        input.begin_frame([], Some(Vec2::new(13.0, 12.0)), [], Vec2::ZERO, Vec::new());
        assert_eq!(input.mouse_delta(), Vec2::new(3.0, 2.0));
        input.begin_frame([], None, [], Vec2::ZERO, Vec::new());
        assert_eq!(input.mouse_delta(), Vec2::ZERO, "cursor left the surface");
    }
}
