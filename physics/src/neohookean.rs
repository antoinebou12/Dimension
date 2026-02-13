//! Neo-Hookean XPBD constraints for tetrahedral soft bodies.
//!
//! Each tetrahedron produces two constraints derived from the Stable
//! Neo-Hookean strain energy (Smith 2018 / Macklin & Müller 2021):
//!
//! - **Deviatoric** — `C_dev = ‖F‖_F` (resists shearing/stretching)
//! - **Hydrostatic** — `C_hydro = det(F) − (1 + μ/λ)` (resists volume change)
//!
//! The deformation gradient is `F = Ds · Dm⁻¹` where Ds is built from the
//! current (deformed) vertex positions and Dm⁻¹ was precomputed at rest.

use crate::particle::Particle;
use mathlib::math3d_raw::{
    mat3_adjugate, mat3_det, mat3_frobenius_norm, mat3_mul, mat3_mul_vec, vec3_add, vec3_length_sq,
    vec3_neg, vec3_scale,
};

/// Result of solving one Neo-Hookean constraint on a tetrahedron.
///
/// Contains per-vertex position deltas for the four vertices.
#[derive(Clone, Debug)]
pub struct TetSolveResult {
    /// Position deltas for vertices 0..4.
    pub dx: [[f32; 3]; 4],
}

/// Compute the deformation gradient F = Ds · Dm⁻¹ for a tetrahedron.
///
/// `predicted` contains the current candidate positions.
///
/// Returns F as a row-major 3×3 matrix.
#[must_use]
pub fn deformation_gradient(
    predicted: &[[f32; 3]],
    tet: &[usize; 4],
    dm_inv: &[f32; 9],
) -> [f32; 9] {
    let p0 = predicted[tet[0]];
    let p1 = predicted[tet[1]];
    let p2 = predicted[tet[2]];
    let p3 = predicted[tet[3]];

    // Ds = [p1-p0 | p2-p0 | p3-p0] in columns (row-major storage)
    let ds = [
        p1[0] - p0[0],
        p2[0] - p0[0],
        p3[0] - p0[0],
        p1[1] - p0[1],
        p2[1] - p0[1],
        p3[1] - p0[1],
        p1[2] - p0[2],
        p2[2] - p0[2],
        p3[2] - p0[2],
    ];

    // F = Ds * Dm_inv (3×3 multiply)
    mat3_mul(&ds, dm_inv)
}

/// Solve the **deviatoric** Neo-Hookean constraint for one tetrahedron.
///
/// `C_dev = ‖F‖_F` (Frobenius norm of deformation gradient).
///
/// Returns the XPBD position corrections for the 4 vertices.
///
/// # Arguments
///
/// * `particles` — all particles (for masses).
/// * `predicted` — current predicted positions.
/// * `tet` — the 4 vertex indices.
/// * `dm_inv` — precomputed Dm⁻¹ for this tet.
/// * `compliance` — XPBD compliance α̃ = 1/(μ V₀ Δτ²).
/// * `lambda_accum` — accumulated Lagrange multiplier (mutated).
pub fn solve_deviatoric(
    particles: &[Particle],
    predicted: &[[f32; 3]],
    tet: &[usize; 4],
    dm_inv: &[f32; 9],
    compliance: f32,
    lambda_accum: &mut f32,
) -> TetSolveResult {
    let f = deformation_gradient(predicted, tet, dm_inv);
    let f_norm = mat3_frobenius_norm(&f).max(1e-8);

    // C = ‖F‖_F,  target drives toward √3 (rest state)
    let c_val = f_norm;

    // ∂C/∂F = F / ‖F‖_F
    // ∂F/∂x_k: for vertex k (k=1,2,3), gradient uses row (k-1) of Dm⁻¹
    // ∇_{x_k} C = (1/‖F‖) · F · row_{k-1}(Dm⁻¹)
    let f_scaled = [
        f[0] / f_norm,
        f[1] / f_norm,
        f[2] / f_norm,
        f[3] / f_norm,
        f[4] / f_norm,
        f[5] / f_norm,
        f[6] / f_norm,
        f[7] / f_norm,
        f[8] / f_norm,
    ];

    // row k of dm_inv (row-major): dm_inv[k*3 .. k*3+3]
    let grad1 = mat3_mul_vec(&f_scaled, &[dm_inv[0], dm_inv[1], dm_inv[2]]);
    let grad2 = mat3_mul_vec(&f_scaled, &[dm_inv[3], dm_inv[4], dm_inv[5]]);
    let grad3 = mat3_mul_vec(&f_scaled, &[dm_inv[6], dm_inv[7], dm_inv[8]]);
    let grad0 = vec3_neg(&vec3_add(&grad1, &vec3_add(&grad2, &grad3)));

    let grads = [grad0, grad1, grad2, grad3];

    xpbd_solve_tet(particles, tet, &grads, c_val, compliance, lambda_accum)
}

