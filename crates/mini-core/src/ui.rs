//! Immediate-mode UI on top of the renderer.
//!
//! Create a [`Ui`] once per scene. Call widgets **once per frame, from
//! `render`** — each call draws *and* hit-tests. Widget calls buffer
//! [`UiEvent`]s; drain them with [`Ui::take_events`] in `update` (one frame
//! of latency, in exchange for zero retained widget trees).
//!
//! ```ignore
//! // render:
//! self.ui.button(ctx.input, r, WidgetId::from_name("play"), rect, "PLAY");
//! // update:
//! for ev in self.ui.take_events() {
//!     if ev == UiEvent::Click(WidgetId::from_name("play")) { /* start game */ }
//! }
//! ```

use crate::input::{Input, MouseButton};
use crate::math::{Rect, Vec2};
use crate::render::{Color, Renderer};

/// Stable widget identity: [`WidgetId::from_name`] (FNV-1a of the name) or
/// [`WidgetId::raw`] for hand-numbered widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(u64);

impl WidgetId {
    /// Hash a stable name into an id (`WidgetId::from_name("play")`).
    pub fn from_name(name: &str) -> Self {
        let mut h = 0xcbf2_9ce4_8422_2325_u64;
        for b in name.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        WidgetId(h)
    }

    /// A hand-assigned id.
    pub const fn raw(v: u64) -> Self {
        WidgetId(v)
    }
}

/// What a widget did this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UiEvent {
    /// A button was clicked (press + release both inside).
    Click(WidgetId),
    /// A checkbox toggled; the payload is the new state.
    Toggled(WidgetId, bool),
    /// A slider moved; the payload is the new value in `0.0..=1.0`.
    Slid(WidgetId, f32),
}

/// Hot/active state plus the event queue. One per scene.
#[derive(Debug, Default)]
pub struct Ui {
    active: Option<WidgetId>,
    events: Vec<UiEvent>,
}

const PANEL_BG: Color = Color::rgb(20, 24, 38);
const PANEL_EDGE: Color = Color::rgb(70, 78, 100);
const BTN_BG: Color = Color::rgb(38, 44, 66);
const BTN_HOT: Color = Color::rgb(56, 66, 98);
const BTN_DOWN: Color = Color::rgb(26, 31, 48);
const TEXT: Color = Color::rgb(190, 198, 214);
const TRACK: Color = Color::rgb(30, 35, 52);
const FILL: Color = Color::rgb(70, 220, 120);

impl Ui {
    /// A fresh UI with no hot widget and no queued events.
    pub fn new() -> Self {
        Self::default()
    }

    /// Drain the events buffered by this frame's widget calls.
    pub fn take_events(&mut self) -> Vec<UiEvent> {
        std::mem::take(&mut self.events)
    }

    /// How many events are queued (test helper).
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    fn over(input: &Input, rect: Rect) -> bool {
        input.mouse_pos().is_some_and(|p| rect.contains(p))
    }

    /// Filled panel with an outline, for grouping widgets.
    pub fn panel(&mut self, r: &mut Renderer, rect: Rect) {
        r.rect(rect, PANEL_BG);
        r.rect_outline(rect, PANEL_EDGE);
    }

    /// Static text centered in `rect` (scale 2).
    pub fn label(&mut self, r: &mut Renderer, rect: Rect, text: &str, color: Color) {
        draw_label(r, rect, text, color);
    }

    /// Button. Emits [`UiEvent::Click`] when pressed and released inside.
    pub fn button(&mut self, input: &Input, r: &mut Renderer, id: WidgetId, rect: Rect, text: &str) {
        let hovered = Self::over(input, rect) && self.active.map_or(true, |a| a == id);
        if hovered && input.mouse_pressed(MouseButton::Left) {
            self.active = Some(id);
        }
        let held = self.active == Some(id) && input.mouse_down(MouseButton::Left);
        let clicked = self.active == Some(id) && hovered && input.mouse_released(MouseButton::Left);
        if input.mouse_released(MouseButton::Left) {
            self.active = None;
        }
        if clicked {
            self.events.push(UiEvent::Click(id));
        }

        let (bg, edge) = if held {
            (BTN_DOWN, TEXT)
        } else if hovered {
            (BTN_HOT, TEXT)
        } else {
            (BTN_BG, PANEL_EDGE)
        };
        r.rect(rect, bg);
        r.rect_outline(rect, edge);
        draw_label(r, rect, text, TEXT);
    }

    /// Checkbox. Emits [`UiEvent::Toggled`] with the new state on click.
    pub fn checkbox(
        &mut self,
        input: &Input,
        r: &mut Renderer,
        id: WidgetId,
        rect: Rect,
        checked: bool,
    ) {
        let hovered = Self::over(input, rect) && self.active.map_or(true, |a| a == id);
        if hovered && input.mouse_pressed(MouseButton::Left) {
            self.active = Some(id);
        }
        let clicked = self.active == Some(id) && hovered && input.mouse_released(MouseButton::Left);
        if input.mouse_released(MouseButton::Left) {
            self.active = None;
        }
        if clicked {
            self.events.push(UiEvent::Toggled(id, !checked));
        }

        r.rect(rect, BTN_BG);
        r.rect_outline(rect, if hovered { TEXT } else { PANEL_EDGE });
        if checked {
            let inner = rect.expand(-3.0);
            r.rect(inner, FILL);
            let (x0, y0) = (inner.x as i32, inner.y as i32);
            let (x1, y1) =
                (inner.x as i32 + inner.w as i32 - 1, inner.y as i32 + inner.h as i32 - 1);
            r.line(x0, y0, x1, y1, PANEL_BG);
            r.line(x0, y1, x1, y0, PANEL_BG);
        }
    }

