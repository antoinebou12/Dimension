//! Hessian IK: exact Hessian Newton method (Erleben & Andrews, MIG 2017).
//!
//! Minimizes f(θ) = ½‖g - F(θ)‖² using the exact Hessian H = J^T J - K:r with Newton's method.
//! The second-order correction is applied symmetrically to (i,j) and (j,i) to match the
//! analytic Hessian of ½‖r‖² and the MATLAB reference (Andrews, HDM05).
//! Includes adaptive regularization for indefinite H, gradient-step fallback, optional
//! conjugate gradient for the Newton step (useful for large DOF), and line search.
//!
//! **Execution:** This solver is CPU-only and uses mathlib's SIMD/parallel backends when
//! the `simd` and `parallel` features are enabled. It does not use wgpu because the
//! implementation is f64 and mathlib's GPU backend is f32-only.

use mathlib::cpu;
use mathlib::vec3_cross_f64;
use mathlib::{CholError, Cholesky, Matrix, Storage, Vector, Vector3f, solve_cg};

use super::chain::{apply_joint_step, build_position_jacobian_and_axes};
use crate::armature::Armature;

/// Default PCG tolerance when using conjugate gradient for the Newton step.
const DEFAULT_PCG_TOL: f64 = 1e-2;
/// Default PCG max iterations (e.g. ndof or 2*ndof).
const DEFAULT_PCG_MAX_ITERS_FACTOR: usize = 2;

/// Hessian IK solver (exact Hessian Newton + line search).
pub struct HessianIk<'a> {
    armature: &'a mut Armature,
    end_effector_idx: usize,
    target: Vector3f,
    max_iters: usize,
    tolerance: f32,
    regularization_base: f64,
    trust_radius: Option<f32>,
    /// When true, use conjugate gradient to solve H p = b instead of Cholesky (useful for large DOF).
    use_pcg: bool,
    pcg_tol: f64,
    pcg_max_iters: Option<usize>,
    /// When true, use trust-region accept/reject and update radius from actual vs predicted reduction.
    trust_region_adaptive: bool,
}

impl<'a> HessianIk<'a> {
    /// Creates a Hessian IK solver.
    pub fn new(armature: &'a mut Armature, end_effector_idx: usize, target: Vector3f) -> Self {
        Self {
            armature,
            end_effector_idx,
            target,
            max_iters: 32,
            tolerance: 1e-5,
            regularization_base: 1e-4,
            trust_radius: Some(0.5),
            use_pcg: false,
            pcg_tol: DEFAULT_PCG_TOL,
            pcg_max_iters: None,
            trust_region_adaptive: false,
        }
    }

    /// Sets maximum iterations.
    #[must_use]
    pub fn with_max_iters(mut self, n: usize) -> Self {
        self.max_iters = n;
        self
    }

    /// Sets position tolerance for early exit.
    #[must_use]
    pub fn with_tolerance(mut self, tol: f32) -> Self {
        self.tolerance = tol;
        self
    }

    /// Sets regularization base for indefinite Hessian (λ = base * ‖r‖²).
    #[must_use]
    pub fn with_regularization(mut self, base: f64) -> Self {
        self.regularization_base = base;
        self
    }

    /// Sets trust-region clamp on step norm (radians). None disables.
    #[must_use]
    pub fn with_trust_radius(mut self, r: Option<f32>) -> Self {
        self.trust_radius = r;
        self
    }

    /// Use conjugate gradient to solve H p = b instead of Cholesky. Useful for larger DOF.
    #[must_use]
    pub fn with_pcg(mut self, use_pcg: bool) -> Self {
        self.use_pcg = use_pcg;
        self
    }

    /// Sets PCG stopping tolerance (default 1e-2). Used when [`Self::with_pcg`] is true.
    #[must_use]
    pub fn with_pcg_tol(mut self, tol: f64) -> Self {
        self.pcg_tol = tol;
        self
    }

    /// Sets PCG max iterations. None = 2 * ndof. Used when [`Self::with_pcg`] is true.
    #[must_use]
    pub fn with_pcg_max_iters(mut self, n: Option<usize>) -> Self {
        self.pcg_max_iters = n;
        self
    }

    /// Use adaptive trust region: accept/reject step by actual vs predicted reduction and update radius.
    #[must_use]
    pub fn with_trust_region_adaptive(mut self, adaptive: bool) -> Self {
        self.trust_region_adaptive = adaptive;
        self
    }

    /// Updates the target position.
    pub fn set_target(&mut self, target: Vector3f) {
        self.target = target;
    }