/// Solve the **hydrostatic** Neo-Hookean constraint for one tetrahedron.
///
/// `C_hydro = det(F) − (1 + μ/λ)`.
///
/// # Arguments
///
/// * `alpha_nh` — the ratio μ/λ (material dependent).
/// * Other args same as [`solve_deviatoric`].
pub fn solve_hydrostatic(
    particles: &[Particle],
    predicted: &[[f32; 3]],
    tet: &[usize; 4],
    dm_inv: &[f32; 9],
    alpha_nh: f32,
    compliance: f32,
    lambda_accum: &mut f32,
) -> TetSolveResult {
    let f = deformation_gradient(predicted, tet, dm_inv);
    let j = mat3_det(&f).max(1e-6); // clamp to avoid inversion singularity

    // C = det(F) - (1 + alpha_nh)
    let c_val = j - (1.0 + alpha_nh);

    // ∂det(F)/∂F = adj(F)  (adjugate / cofactor transpose)
    // ∇_{x_k} C = adj(F) · row_{k-1}(Dm⁻¹)
    let adj = mat3_adjugate(&f);

    // row k of dm_inv (row-major): dm_inv[k*3 .. k*3+3]
    let grad1 = mat3_mul_vec(&adj, &[dm_inv[0], dm_inv[1], dm_inv[2]]);
    let grad2 = mat3_mul_vec(&adj, &[dm_inv[3], dm_inv[4], dm_inv[5]]);
    let grad3 = mat3_mul_vec(&adj, &[dm_inv[6], dm_inv[7], dm_inv[8]]);
    let grad0 = vec3_neg(&vec3_add(&grad1, &vec3_add(&grad2, &grad3)));

    let grads = [grad0, grad1, grad2, grad3];

    xpbd_solve_tet(particles, tet, &grads, c_val, compliance, lambda_accum)
}

/// Apply the XPBD position correction for a 4-vertex constraint.
fn xpbd_solve_tet(
    particles: &[Particle],
    tet: &[usize; 4],
    grads: &[[f32; 3]; 4],
    c_val: f32,
    compliance: f32,
    lambda_accum: &mut f32,
) -> TetSolveResult {
    // w_k = inv_mass_k * ‖∇_k C‖²
    let mut w_total = 0.0_f32;
    for k in 0..4 {
        let w = particles[tet[k]].inv_mass;
        if w > 0.0 {
            w_total += w * vec3_length_sq(&grads[k]);
        }
    }

    if w_total < 1e-20 {
        return TetSolveResult { dx: [[0.0; 3]; 4] };
    }

    // Δλ = (−C − α̃ · λ_accum) / (w_total + α̃)
    let delta_lambda = (-c_val - compliance * *lambda_accum) / (w_total + compliance);

    // Safety: clamp delta_lambda to prevent blow-up
    let dl_clamped = delta_lambda.clamp(-1e4, 1e4);
    if dl_clamped.is_nan() {
        return TetSolveResult { dx: [[0.0; 3]; 4] };
    }
    *lambda_accum += dl_clamped;

    let mut dx = [[0.0_f32; 3]; 4];
    for k in 0..4 {
        let w = particles[tet[k]].inv_mass;
        if w > 0.0 {
            dx[k] = vec3_scale(&grads[k], w * dl_clamped);
        }
    }

    TetSolveResult { dx }
}

/// Solve both Neo-Hookean constraints (deviatoric + hydrostatic) for one tet
/// and apply the results directly to `predicted`.
///
/// This is the main entry point used by the solver loop.
pub fn solve_neohookean_tet(
    particles: &[Particle],
    predicted: &mut [[f32; 3]],
    tet: &[usize; 4],
    dm_inv: &[f32; 9],
    mu: f32,
    lambda: f32,
    rest_volume: f32,
    dt_sq: f32,
    lambda_dev: &mut f32,
    lambda_hydro: &mut f32,
) {
    let compliance_dev = if mu * rest_volume * dt_sq > 1e-20 {
        1.0 / (mu * rest_volume * dt_sq)
    } else {
        0.0
    };
    let compliance_hydro = if lambda * rest_volume * dt_sq > 1e-20 {
        1.0 / (lambda * rest_volume * dt_sq)
    } else {
        0.0
    };
    let alpha_nh = if lambda.abs() > 1e-12 {
        mu / lambda
    } else {
        0.0
    };

    // Deviatoric
    let dev = solve_deviatoric(
        particles,
        predicted,
        tet,
        dm_inv,
        compliance_dev,
        lambda_dev,
    );
    for k in 0..4 {
        predicted[tet[k]] = vec3_add(&predicted[tet[k]], &dev.dx[k]);
    }

    // Hydrostatic
    let hydro = solve_hydrostatic(
        particles,
        predicted,
        tet,
        dm_inv,
        alpha_nh,
        compliance_hydro,
        lambda_hydro,
    );
    for k in 0..4 {
        predicted[tet[k]] = vec3_add(&predicted[tet[k]], &hydro.dx[k]);
    }
}
