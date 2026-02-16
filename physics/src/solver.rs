//! Unified XPBD solver: predict, constraint solve, velocity update.
//!
//! Handles particles, rigid bodies, and soft bodies (Neo-Hookean) in one loop.

use crate::body::RigidBody;
use crate::body_constraint::{PositionalConstraint, RigidContactConstraint};
use crate::constraint::Constraint;
use crate::hooks::SubstepHooks;
use crate::neohookean;
use crate::schur_cg;
use crate::state::{PhysicsState, RigidBilateralSolver};
use mathlib::math3d_raw::{
    generalized_inverse_mass_bilinear_two_r, quat_conjugate, quat_mul, quat_normalize,
    quat_rotate_vec, vec3_add, vec3_length_sq, vec3_scale, vec3_sub,
};

/// Full PBD step (backward-compatible, no rigid/joint constraints).
pub fn step_pbd(state: &mut PhysicsState, dt: f32) {
    step_xpbd(
        state,
        &[],
        &mut [],
        &mut [],
        dt,
        &mut SubstepHooks::default(),
    );
}

/// Explicit (semi-implicit) Euler: v += g*dt, x += v*dt.
pub fn step_explicit_euler(state: &mut PhysicsState, dt: f32) {
    let g = state.config.gravity;
    for p in &mut state.particles {
        if p.inv_mass > 0.0 {
            p.v[0] += g[0] * dt;
            p.v[1] += g[1] * dt;
            p.v[2] += g[2] * dt;
            p.x[0] += p.v[0] * dt;
            p.x[1] += p.v[1] * dt;
            p.x[2] += p.v[2] * dt;
        }
    }
    for rb in &mut state.rigid_bodies {
        if rb.inv_mass > 0.0 {
            rb.v[0] += g[0] * dt;
            rb.v[1] += g[1] * dt;
            rb.v[2] += g[2] * dt;
            rb.x[0] += rb.v[0] * dt;
            rb.x[1] += rb.v[1] * dt;
            rb.x[2] += rb.v[2] * dt;
        }
    }
}

