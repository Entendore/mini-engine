use super::vec2::Vec2;

/// Axis-aligned rectangle. `x, y` is the top-left corner (y is down).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn from_center(center: Vec2, w: f32, h: f32) -> Self {
        Self { x: center.x - w * 0.5, y: center.y - h * 0.5, w, h }
    }

    pub fn center(self) -> Vec2 {
        Vec2::new(self.x + self.w * 0.5, self.y + self.h * 0.5)
    }

    pub fn min(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn max(self) -> Vec2 {
        Vec2::new(self.x + self.w, self.y + self.h)
    }

    pub fn contains(self, p: Vec2) -> bool {
        p.x >= self.x && p.x < self.x + self.w && p.y >= self.y && p.y < self.y + self.h
    }

    /// AABB vs AABB overlap.
    pub fn intersects(self, other: Rect) -> bool {
        self.x < other.x + other.w
            && other.x < self.x + self.w
            && self.y < other.y + other.h
            && other.y < self.y + self.h
    }

    /// Grow by `amount` on every side (negative amounts shrink).
    pub fn expand(self, amount: f32) -> Self {
        Self {
            x: self.x - amount,
            y: self.y - amount,
            w: self.w + amount * 2.0,
            h: self.h + amount * 2.0,
        }
    }

    pub fn translate(self, by: Vec2) -> Self {
        Self { x: self.x + by.x, y: self.y + by.y, ..self }
    }

    /// Does this rect fully contain `other`?
    pub fn contains_rect(self, other: Rect) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.x + other.w <= self.x + self.w
            && other.y + other.h <= self.y + self.h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aabb_intersection() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        assert!(a.intersects(Rect::new(5.0, 5.0, 10.0, 10.0)));
        assert!(!a.intersects(Rect::new(11.0, 0.0, 5.0, 5.0)));
    }

    #[test]
    fn center_and_contains() {
        let r = Rect::new(0.0, 0.0, 10.0, 10.0);
        assert_eq!(r.center(), Vec2::new(5.0, 5.0));
        assert!(r.contains(Vec2::new(0.0, 0.0)));
        assert!(!r.contains(Vec2::new(10.0, 5.0)), "right edge is exclusive");
        assert_eq!(Rect::from_center(r.center(), 10.0, 10.0), r);
    }

    #[test]
    fn expand_translate_contains_rect() {
        let r = Rect::new(10.0, 10.0, 10.0, 10.0);
        assert_eq!(r.expand(2.0), Rect::new(8.0, 8.0, 14.0, 14.0));
        assert_eq!(r.translate(Vec2::new(5.0, -5.0)), Rect::new(15.0, 5.0, 10.0, 10.0));
        assert!(r.contains_rect(r.expand(-2.0)));
        assert!(!r.contains_rect(r.expand(2.0)));
    }
}
