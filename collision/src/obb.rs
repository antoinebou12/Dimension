//! Oriented bounding box (OBB): center, half-extents, and rotation (3x3 column-major).

use crate::Aabb;

/// Oriented bounding box: center, half-extents along local axes, and 3x3 rotation (column-major).
#[derive(Clone, Debug, PartialEq)]
pub struct Obb {
    /// Center (x, y, z).
    pub center: [f32; 3],
    /// Half-extents along local X, Y, Z (positive).
    pub half_extents: [f32; 3],
    /// Rotation 3x3 matrix column-major: columns are local axes (right, up, forward).
    pub rotation: [f32; 9],
}

impl Obb {
    /// Creates an OBB from center, half-extents, and 3x3 rotation (column-major).
    #[must_use]
    pub const fn new(center: [f32; 3], half_extents: [f32; 3], rotation: [f32; 9]) -> Self {
        Self {
            center,
            half_extents,
            rotation,
        }
    }

    /// Returns an AABB that bounds this OBB in world space (conservative).
    #[must_use]
    pub fn aabb(&self) -> Aabb {
        let r = &self.rotation;
        let h = &self.half_extents;
        let ax = (
            r[0].abs() * h[0] + r[3].abs() * h[1] + r[6].abs() * h[2],
            r[1].abs() * h[0] + r[4].abs() * h[1] + r[7].abs() * h[2],
            r[2].abs() * h[0] + r[5].abs() * h[1] + r[8].abs() * h[2],
        );
        let min = [
            self.center[0] - ax.0,
            self.center[1] - ax.1,
            self.center[2] - ax.2,
        ];
        let max = [
            self.center[0] + ax.0,
            self.center[1] + ax.1,
            self.center[2] + ax.2,
        ];
        Aabb::new(min, max)
    }

    /// Ray–OBB intersection (slab in local space). Returns smallest non-negative `t` or `None`.
    #[must_use]
    pub fn ray_intersect(&self, origin: &[f32; 3], dir: &[f32; 3]) -> Option<f32> {
        let r = &self.rotation;
        let ox = origin[0] - self.center[0];
        let oy = origin[1] - self.center[1];
        let oz = origin[2] - self.center[2];
        let orig_local = [
            r[0] * ox + r[1] * oy + r[2] * oz,
            r[3] * ox + r[4] * oy + r[5] * oz,
            r[6] * ox + r[7] * oy + r[8] * oz,
        ];
        let dir_local = [
            r[0] * dir[0] + r[1] * dir[1] + r[2] * dir[2],
            r[3] * dir[0] + r[4] * dir[1] + r[5] * dir[2],
            r[6] * dir[0] + r[7] * dir[1] + r[8] * dir[2],
        ];
        let aabb = Aabb::new(
            [
                -self.half_extents[0],
                -self.half_extents[1],
                -self.half_extents[2],
            ],
            self.half_extents,
        );
        crate::aabb::ray_aabb(&orig_local, &dir_local, &aabb)
    }
}

#[cfg(test)]
mod tests {
    use super::Obb;
    use crate::ray_obb;

    #[test]
    fn obb_axis_aligned_ray() {
        // Identity rotation -> OBB is axis-aligned
        let rot = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        let obb = Obb::new([0.0, 0.0, 0.0], [0.5, 0.5, 0.5], rot);
        let origin = [0.0, 0.0, 2.0];
        let dir = [0.0, 0.0, -1.0];
        let t = ray_obb(&origin, &dir, &obb).unwrap();
        assert!(t > 0.0 && t < 3.0);
    }

    #[test]
    fn obb_ray_miss() {
        let rot = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        let obb = Obb::new([0.0, 0.0, 0.0], [0.5, 0.5, 0.5], rot);
        let origin = [0.0, 0.0, 2.0];
        let dir = [0.0, 0.0, 1.0];
        assert!(ray_obb(&origin, &dir, &obb).is_none());
    }

    #[test]
    fn obb_aabb_axis_aligned() {
        let rot = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        let obb = Obb::new([1.0, 2.0, 3.0], [0.5, 0.5, 0.5], rot);
        let aabb = obb.aabb();
        assert_eq!(aabb.min, [0.5, 1.5, 2.5]);
        assert_eq!(aabb.max, [1.5, 2.5, 3.5]);
    }
}