    /// Runs Newton iterations, returning final error magnitude.
    pub fn solve(&mut self) -> f32 {
        let mut best_err = f32::MAX;
        let mut best_state = self.armature.pack();
        let mut current_trust_radius: f64 = self.trust_radius.map(f64::from).unwrap_or(0.5);

        for _ in 0..self.max_iters {
            self.armature.update_kinematics();
            let ee = self.armature.end_effector_position(self.end_effector_idx);
            let r = self.residual(&ee);
            let err = self.norm3(r[0] as f32, r[1] as f32, r[2] as f32);

            if err < best_err {
                best_err = err;
                best_state = self.armature.pack();
            }
            if err < self.tolerance {
                break;
            }

            let path = self.armature.path_to(self.end_effector_idx);
            let dof_total: usize = path
                .iter()
                .map(|&i| self.armature.tree().nodes[i].data.joint.dof_count())
                .sum();
            if dof_total == 0 {
                break;
            }

            let (j_pos, axes, is_prismatic) =
                build_position_jacobian_and_axes(self.armature, &path, &ee);

            let r_vec = Vector::from_slice(&[r[0], r[1], r[2]]);
            let jt_r = &j_pos.transpose() * &r_vec;

            let hess = build_hessian(&j_pos, &axes, &is_prismatic, &r_vec);
            let p = if self.use_pcg {
                let max_it = self
                    .pcg_max_iters
                    .unwrap_or(dof_total * DEFAULT_PCG_MAX_ITERS_FACTOR);
                match solve_cg(&hess, &jt_r, self.pcg_tol, max_it) {
                    Ok(step) => step,
                    Err(_) => {
                        match solve_newton_step(&hess, &jt_r, self.regularization_base, &r_vec) {
                            Ok(step) => step,
                            Err(_) => jt_r.clone(),
                        }
                    }
                }
            } else {
                match solve_newton_step(&hess, &jt_r, self.regularization_base, &r_vec) {
                    Ok(step) => step,
                    Err(_) => jt_r.clone(),
                }
            };

            let mut p_vec = p;
            let tr = if self.trust_region_adaptive {
                Some(current_trust_radius as f32)
            } else {
                self.trust_radius
            };
            if let Some(tr_val) = tr {
                let p_norm = vector_norm(&p_vec);
                if p_norm > tr_val as f64 && p_norm > 1e-12 {
                    let scale = (tr_val as f64) / p_norm;
                    for i in 0..p_vec.rows() {
                        p_vec.set(i, p_vec.get(i) * scale);
                    }
                }
            }

            let dof_mapping = self.armature.path_to_dfs_dof_mapping(&path);
            let theta = self.armature.pack();
            let p_f32: Vec<f32> = (0..p_vec.rows()).map(|i| p_vec.get(i) as f32).collect();

            const ALPHAS: [f32; 6] = [1.0, 0.5, 0.25, 0.125, 0.0625, 0.03125];
            let mut improved = false;
            let mut alpha_used = 0.0_f32;
            for alpha in ALPHAS {
                let mut theta_new = theta.clone();
                apply_joint_step(
                    self.armature.tree(),
                    &path,
                    &dof_mapping,
                    alpha,
                    &p_f32,
                    &mut theta_new,
                );
                self.armature.unpack(&theta_new);
                self.armature.update_kinematics();
                let ee_new = self.armature.end_effector_position(self.end_effector_idx);
                let r_new = self.residual(&ee_new);
                let err_new = self.norm3(r_new[0] as f32, r_new[1] as f32, r_new[2] as f32);
                if err_new < err {
                    improved = true;
                    alpha_used = alpha;
                    if err_new < best_err {
                        best_err = err_new;
                        best_state = theta_new;
                    }
                    break;
                }
            }
            if !improved {
                if self.trust_region_adaptive {
                    current_trust_radius *= 0.5;
                    self.armature.unpack(&theta);
                    self.armature.update_kinematics();
                    // Continue with smaller radius next iteration
                } else {
                    self.armature.unpack(&theta);
                    self.armature.update_kinematics();
                    break;
                }
            } else if self.trust_region_adaptive {
                let actual = 0.5 * (err * err - best_err * best_err) as f64;
                let alpha_f = alpha_used as f64;
                let mut step_applied = Vector::with_capacity(p_vec.rows());
                step_applied.resize(p_vec.rows());
                for i in 0..p_vec.rows() {
                    step_applied.set(i, p_vec.get(i) * alpha_f);
                }
                let h_step = &hess * &step_applied;
                let predicted =
                    vector_dot(&jt_r, &step_applied) - 0.5 * vector_dot(&step_applied, &h_step);
                if predicted > 1e-20 {
                    let rho = actual / predicted;
                    if rho > 0.75 {
                        current_trust_radius *= 2.0;
                    } else if rho < 0.25 {
                        current_trust_radius *= 0.5;
                    }
                }
            }
        }

        self.armature.unpack(&best_state);
        self.armature.update_kinematics();
        best_err
    }

