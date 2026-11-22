//! Drawing — primitives, camera, images.
//! Top: a screen-space reference sheet. Middle: the same shapes drawn under
//! a camera offset. Bottom: procedural image blits (tint / alpha / flip).

use mini_engine::prelude::*;

/// A soft-edged disc with per-pixel alpha — generated, no asset files.
fn disc(size: u32) -> Image {
    let r = size as f32 * 0.5;
    let mut px = Vec::with_capacity((size * size) as usize);
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - r;
            let dy = y as f32 + 0.5 - r;
            let d = ((dx * dx + dy * dy).sqrt() / r).clamp(0.0, 1.0);
            let a = ((1.0 - d) * 2.0).min(1.0);
            let v = (255.0 * (1.0 - d * 0.4)) as u32;
            px.push(((a * 255.0) as u32) << 24 | (v << 16) | (v << 8) | v);
        }
    }
    Image::from_pixels(size, size, px).expect("size*size pixels")
}

struct DrawingDemo;

impl Game for DrawingDemo {
    fn update(&mut self, _ctx: &mut Ctx) {}

    fn render(&mut self, ctx: &mut Ctx) {
        let r = &mut ctx.renderer;
        r.v_gradient(Color::rgb(14, 16, 26), Color::rgb(24, 22, 40));
        let (w, h) = r.size();

        // -- screen space (camera at ZERO) ----------------------------------
        r.rect(Rect::new(16.0, 16.0, 40.0, 24.0), Color::GREEN);
        r.rect_outline(Rect::new(70.0, 16.0, 40.0, 24.0), Color::GREEN);
        r.line(130, 20, 170, 40, Color::YELLOW);
        r.circle(205, 28, 14, Color::BLUE);
        r.circle_outline(245, 28, 14, Color::BLUE);
        for i in 0..24 {
            r.pixel(275 + (i * 7 % 50), 20 + (i * 13 % 20), Color::GRAY);
        }

        r.text(16, 56, "SCALE 1 / 2 / 4:", 2, Color::GRAY);
        r.text(16, 74, "ABC", 1, Color::WHITE);
        r.text(50, 74, "ABC", 2, Color::WHITE);
        r.text(110, 74, "ABC", 4, Color::WHITE);
        r.text_right(w as i32 - 16, 56, "RIGHT-ALIGNED", 2, Color::GRAY);

        // -- world space: everything until reset is shifted by the camera ---
        r.set_camera(Vec2::new(24.0, 12.0)); // screen = world - camera
        r.rect(Rect::new(16.0, 110.0, 40.0, 24.0), Color::ORANGE);
        r.rect_outline(Rect::new(70.0, 110.0, 40.0, 24.0), Color::ORANGE);
        r.circle(205, 122, 14, Color::RED);
        r.text(16, 150, "SHIFTED BY CAMERA", 2, Color::GRAY);
        r.set_camera(Vec2::ZERO); // HUDs usually reset it first

        // -- images -----------------------------------------------------------
        let img = disc(28);
        r.blit(&img, 300, 96); // plain
        r.blit_opts(&img, 340, 96, Blit::tinted(Color::MAGENTA)); // tinted
        r.blit_opts(&img, 380, 96, Blit { alpha: 0.4, ..Blit::default() }); // 40% opacity
        r.blit_opts(&img, 420, 96, Blit { flip_x: true, ..Blit::default() }); // mirrored

        r.text(
            16,
            h as i32 - 24,
            "SET_CAMERA SHIFTS WORLD DRAWING - RESET IT FOR HUDS",
            2,
            Color::GRAY,
        );
    }
}

fn main() -> Result<()> {
    run(EngineConfig::new().title("03 - drawing").size(480, 270), &mut DrawingDemo)
}
