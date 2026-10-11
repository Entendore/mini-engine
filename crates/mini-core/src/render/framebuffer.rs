//! Software-rendered pixel buffer with a small 2D drawing API.
//!
//! Every primitive except [`Renderer::clear`] and
//! [`Renderer::v_gradient`] is offset by `-camera`, so the game can
//! shake/scroll the world and then [`Renderer::set_camera`] back to
//! [`Vec2::ZERO`](crate::math::Vec2::ZERO) to draw a steady HUD.

use super::color::Color;
use super::font;
use super::image::{Blit, Image};
use crate::math::{Rect, Vec2};

/// Software-rendered pixel buffer with a small 2D drawing API.
pub struct Renderer {
    width: usize,
    height: usize,
    buf: Vec<u32>,
    camera: Vec2,
}

impl Renderer {
    pub(crate) fn new(width: usize, height: usize) -> Self {
        Self { width, height, buf: vec![0xFF00_0000; width * height], camera: Vec2::ZERO }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Raw framebuffer (0xAARRGGBB), row-major, `width * height` pixels.
    pub fn pixels(&self) -> &[u32] {
        &self.buf
    }

    /// Set the world-space camera offset (screen = world - camera).
    pub fn set_camera(&mut self, pos: Vec2) {
        self.camera = pos;
    }

    pub fn camera(&self) -> Vec2 {
        self.camera
    }

    /// Overwrite every pixel (unaffected by the camera).
    pub fn clear(&mut self, color: Color) {
        self.buf.fill(color.to_argb());
    }

    /// Vertical gradient background (unaffected by the camera).
    pub fn v_gradient(&mut self, top: Color, bottom: Color) {
        let h = (self.height.max(2) - 1) as f32;
        for y in 0..self.height {
            let c = top.lerp(bottom, y as f32 / h);
            let row = y * self.width;
            self.buf[row..row + self.width].fill(c.to_argb());
        }
    }

    /// Plot a single pixel in *screen space* (no camera); out-of-range is
    /// silently ignored.
    pub fn pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        self.buf[y as usize * self.width + x as usize] = color.to_argb();
    }

    /// Filled rectangle in world space (floor/ceil edges, clipped to screen).
    pub fn rect(&mut self, r: Rect, color: Color) {
        if r.w <= 0.0 || r.h <= 0.0 {
            return;
        }
        let x = r.x - self.camera.x;
        let y = r.y - self.camera.y;
        let x0 = (x.floor() as i32).clamp(0, self.width as i32);
        let y0 = (y.floor() as i32).clamp(0, self.height as i32);
        let x1 = ((x + r.w).ceil() as i32).clamp(0, self.width as i32);
        let y1 = ((y + r.h).ceil() as i32).clamp(0, self.height as i32);
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        for y in y0..y1 {
            for x in x0..x1 {
                self.buf[y as usize * self.width + x as usize] = color.to_argb();
            }
        }
    }

    /// 1-pixel rectangle outline in world space.
    pub fn rect_outline(&mut self, r: Rect, color: Color) {
        if r.w <= 0.0 || r.h <= 0.0 {
            return;
        }
        let x0 = (r.x - self.camera.x).floor() as i32;
        let y0 = (r.y - self.camera.y).floor() as i32;
        let x1 = ((r.x + r.w - self.camera.x).ceil()) as i32;
        let y1 = ((r.y + r.h - self.camera.y).ceil()) as i32;
        for x in x0..x1 {
            self.pixel(x, y0, color);
            self.pixel(x, y1 - 1, color);
        }
        for y in y0..y1 {
            self.pixel(x0, y, color);
            self.pixel(x1 - 1, y, color);
        }
    }

    /// Bresenham line in world space.
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        let (ox, oy) = (self.camera.x.round() as i32, self.camera.y.round() as i32);
        let (x0, y0, x1, y1) = (x0 - ox, y0 - oy, x1 - ox, y1 - oy);
        let (mut x, mut y) = (x0, y0);
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.pixel(x, y, color);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Filled circle in world space.
    pub fn circle(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        let (ox, oy) = (self.camera.x.round() as i32, self.camera.y.round() as i32);
        let (cx, cy) = (cx - ox, cy - oy);
        for y in (cy - radius)..=(cy + radius) {
            for x in (cx - radius)..=(cx + radius) {
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy <= radius * radius {
                    self.pixel(x, y, color);
                }
            }
        }
    }

