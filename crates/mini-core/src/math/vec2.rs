use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

/// 2D vector. Screen space: +x is right, +y is **down**.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Unit vector; stays ZERO if the vector has no length.
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len > f32::EPSILON {
            self / len
        } else {
            Self::ZERO
        }
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }

    /// Restrict the magnitude to `max` (cheap velocity clamp).
    pub fn clamp_length(self, max: f32) -> Self {
        let len = self.length();
        if len > max && len > 0.0 {
            self * (max / len)
        } else {
            self
        }
    }

    /// Unit vector at `radians` (0 = +x; π/2 points *down*, since +y is down).
    pub fn from_angle(radians: f32) -> Self {
        Self::new(radians.cos(), radians.sin())
    }

    /// 90° counter-clockwise in math terms (visually clockwise on screen).
    pub fn perpendicular(self) -> Self {
        Self::new(-self.y, self.x)
    }

    /// Rotate by `radians`.
    pub fn rotated(self, radians: f32) -> Self {
        let (s, c) = radians.sin_cos();
        Self::new(self.x * c - self.y * s, self.x * s + self.y * c)
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x + o.x, self.y + o.y)
    }
}
impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x - o.x, self.y - o.y)
    }
}
impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, s: f32) -> Vec2 {
        Vec2::new(self.x * s, self.y * s)
    }
}
impl Div<f32> for Vec2 {
    type Output = Vec2;
    fn div(self, s: f32) -> Vec2 {
        Vec2::new(self.x / s, self.y / s)
    }
}
impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}
impl AddAssign for Vec2 {
    fn add_assign(&mut self, o: Vec2) {
        *self = *self + o;
    }
}
impl SubAssign for Vec2 {
    fn sub_assign(&mut self, o: Vec2) {
        *self = *self - o;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_and_handles_zero() {
        assert!((Vec2::new(3.0, 4.0).normalized().length() - 1.0).abs() < 1e-5);
        assert_eq!(Vec2::ZERO.normalized(), Vec2::ZERO);
    }

    #[test]
    fn ops_work() {
        let v = Vec2::new(1.0, 2.0) + Vec2::new(3.0, 4.0);
        assert_eq!(v, Vec2::new(4.0, 6.0));
        assert_eq!(v * 2.0, Vec2::new(8.0, 12.0));
        assert_eq!(v - Vec2::ONE, Vec2::new(3.0, 5.0));
        assert_eq!(-v, Vec2::new(-4.0, -6.0));
    }

    #[test]
    fn angle_and_rotation() {
        let v = Vec2::from_angle(0.0);
        assert_eq!(v, Vec2::new(1.0, 0.0));
        let quarter = Vec2::new(1.0, 0.0).rotated(std::f32::consts::FRAC_PI_2);
        assert!((quarter.x.abs()) < 1e-6 && (quarter.y - 1.0).abs() < 1e-6);
        assert!((Vec2::new(1.0, 0.0).perpendicular() - Vec2::new(0.0, 1.0)).length() < 1e-6);
    }

    #[test]
    fn clamp_length_never_exceeds() {
        let v = Vec2::new(30.0, 40.0).clamp_length(10.0);
        assert!((v.length() - 10.0).abs() < 1e-5);
    }
}
