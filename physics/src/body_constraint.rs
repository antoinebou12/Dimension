//! Rigid-body XPBD constraints: positional (ball joint), angular, contact.
//!
//! These constraints operate on [`RigidBody`] pairs (or a rigid body and a
//! particle) and produce both positional and angular corrections.

use crate::body::RigidBody;
use mathlib::math3d_raw::{
    generalized_inverse_mass, mat3_mul_vec, quat_normalize, quat_rotate_vec, vec3_add, vec3_cross,
    vec3_dot, vec3_length, vec3_scale, vec3_sub,
};

/// Positional (ball-joint) constraint between two rigid bodies.
///
/// Keeps the attachment points on both bodies coincident:
/// `C = (x_A + R_A · r_A) − (x_B + R_B · r_B) = 0`.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PositionalConstraint {
    /// Index of body A in `PhysicsState::rigid_bodies`.
    pub body_a: usize,
    /// Index of body B in `PhysicsState::rigid_bodies`.
    pub body_b: usize,
    /// Attachment point in body A's local frame.
    pub r_a: [f32; 3],
    /// Attachment point in body B's local frame.
    pub r_b: [f32; 3],
    /// XPBD compliance (0 = infinitely stiff).
    pub compliance: f32,
    /// Accumulated Lagrange multiplier (per-axis).
    #[cfg_attr(feature = "serde", serde(skip))]
    pub lambda_accum: [f32; 3],
}

impl PositionalConstraint {
    /// Creates a ball-joint constraint between two bodies.
    #[must_use]
    pub fn new(
        body_a: usize,
        body_b: usize,
        r_a: [f32; 3],
        r_b: [f32; 3],
        compliance: f32,
    ) -> Self {
        Self {
            body_a,
            body_b,
            r_a,
            r_b,
            compliance,
            lambda_accum: [0.0; 3],
        }
    }

    /// Reset accumulated multiplier (call at start of each substep).
    pub fn reset_lambda(&mut self) {
        self.lambda_accum = [0.0; 3];
    }

    /// Solve one iteration of this constraint, mutating `bodies` in place.
    pub fn solve(&mut self, bodies: &mut [RigidBody]) {
        let (a_slice, b_slice) = if self.body_a < self.body_b {
            let (lo, hi) = bodies.split_at_mut(self.body_b);
            (&mut lo[self.body_a], &mut hi[0])
        } else if self.body_a > self.body_b {
            let (lo, hi) = bodies.split_at_mut(self.body_a);
            (&mut hi[0], &mut lo[self.body_b])
        } else {
            return; // same body — skip
        };

        let ra_world = quat_rotate_vec(&a_slice.predicted_q, &self.r_a);
        let rb_world = quat_rotate_vec(&b_slice.predicted_q, &self.r_b);

        let pa = vec3_add(&a_slice.predicted_x, &ra_world);
        let pb = vec3_add(&b_slice.predicted_x, &rb_world);

        // C = pa - pb (3-vector constraint)
        let c = vec3_sub(&pa, &pb);
        let c_len = vec3_length(&c);
        if c_len < 1e-10 {
            return;
        }

        // Solve each axis independently
        for axis in 0..3 {
            let n = match axis {
                0 => [1.0, 0.0, 0.0],
                1 => [0.0, 1.0, 0.0],
                _ => [0.0, 0.0, 1.0],
            };
            let c_val = c[axis];

            let inv_inertia_a = a_slice.inv_inertia_world();
            let inv_inertia_b = b_slice.inv_inertia_world();

            let w_a = generalized_inverse_mass(a_slice.inv_mass, &inv_inertia_a, &ra_world, &n);
            let w_b = generalized_inverse_mass(b_slice.inv_mass, &inv_inertia_b, &rb_world, &n);

            let w_total = w_a + w_b;
            if w_total < 1e-20 {
                continue;
            }

            let dlambda =
                (-c_val - self.compliance * self.lambda_accum[axis]) / (w_total + self.compliance);
            self.lambda_accum[axis] += dlambda;

            // Position correction
            let p = vec3_scale(&n, dlambda);
            if a_slice.inv_mass > 0.0 {
                a_slice.predicted_x =
                    vec3_add(&a_slice.predicted_x, &vec3_scale(&p, a_slice.inv_mass));
                // Angular correction: Δq = 0.5 * [0, I⁻¹(r×p)] ⊗ q
                let torque = vec3_cross(&ra_world, &p);
                let ang = mat3_mul_vec(&inv_inertia_a, &torque);
                apply_angular_correction(&mut a_slice.predicted_q, &ang, 0.5);
            }
            if b_slice.inv_mass > 0.0 {
                b_slice.predicted_x =
                    vec3_sub(&b_slice.predicted_x, &vec3_scale(&p, b_slice.inv_mass));
                let torque = vec3_cross(&rb_world, &p);
                let ang = mat3_mul_vec(&inv_inertia_b, &torque);
                apply_angular_correction(&mut b_slice.predicted_q, &ang, -0.5);
            }
        }
    }
}

