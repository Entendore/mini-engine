//! Persistence — `ctx.storage` is a tiny key/value contract.
//! With the mini-engine facade it is file-backed, under your OS data dir
//! (~/.local/share/<window title>/ on Linux, %APPDATA%\<title> on Windows).
//! Values are strings — pick your own encoding (numbers, JSON, bincode...).

use mini_engine::prelude::*;

struct Counter {
    clicks: u32,
    status: String,
}

impl Game for Counter {
    fn init(&mut self, ctx: &mut Ctx) {
        // Load on start; absent key -> default.
        self.clicks = ctx.storage.load("clicks").and_then(|s| s.parse().ok()).unwrap_or(0);
        self.status = "loaded".into();
    }

    fn update(&mut self, ctx: &mut Ctx) {
        if ctx.input.mouse_pressed(MouseButton::Left) {
            self.clicks += 1;
            match ctx.storage.save("clicks", &self.clicks.to_string()) {
                Ok(()) => self.status = "saved".into(),
                Err(e) => self.status = format!("save failed: {e}"),
            }
        }
        if ctx.input.pressed(KeyCode::D) {
            ctx.storage.delete("clicks");
            self.clicks = 0;
            self.status = "key deleted".into();
        }
    }

    fn render(&mut self, ctx: &mut Ctx) {
        let r = &mut ctx.renderer;
        r.clear(Color::rgb(14, 16, 26));
        r.text_centered(70, &format!("{}", self.clicks), 8, Color::GREEN);
        r.text_centered(140, "PERSISTENT CLICKS", 3, Color::WHITE);
        r.text_centered(175, &self.status, 2, Color::GRAY);
        r.text_centered(210, "CLICK +1 - D DELETES THE KEY - CLOSE AND RERUN ME", 2, Color::GRAY);
    }
}

fn main() -> Result<()> {
    let mut game = Counter { clicks: 0, status: String::new() };
    run(EngineConfig::new().title("10 - storage demo").size(480, 270), &mut game)
}
