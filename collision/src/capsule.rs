//! Capsule: line segment plus radius; ray intersection.

use crate::Aabb;

/// Ray–capsule intersection. Returns the smallest non-negative `t` or `None`.
#[must_use]
pub fn ray_capsule(origin: &[f32; 3], dir: &[f32; 3], capsule: &Capsule) -> Option<f32> {
    capsule.ray_intersect(origin, dir)
}

/// Capsule: segment from `a` to `b` with radius.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Capsule {
    /// Segment start (x, y, z).
    pub a: [f32; 3],
    /// Segment end (x, y, z).
    pub b: [f32; 3],
    /// Radius (non-negative).
    pub radius: f32,
}

impl Capsule {
    /// Creates a capsule from segment endpoints and radius.
    #[must_use]
    pub const fn new(a: [f32; 3], b: [f32; 3], radius: f32) -> Self {
        Self { a, b, radius }
    }

    /// Returns an AABB that bounds this capsule.
    #[must_use]
    pub fn aabb(&self) -> Aabb {
        let min = [
            self.a[0].min(self.b[0]) - self.radius,
            self.a[1].min(self.b[1]) - self.radius,
            self.a[2].min(self.b[2]) - self.radius,
        ];
        let max = [
            self.a[0].max(self.b[0]) + self.radius,
            self.a[1].max(self.b[1]) + self.radius,
            self.a[2].max(self.b[2]) + self.radius,
        ];
        Aabb::new(min, max)
    }

    /// Ray–capsule intersection: treat capsule as segment plus radius (distance from segment).
    /// Returns smallest non-negative `t` or `None`.
    #[must_use]
    pub fn ray_intersect(&self, origin: &[f32; 3], dir: &[f32; 3]) -> Option<f32> {
        let t_seg = crate::ray::ray_segment(origin, dir, &self.a, &self.b)?;
        let hit = [
            origin[0] + t_seg * dir[0],
            origin[1] + t_seg * dir[1],
            origin[2] + t_seg * dir[2],
        ];
        let pa = [hit[0] - self.a[0], hit[1] - self.a[1], hit[2] - self.a[2]];
        let ba = [
            self.b[0] - self.a[0],
            self.b[1] - self.a[1],
            self.b[2] - self.a[2],
        ];
        let ba_len_sq = ba[0] * ba[0] + ba[1] * ba[1] + ba[2] * ba[2];
        let eps = 1e-10;
        if ba_len_sq < eps {
            let ca = [
                origin[0] - self.a[0],
                origin[1] - self.a[1],
                origin[2] - self.a[2],
            ];
            let dist_sq = ca[0] * ca[0] + ca[1] * ca[1] + ca[2] * ca[2];
            let r_sq = self.radius * self.radius;
            if dist_sq <= r_sq {
                return Some(0.0);
            }
            return None;
        }
        let h = (pa[0] * ba[0] + pa[1] * ba[1] + pa[2] * ba[2]) / ba_len_sq;
        let h = h.clamp(0.0, 1.0);
        let closest = [
            self.a[0] + h * ba[0],
            self.a[1] + h * ba[1],
            self.a[2] + h * ba[2],
        ];
        let to_closest = [
            closest[0] - origin[0],
            closest[1] - origin[1],
            closest[2] - origin[2],
        ];
        let dir_len_sq = dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2];
        if dir_len_sq < eps {
            let d_sq = to_closest[0] * to_closest[0]
                + to_closest[1] * to_closest[1]
                + to_closest[2] * to_closest[2];
            if d_sq <= self.radius * self.radius {
                return Some(0.0);
            }
            return None;
        }
        let proj =
            (to_closest[0] * dir[0] + to_closest[1] * dir[1] + to_closest[2] * dir[2]) / dir_len_sq;
        if proj < 0.0 {
            let d_sq = to_closest[0] * to_closest[0]
                + to_closest[1] * to_closest[1]
                + to_closest[2] * to_closest[2];
            if d_sq <= self.radius * self.radius {
                return Some(0.0);
            }
            return None;
        }
        let t = proj;
        let pt = [
            origin[0] + t * dir[0] - closest[0],
            origin[1] + t * dir[1] - closest[1],
            origin[2] + t * dir[2] - closest[2],
        ];
        let dist_sq = pt[0] * pt[0] + pt[1] * pt[1] + pt[2] * pt[2];
        if dist_sq <= self.radius * self.radius {
            Some(t)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Capsule;
    use crate::ray_capsule;

    #[test]
    fn capsule_aabb() {
        let c = Capsule::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.5);
        let a = c.aabb();
        assert_eq!(a.min[0], -0.5);
        assert_eq!(a.max[0], 1.5);
    }

    #[test]
    fn ray_capsule_hit() {
        let cap = Capsule::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.5);
        let origin = [0.5, 0.0, 2.0];
        let dir = [0.0, 0.0, -1.0];
        let t = ray_capsule(&origin, &dir, &cap).unwrap();
        assert!(t > 0.0 && t < 3.0);
    }

    #[test]
    fn ray_capsule_miss() {
        let cap = Capsule::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.1);
        let origin = [5.0, 5.0, 1.0];
        let dir = [0.0, 0.0, -1.0];
        assert!(ray_capsule(&origin, &dir, &cap).is_none());
    }
}
