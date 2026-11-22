//! Grid sprite atlases and a frame animator. Frames are extracted once, at
//! construction, into `Rc` clones — handing them out is then free.

use std::rc::Rc;

use super::image::Image;

/// A uniform grid atlas: one image sliced into `cols × rows` equal cells.
pub struct Atlas {
    cols: u32,
    rows: u32,
    cell_w: u32,
    cell_h: u32,
    frames: Vec<Rc<Image>>,
}

impl Atlas {
    /// Slices `image` into a grid; `None` unless it divides evenly.
    pub fn new(image: Image, cols: u32, rows: u32) -> Option<Self> {
        if cols == 0 || rows == 0 || image.width() % cols != 0 || image.height() % rows != 0 {
            return None;
        }
        let (cell_w, cell_h) = (image.width() / cols, image.height() / rows);
        let mut frames = Vec::with_capacity((cols * rows) as usize);
        for i in 0..cols * rows {
            let x = (i % cols) * cell_w;
            let y = (i / cols) * cell_h;
            frames.push(Rc::new(image.region(x as i32, y as i32, cell_w, cell_h)?));
        }
        Some(Self { cols, rows, cell_w, cell_h, frames })
    }

    /// `(cell_w, cell_h)` in pixels.
    pub fn cell_size(&self) -> (u32, u32) {
        (self.cell_w, self.cell_h)
    }

    pub fn cols(&self) -> u32 {
        self.cols
    }

    pub fn rows(&self) -> u32 {
        self.rows
    }

    pub fn frame_count(&self) -> u32 {
        self.cols * self.rows
    }

    /// Frame `index` (row-major, left→right, top→bottom) as a cheap `Rc` clone.
    pub fn frame(&self, index: u32) -> Option<Rc<Image>> {
        self.frames.get(index as usize).cloned()
    }
}

/// Steps through frame indices at a fixed fps.
#[derive(Debug, Clone, Copy)]
pub struct Animator {
    fps: f32,
    frames: u32,
    looping: bool,
    index: u32,
    t: f32,
    finished: bool,
}

impl Animator {
    /// `frames` frames at `fps` frames-per-second; loop or play once.
    pub fn new(fps: f32, frames: u32, looping: bool) -> Self {
        Self { fps: fps.max(f32::EPSILON), frames, looping, index: 0, t: 0.0, finished: false }
    }

    /// Advance by `dt` seconds.
    pub fn advance(&mut self, dt: f32) {
        if self.finished || self.frames <= 1 {
            return;
        }
        let frame_dt = 1.0 / self.fps;
        self.t += dt;
        while self.t >= frame_dt {
            self.t -= frame_dt;
            self.index += 1;
            if self.index >= self.frames {
                if self.looping {
                    self.index = 0;
                } else {
                    self.index = self.frames - 1;
                    self.finished = true;
                    self.t = 0.0;
                    break;
                }
            }
        }
    }

    /// Current frame index (feed to [`Atlas::frame`]).
    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn restart(&mut self) {
        self.index = 0;
        self.t = 0.0;
        self.finished = false;
    }
}

#[cfg(test)]
mod tests {
    use super::super::Color;
    use super::*;

    #[test]
    fn atlas_slices_evenly_or_rejects() {
        let img = Image::solid(4, 2, Color::WHITE);
        assert!(Atlas::new(img.clone(), 2, 2).is_some());
        assert!(Atlas::new(img.clone(), 3, 1).is_none());
        let atlas = Atlas::new(img, 2, 1).unwrap();
        assert_eq!(atlas.cell_size(), (2, 2));
        assert_eq!(atlas.frame_count(), 2);
        assert!(atlas.frame(2).is_none());
        assert_eq!(atlas.frame(1).unwrap().pixels().len(), 4);
    }

    #[test]
    fn animator_loops_and_finishes() {
        let mut looped = Animator::new(1.0, 3, true);
        looped.advance(1.0);
        assert_eq!(looped.index(), 1);
        looped.advance(2.0);
        assert_eq!(looped.index(), 0, "wrapped around");

        let mut once = Animator::new(1.0, 2, false);
        once.advance(5.0);
        assert_eq!(once.index(), 1);
        assert!(once.is_finished());
        once.restart();
        assert_eq!(once.index(), 0);
    }
}
