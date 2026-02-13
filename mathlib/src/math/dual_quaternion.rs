//! Dual quaternions for rigid transforms (rotation + translation).
//!
//! A dual quaternion \( \widehat{Q} = \widehat{q} + \epsilon \widehat{q}^0 \) with \( \epsilon^2 = 0 \)
//! represents a rigid motion: the real part \( \widehat{q} \) is a unit quaternion (rotation),
//! and the dual part encodes translation. Multiplication composes transforms.
//!
//! See [Quat4f](crate::quaternion::Quat4f) for rotation quaternions and
//! [`transform_point`](crate::cg::transform_point) for applying a 4×4 matrix to a point.
//!
//! # Examples
//!
//! ```
//! use mathlib::{DualQuat4f, Quat4f, Vector3f, cg::{vector3, transform_point}};
//!
//! let rot = Quat4f::identity();
//! let t = vector3(1.0, 0.0, 0.0);
//! let dq = DualQuat4f::from_rotation_and_translation(&rot, &t);
//! let m = dq.to_matrix4();
//! let p = vector3(0.0, 0.0, 0.0);
//! let out_dq = dq.transform_point(&p);
//! let out_m = transform_point(&m, &p);
//! assert!((out_dq.get(0) - out_m.get(0)).abs() < 1e-5);
//! ```

use super::math3d::{Matrix3f, Matrix4f, Vector3f};
use super::quaternion::Quat4f;
use crate::cg::{matrix4f_translation, transform_point};
use crate::types::Storage;
use std::ops::Mul;

/// Dual quaternion for rigid transforms: real part (rotation) + dual part (translation).
///
/// Stored as eight `f32`: `real_w`, `real_x`, `real_y`, `real_z` (real quaternion, scalar-first),
/// then `dual_w`, `dual_x`, `dual_y`, `dual_z` (dual quaternion). Aligned with libdq-style layout
/// for formulas (dual in our storage: `dual_w` = libdq Q[7], `dual_x,y,z` = libdq Q[4],Q[5],Q[6]).
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DualQuat4f {
    /// Real part: scalar (w).
    pub real_w: f32,
    /// Real part: i component.
    pub real_x: f32,
    /// Real part: j component.
    pub real_y: f32,
    /// Real part: k component.
    pub real_z: f32,
    /// Dual part: scalar (ε).
    pub dual_w: f32,
    /// Dual part: iε component.
    pub dual_x: f32,
    /// Dual part: jε component.
    pub dual_y: f32,
    /// Dual part: kε component.
    pub dual_z: f32,
}

impl DualQuat4f {
    /// Identity rigid transform (no rotation, no translation).
    #[must_use]
    pub fn identity() -> Self {
        Self {
            real_w: 1.0,
            real_x: 0.0,
            real_y: 0.0,
            real_z: 0.0,
            dual_w: 0.0,
            dual_x: 0.0,
            dual_y: 0.0,
            dual_z: 0.0,
        }
    }

    /// Build from a unit rotation quaternion and translation vector.
    ///
    /// Equivalent to libdq `dq_cr_homo`: DQ = `translation_dq` * `rotation_dq` so that
    /// applying the transform gives R*p + t.
    #[must_use]
    pub fn from_rotation_and_translation(rot: &Quat4f, t: &Vector3f) -> Self {
        assert!(t.rows() >= 3);
        // Translation DQ: real = (1,0,0,0), dual = (0, t/2) in (w, i, j, k)
        let q_t = Self {
            real_w: 1.0,
            real_x: 0.0,
            real_y: 0.0,
            real_z: 0.0,
            dual_w: 0.0,
            dual_x: t.get(0) * 0.5,
            dual_y: t.get(1) * 0.5,
            dual_z: t.get(2) * 0.5,
        };
        // Rotation DQ: real = rot, dual = 0
        let q_r = Self {
            real_w: rot.w,
            real_x: rot.x,
            real_y: rot.y,
            real_z: rot.z,
            dual_w: 0.0,
            dual_x: 0.0,
            dual_y: 0.0,
            dual_z: 0.0,
        };
        // Composed: first rotate then translate = q_t * q_r
        q_t * q_r
    }

