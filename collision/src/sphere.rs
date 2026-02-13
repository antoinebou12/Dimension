//! Bounding sphere: center and radius; ray intersection and AABB test.

use crate::Aabb;

/// Sphere (center and radius).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sphere {
    /// Center (x, y, z).
    pub center: [f32; 3],
    /// Radius (non-negative).
    pub radius: f32,
}

impl Sphere {
    /// Creates a sphere from center and radius.
    #[must_use]
    pub const fn new(center: [f32; 3], radius: f32) -> Self {
        Self { center, radius }
    }

    /// Returns an AABB that tightly bounds this sphere.
    #[must_use]
    pub fn aabb(&self) -> Aabb {
        let r = self.radius;
        Aabb::new(
            [self.center[0] - r, self.center[1] - r, self.center[2] - r],
            [self.center[0] + r, self.center[1] + r, self.center[2] + r],
        )
    }

    /// Returns true if this sphere intersects the given AABB.
    #[must_use]
    pub fn intersects_aabb(&self, aabb: &Aabb) -> bool {
        let cx = self.center[0].clamp(aabb.min[0], aabb.max[0]);
        let cy = self.center[1].clamp(aabb.min[1], aabb.max[1]);
        let cz = self.center[2].clamp(aabb.min[2], aabb.max[2]);
        let dx = self.center[0] - cx;
        let dy = self.center[1] - cy;
        let dz = self.center[2] - cz;
        let dist_sq = dx * dx + dy * dy + dz * dz;
        dist_sq <= self.radius * self.radius
    }

    /// Ray–sphere intersection. Returns the smallest non-negative `t` or `None`.
    #[must_use]
    pub fn ray_intersect(&self, origin: &[f32; 3], dir: &[f32; 3]) -> Option<f32> {
        let oc = [
            origin[0] - self.center[0],
            origin[1] - self.center[1],
            origin[2] - self.center[2],
        ];
        let a = dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2];
        let b = 2.0 * (oc[0] * dir[0] + oc[1] * dir[1] + oc[2] * dir[2]);
        let c = oc[0] * oc[0] + oc[1] * oc[1] + oc[2] * oc[2] - self.radius * self.radius;
        let disc = b * b - 4.0 * a * c;
        if disc < 0.0 {
            return None;
        }
        let sqrt_d = disc.sqrt();
        let t = (-b - sqrt_d) / (2.0 * a);
        if t >= 0.0 {
            Some(t)
        } else {
            let t2 = (-b + sqrt_d) / (2.0 * a);
            if t2 >= 0.0 {
                Some(t2)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Sphere;
    use crate::{ray_sphere, Aabb};

    #[test]
    fn sphere_aabb() {
        let s = Sphere::new([1.0, 2.0, 3.0], 0.5);
        let a = s.aabb();
        assert_eq!(a.min, [0.5, 1.5, 2.5]);
        assert_eq!(a.max, [1.5, 2.5, 3.5]);
    }

    #[test]
    fn sphere_ray_hit() {
        let s = Sphere::new([0.0, 0.0, 0.0], 1.0);
        let origin = [0.0, 0.0, 2.0];
        let dir = [0.0, 0.0, -1.0];
        let t = ray_sphere(&origin, &dir, &s).unwrap();
        assert!((t - 1.0).abs() < 1e-5);
    }

    #[test]
    fn sphere_ray_miss() {
        let s = Sphere::new([0.0, 0.0, 0.0], 1.0);
        let origin = [0.0, 0.0, 2.0];
        let dir = [0.0, 0.0, 1.0];
        assert!(ray_sphere(&origin, &dir, &s).is_none());
    }

    #[test]
    fn sphere_intersects_aabb_true() {
        let s = Sphere::new([1.0, 1.0, 1.0], 1.0);
        let aabb = Aabb::new([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        assert!(s.intersects_aabb(&aabb));
    }

    #[test]
    fn sphere_intersects_aabb_false() {
        let s = Sphere::new([5.0, 5.0, 5.0], 0.5);
        let aabb = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(!s.intersects_aabb(&aabb));
    }
}
