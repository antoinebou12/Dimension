//! Matrix-free conjugate gradient solver for bilateral (positional) constraints.
//!
//! Solves the velocity-level Schur system `(J M⁻¹ Jᵀ + εI) λ = b` with
//! `b = -γ φ / h - J·v`, then applies λ as velocity impulses and updates predicted positions.

use crate::body::RigidBody;
use crate::body_constraint::PositionalConstraint;
use mathlib::math3d_raw::{
    generalized_inverse_mass_bilinear, generalized_inverse_mass_bilinear_two_r, mat3_mul_vec,
    quat_rotate_vec, vec3_add, vec3_cross, vec3_scale, vec3_sub,
};

const AXES: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

const REG_EPS: f32 = 1e-9;

/// Run CG for positional constraints and apply λ to predicted state.
///
/// Uses current `predicted_x`, `predicted_q` and body `v`, `omega` to build RHS and J.
/// On return, `rigid_bodies` predicted state is updated from the CG solution.
pub fn solve_positional_cg(
    positionals: &[PositionalConstraint],
    bodies: &mut [RigidBody],
    inv_inertia_cache: &[[f32; 9]],
    dtau: f32,
    gamma: f32,
    max_iter: u32,
    tol_sq: f32,
) {
    let n = positionals.len();
    if n == 0 {
        return;
    }
    let dim = n * 3;

    let hinv = 1.0 / dtau.max(1e-10);

    // Build RHS: b = -γ φ / h - J·v (per constraint, 3-vector)
    let mut b = vec![0.0_f32; dim];
    for (i, pos) in positionals.iter().enumerate() {
        let (_ra, _rb, pa, pb, va, vb) = constraint_geometry(pos, bodies, inv_inertia_cache);
        let phi = vec3_sub(&pa, &pb); // position error
        let jv = vec3_sub(&va, &vb);
        let base = i * 3;
        b[base] = -gamma * hinv * phi[0] - jv[0];
        b[base + 1] = -gamma * hinv * phi[1] - jv[1];
        b[base + 2] = -gamma * hinv * phi[2] - jv[2];
    }

    // CG: x = 0, r = b - Ax = b (since x=0, Ax=0), p = r
    let mut x = vec![0.0_f32; dim];
    let mut r = b.clone();
    let mut p = r.clone();

    let mut r_tr = dot(&r, &r);
    let mut ax_buf = vec![0.0_f32; dim];
    compute_ax(positionals, bodies, inv_inertia_cache, &p, &mut ax_buf);
    let mut p_ap = dot(&p, &ax_buf);

    let mut iter = 0u32;
    while iter < max_iter && r_tr > tol_sq && p_ap > 1e-20 {
        let alpha = r_tr / p_ap;
        for i in 0..dim {
            x[i] += alpha * p[i];
            r[i] -= alpha * ax_buf[i];
        }
        let r_tr_next = dot(&r, &r);
        let beta = r_tr_next / r_tr;
        r_tr = r_tr_next;
        for i in 0..dim {
            p[i] = r[i] + beta * p[i];
        }
        compute_ax(positionals, bodies, inv_inertia_cache, &p, &mut ax_buf);
        p_ap = dot(&p, &ax_buf);
        iter += 1;
    }

    // Apply λ: velocity impulses then integrate into predicted state
    apply_lambda_to_predicted(positionals, bodies, inv_inertia_cache, &x, dtau);
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// (ra_world, rb_world, pa, pb, va, vb) for constraint.
fn constraint_geometry(
    pos: &PositionalConstraint,
    bodies: &[RigidBody],
    _inv_inertia_cache: &[[f32; 9]],
) -> ([f32; 3], [f32; 3], [f32; 3], [f32; 3], [f32; 3], [f32; 3]) {
    let a = &bodies[pos.body_a];
    let b = &bodies[pos.body_b];
    let ra = quat_rotate_vec(&a.predicted_q, &pos.r_a);
    let rb = quat_rotate_vec(&b.predicted_q, &pos.r_b);
    let pa = vec3_add(&a.predicted_x, &ra);
    let pb = vec3_add(&b.predicted_x, &rb);
    let va = vec3_add(&a.v, &vec3_cross(&a.omega, &ra));
    let vb = vec3_add(&b.v, &vec3_cross(&b.omega, &rb));
    (ra, rb, pa, pb, va, vb)
}

/// (Ax)_i = (J_i M⁻¹ Jᵀ + ε I) x  for constraint i (3 DoF), with coupling from j on same bodies.
fn compute_ax(
    positionals: &[PositionalConstraint],
    bodies: &[RigidBody],
    inv_inertia_cache: &[[f32; 9]],
    x: &[f32],
    out: &mut [f32],
) {
    let n = positionals.len();
    out.fill(0.0);

    for i in 0..n {
        let pi = &positionals[i];
        let base_i = i * 3;
        let body_a = pi.body_a;
        let body_b = pi.body_b;
        let ra_i = quat_rotate_vec(&bodies[body_a].predicted_q, &pi.r_a);
        let rb_i = quat_rotate_vec(&bodies[body_b].predicted_q, &pi.r_b);
        let inv_ia = &inv_inertia_cache[body_a];
        let inv_ib = &inv_inertia_cache[body_b];

        // Diagonal block (self) + regularization on diagonal
        for k in 0..3 {
            for l in 0..3 {
                let w = generalized_inverse_mass_bilinear(
                    bodies[body_a].inv_mass,
                    inv_ia,
                    &ra_i,
                    &AXES[k],
                    &AXES[l],
                ) + generalized_inverse_mass_bilinear(
                    bodies[body_b].inv_mass,
                    inv_ib,
                    &rb_i,
                    &AXES[k],
                    &AXES[l],
                );
                let reg = if k == l { REG_EPS } else { 0.0 };
                out[base_i + k] += (w + reg) * x[base_i + l];
            }
        }

        // Coupling from other constraints sharing a body
        for (j, pj) in positionals.iter().enumerate() {
            if j == i {
                continue;
            }
            let base_j = j * 3;
            let xj = [x[base_j], x[base_j + 1], x[base_j + 2]];

            if pj.body_a == body_a || pj.body_b == body_a {
                let rj_a = if pj.body_a == body_a {
                    quat_rotate_vec(&bodies[body_a].predicted_q, &pj.r_a)
                } else {
                    quat_rotate_vec(&bodies[body_a].predicted_q, &pj.r_b)
                };
                let inv_m = bodies[body_a].inv_mass;
                for k in 0..3 {
                    for l in 0..3 {
                        out[base_i + k] += generalized_inverse_mass_bilinear_two_r(
                            inv_m, inv_ia, &ra_i, &rj_a, &AXES[k], &AXES[l],
                        ) * xj[l];
                    }
                }
            }
            if pj.body_a == body_b || pj.body_b == body_b {
                let rj_b = if pj.body_a == body_b {
                    quat_rotate_vec(&bodies[body_b].predicted_q, &pj.r_a)
                } else {
                    quat_rotate_vec(&bodies[body_b].predicted_q, &pj.r_b)
                };
                let inv_m = bodies[body_b].inv_mass;
                for k in 0..3 {
                    for l in 0..3 {
                        out[base_i + k] += generalized_inverse_mass_bilinear_two_r(
                            inv_m, inv_ib, &rb_i, &rj_b, &AXES[k], &AXES[l],
                        ) * xj[l];
                    }
                }
            }
        }
    }
}

/// Apply λ as impulses and integrate into predicted_x / predicted_q.
fn apply_lambda_to_predicted(
    positionals: &[PositionalConstraint],
    bodies: &mut [RigidBody],
    inv_inertia_cache: &[[f32; 9]],
    lambda: &[f32],
    dtau: f32,
) {
    // Accumulate linear and angular impulse per body
    let n_bodies = bodies.len();
    let mut linear_impulse = vec![[0.0_f32; 3]; n_bodies];
    let mut angular_impulse = vec![[0.0_f32; 3]; n_bodies];

    for (i, pos) in positionals.iter().enumerate() {
        let base = i * 3;
        let lam = [lambda[base], lambda[base + 1], lambda[base + 2]];
        let p_impulse = [lam[0], lam[1], lam[2]]; // impulse vector (same as lambda for world axes)

        let ra = quat_rotate_vec(&bodies[pos.body_a].predicted_q, &pos.r_a);
        let rb = quat_rotate_vec(&bodies[pos.body_b].predicted_q, &pos.r_b);

        linear_impulse[pos.body_a][0] += p_impulse[0];
        linear_impulse[pos.body_a][1] += p_impulse[1];
        linear_impulse[pos.body_a][2] += p_impulse[2];
        let ta = vec3_cross(&ra, &p_impulse);
        angular_impulse[pos.body_a][0] += ta[0];
        angular_impulse[pos.body_a][1] += ta[1];
        angular_impulse[pos.body_a][2] += ta[2];

        linear_impulse[pos.body_b][0] -= p_impulse[0];
        linear_impulse[pos.body_b][1] -= p_impulse[1];
        linear_impulse[pos.body_b][2] -= p_impulse[2];
        let tb = vec3_cross(&rb, &p_impulse);
        angular_impulse[pos.body_b][0] -= tb[0];
        angular_impulse[pos.body_b][1] -= tb[1];
        angular_impulse[pos.body_b][2] -= tb[2];
    }

    for (bi, rb) in bodies.iter_mut().enumerate() {
        if rb.inv_mass <= 0.0 {
            continue;
        }
        let dv = vec3_scale(&linear_impulse[bi], rb.inv_mass);
        rb.predicted_x[0] += dv[0] * dtau;
        rb.predicted_x[1] += dv[1] * dtau;
        rb.predicted_x[2] += dv[2] * dtau;

        let inv_i = &inv_inertia_cache[bi];
        let domega = mat3_mul_vec(inv_i, &angular_impulse[bi]);
        // Integrate angular: q += 0.5 * dt * [0, domega] * q then normalize
        let dq =
            mathlib::math3d_raw::quat_mul(&[0.0, domega[0], domega[1], domega[2]], &rb.predicted_q);
        let half_dt = 0.5 * dtau;
        rb.predicted_q[0] += half_dt * dq[0];
        rb.predicted_q[1] += half_dt * dq[1];
        rb.predicted_q[2] += half_dt * dq[2];
        rb.predicted_q[3] += half_dt * dq[3];
        rb.predicted_q = mathlib::math3d_raw::quat_normalize(&rb.predicted_q);
    }
}