    /// Midpoint circle outline (crisp at any radius) in world space.
    pub fn circle_outline(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        let (ox, oy) = (self.camera.x.round() as i32, self.camera.y.round() as i32);
        let (cx, cy) = (cx - ox, cy - oy);
        if radius <= 0 {
            self.pixel(cx, cy, color);
            return;
        }
        let (mut x, mut y) = (radius, 0);
        let mut err = 1 - radius;
        while x >= y {
            for (px, py) in [
                (cx + x, cy + y),
                (cx + y, cy + x),
                (cx - y, cy + x),
                (cx - x, cy + y),
                (cx - x, cy - y),
                (cx - y, cy - x),
                (cx + y, cy - x),
                (cx + x, cy - y),
            ] {
                self.pixel(px, py, color);
            }
            y += 1;
            if err < 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x) + 1;
            }
        }
    }

    /// Blit an image in world space. Fully transparent pixels are skipped;
    /// semi-transparent pixels are alpha-blended onto the destination.
    pub fn blit(&mut self, image: &Image, x: i32, y: i32) {
        self.blit_opts(image, x, y, Blit::default());
    }

    /// Blit with tint / opacity / flips.
    pub fn blit_opts(&mut self, image: &Image, x: i32, y: i32, opts: Blit) {
        let ox = self.camera.x.round() as i32;
        let oy = self.camera.y.round() as i32;
        let (x, y) = (x - ox, y - oy);
        let alpha_mult = opts.alpha.clamp(0.0, 1.0);
        for row in 0..image.height() as i32 {
            let py = y + row;
            if py < 0 || py >= self.height as i32 {
                continue;
            }
            let sy = if opts.flip_y { image.height() as i32 - 1 - row } else { row };
            for col in 0..image.width() as i32 {
                let px = x + col;
                if px < 0 || px >= self.width as i32 {
                    continue;
                }
                let sx = if opts.flip_x { image.width() as i32 - 1 - col } else { col };
                let argb = image.get(sx, sy).unwrap_or(0);
                let a = ((argb >> 24) & 0xFF) as f32 / 255.0 * alpha_mult;
                if a <= 0.0 {
                    continue;
                }
                let (mut sr, mut sg, mut sb) = (
                    ((argb >> 16) & 0xFF) as f32,
                    ((argb >> 8) & 0xFF) as f32,
                    (argb & 0xFF) as f32,
                );
                if let Some(t) = opts.tint {
                    sr = sr * t.r as f32 / 255.0;
                    sg = sg * t.g as f32 / 255.0;
                    sb = sb * t.b as f32 / 255.0;
                }
                let idx = py as usize * self.width + px as usize;
                if a >= 0.999 {
                    self.buf[idx] = Color::rgb(sr as u8, sg as u8, sb as u8).to_argb();
                } else {
                    let d = self.buf[idx];
                    let mix = |s: f32, dv: u32| (s + (dv as f32 - s) * a).round() as u8;
                    self.buf[idx] = Color::rgb(
                        mix(sr, (d >> 16) & 0xFF),
                        mix(sg, (d >> 8) & 0xFF),
                        mix(sb, d & 0xFF),
                    )
                    .to_argb();
                }
            }
        }
    }

    /// Blit centered on a world-space point.
    pub fn blit_centered(&mut self, image: &Image, center: Vec2) {
        self.blit_centered_opts(image, center, Blit::default());
    }

    /// [`blit_centered`] with tint / opacity / flips.
    pub fn blit_centered_opts(&mut self, image: &Image, center: Vec2, opts: Blit) {
        let x = (center.x - image.width() as f32 * 0.5).round() as i32;
        let y = (center.y - image.height() as f32 * 0.5).round() as i32;
        self.blit_opts(image, x, y, opts);
    }

    /// Bitmap-font text in world space, `scale` pixels per font pixel.
    pub fn text(&mut self, x: i32, y: i32, text: &str, scale: u32, color: Color) {
        let mut cursor_x = x - self.camera.x.round() as i32;
        let y = y - self.camera.y.round() as i32;
        for ch in text.chars() {
            if let Some(rows) = font::glyph(ch) {
                for (row, bits) in rows.iter().enumerate() {
                    for col in 0..font::GLYPH_W {
                        if bits & (0b100u8 >> col) != 0 {
                            for dy in 0..scale as i32 {
                                for dx in 0..scale as i32 {
                                    self.pixel(
                                        cursor_x + col as i32 * scale as i32 + dx,
                                        y + row as i32 * scale as i32 + dy,
                                        color,
                                    );
                                }
                            }
                        }
                    }
                }
            }
            cursor_x += (font::GLYPH_W as i32 + 1) * scale as i32;
        }
    }

    /// Screen-space centered text — for HUDs (reset the camera first).
    pub fn text_centered(&mut self, y: i32, text: &str, scale: u32, color: Color) {
        let w = self.text_width(text, scale);
        self.text((self.width as i32 - w) / 2, y, text, scale, color);
    }

    /// Screen-space right-aligned text — HUD corners.
    pub fn text_right(&mut self, x_right: i32, y: i32, text: &str, scale: u32, color: Color) {
        let w = self.text_width(text, scale);
        self.text(x_right - w, y, text, scale, color);
    }

    /// Width in pixels that `text` would occupy at the given scale.
    pub fn text_width(&self, text: &str, scale: u32) -> i32 {
        let chars = text.chars().count() as i32;
        if chars == 0 {
            return 0;
        }
        chars * ((font::GLYPH_W as i32 + 1) * scale as i32) - scale as i32
    }

    #[cfg(test)]
    pub(crate) fn pixel_at(&self, x: i32, y: i32) -> Option<Color> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        let v = self.buf[y as usize * self.width + x as usize];
        Some(Color::rgb(((v >> 16) & 0xFF) as u8, ((v >> 8) & 0xFF) as u8, (v & 0xFF) as u8))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_offsets_drawing() {
        let mut r = Renderer::new(32, 32);
        r.set_camera(Vec2::new(5.0, 0.0));
        r.rect(Rect::new(10.0, 10.0, 2.0, 2.0), Color::WHITE);
        assert_eq!(r.pixel_at(5, 11), Some(Color::WHITE)); // 10 - 5
        assert_eq!(r.pixel_at(10, 11), None);
    }

    #[test]
    fn one_pixel_rect_survives_rounding() {
        let mut r = Renderer::new(32, 32);
        r.rect(Rect::new(10.0, 10.0, 1.0, 1.0), Color::WHITE);
        assert_eq!(r.pixel_at(10, 10), Some(Color::WHITE));
    }

    #[test]
    fn blit_skips_transparent_and_blends() {
        let mut r = Renderer::new(4, 4);
        r.clear(Color::BLACK);
        let mut img = Image::solid(2, 2, Color::RED);
        img.set(1, 0, 0); // fully transparent pixel
        r.blit(&img, 0, 0);
        assert_eq!(r.pixel_at(0, 0), Some(Color::RED));
        assert_eq!(r.pixel_at(1, 0), Some(Color::BLACK), "alpha-0 pixel skipped");

        r.clear(Color::BLACK);
        r.blit_opts(&Image::solid(1, 1, Color::RED), 0, 0, Blit { alpha: 0.5, ..Blit::default() });
        let c = r.pixel_at(0, 0).unwrap();
        assert!(c.r > 120 && c.r < 135, "half red over black ≈ {}", c.r);
    }

    #[test]
    fn text_draws_known_glyphs() {
        let mut r = Renderer::new(40, 12);
        r.text(1, 1, "HI!", 2, Color::WHITE);
        let any = |r: &Renderer| {
            r.buf.iter().any(|&v| v == Color::WHITE.to_argb())
        };
        assert!(any(&r), "glyph pixels landed");
    }
}
