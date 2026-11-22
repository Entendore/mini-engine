//! Software images: ARGB pixels with per-pixel alpha, ready for
//! [`Renderer::blit`](super::Renderer::blit). Decoders (PNG in the facade)
//! and procedural generators both produce these.

use super::color::Color;

/// An ARGB bitmap, row-major.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    width: u32,
    height: u32,
    pixels: Vec<u32>,
}

impl Image {
    /// A solid-color image.
    pub fn solid(width: u32, height: u32, color: Color) -> Self {
        Self { width, height, pixels: vec![color.to_argb(); (width * height) as usize] }
    }

    /// Adopt existing pixels; `None` if `pixels.len() != width * height`.
    pub fn from_pixels(width: u32, height: u32, pixels: Vec<u32>) -> Option<Self> {
        if pixels.len() == (width as usize) * (height as usize) {
            Some(Self { width, height, pixels })
        } else {
            None
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    /// Raw `0xAARRGGBB` pixel, or `None` out of bounds.
    pub fn get(&self, x: i32, y: i32) -> Option<u32> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        Some(self.pixels[y as usize * self.width as usize + x as usize])
    }

    /// Write a raw `0xAARRGGBB` pixel (ignored out of bounds).
    pub fn set(&mut self, x: i32, y: i32, argb: u32) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        self.pixels[y as usize * self.width as usize + x as usize] = argb;
    }

    /// Crop a sub-image (copy). Spritesheet slicing without an [`Atlas`].
    pub fn region(&self, x: i32, y: i32, w: u32, h: u32) -> Option<Image> {
        if x < 0 || y < 0 || x + w as i32 > self.width as i32 || y + h as i32 > self.height as i32 {
            return None;
        }
        let mut pixels = Vec::with_capacity((w * h) as usize);
        for row in y..y + h as i32 {
            let start = row as usize * self.width as usize + x as usize;
            pixels.extend_from_slice(&self.pixels[start..start + w as usize]);
        }
        Image::from_pixels(w, h, pixels)
    }
}

/// Options for [`Renderer::blit_opts`](super::Renderer::blit_opts).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blit {
    /// Per-channel multiply applied before blending (`None` = untouched).
    pub tint: Option<Color>,
    /// Extra opacity multiplier in `0.0..=1.0`.
    pub alpha: f32,
    /// Mirror horizontally.
    pub flip_x: bool,
    /// Mirror vertically.
    pub flip_y: bool,
}

impl Default for Blit {
    fn default() -> Self {
        Self { tint: None, alpha: 1.0, flip_x: false, flip_y: false }
    }
}

impl Blit {
    /// Default options with a tint.
    pub fn tinted(color: Color) -> Self {
        Self { tint: Some(color), ..Self::default() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_crops() {
        let img = Image::from_pixels(2, 2, vec![1, 2, 3, 4]).unwrap();
        let r = img.region(1, 0, 1, 2).unwrap();
        assert_eq!(r.pixels(), &[2, 4]);
        assert!(img.region(1, 1, 2, 2).is_none(), "out of bounds rejected");
    }

    #[test]
    fn from_pixels_validates_length() {
        assert!(Image::from_pixels(2, 2, vec![0; 3]).is_none());
        assert!(Image::from_pixels(2, 2, vec![0; 4]).is_some());
    }

    #[test]
    fn get_set_bounds() {
        let mut img = Image::solid(2, 2, Color::BLACK);
        img.set(5, 5, 0xFFFFFFFF);
        assert_eq!(img.get(5, 5), None);
        img.set(1, 1, 0xFF00FFFF);
        assert_eq!(img.get(1, 1), Some(0xFF00FFFF));
    }
}