    /// Build from a 3×3 rotation matrix and translation vector.
    #[must_use]
    pub fn from_rotation_matrix_and_translation(r: &Matrix3f, t: &Vector3f) -> Self {
        let q = Quat4f::from_rotation_matrix3(r);
        Self::from_rotation_and_translation(&q, t)
    }

    /// Build from a 4×4 homogeneous matrix (3×3 rotation block + translation column).
    #[must_use]
    pub fn from_matrix4(m: &Matrix4f) -> Self {
        let mut r = Matrix3f::with_storage(3, 3, Storage::Column);
        for i in 0..3 {
            for j in 0..3 {
                r.set(i, j, m.get(i, j));
            }
        }
        let t = matrix4f_translation(m);
        Self::from_rotation_matrix_and_translation(&r, &t)
    }

    /// Conjugate: real part quaternion conjugate, dual part conjugated with dual scalar sign flip (libdq `dq_cr_conj`).
    #[must_use]
    pub fn conjugate(self) -> Self {
        Self {
            real_w: self.real_w,
            real_x: -self.real_x,
            real_y: -self.real_y,
            real_z: -self.real_z,
            dual_w: self.dual_w,
            dual_x: -self.dual_x,
            dual_y: -self.dual_y,
            dual_z: -self.dual_z,
        }
    }

    /// Inverse. For unit dual quaternions this equals conjugate.
    #[must_use]
    pub fn inverse(self) -> Self {
        let (real, dual) = self.norm2();
        if real.abs() < 1e-20 {
            return self.conjugate();
        }
        let inv_real = 1.0 / real;
        let inv_real_sq = inv_real * inv_real;
        // General inverse: O = Q* / ||Q||^2; dual part = Q*_dual/real - dual*Q*_real/real^2
        Self {
            real_w: self.real_w * inv_real,
            real_x: -self.real_x * inv_real,
            real_y: -self.real_y * inv_real,
            real_z: -self.real_z * inv_real,
            dual_w: self.dual_w * inv_real - dual * self.real_w * inv_real_sq,
            dual_x: -self.dual_x * inv_real + dual * self.real_x * inv_real_sq,
            dual_y: -self.dual_y * inv_real + dual * self.real_y * inv_real_sq,
            dual_z: -self.dual_z * inv_real + dual * self.real_z * inv_real_sq,
        }
    }

    /// Squared norm as dual number: (`real_part`, `dual_part`). Unit DQ has real=1, dual=0.
    #[must_use]
    pub fn norm2(&self) -> (f32, f32) {
        let real = self.real_w * self.real_w
            + self.real_x * self.real_x
            + self.real_y * self.real_y
            + self.real_z * self.real_z;
        let dual = 2.0
            * (self.real_w * self.dual_w
                + self.real_x * self.dual_x
                + self.real_y * self.dual_y
                + self.real_z * self.dual_z);
        (real, dual)
    }

    /// Whether this is a unit dual quaternion (real part norm 1, dual part 0) within `precision`.
    #[must_use]
    pub fn is_unit(&self, precision: f32) -> bool {
        let (real, dual) = self.norm2();
        (real - 1.0).abs() <= precision && dual.abs() <= precision
    }

    /// Extract 4×4 rigid transform matrix (3×3 rotation + translation column).
    #[must_use]
    pub fn to_matrix4(&self) -> Matrix4f {
        let r = Quat4f {
            w: self.real_w,
            x: self.real_x,
            y: self.real_y,
            z: self.real_z,
        }
        .to_rotation_matrix3();
        let d0 = 2.0
            * (self.real_w * self.dual_x - self.real_x * self.dual_w + self.real_y * self.dual_z
                - self.real_z * self.dual_y);
        let d1 = 2.0
            * (self.real_w * self.dual_y - self.real_y * self.dual_w - self.real_x * self.dual_z
                + self.real_z * self.dual_x);
        let d2 = 2.0
            * (self.real_w * self.dual_z - self.real_z * self.dual_w + self.real_x * self.dual_y
                - self.real_y * self.dual_x);
        let mut m = Matrix4f::with_storage(4, 4, Storage::Column);
        m.set_zero();
        for i in 0..3 {
            for j in 0..3 {
                m.set(i, j, r.get(i, j));
            }
        }
        m.set(0, 3, d0);
        m.set(1, 3, d1);
        m.set(2, 3, d2);
        m.set(3, 3, 1.0);
        m
    }