    /// Horizontal slider. Emits [`UiEvent::Slid`] on change; pressing the
    /// track jumps to that position, and dragging continues outside `rect`.
    pub fn slider(&mut self, input: &Input, r: &mut Renderer, id: WidgetId, rect: Rect, value: f32) {
        let hovering = Self::over(input, rect);
        if hovering && input.mouse_pressed(MouseButton::Left) {
            self.active = Some(id);
        }
        let mut value = value.clamp(0.0, 1.0);
        let dragging = self.active == Some(id) && input.mouse_down(MouseButton::Left);
        if dragging {
            if let Some(p) = input.mouse_pos() {
                let t = ((p.x - rect.x) / rect.w.max(1.0)).clamp(0.0, 1.0);
                if t != value {
                    value = t;
                    self.events.push(UiEvent::Slid(id, value));
                }
            }
        }
        if input.mouse_released(MouseButton::Left) {
            self.active = None;
        }

        r.rect(rect, TRACK);
        r.rect_outline(rect, PANEL_EDGE);
        let knob_w = rect.h + 4.0;
        let knob_x = rect.x + value * (rect.w - knob_w).max(0.0);
        let knob = Rect::new(knob_x, rect.y - 2.0, knob_w, rect.h + 4.0);
        r.rect(knob, if dragging { BTN_DOWN } else if hovering { BTN_HOT } else { BTN_BG });
        r.rect_outline(knob, TEXT);
    }

    /// Progress bar (display only).
    pub fn progress(&mut self, r: &mut Renderer, rect: Rect, fraction: f32) {
        r.rect(rect, TRACK);
        let w = (rect.w * fraction.clamp(0.0, 1.0)).round();
        if w >= 1.0 {
            r.rect(Rect::new(rect.x, rect.y, w, rect.h), FILL);
        }
    }
}

fn draw_label(r: &mut Renderer, rect: Rect, text: &str, color: Color) {
    let scale = 2;
    let w = r.text_width(text, scale) as f32;
    let h = (5 * scale) as f32;
    let x = rect.x + (rect.w - w) * 0.5;
    let y = rect.y + (rect.h - h) * 0.5;
    r.text(x as i32, y as i32, text, scale, color);
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: WidgetId = WidgetId::raw(1);
    const RECT: Rect = Rect::new(10.0, 10.0, 40.0, 16.0);

    fn input_at(x: f32, y: f32, down: bool) -> Input {
        let mut input = Input::default();
        let buttons: [MouseButton; 1] = if down { [MouseButton::Left] } else { [MouseButton::Left] };
        // Encode "not down" as an empty iterator instead:
        let _ = buttons;
        if down {
            input.begin_frame([], Some(Vec2::new(x, y)), [MouseButton::Left], Vec2::ZERO, Vec::new());
        } else {
            input.begin_frame([], Some(Vec2::new(x, y)), [], Vec2::ZERO, Vec::new());
        }
        input
    }

    #[test]
    fn button_clicks_exactly_once() {
        let mut ui = Ui::new();
        let mut r = Renderer::new(64, 64);

        ui.button(&input_at(20.0, 18.0, true), &mut r, ID, RECT, "OK"); // press
        ui.button(&input_at(20.0, 18.0, false), &mut r, ID, RECT, "OK"); // release
        assert_eq!(ui.take_events(), vec![UiEvent::Click(ID)]);

        ui.button(&input_at(20.0, 18.0, false), &mut r, ID, RECT, "OK"); // idle hover
        assert!(ui.take_events().is_empty());
    }

    #[test]
    fn click_cancels_when_released_outside() {
        let mut ui = Ui::new();
        let mut r = Renderer::new(64, 64);
        ui.button(&input_at(20.0, 18.0, true), &mut r, ID, RECT, "OK"); // press inside
        ui.button(&input_at(60.0, 18.0, false), &mut r, ID, RECT, "OK"); // release outside
        assert!(ui.take_events().is_empty());
    }

    #[test]
    fn slider_drags_and_jumps_to_click() {
        let mut ui = Ui::new();
        let mut r = Renderer::new(64, 64);

        // Press at 75% across the track.
        ui.slider(&input_at(40.0, 16.0, true), &mut r, ID, RECT, 0.0);
        let ev = ui.take_events();
        assert_eq!(ev.len(), 1);
        assert!(matches!(ev[0], UiEvent::Slid(ID, v) if (v - 0.75).abs() < 0.1));

        // Drag to the far end, then release.
        ui.slider(&input_at(50.0, 16.0, true), &mut r, ID, RECT, 0.75);
        assert!(matches!(ui.take_events()[0], UiEvent::Slid(ID, v) if v > 0.9));
        ui.slider(&input_at(50.0, 16.0, false), &mut r, ID, RECT, 1.0);
        assert!(ui.take_events().is_empty());
    }

    #[test]
    fn checkbox_toggles() {
        let mut ui = Ui::new();
        let mut r = Renderer::new(64, 64);
        ui.checkbox(&input_at(20.0, 18.0, true), &mut r, ID, RECT, false); // press
        ui.checkbox(&input_at(20.0, 18.0, false), &mut r, ID, RECT, false); // release
        assert_eq!(ui.take_events(), vec![UiEvent::Toggled(ID, true)]);
    }
}
