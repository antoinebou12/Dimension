//! Rigid-body XPBD constraints: positional (ball joint), angular, contact.
//!
//! These constraints operate on [`RigidBody`] pairs (or a rigid body and a
//! particle) and produce both positional and angular corrections.

use crate::body::RigidBody;
use mathlib::math3d_raw::{
    generalized_inverse_mass, generalized_inverse_mass_bilinear, mat3_mul_vec, quat_normalize,
    quat_rotate_vec, vec3_add, vec3_cross, vec3_dot, vec3_length, vec3_scale, vec3_sub,
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
    /// `inv_inertia_cache` should be precomputed from each body's `predicted_q` for this iteration.
    /// `coupling` is subtracted from the RHS (PGS coupling from other constraints on the same bodies); use `[0.0; 3]` if none.
    pub fn solve(
        &mut self,
        bodies: &mut [RigidBody],
        inv_inertia_cache: &[[f32; 9]],
        coupling: [f32; 3],
    ) {
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

        let inv_inertia_a = &inv_inertia_cache[self.body_a];
        let inv_inertia_b = &inv_inertia_cache[self.body_b];

        // Solve each axis independently
        for axis in 0..3 {
            let n = match axis {
                0 => [1.0, 0.0, 0.0],
                1 => [0.0, 1.0, 0.0],
                _ => [0.0, 0.0, 1.0],
            };
            let c_val = c[axis];

            let w_a = generalized_inverse_mass(a_slice.inv_mass, inv_inertia_a, &ra_world, &n);
            let w_b = generalized_inverse_mass(b_slice.inv_mass, inv_inertia_b, &rb_world, &n);

            let w_total = w_a + w_b;
            if w_total < 1e-20 {
                continue;
            }

            let dlambda = (-c_val - self.compliance * self.lambda_accum[axis] - coupling[axis])
                / (w_total + self.compliance);
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
/// Enforces non-penetration along the contact normal and Coulomb friction along
/// two tangent directions (box PGS: normal λ ∈ [0, ∞), tangents ∈ [-μ·λ_n, μ·λ_n]).
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
    /// First tangent direction (world space, orthonormal with normal and tangent2).
    pub tangent1: [f32; 3],
    /// Second tangent direction (world space).
    pub tangent2: [f32; 3],
    /// Penetration depth (positive = overlapping).
    pub depth: f32,
    /// Accumulated Lagrange multipliers: [normal, tangent1, tangent2].
    #[cfg_attr(feature = "serde", serde(skip))]
    pub lambda_accum: [f32; 3],
}

/// Build two orthonormal tangent directions from a contact normal.
/// Uses a reference axis to avoid degeneracy when normal is aligned with an axis.
#[must_use]
pub fn tangents_from_normal(normal: &[f32; 3]) -> ([f32; 3], [f32; 3]) {
    let ref_axis = if normal[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let t1 = vec3_cross(normal, &ref_axis);
    let len = vec3_length(&t1);
    if len < 1e-8 {
        let alt = [0.0, 0.0, 1.0];
        let t1_alt = vec3_cross(normal, &alt);
        let len_alt = vec3_length(&t1_alt);
        let t1_n = vec3_scale(&t1_alt, 1.0 / len_alt);
        let t2_alt = vec3_cross(normal, &t1_n);
        let t2_len_alt = vec3_length(&t2_alt);
        return (t1_n, vec3_scale(&t2_alt, 1.0 / t2_len_alt));
    }
    let t1 = vec3_scale(&t1, 1.0 / len);
    let t2 = vec3_cross(normal, &t1);
    let t2_len = vec3_length(&t2);
    (t1, vec3_scale(&t2, 1.0 / t2_len))
}

impl RigidContactConstraint {
    /// Creates a contact constraint with tangents derived from the normal.
    #[must_use]
    pub fn new(
        body_a: usize,
        body_b: usize,
        r_a: [f32; 3],
        r_b: [f32; 3],
        normal: [f32; 3],
        depth: f32,
    ) -> Self {
        let (tangent1, tangent2) = tangents_from_normal(&normal);
        Self {
            body_a,
            body_b,
            r_a,
            r_b,
            normal,
            tangent1,
            tangent2,
            depth,
            lambda_accum: [0.0; 3],
        }
    }

    /// Solve one iteration of this contact constraint (normal + box friction).
    /// `inv_inertia_cache` should be precomputed from each body's `predicted_q` for this iteration.
    /// `mu` is the Coulomb friction coefficient (use `config.contact_friction`).
    /// `coupling` is subtracted from the RHS (PGS coupling from other constraints on the same bodies); use `[0.0; 3]` if none.
    pub fn solve(
        &mut self,
        bodies: &mut [RigidBody],
        inv_inertia_cache: &[[f32; 9]],
        mu: f32,
        coupling: [f32; 3],
    ) {
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

        let inv_inertia_a = &inv_inertia_cache[self.body_a];
        let inv_inertia_b = &inv_inertia_cache[self.body_b];

        // 3×3 effective mass block A = J M⁻¹ Jᵀ (normal, tangent1, tangent2)
        let dirs: [&[f32; 3]; 3] = [&self.normal, &self.tangent1, &self.tangent2];
        let mut a = [[0.0_f32; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                a[i][j] = generalized_inverse_mass_bilinear(
                    a_slice.inv_mass,
                    inv_inertia_a,
                    &ra_world,
                    dirs[i],
                    dirs[j],
                ) + generalized_inverse_mass_bilinear(
                    b_slice.inv_mass,
                    inv_inertia_b,
                    &rb_world,
                    dirs[i],
                    dirs[j],
                );
            }
        }
        // Regularization for numerical stability
        const EPS: f32 = 1e-6;
        a[0][0] += EPS;
        a[1][1] += EPS;
        a[2][2] += EPS;

        // RHS: b[0] = normal residual (negative penetration), b[1], b[2] = relative velocity along tangents; subtract coupling
        let va = vec3_add(&a_slice.v, &vec3_cross(&a_slice.omega, &ra_world));
        let vb = vec3_add(&b_slice.v, &vec3_cross(&b_slice.omega, &rb_world));
        let rel_vel = vec3_sub(&va, &vb);
        let b = [
            -c_val - coupling[0],
            vec3_dot(&rel_vel, &self.tangent1) - coupling[1],
            vec3_dot(&rel_vel, &self.tangent2) - coupling[2],
        ];

        // Solve A * x = b (3×3)
        let a_flat: [f32; 9] = [
            a[0][0], a[0][1], a[0][2], a[1][0], a[1][1], a[1][2], a[2][0], a[2][1], a[2][2],
        ];
        let Some(inv_a) = mathlib::math3d_raw::mat3_inverse(&a_flat) else {
            return;
        };
        let x = mathlib::math3d_raw::mat3_mul_vec(&inv_a, &b);

        let old_lambda = self.lambda_accum;
        self.lambda_accum[0] = old_lambda[0] + x[0];
        self.lambda_accum[1] = old_lambda[1] + x[1];
        self.lambda_accum[2] = old_lambda[2] + x[2];

        // Project: normal ∈ [0, ∞), tangents ∈ [-μ·λ_n, μ·λ_n]
        if self.lambda_accum[0] < 0.0 {
            self.lambda_accum[0] = 0.0;
        }
        let lambda_n = self.lambda_accum[0];
        let bound = mu * lambda_n;
        self.lambda_accum[1] = self.lambda_accum[1].clamp(-bound, bound);
        self.lambda_accum[2] = self.lambda_accum[2].clamp(-bound, bound);

        let dlambda = [
            self.lambda_accum[0] - old_lambda[0],
            self.lambda_accum[1] - old_lambda[1],
            self.lambda_accum[2] - old_lambda[2],
        ];
        let p = vec3_add(
            &vec3_scale(&self.normal, dlambda[0]),
            &vec3_add(
                &vec3_scale(&self.tangent1, dlambda[1]),
                &vec3_scale(&self.tangent2, dlambda[2]),
            ),
        );

        if a_slice.inv_mass > 0.0 {
            a_slice.predicted_x = vec3_add(&a_slice.predicted_x, &vec3_scale(&p, a_slice.inv_mass));
            let torque = vec3_cross(&ra_world, &p);
            let ang = mat3_mul_vec(inv_inertia_a, &torque);
            apply_angular_correction(&mut a_slice.predicted_q, &ang, 0.5);
        }
        if b_slice.inv_mass > 0.0 {
            b_slice.predicted_x = vec3_sub(&b_slice.predicted_x, &vec3_scale(&p, b_slice.inv_mass));
            let torque = vec3_cross(&rb_world, &p);
            let ang = mat3_mul_vec(inv_inertia_b, &torque);
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
