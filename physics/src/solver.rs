//! Unified XPBD solver: predict, constraint solve, velocity update.
//!
//! Handles particles, rigid bodies, and soft bodies (Neo-Hookean) in one loop.

use crate::body_constraint::{PositionalConstraint, RigidContactConstraint};
use crate::constraint::Constraint;
use crate::hooks::SubstepHooks;
use crate::neohookean;
use crate::state::PhysicsState;
use mathlib::math3d_raw::{
    quat_conjugate, quat_mul, quat_normalize, vec3_add, vec3_length_sq, vec3_scale, vec3_sub,
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
            c.lambda_accum = 0.0;
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

            // Rigid body constraints
            for c in positional_constraints.iter_mut() {
                c.solve(&mut state.rigid_bodies);
            }
            for c in rigid_contacts.iter_mut() {
                c.solve(&mut state.rigid_bodies);
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
