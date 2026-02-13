//! Axis-aligned bounding box (AABB).

/// Axis-aligned bounding box (min and max corners).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    /// Minimum corner (x, y, z).
    pub min: [f32; 3],
    /// Maximum corner (x, y, z).
    pub max: [f32; 3],
}

impl Aabb {
    /// Creates an AABB from min and max corners.
    #[must_use]
    pub const fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    /// Returns the eight corners of the AABB (min first, then max and combinations).
    #[must_use]
    pub fn corners(&self) -> [[f32; 3]; 8] {
        let [mx, my, mz] = self.min;
        let [max_x, max_y, max_z] = self.max;
        [
            [mx, my, mz],
            [max_x, my, mz],
            [max_x, max_y, mz],
            [mx, max_y, mz],
            [mx, my, max_z],
            [max_x, my, max_z],
            [max_x, max_y, max_z],
            [mx, max_y, max_z],
        ]
    }

    /// Returns the union of this AABB with another (smallest AABB containing both).
    #[must_use]
    pub fn union(&self, other: &Aabb) -> Aabb {
        #[cfg(feature = "simd")]
        {
            use wide::f32x4;
            let min_a = f32x4::from([self.min[0], self.min[1], self.min[2], 0.0]);
            let min_b = f32x4::from([other.min[0], other.min[1], other.min[2], 0.0]);
            let max_a = f32x4::from([self.max[0], self.max[1], self.max[2], 0.0]);
            let max_b = f32x4::from([other.max[0], other.max[1], other.max[2], 0.0]);
            let out_min = min_a.fast_min(min_b);
            let out_max = max_a.fast_max(max_b);
            let min: [f32; 4] = out_min.to_array();
            let max: [f32; 4] = out_max.to_array();
            Aabb::new([min[0], min[1], min[2]], [max[0], max[1], max[2]])
        }
        #[cfg(not(feature = "simd"))]
        {
            let min = [
                self.min[0].min(other.min[0]),
                self.min[1].min(other.min[1]),
                self.min[2].min(other.min[2]),
            ];
            let max = [
                self.max[0].max(other.max[0]),
                self.max[1].max(other.max[1]),
                self.max[2].max(other.max[2]),
            ];
            Aabb::new(min, max)
        }
    }

    /// Returns the union of multiple AABBs, or an empty-like AABB if the iterator is empty.
    #[must_use]
    pub fn union_many(aabbs: impl Iterator<Item = Aabb>) -> Option<Aabb> {
        let mut it = aabbs;
        let first = it.next()?;
        Some(it.fold(first, |acc, a| acc.union(&a)))
    }

    /// Returns the center (midpoint) of the AABB.
    ///
    /// # Examples
    ///
    /// ```
    /// use collision::Aabb;
    /// let aabb = Aabb::new([0.0, 0.0, 0.0], [2.0, 4.0, 6.0]);
    /// assert_eq!(aabb.center(), [1.0, 2.0, 3.0]);
    /// ```
    #[must_use]
    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    /// Returns the half-extents (half the size along each axis).
    ///
    /// Useful for surface area and SAH cost: surface area = 2 * (wx*hy + wx*hz + hy*hz)
    /// where (wx, hy, hz) = half_extents().
    ///
    /// # Examples
    ///
    /// ```
    /// use collision::Aabb;
    /// let aabb = Aabb::new([0.0, 0.0, 0.0], [2.0, 4.0, 6.0]);
    /// assert_eq!(aabb.half_extents(), [1.0, 2.0, 3.0]);
    /// ```
    #[must_use]
    pub fn half_extents(&self) -> [f32; 3] {
        [
            (self.max[0] - self.min[0]) * 0.5,
            (self.max[1] - self.min[1]) * 0.5,
            (self.max[2] - self.min[2]) * 0.5,
        ]
    }

    /// Returns a new AABB expanded by `margin` on all axes.
    ///
    /// Min corner is shifted by `-margin`, max corner by `+margin` on each axis.
    /// Useful for conservative bounds and padding.
    ///
    /// # Examples
    ///
    /// ```
    /// use collision::Aabb;
    /// let aabb = Aabb::new([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
    /// let expanded = aabb.expand(0.5);
    /// assert_eq!(expanded.min, [0.5, 1.5, 2.5]);
    /// assert_eq!(expanded.max, [4.5, 5.5, 6.5]);
    /// ```
    #[must_use]
    pub fn expand(&self, margin: f32) -> Aabb {
        Aabb::new(
            [
                self.min[0] - margin,
                self.min[1] - margin,
                self.min[2] - margin,
            ],
            [
                self.max[0] + margin,
                self.max[1] + margin,
                self.max[2] + margin,
            ],
        )
    }

    /// Returns true if this AABB overlaps the other (intersection on all three axes).
    ///
    /// # Examples
    ///
    /// ```
    /// use collision::Aabb;
    /// let a = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    /// let b = Aabb::new([0.5, 0.5, 0.5], [1.5, 1.5, 1.5]);
    /// assert!(a.intersects(&b));
    /// let c = Aabb::new([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]);
    /// assert!(!a.intersects(&c));
    /// ```
    #[must_use]
    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min[0] <= other.max[0]
            && self.max[0] >= other.min[0]
            && self.min[1] <= other.max[1]
            && self.max[1] >= other.min[1]
            && self.min[2] <= other.max[2]
            && self.max[2] >= other.min[2]
    }
}