/// Unified XPBD step with all body types.
pub fn step_xpbd(
    state: &mut PhysicsState,
    particle_constraints: &[Box<dyn Constraint>],
    positional_constraints: &mut [PositionalConstraint],
    rigid_contacts: &mut [RigidContactConstraint],
    dt: f32,
    hooks: &mut SubstepHooks,
) {
    let substeps = state.config.substeps.max(1);
    let dtau = dt / substeps as f32;
    let dtau_sq = dtau * dtau;
    let iterations = state.config.solver_iterations.max(1);
    let sor = state.config.sor_omega;
    let sleep_sq = state.config.sleep_threshold * state.config.sleep_threshold;
    let gravity = state.config.gravity;
    let n_particles = state.particles.len();

    for substep in 0..substeps {
        hooks.on_pre_substep(substep, dtau);

        // == Predict: particles ==
        let mut predicted: Vec<[f32; 3]> = Vec::with_capacity(n_particles);
        for p in &mut state.particles {
            if p.inv_mass > 0.0 {
                p.v[0] += gravity[0] * dtau;
                p.v[1] += gravity[1] * dtau;
                p.v[2] += gravity[2] * dtau;
            }
            predicted.push([
                p.x[0] + p.v[0] * dtau,
                p.x[1] + p.v[1] * dtau,
                p.x[2] + p.v[2] * dtau,
            ]);
        }

        // == Predict: rigid bodies ==
        for rb in &mut state.rigid_bodies {
            if rb.inv_mass > 0.0 {
                rb.v[0] += gravity[0] * dtau;
                rb.v[1] += gravity[1] * dtau;
                rb.v[2] += gravity[2] * dtau;
                rb.predicted_x = vec3_add(&rb.x, &vec3_scale(&rb.v, dtau));
                rb.predicted_q = integrate_quaternion(&rb.q, &rb.omega, dtau);
            } else {
                rb.predicted_x = rb.x;
                rb.predicted_q = rb.q;
            }
        }

        // == Reset Lagrange multipliers ==
        for c in positional_constraints.iter_mut() {
            c.reset_lambda();
        }
        for c in rigid_contacts.iter_mut() {
            c.lambda_accum = [0.0; 3];
        }

        // Optional: solve bilateral (positional) constraints with CG once per substep
        let use_cg = state.config.rigid_bilateral_solver == RigidBilateralSolver::ConjugateGradient
            && !positional_constraints.is_empty();
        if use_cg {
            let inv_inertia_cg: Vec<[f32; 9]> = state
                .rigid_bodies
                .iter()
                .map(|rb| RigidBody::inv_inertia_world_from_q(&rb.inv_inertia, &rb.predicted_q))
                .collect();
            schur_cg::solve_positional_cg(
                &*positional_constraints,
                &mut state.rigid_bodies,
                &inv_inertia_cg,
                dtau,
                state.config.constraint_gamma,
                state.config.cg_max_iter,
                state.config.cg_tolerance,
            );
        }

        // Soft body Lagrange multipliers (deviatoric + hydrostatic per tet)
        let mut soft_lambdas: Vec<Vec<(f32, f32)>> = state
            .soft_bodies
            .iter()
            .map(|sb| vec![(0.0_f32, 0.0_f32); sb.mesh.num_tets()])
            .collect();

        // == Constraint solve iterations ==
        let mut dx: Vec<[f32; 3]> = vec![[0.0; 3]; n_particles];
        let mut n_affecting: Vec<u32> = vec![0; n_particles];
        let n_bodies = state.rigid_bodies.len();
        let mut inv_inertia_cache: Vec<[f32; 9]> = vec![[0.0; 9]; n_bodies];

        for _iter in 0..iterations {
            for d in dx.iter_mut() {
                *d = [0.0; 3];
            }
            for n in n_affecting.iter_mut() {
                *n = 0;
            }

            // Particle constraints
            for constraint in particle_constraints {
                constraint.solve(&state.particles, &predicted, &mut dx, &mut n_affecting);
            }

            // Apply with SOR averaging
            for i in 0..n_particles {
                if n_affecting[i] > 0 && state.particles[i].inv_mass > 0.0 {
                    let inv_n = sor / n_affecting[i] as f32;
                    predicted[i][0] += dx[i][0] * inv_n;
                    predicted[i][1] += dx[i][1] * inv_n;
                    predicted[i][2] += dx[i][2] * inv_n;
                }
            }

            // Cache world-frame inverse inertia per body (from predicted_q) for rigid constraints.
            // Avoids O(contacts) recomputation; each body's inertia is computed once per iteration.
            for (i, rb) in state.rigid_bodies.iter().enumerate() {
                inv_inertia_cache[i] =
                    RigidBody::inv_inertia_world_from_q(&rb.inv_inertia, &rb.predicted_q);
            }

            // PGS coupling: build body -> constraint index lists and compute RHS coupling terms (contacts only when using CG for positionals)
            let n_bodies = state.rigid_bodies.len();
            let body_to_pos = {
                let pos = &*positional_constraints;
                build_body_to_constraint_indices(n_bodies, pos.len(), |i| {
                    [pos[i].body_a, pos[i].body_b]
                })
            };
            let body_to_contact = {
                let con = &*rigid_contacts;
                build_body_to_constraint_indices(n_bodies, con.len(), |i| {
                    [con[i].body_a, con[i].body_b]
                })
            };
            let coupling_pos = compute_positional_coupling(
                &*positional_constraints,
                &state.rigid_bodies,
                &inv_inertia_cache,
                &body_to_pos,
            );
            let coupling_contact = compute_contact_coupling(
                &*rigid_contacts,
                &state.rigid_bodies,
                &inv_inertia_cache,
                &body_to_contact,
            );

            // Rigid body bilateral constraints: PGS (with coupling) or skip when CG was used
            if !use_cg {
                for (c, coupling) in positional_constraints
                    .iter_mut()
                    .zip(coupling_pos.into_iter())
                {
                    c.solve(&mut state.rigid_bodies, &inv_inertia_cache, coupling);
                }
            }
            for (c, coupling) in rigid_contacts.iter_mut().zip(coupling_contact.into_iter()) {
                c.solve(
                    &mut state.rigid_bodies,
                    &inv_inertia_cache,
                    state.config.contact_friction,
                    coupling,
                );
            }

            // Soft body Neo-Hookean constraints
            for (sb_idx, sb) in state.soft_bodies.iter().enumerate() {
                for t in 0..sb.mesh.num_tets() {
                    let local_tet = sb.mesh.tets[t];
                    let global_tet = [
                        sb.particle_offset + local_tet[0],
                        sb.particle_offset + local_tet[1],
                        sb.particle_offset + local_tet[2],
                        sb.particle_offset + local_tet[3],
                    ];
                    let (ld, lh) = &mut soft_lambdas[sb_idx][t];
                    neohookean::solve_neohookean_tet(
                        &state.particles,
                        &mut predicted,
                        &global_tet,
                        &sb.mesh.dm_inv[t],
                        sb.mu,
                        sb.lambda,
                        sb.mesh.rest_volumes[t],
                        dtau_sq,
                        ld,
                        lh,
                    );
                }
            }
        }

        // == Velocity update: particles ==
        for i in 0..n_particles {
            if state.particles[i].inv_mass > 0.0 {
                let disp = vec3_sub(&predicted[i], &state.particles[i].x);
                if vec3_length_sq(&disp) < sleep_sq {
                    predicted[i] = state.particles[i].x;
                } else {
                    state.particles[i].v = vec3_scale(&disp, 1.0 / dtau);
                }
                state.particles[i].x = predicted[i];
            }
        }

        // == Velocity update: rigid bodies ==
        for rb in &mut state.rigid_bodies {
            if rb.inv_mass > 0.0 {
                let disp = vec3_sub(&rb.predicted_x, &rb.x);
                if vec3_length_sq(&disp) < sleep_sq {
                    rb.predicted_x = rb.x;
                    rb.predicted_q = rb.q;
                } else {
                    rb.v = vec3_scale(&disp, 1.0 / dtau);
                    let dq = quat_mul(&rb.predicted_q, &quat_conjugate(&rb.q));
                    let sign = if dq[0] < 0.0 { -1.0 } else { 1.0 };
                    rb.omega = [
                        2.0 * dq[1] * sign / dtau,
                        2.0 * dq[2] * sign / dtau,
                        2.0 * dq[3] * sign / dtau,
                    ];
                }
                rb.x = rb.predicted_x;
                rb.q = rb.predicted_q;
            }
        }

        hooks.on_post_substep(substep, dtau);
    }
}

