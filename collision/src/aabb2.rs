//! Axis-aligned bounding box in 2D (Aabb2).

/// Axis-aligned bounding box in 2D (min and max corners).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb2 {
    /// Minimum corner (x, y).
    pub min: [f32; 2],
    /// Maximum corner (x, y).
    pub max: [f32; 2],
}

impl Aabb2 {
    /// Creates an Aabb2 from min and max corners.
    #[must_use]
    pub const fn new(min: [f32; 2], max: [f32; 2]) -> Self {
        Self { min, max }
    }

    /// Creates an Aabb2 from center and half-extents (half width and half height).
    #[must_use]
    pub fn from_center_half_extents(center: [f32; 2], half_extents: [f32; 2]) -> Self {
        Self::new(
            [center[0] - half_extents[0], center[1] - half_extents[1]],
            [center[0] + half_extents[0], center[1] + half_extents[1]],
        )
    }

    /// Returns the four corners of the Aabb2 (min first, then max and combinations).
    #[must_use]
    pub fn corners(&self) -> [[f32; 2]; 4] {
        let [mx, my] = self.min;
        let [max_x, max_y] = self.max;
        [[mx, my], [max_x, my], [max_x, max_y], [mx, max_y]]
    }

    /// Returns true if this Aabb2 overlaps the other (intersection on both axes).
    #[must_use]
    pub fn intersects(&self, other: &Aabb2) -> bool {
        self.min[0] <= other.max[0]
            && self.max[0] >= other.min[0]
            && self.min[1] <= other.max[1]
            && self.max[1] >= other.min[1]
    }

    /// Returns true if this Aabb2 fully contains the other.
    #[must_use]
    pub fn contains(&self, other: &Aabb2) -> bool {
        self.min[0] <= other.min[0]
            && self.max[0] >= other.max[0]
            && self.min[1] <= other.min[1]
            && self.max[1] >= other.max[1]
    }

    /// Returns a new Aabb2 expanded by `margin` on both axes (min -= margin, max += margin).
    /// Useful for conservative bounds (e.g. "loosened" bounds).
    #[must_use]
    pub fn expand(&self, margin: f32) -> Aabb2 {
        Aabb2::new(
            [self.min[0] - margin, self.min[1] - margin],
            [self.max[0] + margin, self.max[1] + margin],
        )
    }

    /// Returns the union of this Aabb2 with another (smallest Aabb2 containing both).
    #[must_use]
    pub fn union(&self, other: &Aabb2) -> Aabb2 {
        #[cfg(feature = "simd")]
        {
            use wide::f32x4;
            let min_a = f32x4::from([self.min[0], self.min[1], other.min[0], other.min[1]]);
            let min_b = f32x4::from([other.min[0], other.min[1], self.min[0], self.min[1]]);
            let max_a = f32x4::from([self.max[0], self.max[1], other.max[0], other.max[1]]);
            let max_b = f32x4::from([other.max[0], other.max[1], self.max[0], self.max[1]]);
            let out_min = min_a.fast_min(min_b);
            let out_max = max_a.fast_max(max_b);
            let min_arr: [f32; 4] = out_min.to_array();
            let max_arr: [f32; 4] = out_max.to_array();
            Aabb2::new(
                [min_arr[0].min(min_arr[2]), min_arr[1].min(min_arr[3])],
                [max_arr[0].max(max_arr[2]), max_arr[1].max(max_arr[3])],
            )
        }
        #[cfg(not(feature = "simd"))]
        {
            let min = [self.min[0].min(other.min[0]), self.min[1].min(other.min[1])];
            let max = [self.max[0].max(other.max[0]), self.max[1].max(other.max[1])];
            Aabb2::new(min, max)
        }
    }
}