/// Ray–AABB intersection (slab method).
///
/// Returns the smallest non-negative `t` such that `origin + t * dir` lies in the AABB,
/// or `None` if the ray does not hit the box (or hits only behind the origin).
#[must_use]
pub fn ray_aabb(origin: &[f32; 3], dir: &[f32; 3], aabb: &Aabb) -> Option<f32> {
    #[cfg(feature = "simd")]
    {
        const EPS: f32 = 1e-8;
        if dir[0].abs() < EPS || dir[1].abs() < EPS || dir[2].abs() < EPS {
            return ray_aabb_scalar(origin, dir, aabb);
        }
        use wide::f32x4;
        let o = f32x4::from([origin[0], origin[1], origin[2], 0.0]);
        let bmin = f32x4::from([aabb.min[0], aabb.min[1], aabb.min[2], 0.0]);
        let bmax = f32x4::from([aabb.max[0], aabb.max[1], aabb.max[2], 0.0]);
        let inv_d = f32x4::from([1.0 / dir[0], 1.0 / dir[1], 1.0 / dir[2], 0.0]);
        let t0 = (bmin - o) * inv_d;
        let t1 = (bmax - o) * inv_d;
        let t_min_v = t0.fast_min(t1);
        let t_max_v = t0.fast_max(t1);
        let t_min_arr: [f32; 4] = t_min_v.to_array();
        let t_max_arr: [f32; 4] = t_max_v.to_array();
        let t_min = t_min_arr[0].max(t_min_arr[1]).max(t_min_arr[2]);
        let t_max = t_max_arr[0].min(t_max_arr[1]).min(t_max_arr[2]);
        if t_min <= t_max && t_max >= 0.0 {
            let t = if t_min >= 0.0 { t_min } else { t_max };
            Some(t)
        } else {
            None
        }
    }
    #[cfg(not(feature = "simd"))]
    {
        ray_aabb_scalar(origin, dir, aabb)
    }
}

#[inline]
fn ray_aabb_scalar(origin: &[f32; 3], dir: &[f32; 3], aabb: &Aabb) -> Option<f32> {
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
    let (t_min_z, t_max_z) = if dir[2].abs() < eps {
        if origin[2] < aabb.min[2] || origin[2] > aabb.max[2] {
            return None;
        }
        (f32::NEG_INFINITY, f32::INFINITY)
    } else {
        let inv = 1.0 / dir[2];
        let t0 = (aabb.min[2] - origin[2]) * inv;
        let t1 = (aabb.max[2] - origin[2]) * inv;
        (t0.min(t1), t0.max(t1))
    };
    let t_min = t_min_x.max(t_min_y).max(t_min_z);
    let t_max = t_max_x.min(t_max_y).min(t_max_z);
    if t_min <= t_max && t_max >= 0.0 {
        let t = if t_min >= 0.0 { t_min } else { t_max };
        Some(t)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{ray_aabb, Aabb};

    #[test]
    fn aabb_new_corners() {
        let a = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let c = a.corners();
        assert_eq!(c[0], [0.0, 0.0, 0.0]);
        assert_eq!(c[6], [1.0, 1.0, 1.0]);
    }

    #[test]
    fn aabb_union() {
        let a = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = Aabb::new([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]);
        let u = a.union(&b);
        assert_eq!(u.min, [0.0, 0.0, 0.0]);
        assert_eq!(u.max, [3.0, 1.0, 1.0]);
    }

    #[test]
    fn aabb_union_many_empty() {
        let v: Vec<Aabb> = vec![];
        assert!(Aabb::union_many(v.into_iter()).is_none());
    }

    #[test]
    fn aabb_union_many_single() {
        let a = Aabb::new([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        let u = Aabb::union_many([a].into_iter()).unwrap();
        assert_eq!(u.min, a.min);
        assert_eq!(u.max, a.max);
    }

    #[test]
    fn aabb_union_many_two() {
        let a = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = Aabb::new([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]);
        let u = Aabb::union_many([a, b].into_iter()).unwrap();
        assert_eq!(u.min, [0.0, 0.0, 0.0]);
        assert_eq!(u.max, [3.0, 1.0, 1.0]);
    }

    #[test]
    fn aabb_union_many_three() {
        let a = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = Aabb::new([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]);
        let c = Aabb::new([0.0, 2.0, 0.0], [1.0, 3.0, 1.0]);
        let u = Aabb::union_many([a, b, c].into_iter()).unwrap();
        assert_eq!(u.min, [0.0, 0.0, 0.0]);
        assert_eq!(u.max, [3.0, 3.0, 1.0]);
    }

    #[test]
    fn ray_aabb_hit() {
        let aabb = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let origin = [0.5, 0.5, 2.0];
        let dir = [0.0, 0.0, -1.0];
        let t = ray_aabb(&origin, &dir, &aabb).unwrap();
        assert!((t - 1.0).abs() < 1e-5);
    }

    #[test]
    fn ray_aabb_miss() {
        let aabb = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let origin = [2.0, 2.0, 2.0];
        let dir = [1.0, 0.0, 0.0];
        assert!(ray_aabb(&origin, &dir, &aabb).is_none());
    }

    #[test]
    fn ray_aabb_parallel_axis() {
        let aabb = Aabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let origin = [0.5, 0.5, -1.0];
        let dir = [0.0, 0.0, 1.0];
        let t = ray_aabb(&origin, &dir, &aabb).unwrap();
        assert!(t > 0.0 && t < 2.0);
    }

    #[test]
    fn aabb_center() {
        let aabb = Aabb::new([0.0, 0.0, 0.0], [2.0, 4.0, 6.0]);
        assert_eq!(aabb.center(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn aabb_half_extents() {
        let aabb = Aabb::new([0.0, 0.0, 0.0], [2.0, 4.0, 6.0]);
        assert_eq!(aabb.half_extents(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn aabb_expand() {
        let aabb = Aabb::new([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        let expanded = aabb.expand(0.5);
        assert_eq!(expanded.min, [0.5, 1.5, 2.5]);
        assert_eq!(expanded.max, [4.5, 5.5, 6.5]);
    }
}