fn integrate_quaternion(q: &[f32; 4], omega: &[f32; 3], dt: f32) -> [f32; 4] {
    let omega_q = [0.0, omega[0], omega[1], omega[2]];
    let dq = quat_mul(&omega_q, q);
    let half_dt = 0.5 * dt;
    quat_normalize(&[
        q[0] + half_dt * dq[0],
        q[1] + half_dt * dq[1],
        q[2] + half_dt * dq[2],
        q[3] + half_dt * dq[3],
    ])
}

/// Build per-body lists of constraint indices (positional or contact) that touch that body.
fn build_body_to_constraint_indices<F>(
    n_bodies: usize,
    n_constraints: usize,
    body_pair: F,
) -> Vec<Vec<usize>>
where
    F: Fn(usize) -> [usize; 2],
{
    let mut out = vec![Vec::new(); n_bodies];
    for i in 0..n_constraints {
        let [a, b] = body_pair(i);
        if a < n_bodies {
            out[a].push(i);
        }
        if b < n_bodies && b != a {
            out[b].push(i);
        }
    }
    out
}

/// PGS coupling for positionals: for each constraint i, subtract sum_j (J_i M⁻¹ J_jᵀ) λ_j over j on same bodies.
fn compute_positional_coupling(
    positionals: &[PositionalConstraint],
    bodies: &[RigidBody],
    inv_inertia_cache: &[[f32; 9]],
    body_to_pos: &[Vec<usize>],
) -> Vec<[f32; 3]> {
    const AXES: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let n = positionals.len();
    let mut coupling = vec![[0.0; 3]; n];
    for i in 0..n {
        let pi = &positionals[i];
        let bodies_i = [pi.body_a, pi.body_b];
        for &bi in &bodies_i {
            if bi >= bodies.len() {
                continue;
            }
            let rb = &bodies[bi];
            let inv_mass = rb.inv_mass;
            let inv_i = &inv_inertia_cache[bi];
            let ri = if pi.body_a == bi {
                quat_rotate_vec(&rb.predicted_q, &pi.r_a)
            } else {
                quat_rotate_vec(&rb.predicted_q, &pi.r_b)
            };
            for &j in &body_to_pos[bi] {
                if j == i {
                    continue;
                }
                let pj = &positionals[j];
                let rj = if pj.body_a == bi {
                    quat_rotate_vec(&rb.predicted_q, &pj.r_a)
                } else {
                    quat_rotate_vec(&rb.predicted_q, &pj.r_b)
                };
                let mut block = [[0.0_f32; 3]; 3];
                for (k, ek) in AXES.iter().enumerate() {
                    for (l, el) in AXES.iter().enumerate() {
                        block[k][l] = generalized_inverse_mass_bilinear_two_r(
                            inv_mass, inv_i, &ri, &rj, ek, el,
                        );
                    }
                }
                let lam = &pj.lambda_accum;
                coupling[i][0] +=
                    block[0][0] * lam[0] + block[0][1] * lam[1] + block[0][2] * lam[2];
                coupling[i][1] +=
                    block[1][0] * lam[0] + block[1][1] * lam[1] + block[1][2] * lam[2];
                coupling[i][2] +=
                    block[2][0] * lam[0] + block[2][1] * lam[1] + block[2][2] * lam[2];
            }
        }
    }
    coupling
}