    /// Extract rotation as quaternion and translation as vector.
    #[must_use]
    pub fn to_rotation_and_translation(&self) -> (Quat4f, Vector3f) {
        let q = Quat4f {
            w: self.real_w,
            x: self.real_x,
            y: self.real_y,
            z: self.real_z,
        };
        let m = self.to_matrix4();
        let t = matrix4f_translation(&m);
        (q, t)
    }

    /// Transform a 3D point: R*p + t.
    #[must_use]
    pub fn transform_point(&self, p: &Vector3f) -> Vector3f {
        let m = self.to_matrix4();
        transform_point(&m, p)
    }

    /// Transform many points. When the `parallel` feature is enabled (and not targeting wasm32),
    /// points are processed in parallel.
    #[must_use]
    pub fn batch_transform_points(&self, points: &[Vector3f]) -> Vec<Vector3f> {
        #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
        {
            use par_iter::prelude::*;
            points.par_iter().map(|p| self.transform_point(p)).collect()
        }

        #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
        {
            points.iter().map(|p| self.transform_point(p)).collect()
        }
    }
}

/// Quaternion multiply: left as (w,x,y,z) tuple, right as Quat4f. Returns (w,x,y,z).
fn quat_mul_dual(left: impl QuatLike, right: impl QuatLike) -> (f32, f32, f32, f32) {
    let (w1, x1, y1, z1) = left.to_wxyz();
    let (w2, x2, y2, z2) = right.to_wxyz();
    (
        w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2,
        w1 * x2 + x1 * w2 + y1 * z2 - z1 * y2,
        w1 * y2 - x1 * z2 + y1 * w2 + z1 * x2,
        w1 * z2 + x1 * y2 - y1 * x2 + z1 * w2,
    )
}

trait QuatLike {
    fn to_wxyz(self) -> (f32, f32, f32, f32);
}

impl QuatLike for Quat4f {
    fn to_wxyz(self) -> (f32, f32, f32, f32) {
        (self.w, self.x, self.y, self.z)
    }
}

impl QuatLike for (f32, f32, f32, f32) {
    fn to_wxyz(self) -> (f32, f32, f32, f32) {
        self
    }
}

impl Mul for DualQuat4f {
    type Output = Self;

    /// Dual quaternion product: `(P * Q).transform_point(p)` = `P.transform_point(Q.transform_point(p))`.
    fn mul(self, other: Self) -> Self {
        let p_real = Quat4f {
            w: self.real_w,
            x: self.real_x,
            y: self.real_y,
            z: self.real_z,
        };
        let q_real = Quat4f {
            w: other.real_w,
            x: other.real_x,
            y: other.real_y,
            z: other.real_z,
        };
        let real_product = p_real * q_real;
        // Dual part: p_real * q_dual + p_dual * q_real (quaternion products)
        let d1 = quat_mul_dual(
            p_real,
            (other.dual_w, other.dual_x, other.dual_y, other.dual_z),
        );
        let d2 = quat_mul_dual((self.dual_w, self.dual_x, self.dual_y, self.dual_z), q_real);
        Self {
            real_w: real_product.w,
            real_x: real_product.x,
            real_y: real_product.y,
            real_z: real_product.z,
            dual_w: d1.0 + d2.0,
            dual_x: d1.1 + d2.1,
            dual_y: d1.2 + d2.2,
            dual_z: d1.3 + d2.3,
        }
    }
}

impl Mul for &DualQuat4f {
    type Output = DualQuat4f;

    fn mul(self, other: Self) -> DualQuat4f {
        *self * *other
    }
}
