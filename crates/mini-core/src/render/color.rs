/// 24-bit RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red channel, 0-255.
    pub r: u8,
    /// Green channel, 0-255.
    pub g: u8,
    /// Blue channel, 0-255.
    pub b: u8,
}

impl Color {
    /// Build from three channels.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Build from `0xRRGGBB`.
    pub const fn hex(hex: u32) -> Self {
        Self::rgb(((hex >> 16) & 0xFF) as u8, ((hex >> 8) & 0xFF) as u8, (hex & 0xFF) as u8)
    }

    pub const WHITE: Color = Color::rgb(236, 240, 244);
    pub const BLACK: Color = Color::rgb(10, 12, 18);
    pub const GRAY: Color = Color::rgb(120, 126, 134);
    pub const DARK_GRAY: Color = Color::rgb(56, 60, 70);
    pub const RED: Color = Color::rgb(235, 87, 87);
    pub const GREEN: Color = Color::rgb(80, 220, 120);
    pub const BLUE: Color = Color::rgb(90, 150, 255);
    pub const CYAN: Color = Color::rgb(70, 220, 230);
    pub const MAGENTA: Color = Color::rgb(230, 100, 230);
    pub const YELLOW: Color = Color::rgb(250, 210, 70);
    pub const ORANGE: Color = Color::rgb(250, 150, 60);

    /// Pack into the 0xAARRGGBB layout the framebuffer uses.
    pub const fn to_argb(self) -> u32 {
        0xFF00_0000 | ((self.r as u32) << 16) | ((self.g as u32) << 8) | self.b as u32
    }

    /// Linear blend; `t` clamped to `0.0..=1.0`.
    pub fn lerp(self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color::rgb(
            (self.r as f32 + (other.r as f32 - self.r as f32) * t) as u8,
            (self.g as f32 + (other.g as f32 - self.g as f32) * t) as u8,
            (self.b as f32 + (other.b as f32 - self.b as f32) * t) as u8,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_and_argb_round_trip() {
        let c = Color::hex(0x1A2B3C);
        assert_eq!((c.r, c.g, c.b), (0x1A, 0x2B, 0x3C));
        assert_eq!(c.to_argb(), 0xFF1A_2B3C);
    }

    #[test]
    fn lerp_hits_endpoints() {
        let a = Color::rgb(0, 0, 0);
        let b = Color::rgb(255, 255, 255);
        assert_eq!(a.lerp(b, 0.0), a);
        assert_eq!(a.lerp(b, 1.0), b);
    }
}
