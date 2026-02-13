//! View frustum from view–projection matrix; AABB intersection test.

use mathlib::cg::matrix4f_to_array;
use mathlib::math3d::Matrix4f;

use crate::Aabb;

/// View frustum (six planes from view * projection matrix).
/// Plane equation: nx*x + ny*y + nz*z + d = 0; inside when value >= 0.
#[derive(Clone, Debug)]
pub struct Frustum {
    /// Left, right, bottom, top, near, far (each plane: [nx, ny, nz, d]).
    planes: [[f32; 4]; 6],
}

impl Frustum {
    /// Builds a frustum from the combined view–projection matrix (view * proj).
    /// Matrix is column-major. Planes are extracted so that points inside the
    /// view volume have non-negative signed distance.
    #[must_use]
    pub fn from_view_proj(view_proj: &Matrix4f) -> Self {
        let m = matrix4f_to_array(view_proj);
        let left = [m[3] + m[0], m[7] + m[4], m[11] + m[8], m[15] + m[12]];
        let right = [m[3] - m[0], m[7] - m[4], m[11] - m[8], m[15] - m[12]];
        let bottom = [m[3] + m[1], m[7] + m[5], m[11] + m[9], m[15] + m[13]];
        let top = [m[3] - m[1], m[7] - m[5], m[11] - m[9], m[15] - m[13]];
        let near = [m[3] + m[2], m[7] + m[6], m[11] + m[10], m[15] + m[14]];
        let far = [m[3] - m[2], m[7] - m[6], m[11] - m[10], m[15] - m[14]];
        Self {
            planes: [left, right, bottom, top, near, far],
        }
    }

    /// Returns true if the AABB intersects the frustum (not fully outside).
    /// Uses the "positive vertex" test per plane.
    #[must_use]
    pub fn intersects_aabb(&self, aabb: &Aabb) -> bool {
        #[cfg(feature = "simd")]
        {
            use wide::{f32x4, CmpGt};
            let min_vec = f32x4::from([aabb.min[0], aabb.min[1], aabb.min[2], 1.0]);
            let max_vec = f32x4::from([aabb.max[0], aabb.max[1], aabb.max[2], 1.0]);
            for plane in &self.planes {
                let p = f32x4::from(*plane);
                let mask = p.cmp_gt(f32x4::ZERO);
                let vertex = mask.blend(max_vec, min_vec);
                let dot = (vertex * p).reduce_add();
                if dot < 0.0 {
                    return false;
                }
            }
            true
        }
        #[cfg(not(feature = "simd"))]
        {
            for plane in &self.planes {
                let [nx, ny, nz, d] = *plane;
                let px = if nx >= 0.0 { aabb.max[0] } else { aabb.min[0] };
                let py = if ny >= 0.0 { aabb.max[1] } else { aabb.min[1] };
                let pz = if nz >= 0.0 { aabb.max[2] } else { aabb.min[2] };
                if nx * px + ny * py + nz * pz + d < 0.0 {
                    return false;
                }
            }
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Frustum;
    use crate::Aabb;
    use mathlib::cg::matrix4f_identity;

    #[test]
    fn frustum_identity_contains_origin_aabb() {
        let view_proj = matrix4f_identity();
        let frustum = Frustum::from_view_proj(&view_proj);
        let aabb = Aabb::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        assert!(frustum.intersects_aabb(&aabb));
    }

    #[test]
    fn frustum_aabb_outside() {
        let view_proj = matrix4f_identity();
        let frustum = Frustum::from_view_proj(&view_proj);
        // AABB entirely outside the identity NDC volume (e.g. far away)
        let aabb = Aabb::new([10.0, 10.0, 10.0], [11.0, 11.0, 11.0]);
        assert!(!frustum.intersects_aabb(&aabb));
    }
}