    fn residual(&self, ee: &Vector3f) -> [f64; 3] {
        [
            self.target.get(0) as f64 - ee.get(0) as f64,
            self.target.get(1) as f64 - ee.get(1) as f64,
            self.target.get(2) as f64 - ee.get(2) as f64,
        ]
    }

    fn norm3(&self, x: f32, y: f32, z: f32) -> f32 {
        (x * x + y * y + z * z).sqrt()
    }
}

/// Builds the Hessian H = J^T J − K:r for the current armature state and target (one Newton step).
/// Returns the flat Hessian (row-major, n×n), size n, and current position error magnitude.
/// Use for visualization (e.g. heatmap) without running the full solver.
///
/// Returns `None` if the path to the end-effector has zero DOF.
#[must_use]
pub fn hessian_snapshot(
    armature: &mut Armature,
    end_effector_idx: usize,
    target: Vector3f,
) -> Option<(Vec<f64>, usize, f32)> {
    armature.update_kinematics();
    let ee = armature.end_effector_position(end_effector_idx);
    let path = armature.path_to(end_effector_idx);
    let dof_total: usize = path
        .iter()
        .map(|&i| armature.tree().nodes[i].data.joint.dof_count())
        .sum();
    if dof_total == 0 {
        return None;
    }
    let (j_pos, axes, is_prismatic) = build_position_jacobian_and_axes(armature, &path, &ee);
    let r = [
        target.get(0) as f64 - ee.get(0) as f64,
        target.get(1) as f64 - ee.get(1) as f64,
        target.get(2) as f64 - ee.get(2) as f64,
    ];
    let r_vec = Vector::from_slice(&r);
    let hess = build_hessian(&j_pos, &axes, &is_prismatic, &r_vec);
    let n = hess.rows();
    let mut flat = Vec::with_capacity(n * n);
    for i in 0..n {
        for j in 0..n {
            flat.push(hess.get(i, j));
        }
    }
    let err = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt() as f32;
    Some((flat, n, err))
}

/// H = J^T J − K:r. Correction applied symmetrically to (i,j) and (j,i) so H stays symmetric
/// (analytic Hessian of ½‖r‖²; matches MATLAB reference Andrews HDM05).
#[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
fn build_hessian(
    j: &Matrix<f64>,
    axes: &[[f64; 3]],
    is_prismatic: &[bool],
    r: &Vector<f64>,
) -> Matrix<f64> {
    build_hessian_impl(j, axes, is_prismatic, r)
}

#[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
fn build_hessian(
    j: &Matrix<f64>,
    axes: &[[f64; 3]],
    is_prismatic: &[bool],
    r: &Vector<f64>,
) -> Matrix<f64> {
    use par_iter::prelude::*;
    let n = j.cols();
    let mut h = Matrix::with_storage(n, n, Storage::Column);
    let rows: Vec<Vec<f64>> = (0..n)
        .into_par_iter()
        .map(|i| {
            let mut row = vec![0.0; n];
            for k in 0..n {
                let mut dot = 0.0;
                for row_idx in 0..3 {
                    dot += j.get(row_idx, i) * j.get(row_idx, k);
                }
                row[k] = dot;
            }
            let r_slice: [f64; 3] = [r.get(0), r.get(1), r.get(2)];
            let ji = [j.get(0, i), j.get(1, i), j.get(2, i)];
            for jj in 0..n {
                if is_prismatic[i] || is_prismatic[jj] {
                    continue;
                }
                let w = axes[jj];
                let k_cross = vec3_cross_f64(&w, &ji);
                let k_dot_r = cpu::dot_f64(&k_cross, &r_slice);
                row[jj] -= k_dot_r;
            }
            row
        })
        .collect();
    for (i, row) in rows.into_iter().enumerate() {
        for (k, &v) in row.iter().enumerate() {
            h.set(i, k, v);
        }
    }
    // Enforce symmetry: use same correction for (j,i) as (i,j) (copy upper triangle to lower).
    for i in 0..n {
        for jj in (i + 1)..n {
            h.set(jj, i, h.get(i, jj));
        }
    }
    h
}

