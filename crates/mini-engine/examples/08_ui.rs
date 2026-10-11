//! Immediate-mode UI. Widget calls happen in `render` — each one draws AND
//! hit-tests — and buffer events that you drain in `update`. No widget tree,
//! no retained state: `Ui` only tracks hover/active between frames.

use mini_engine::prelude::*;

const W: f32 = 480.0;
const H: f32 = 270.0;

struct UiDemo {
    ui: Ui,
    plays: u32,
    checked: bool,
    radius: f32, // slider value, 0..=1
    bar: Timer,
}

impl Game for UiDemo {
    fn update(&mut self, ctx: &mut Ctx) {
        // Widgets were drawn (and hit-tested) during the previous render;
        // their events arrive here, one frame later.
        for ev in self.ui.take_events() {
            match ev {
                UiEvent::Click(id) => {
                    if id == WidgetId::from_name("play") {
                        self.plays += 1;
                    }
                    if id == WidgetId::from_name("quit") {
                        ctx.quit();
                    }
                }
                UiEvent::Toggled(id, now) if id == WidgetId::from_name("grid") => {
                    self.checked = now;
                }
                UiEvent::Slid(id, v) if id == WidgetId::from_name("radius") => {
                    self.radius = v;
                }
                _ => {}
            }
        }

        self.bar.advance(ctx.time.delta());
        if self.bar.finished() {
            self.bar.restart();
        }
    }

    fn render(&mut self, ctx: &mut Ctx) {
        let r = &mut ctx.renderer;
        r.clear(Color::rgb(14, 16, 26));

        // The checkbox toggles a grid; the slider drives the circle radius.
        if self.checked {
            for x in 0..(W as i32 / 24) {
                r.line(x * 24, 0, x * 24, H as i32, Color::rgb(28, 32, 48));
            }
            for y in 0..(H as i32 / 24) {
                r.line(0, y * 24, W as i32, y * 24, Color::rgb(28, 32, 48));
            }
        }
        r.circle(W as i32 / 2, 210, (10.0 + self.radius * 40.0) as i32, Color::CYAN);

        // The panel groups everything; all coordinates are plain Rects.
        self.ui.panel(r, Rect::new(90.0, 40.0, 300.0, 130.0));
        self.ui.label(r, Rect::new(90.0, 46.0, 300.0, 22.0), "SETTINGS", Color::WHITE);

        self.ui.button(
            ctx.input,
            r,
            WidgetId::from_name("play"),
            Rect::new(110.0, 76.0, 120.0, 26.0),
            &format!("PLAY x{}", self.plays),
        );
        self.ui.button(
            ctx.input,
            r,
            WidgetId::from_name("quit"),
            Rect::new(250.0, 76.0, 120.0, 26.0),
            "QUIT",
        );

        self.ui.checkbox(
            ctx.input,
            r,
            WidgetId::from_name("grid"),
            Rect::new(110.0, 114.0, 18.0, 18.0),
            self.checked,
        );
        self.ui.label(r, Rect::new(136.0, 112.0, 100.0, 22.0), "SHOW GRID", Color::GRAY);

        self.ui.slider(
            ctx.input,
            r,
            WidgetId::from_name("radius"),
            Rect::new(240.0, 118.0, 130.0, 10.0),
            self.radius,
        );

        // Progress bar (display only) — here fed by a looping Timer.
        self.ui.progress(r, Rect::new(110.0, 186.0, 260.0, 12.0), self.bar.fraction());
        r.text(8, 8, "WIDGETS DRAW IN RENDER - EVENTS ARRIVE IN UPDATE", 2, Color::GRAY);
    }
}

fn main() -> Result<()> {
    let mut game =
        UiDemo { ui: Ui::new(), plays: 0, checked: false, radius: 0.5, bar: Timer::new(3.0) };
    run(EngineConfig::new().title("08 - ui").size(480, 270), &mut game)
}
