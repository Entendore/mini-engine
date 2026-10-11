//! Asset loading. Kept in the facade so `mini-core` stays dependency-free.
//!
//! Enable the `png` cargo feature for [`load_png`]:
//!
//! ```ignore
//! let img = mini_engine::assets::load_png("assets/player.png")?;
//! let atlas = Atlas::new(img, 8, 2).ok_or("grid must divide evenly")?;
//! ```

/// Decode a PNG file into a blittable [`mini_core::render::Image`].
///
/// Supports 8-bit RGB and RGBA; other bit depths / color types are rejected
/// with [`EngineError::Asset`](mini_core::EngineError::Asset).
#[cfg(feature = "png")]
pub fn load_png(path: impl AsRef<std::path::Path>) -> mini_core::Result<mini_core::render::Image> {
    use mini_core::error::{EngineError, Result};
    use mini_core::render::Image;
    use png::BitDepth;

    let file = std::fs::File::open(path.as_ref())
        .map_err(|e| EngineError::Asset(e.to_string()))?;
    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info().map_err(|e| EngineError::Asset(e.to_string()))?;
    if reader.info().bit_depth != BitDepth::Eight {
        return Err(EngineError::Asset("only 8-bit PNGs are supported".into()));
    }
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).map_err(|e| EngineError::Asset(e.to_string()))?;
    let (w, h) = (info.width, info.height);

    let mut pixels = Vec::with_capacity((w * h) as usize);
    match info.color_type {
        png::ColorType::Rgba => {
            for px in buf[..(w * h * 4) as usize].chunks_exact(4) {
                pixels.push(
                    ((px[3] as u32) << 24)
                        | ((px[0] as u32) << 16)
                        | ((px[1] as u32) << 8)
                        | px[2] as u32,
                );
            }
        }
        png::ColorType::Rgb => {
            for px in buf[..(w * h * 3) as usize].chunks_exact(3) {
                pixels.push(
                    0xFF00_0000 | ((px[0] as u32) << 16) | ((px[1] as u32) << 8) | px[2] as u32,
                );
            }
        }
        other => {
            return Err(EngineError::Asset(format!("unsupported png color type {other:?}")));
        }
    }
    Image::from_pixels(w, h, pixels)
        .ok_or_else(|| EngineError::Asset("png dimension mismatch".into()))
}