/// PGS coupling for contacts: same idea with normal + two tangents.
fn compute_contact_coupling(
    contacts: &[RigidContactConstraint],
    bodies: &[RigidBody],
    inv_inertia_cache: &[[f32; 9]],
    body_to_contact: &[Vec<usize>],
) -> Vec<[f32; 3]> {
    let n = contacts.len();
    let mut coupling = vec![[0.0; 3]; n];
    for i in 0..n {
        let ci = &contacts[i];
        let dirs_i: [&[f32; 3]; 3] = [&ci.normal, &ci.tangent1, &ci.tangent2];
        let bodies_i = [ci.body_a, ci.body_b];
        for &bi in &bodies_i {
            if bi >= bodies.len() {
                continue;
            }
            let rb = &bodies[bi];
            let inv_mass = rb.inv_mass;
            let inv_i = &inv_inertia_cache[bi];
            let ri = if ci.body_a == bi {
                quat_rotate_vec(&rb.predicted_q, &ci.r_a)
            } else {
                quat_rotate_vec(&rb.predicted_q, &ci.r_b)
            };
            for &j in &body_to_contact[bi] {
                if j == i {
                    continue;
                }
                let cj = &contacts[j];
                let rj = if cj.body_a == bi {
                    quat_rotate_vec(&rb.predicted_q, &cj.r_a)
                } else {
                    quat_rotate_vec(&rb.predicted_q, &cj.r_b)
                };
                let dirs_j: [&[f32; 3]; 3] = [&cj.normal, &cj.tangent1, &cj.tangent2];
                let mut block = [[0.0_f32; 3]; 3];
                for k in 0..3 {
                    for l in 0..3 {
                        block[k][l] = generalized_inverse_mass_bilinear_two_r(
                            inv_mass, inv_i, &ri, &rj, dirs_i[k], dirs_j[l],
                        );
                    }
                }
                let lam = &cj.lambda_accum;
                coupling[i][0] +=
                    block[0][0] * lam[0] + block[0][1] * lam[1] + block[0][2] * lam[2];
                coupling[i][1] +=
                    block[1][0] * lam[0] + block[1][1] * lam[1] + block[1][2] * lam[2];
                coupling[i][2] +=
                    block[2][0] * lam[0] + block[2][1] * lam[1] + block[2][2] * lam[2];
            }
        }
    }
    coupling
}