/// Contact constraint between two rigid bodies.
///
/// Enforces non-penetration along a contact normal.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RigidContactConstraint {
    /// Index of body A.
    pub body_a: usize,
    /// Index of body B.
    pub body_b: usize,
    /// Contact point offset in body A's local frame.
    pub r_a: [f32; 3],
    /// Contact point offset in body B's local frame.
    pub r_b: [f32; 3],
    /// Contact normal (from A to B, world space).
    pub normal: [f32; 3],
    /// Penetration depth (positive = overlapping).
    pub depth: f32,
    /// Accumulated Lagrange multiplier.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub lambda_accum: f32,
}

impl RigidContactConstraint {
    /// Solve one iteration of this contact constraint.
    pub fn solve(&mut self, bodies: &mut [RigidBody]) {
        let (a_slice, b_slice) = if self.body_a < self.body_b {
            let (lo, hi) = bodies.split_at_mut(self.body_b);
            (&mut lo[self.body_a], &mut hi[0])
        } else if self.body_a > self.body_b {
            let (lo, hi) = bodies.split_at_mut(self.body_a);
            (&mut hi[0], &mut lo[self.body_b])
        } else {
            return;
        };

        let ra_world = quat_rotate_vec(&a_slice.predicted_q, &self.r_a);
        let rb_world = quat_rotate_vec(&b_slice.predicted_q, &self.r_b);

        let pa = vec3_add(&a_slice.predicted_x, &ra_world);
        let pb = vec3_add(&b_slice.predicted_x, &rb_world);
        let diff = vec3_sub(&pa, &pb);
        let c_val = vec3_dot(&diff, &self.normal);

        // Only enforce if penetrating (C < 0)
        if c_val >= 0.0 {
            return;
        }

        let inv_inertia_a = a_slice.inv_inertia_world();
        let inv_inertia_b = b_slice.inv_inertia_world();

        let w_a =
            generalized_inverse_mass(a_slice.inv_mass, &inv_inertia_a, &ra_world, &self.normal);
        let w_b =
            generalized_inverse_mass(b_slice.inv_mass, &inv_inertia_b, &rb_world, &self.normal);
        let w_total = w_a + w_b;
        if w_total < 1e-20 {
            return;
        }

        let dlambda = -c_val / w_total;
        // Only push apart (unilateral)
        let new_lambda = (self.lambda_accum + dlambda).max(0.0);
        let dlambda = new_lambda - self.lambda_accum;
        self.lambda_accum = new_lambda;

        let p = vec3_scale(&self.normal, dlambda);
        if a_slice.inv_mass > 0.0 {
            a_slice.predicted_x = vec3_add(&a_slice.predicted_x, &vec3_scale(&p, a_slice.inv_mass));
            let torque = vec3_cross(&ra_world, &p);
            let ang = mat3_mul_vec(&inv_inertia_a, &torque);
            apply_angular_correction(&mut a_slice.predicted_q, &ang, 0.5);
        }
        if b_slice.inv_mass > 0.0 {
            b_slice.predicted_x = vec3_sub(&b_slice.predicted_x, &vec3_scale(&p, b_slice.inv_mass));
            let torque = vec3_cross(&rb_world, &p);
            let ang = mat3_mul_vec(&inv_inertia_b, &torque);
            apply_angular_correction(&mut b_slice.predicted_q, &ang, -0.5);
        }
    }
}

/// Apply a small angular correction to a predicted quaternion.
///
/// `q ← normalize(q + scale * [0, ang] ⊗ q)`
fn apply_angular_correction(q: &mut [f32; 4], ang: &[f32; 3], scale: f32) {
    let dq = [0.0, ang[0] * scale, ang[1] * scale, ang[2] * scale];
    // quaternion multiply dq * q
    let r = mathlib::math3d_raw::quat_mul(&dq, q);
    q[0] += r[0];
    q[1] += r[1];
    q[2] += r[2];
    q[3] += r[3];
    *q = quat_normalize(q);
}
