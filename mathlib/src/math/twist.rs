//! Lie algebra helpers for rigid-body twists (6D pose errors).
//!
//! These utilities mirror the “velocity twist” representation used by Halley /
//! `QuIK` style inverse kinematics solvers. They operate on homogeneous
//! transforms (`Matrix4f`) and encode translation + rotation deltas in a single
//! 6-element column vector.

use super::math3d::{Matrix3f, Matrix4f, Vector3f};
use super::quaternion::Quat4f;
use crate::types::Storage;
use crate::vector::Vector;

const SMALL_ANGLE: f32 = 1e-6;

/// Computes the 6×1 twist (Δx, Δθ·axis) from `current` to `target`.
///
/// The translational part is simply `target.translation - current.translation`.
/// The rotational part is the logarithm of the relative rotation matrix (axis
/// multiplied by rotation angle, in radians). This matches the error metric used
/// by Halley/QuIK inverse kinematics solvers.
#[must_use]
pub fn pose_twist_error(current: &Matrix4f, target: &Matrix4f) -> Vector<f32> {
    let mut twist = Vector::with_capacity(6);
    twist.set_zero();

    for i in 0..3 {
        twist.set(i, target.get(i, 3) - current.get(i, 3));
    }

    let rot_current = rotation_block(current);
    let rot_target = rotation_block(target);
    let rot_current_t = rot_current.transpose();
    let mut rot_delta = Matrix3f::with_storage(3, 3, Storage::Column);
    rot_current_t.mul_into(&rot_target, &mut rot_delta);

    let omega = rotation_vector(&rot_delta);
    for i in 0..3 {
        twist.set(3 + i, omega.get(i));
    }
    twist
}

/// Scales the linear (first 3) and angular (last 3) components of a twist in
/// place so that they do not exceed the provided magnitudes.
pub fn clamp_twist(step: &mut Vector<f32>, max_linear: f32, max_angular: f32) {
    assert!(
        step.rows() >= 6,
        "twist vectors must have at least 6 components"
    );
    if max_linear > 0.0 {
        clamp_segment(step, 0, max_linear);
    }
    if max_angular > 0.0 {
        clamp_segment(step, 3, max_angular);
    }
}

fn clamp_segment(step: &mut Vector<f32>, offset: usize, max_norm: f32) {
    let mut norm_sq = 0.0;
    for i in 0..3 {
        let v = step.get(offset + i);
        norm_sq += v * v;
    }
    if norm_sq <= max_norm * max_norm || norm_sq <= f32::EPSILON {
        return;
    }
    let scale = max_norm / norm_sq.sqrt();
    for i in 0..3 {
        let value = step.get(offset + i) * scale;
        step.set(offset + i, value);
    }
}

fn rotation_block(m: &Matrix4f) -> Matrix3f {
    let mut r = Matrix3f::with_storage(3, 3, Storage::Column);
    for i in 0..3 {
        for j in 0..3 {
            r.set(i, j, m.get(i, j));
        }
    }
    r
}

fn rotation_vector(delta: &Matrix3f) -> Vector3f {
    let quat = Quat4f::from_rotation_matrix3(delta);
    let vx = quat.x;
    let vy = quat.y;
    let vz = quat.z;
    let v_norm = (vx * vx + vy * vy + vz * vz).sqrt();
    let mut axis_angle = Vector3f::with_capacity(3);
    axis_angle.set_zero();

    if v_norm < SMALL_ANGLE {
        // Small-angle approximation: sin(theta / 2) ~ theta / 2
        axis_angle.set(0, 2.0 * vx);
        axis_angle.set(1, 2.0 * vy);
        axis_angle.set(2, 2.0 * vz);
        return axis_angle;
    }

    let angle = 2.0 * v_norm.atan2(quat.w);
    if angle.abs() < SMALL_ANGLE {
        axis_angle.set(0, 2.0 * vx);
        axis_angle.set(1, 2.0 * vy);
        axis_angle.set(2, 2.0 * vz);
        return axis_angle;
    }

    let scale = angle / v_norm;
    axis_angle.set(0, vx * scale);
    axis_angle.set(1, vy * scale);
    axis_angle.set(2, vz * scale);
    axis_angle
}
