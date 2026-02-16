//! Shared helpers for IK solvers (FABRIK, CCD, etc.).

use mathlib::cg::vector3;
use mathlib::math3d::vector3_cross;
use mathlib::{Quat4f, Vector3f};

/// Epsilon for length and distance checks.
const EPS: f32 = 1e-10;

/// Threshold for near-antiparallel vectors (dot < this): use stable 180° rotation to avoid
/// numerical instability when cross product is degenerate.
const ANTI_PARALLEL_DOT: f32 = -0.99;

/// Unit quaternion that rotates `from` (normalized) to align with `to` (normalized).
/// Uses a stable 180° rotation when vectors are nearly opposite to avoid numerical issues.
pub(crate) fn quat_from_vec_to_vec(from: &Vector3f, to: &Vector3f) -> Quat4f {
    let dot = from.dot(to);
    let axis = vector3_cross(from, to);
    let axis_norm = axis.norm();
    if axis_norm < EPS {
        if dot >= 0.0 {
            return Quat4f::identity();
        }
        return quat_rotate_180_about_perp(from);
    }
    if dot <= ANTI_PARALLEL_DOT {
        return quat_rotate_180_about_perp(from);
    }
    let angle = dot.clamp(-1.0, 1.0).acos();
    Quat4f::from_axis_angle(&axis.normalize(), angle)
}

/// Stable 180° rotation about an axis perpendicular to `v` (avoids degenerate cross product).
fn quat_rotate_180_about_perp(v: &Vector3f) -> Quat4f {
    let perp = if v.get(0).abs() < 0.9 {
        let mut vec = Vector3f::with_capacity(3);
        vec.set(0, 1.0);
        vec.set(1, 0.0);
        vec.set(2, 0.0);
        vector3_cross(v, &vec)
    } else {
        let mut vec = Vector3f::with_capacity(3);
        vec.set(0, 0.0);
        vec.set(1, 1.0);
        vec.set(2, 0.0);
        vector3_cross(v, &vec)
    };
    let perp_norm = perp.norm();
    if perp_norm < EPS {
        return Quat4f::identity();
    }
    Quat4f::from_axis_angle(&perp.normalize(), std::f32::consts::PI)
}

/// Project vector `v` onto the plane perpendicular to `normal` (assumed non-zero).
/// Returns the component of `v` in that plane.
pub(crate) fn project_onto_plane(v: &Vector3f, normal: &Vector3f) -> Vector3f {
    let n_dot_n = normal.dot(normal);
    if n_dot_n < EPS {
        return v.clone();
    }
    let v_dot_n = v.dot(normal);
    let scale = v_dot_n / n_dot_n;
    vector3(
        v.get(0) - normal.get(0) * scale,
        v.get(1) - normal.get(1) * scale,
        v.get(2) - normal.get(2) * scale,
    )
}
