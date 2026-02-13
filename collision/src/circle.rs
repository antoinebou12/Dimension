//! Circle shape in 2D: center and radius.

use crate::aabb2::Aabb2;

/// Circle in 2D (center and radius).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Circle {
    /// Center (x, y).
    pub center: [f32; 2],
    /// Radius (non-negative).
    pub radius: f32,
}

impl Circle {
    /// Creates a circle from center and radius.
    #[must_use]
    pub const fn new(center: [f32; 2], radius: f32) -> Self {
        Self { center, radius }
    }

    /// Returns an Aabb2 that tightly bounds this circle.
    #[must_use]
    pub fn aabb(&self) -> Aabb2 {
        let r = self.radius;
        Aabb2::new(
            [self.center[0] - r, self.center[1] - r],
            [self.center[0] + r, self.center[1] + r],
        )
    }

    /// Returns true if the point lies inside or on the circle.
    #[must_use]
    pub fn contains_point(&self, p: &[f32; 2]) -> bool {
        let dx = p[0] - self.center[0];
        let dy = p[1] - self.center[1];
        dx * dx + dy * dy <= self.radius * self.radius
    }

    /// Ray–circle intersection in 2D.
    ///
    /// Returns the smallest non-negative `t` such that `origin + t * dir` lies on the circle,
    /// or `None` if the ray does not hit the circle (or hits only behind the origin).
    #[must_use]
    pub fn ray_intersect(&self, origin: &[f32; 2], dir: &[f32; 2]) -> Option<f32> {
        let oc_x = origin[0] - self.center[0];
        let oc_y = origin[1] - self.center[1];
        let a = dir[0] * dir[0] + dir[1] * dir[1];
        let half_b = oc_x * dir[0] + oc_y * dir[1];
        let c = oc_x * oc_x + oc_y * oc_y - self.radius * self.radius;
        let disc = half_b * half_b - a * c;
        if disc < 0.0 {
            return None;
        }
        let sqrt_d = disc.sqrt();
        let t = (-half_b - sqrt_d) / a;
        if t >= 0.0 {
            return Some(t);
        }
        let t = (-half_b + sqrt_d) / a;
        if t >= 0.0 {
            Some(t)
        } else {
            None
        }
    }
}

/// Ray–circle intersection in 2D.
///
/// Returns the smallest non-negative `t` such that `origin + t * dir` hits the circle,
/// or `None` if the ray does not hit (or hits only behind the origin).
#[must_use]
pub fn ray_circle_2d(origin: &[f32; 2], dir: &[f32; 2], circle: &Circle) -> Option<f32> {
    circle.ray_intersect(origin, dir)
}

#[cfg(test)]
mod tests {
    use super::{ray_circle_2d, Circle};

    #[test]
    fn circle_aabb() {
        let c = Circle::new([1.0, 2.0], 3.0);
        let aabb = c.aabb();
        assert_eq!(aabb.min, [-2.0, -1.0]);
        assert_eq!(aabb.max, [4.0, 5.0]);
    }

    #[test]
    fn circle_contains_point() {
        let c = Circle::new([0.0, 0.0], 2.0);
        assert!(c.contains_point(&[0.0, 0.0]));
        assert!(c.contains_point(&[1.0, 1.0]));
        assert!(!c.contains_point(&[3.0, 0.0]));
    }

    #[test]
    fn ray_circle_2d_hit() {
        let circle = Circle::new([0.0, 0.0], 1.0);
        let origin = [0.0, 2.0];
        let dir = [0.0, -1.0];
        let t = ray_circle_2d(&origin, &dir, &circle).unwrap();
        assert!((t - 1.0).abs() < 1e-5);
    }

    #[test]
    fn ray_circle_2d_miss() {
        let circle = Circle::new([0.0, 0.0], 1.0);
        let origin = [2.0, 2.0];
        let dir = [1.0, 0.0];
        assert!(ray_circle_2d(&origin, &dir, &circle).is_none());
    }
}
