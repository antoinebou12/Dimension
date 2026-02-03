//! Unit quaternion for 3D rotation (scalar-first: w, x, y, z).
//!
//! Use `from_axis_angle` to build from axis and angle (radians), then `to_rotation_matrix3` or
//! `to_rotation_matrix4` for matrix form.

use super::math3d::{Matrix3f, Matrix4f, Vector3f};
use crate::types::Storage;
use std::ops::Mul;

/// Quaternion with components (w, x, y, z); scalar w first.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Quat4f {
    /// Scalar part.
    pub w: f32,
    /// First imaginary component.
    pub x: f32,
    /// Second imaginary component.
    pub y: f32,
    /// Third imaginary component.
    pub z: f32,
}

impl Quat4f {
    /// Identity rotation (no rotation).
    #[must_use]
    pub fn identity() -> Self {
        Self {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Build unit quaternion from rotation axis (unit or not) and angle in radians.
    #[must_use]
    pub fn from_axis_angle(axis: &Vector3f, angle_rad: f32) -> Self {
        assert!(axis.rows() >= 3);
        let x = axis.get(0);
        let y = axis.get(1);
        let z = axis.get(2);
        let norm_sq = x * x + y * y + z * z;
        let half = angle_rad * 0.5;
        let w = half.cos();
        if norm_sq < 1e-14 {
            return Self {
                w,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
        }
        let s = half.sin() / norm_sq.sqrt();
        Self {
            w,
            x: x * s,
            y: y * s,
            z: z * s,
        }
    }

    /// Conjugate (inverse for unit quaternion): (w, -x, -y, -z).
    #[must_use]
    pub fn conjugate(&self) -> Self {
        Self {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    /// Normalized copy: q / ||q||.
    #[must_use]
    pub fn normalize(&self) -> Self {
        let n = (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if n < 1e-20 {
            return Self::identity();
        }
        Self {
            w: self.w / n,
            x: self.x / n,
            y: self.y / n,
            z: self.z / n,
        }
    }

    /// 3×3 rotation matrix (column-major).
    #[must_use]
    pub fn to_rotation_matrix3(&self) -> Matrix3f {
        let (w, x, y, z) = (self.w, self.x, self.y, self.z);
        let mut r = Matrix3f::with_storage(3, 3, Storage::Column);
        r.set(0, 0, 1.0 - 2.0 * (y * y + z * z));
        r.set(0, 1, 2.0 * (x * y - z * w));
        r.set(0, 2, 2.0 * (x * z + y * w));
        r.set(1, 0, 2.0 * (x * y + z * w));
        r.set(1, 1, 1.0 - 2.0 * (x * x + z * z));
        r.set(1, 2, 2.0 * (y * z - x * w));
        r.set(2, 0, 2.0 * (x * z - y * w));
        r.set(2, 1, 2.0 * (y * z + x * w));
        r.set(2, 2, 1.0 - 2.0 * (x * x + y * y));
        r
    }

    /// 4×4 rotation matrix (3×3 block + identity 4th row/col).
    #[must_use]
    pub fn to_rotation_matrix4(&self) -> Matrix4f {
        let r3 = self.to_rotation_matrix3();
        let mut m = Matrix4f::with_storage(4, 4, Storage::Column);
        m.set_zero();
        for i in 0..3 {
            for j in 0..3 {
                m.set(i, j, r3.get(i, j));
            }
        }
        m.set(3, 3, 1.0);
        m
    }

    /// Spherical linear interpolation between two unit quaternions (shortest path).
    /// Returns `self` at `t = 0` and `other` at `t = 1`.
    #[must_use]
    pub fn slerp(&self, other: &Self, t: f32) -> Self {
        let dot = self.w * other.w + self.x * other.x + self.y * other.y + self.z * other.z;
        let (other_w, other_x, other_y, other_z) = if dot < 0.0 {
            (-other.w, -other.x, -other.y, -other.z)
        } else {
            (other.w, other.x, other.y, other.z)
        };
        let dot = self.w * other_w + self.x * other_x + self.y * other_y + self.z * other_z;
        let dot = dot.clamp(-1.0, 1.0);
        if dot > 0.9995 {
            let u = 1.0 - t;
            Self {
                w: u * self.w + t * other_w,
                x: u * self.x + t * other_x,
                y: u * self.y + t * other_y,
                z: u * self.z + t * other_z,
            }
            .normalize()
        } else {
            let theta_0 = dot.acos();
            let theta = theta_0 * t;
            let sin_theta = theta.sin();
            let sin_theta_0 = theta_0.sin();
            let s0 = (theta_0 - theta).cos() / sin_theta_0;
            let s1 = sin_theta / sin_theta_0;
            Self {
                w: s0 * self.w + s1 * other_w,
                x: s0 * self.x + s1 * other_x,
                y: s0 * self.y + s1 * other_y,
                z: s0 * self.z + s1 * other_z,
            }
        }
    }
}

impl Mul for Quat4f {
    type Output = Quat4f;

    fn mul(self, other: Quat4f) -> Quat4f {
        let (w1, x1, y1, z1) = (self.w, self.x, self.y, self.z);
        let (w2, x2, y2, z2) = (other.w, other.x, other.y, other.z);
        Quat4f {
            w: w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2,
            x: w1 * x2 + x1 * w2 + y1 * z2 - z1 * y2,
            y: w1 * y2 - x1 * z2 + y1 * w2 + z1 * x2,
            z: w1 * z2 + x1 * y2 - y1 * x2 + z1 * w2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn axis_y() -> Vector3f {
        let mut v = Vector3f::with_capacity(3);
        v.set(0, 0.0);
        v.set(1, 1.0);
        v.set(2, 0.0);
        v
    }

    const EPS: f32 = 1e-5;

    #[test]
    fn slerp_identity() {
        let q = Quat4f::from_axis_angle(&axis_y(), 0.5);
        let interp = q.slerp(&q, 0.3);
        assert!((interp.w - q.w).abs() < EPS);
        assert!((interp.x - q.x).abs() < EPS);
        assert!((interp.y - q.y).abs() < EPS);
        assert!((interp.z - q.z).abs() < EPS);
    }

    #[test]
    fn slerp_endpoints() {
        let q0 = Quat4f::from_axis_angle(&axis_y(), 0.0);
        let q1 = Quat4f::from_axis_angle(&axis_y(), std::f32::consts::FRAC_PI_2);

        let at_0 = q0.slerp(&q1, 0.0);
        assert!((at_0.w - q0.w).abs() < EPS);
        assert!((at_0.x - q0.x).abs() < EPS);
        assert!((at_0.y - q0.y).abs() < EPS);
        assert!((at_0.z - q0.z).abs() < EPS);

        let at_1 = q0.slerp(&q1, 1.0);
        let dot = at_1.w * q1.w + at_1.x * q1.x + at_1.y * q1.y + at_1.z * q1.z;
        assert!(dot.abs() > 1.0 - EPS, "slerp(1) ≈ q1 or -q1");
    }

    #[test]
    fn slerp_interior_half() {
        let q0 = Quat4f::from_axis_angle(&axis_y(), 0.0);
        let q1 = Quat4f::from_axis_angle(&axis_y(), std::f32::consts::FRAC_PI_2);
        let half = q0.slerp(&q1, 0.5);
        let expected = Quat4f::from_axis_angle(&axis_y(), std::f32::consts::FRAC_PI_4);
        let dot =
            half.w * expected.w + half.x * expected.x + half.y * expected.y + half.z * expected.z;
        assert!(dot.abs() > 1.0 - EPS, "slerp(0.5) ≈ half-angle rotation");
    }
}