fn build_hessian_impl(
    j: &Matrix<f64>,
    axes: &[[f64; 3]],
    is_prismatic: &[bool],
    r: &Vector<f64>,
) -> Matrix<f64> {
    let n = j.cols();
    let mut h = Matrix::with_storage(n, n, Storage::Column);
    for i in 0..n {
        for k in 0..n {
            let mut dot = 0.0;
            for row in 0..3 {
                dot += j.get(row, i) * j.get(row, k);
            }
            h.set(i, k, dot);
        }
    }

    let r_slice: [f64; 3] = [r.get(0), r.get(1), r.get(2)];

    // Second-order correction H_ij -= dot(cross(axis_j, J_col_i), r). Apply the same
    // value to both (i,j) and (j,i) so H stays symmetric (matches analytic Hessian of ½‖r‖²).
    for i in 0..n {
        let ji = [j.get(0, i), j.get(1, i), j.get(2, i)];
        for jj in 0..n {
            if is_prismatic[i] || is_prismatic[jj] {
                continue;
            }
            let w = axes[jj];
            let k_cross = vec3_cross_f64(&w, &ji);
            let k_dot_r = cpu::dot_f64(&k_cross, &r_slice);
            let current_ij = h.get(i, jj);
            h.set(i, jj, current_ij - k_dot_r);
            if i != jj {
                let current_ji = h.get(jj, i);
                h.set(jj, i, current_ji - k_dot_r);
            }
        }
    }
    h
}

/// Solve H p = b. Try Cholesky; on NotSPD add λI and retry with increasing λ (iterative regularization).
fn solve_newton_step(
    h: &Matrix<f64>,
    b: &Vector<f64>,
    reg_base: f64,
    r: &Vector<f64>,
) -> Result<Vector<f64>, CholError> {
    match Cholesky::new(h) {
        Ok(c) => return Ok(c.solve(b)),
        Err(CholError::NotSPD) => {}
        Err(e) => return Err(e),
    }

    let r_norm_sq = r.get(0) * r.get(0) + r.get(1) * r.get(1) + r.get(2) * r.get(2);
    let mut lambda = reg_base * r_norm_sq.max(1e-8);
    const MAX_REG_ATTEMPTS: usize = 4;
    const REG_MULTIPLIER: f64 = 4.0;

    for _ in 0..MAX_REG_ATTEMPTS {
        let n = h.rows();
        let mut h_reg = h.clone();
        for i in 0..n {
            h_reg.set(i, i, h_reg.get(i, i) + lambda);
        }
        match Cholesky::new(&h_reg) {
            Ok(c) => return Ok(c.solve(b)),
            Err(CholError::NotSPD) => lambda *= REG_MULTIPLIER,
            Err(e) => return Err(e),
        }
    }
    Err(CholError::NotSPD)
}

fn vector_norm(v: &Vector<f64>) -> f64 {
    cpu::dot_f64(v.data(), v.data()).sqrt()
}

fn vector_dot(a: &Vector<f64>, b: &Vector<f64>) -> f64 {
    cpu::dot_f64(a.data(), b.data())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::armature::{Armature, JointData, JointVariant};
    use crate::joints::{RevoluteJoint, SphericalJoint};
    use mathlib::Quat4f;
    use mathlib::cg::vector3;

    #[test]
    fn hessian_is_symmetric() {
        // Build a small chain (2–3 DOF), form H, assert H - H' ≈ 0.
        let root = JointData::new(JointVariant::Spherical(SphericalJoint::new(
            vector3(0.0, 0.0, 0.0),
            Quat4f::identity(),
        )));
        let mut arm = Armature::new(root);
        arm.add_child(
            0,
            1,
            JointData::new(JointVariant::Revolute(RevoluteJoint::new(
                vector3(0.5, 0.0, 0.0),
                (0.0, 1.0, 0.0),
                0.1,
            ))),
        );
        arm.update_kinematics();
        let path = arm.path_to(1);
        let ee = arm.end_effector_position(1);
        let (j_pos, axes, is_prismatic) = build_position_jacobian_and_axes(&arm, &path, &ee);
        let r_vec = Vector::from_slice(&[0.01, -0.02, 0.01]);
        let h = build_hessian_impl(&j_pos, &axes, &is_prismatic, &r_vec);
        let n = h.rows();
        assert_eq!(n, h.cols());
        for i in 0..n {
            for j in 0..n {
                let diff = (h.get(i, j) - h.get(j, i)).abs();
                assert!(
                    diff < 1e-10,
                    "H({},{}) = {} vs H({},{}) = {} (diff {})",
                    i,
                    j,
                    h.get(i, j),
                    j,
                    i,
                    h.get(j, i),
                    diff
                );
            }
        }
    }
}