/// Ray–Aabb2 intersection (2D slab method).
///
/// Returns the smallest non-negative `t` such that `origin + t * dir` lies in the Aabb2,
/// or `None` if the ray does not hit the box (or hits only behind the origin).
#[must_use]
pub fn ray_aabb_2d(origin: &[f32; 2], dir: &[f32; 2], aabb: &Aabb2) -> Option<f32> {
    let eps = 1e-8;
    let (t_min_x, t_max_x) = if dir[0].abs() < eps {
        if origin[0] < aabb.min[0] || origin[0] > aabb.max[0] {
            return None;
        }
        (f32::NEG_INFINITY, f32::INFINITY)
    } else {
        let inv = 1.0 / dir[0];
        let t0 = (aabb.min[0] - origin[0]) * inv;
        let t1 = (aabb.max[0] - origin[0]) * inv;
        (t0.min(t1), t0.max(t1))
    };
    let (t_min_y, t_max_y) = if dir[1].abs() < eps {
        if origin[1] < aabb.min[1] || origin[1] > aabb.max[1] {
            return None;
        }
        (f32::NEG_INFINITY, f32::INFINITY)
    } else {
        let inv = 1.0 / dir[1];
        let t0 = (aabb.min[1] - origin[1]) * inv;
        let t1 = (aabb.max[1] - origin[1]) * inv;
        (t0.min(t1), t0.max(t1))
    };
    let t_min = t_min_x.max(t_min_y);
    let t_max = t_max_x.min(t_max_y);
    if t_min <= t_max && t_max >= 0.0 {
        let t = if t_min >= 0.0 { t_min } else { t_max };
        Some(t)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{ray_aabb_2d, Aabb2};

    #[test]
    fn aabb2_new_corners() {
        let a = Aabb2::new([0.0, 0.0], [1.0, 1.0]);
        let c = a.corners();
        assert_eq!(c[0], [0.0, 0.0]);
        assert_eq!(c[2], [1.0, 1.0]);
    }

    #[test]
    fn aabb2_union() {
        let a = Aabb2::new([0.0, 0.0], [1.0, 1.0]);
        let b = Aabb2::new([2.0, 0.0], [3.0, 1.0]);
        let u = a.union(&b);
        assert_eq!(u.min, [0.0, 0.0]);
        assert_eq!(u.max, [3.0, 1.0]);
    }

    #[test]
    fn ray_aabb_2d_hit() {
        let aabb = Aabb2::new([0.0, 0.0], [1.0, 1.0]);
        let origin = [0.5, 2.0];
        let dir = [0.0, -1.0];
        let t = ray_aabb_2d(&origin, &dir, &aabb).unwrap();
        assert!((t - 1.0).abs() < 1e-5);
    }

    #[test]
    fn ray_aabb_2d_miss() {
        let aabb = Aabb2::new([0.0, 0.0], [1.0, 1.0]);
        let origin = [2.0, 2.0];
        let dir = [1.0, 0.0];
        assert!(ray_aabb_2d(&origin, &dir, &aabb).is_none());
    }

    #[test]
    fn aabb2_intersects() {
        let a = Aabb2::new([0.0, 0.0], [2.0, 2.0]);
        let b = Aabb2::new([1.0, 1.0], [3.0, 3.0]);
        assert!(a.intersects(&b));
        let c = Aabb2::new([3.0, 0.0], [5.0, 2.0]);
        assert!(!a.intersects(&c));
    }

    #[test]
    fn aabb2_contains() {
        let outer = Aabb2::new([0.0, 0.0], [4.0, 4.0]);
        let inner = Aabb2::new([1.0, 1.0], [3.0, 3.0]);
        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn aabb2_expand() {
        let aabb = Aabb2::new([1.0, 2.0], [4.0, 5.0]);
        let expanded = aabb.expand(0.5);
        assert_eq!(expanded.min, [0.5, 1.5]);
        assert_eq!(expanded.max, [4.5, 5.5]);
    }

    #[test]
    fn aabb2_from_center_half_extents() {
        let aabb = Aabb2::from_center_half_extents([1.0, 2.0], [0.5, 1.0]);
        assert_eq!(aabb.min, [0.5, 1.0]);
        assert_eq!(aabb.max, [1.5, 3.0]);
    }
}
