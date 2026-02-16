//! Jacobian-based IK using damped least-squares (with SVD fallback) and line search.
//!
//! Uses [damped_least_squares](mathlib::damped_least_squares) on the 3×N position Jacobian for
//! speed; falls back to SVD pseudoinverse when the normal equations are singular (e.g.
//! CholError::NotSPD). Step size is capped via full-step scaling (inf-norm), preserving
//! direction while respecting [max_delta_rad](JacobianIk::with_max_delta_rad). Adaptive damping
//! increases when the line search fails to improve, reducing blow-up near singularities.
//! Relative-improvement stagnation exits early when improvement over consecutive steps is tiny.
//!
//! For spherical joints, packed state uses scaled axis (tangent space); the solver
//! converts world-frame angular increments to parent-local before applying.
//! Early exit uses a configurable position tolerance, consistent with
//! [FabrikIk::with_tolerance](super::fabrik::FabrikIk::with_tolerance).

use mathlib::{Vector, Vector3f, damped_least_squares, svd_econ};

use super::chain::{apply_joint_step, build_position_jacobian_and_axes};
use crate::armature::Armature;

/// Relative improvement below this over consecutive steps triggers stagnation exit.
const RELATIVE_STAGNATION_THRESHOLD: f32 = 1e-4;
/// Maximum damping when adapting (avoid overly small steps).
const MAX_DAMPING: f64 = 1.0;
/// Damping multiplier when line search finds no improving step.
const DAMPING_INCREASE_FACTOR: f64 = 2.0;

/// Jacobian IK solver (damped least-squares + line search).
pub struct JacobianIk<'a> {
    armature: &'a mut Armature,
    end_effector_idx: usize,
    target: Vector3f,
    max_iters: usize,
    /// Position error threshold for early exit (stop when error < tolerance), same as FABRIK.
    tolerance: f32,
    /// Damping for damped least-squares; increased adaptively when line search fails.
    damping: f64,
    /// Cap on step size (inf-norm of d_theta in radians); None = no cap.
    max_delta_rad: Option<f32>,
}

impl<'a> JacobianIk<'a> {
    /// Creates a Jacobian IK solver.
    pub fn new(armature: &'a mut Armature, end_effector_idx: usize, target: Vector3f) -> Self {
        Self {
            armature,
            end_effector_idx,
            target,
            max_iters: 50,
            tolerance: 1e-5,
            damping: 1e-2,
            max_delta_rad: Some(0.5),
        }
    }

    /// Sets maximum iterations.
    #[must_use]
    pub fn with_max_iters(mut self, n: usize) -> Self {
        self.max_iters = n;
        self
    }

    /// Sets damping for damped least-squares (default 1e-2). Larger values give smaller steps.
    #[must_use]
    pub fn with_damping(mut self, damping: f64) -> Self {
        self.damping = damping;
        self
    }

    /// Sets maximum joint delta per step in radians (default 0.5). None disables clamping.
    #[must_use]
    pub fn with_max_delta_rad(mut self, max: Option<f32>) -> Self {
        self.max_delta_rad = max;
        self
    }

    /// Sets position tolerance for early exit; solver stops when error is below this (same as FABRIK).
    #[must_use]
    pub fn with_tolerance(mut self, tol: f32) -> Self {
        self.tolerance = tol;
        self
    }

    /// Updates the target position.
    pub fn set_target(&mut self, target: Vector3f) {
        self.target = target;
    }

    /// Performs one or more IK steps; returns final error magnitude.
    ///
    /// Stops when error is below tolerance, or when relative improvement is below
    /// threshold for two consecutive improving steps, or after three non-improving steps, or max_iters.
    pub fn solve(&mut self) -> f32 {
        let mut best_err = f32::MAX;
        let mut stagnant = 0u32;
        let mut small_improvement_count = 0u32;
        let mut current_damping = self.damping;
        for _ in 0..self.max_iters {
            let err = self.step(&mut current_damping);
            if err < self.tolerance {
                return err;
            }
            if err < best_err {
                let prev = best_err;
                best_err = err;
                stagnant = 0;
                if prev < f32::MAX {
                    let rel = (prev - err) / (prev + 1e-9f32);
                    if rel < RELATIVE_STAGNATION_THRESHOLD {
                        small_improvement_count += 1;
                        if small_improvement_count >= 2 {
                            break;
                        }
                    } else {
                        small_improvement_count = 0;
                    }
                }
            } else {
                stagnant += 1;
                small_improvement_count = 0;
                if stagnant >= 3 {
                    break;
                }
            }
        }
        best_err
    }

    /// Performs one IK step; returns error magnitude after the step.
    /// Updates `current_damping` when line search fails (increase) or succeeds (reset to base).
    pub fn step(&mut self, current_damping: &mut f64) -> f32 {
        self.armature.update_kinematics();
        let ee = self.armature.end_effector_position(self.end_effector_idx);
        let dx = self.delta_to_target(&ee);
        let err = self.norm3(dx.get(0), dx.get(1), dx.get(2));
        if err < 1e-6 {
            return err;
        }

        let path = self.armature.path_to(self.end_effector_idx);
        let dof_total: usize = path
            .iter()
            .map(|&i| self.armature.tree().nodes[i].data.joint.dof_count())
            .sum();
        if dof_total == 0 {
            return err;
        }

        let (j_pos, _, _) = build_position_jacobian_and_axes(self.armature, &path, &ee);
        let mut dx_v = Vector::with_capacity(3);
        dx_v.set(0, dx.get(0) as f64);
        dx_v.set(1, dx.get(1) as f64);
        dx_v.set(2, dx.get(2) as f64);

        let mut d_theta = match damped_least_squares(&j_pos, &dx_v, *current_damping) {
            Ok(x) => (0..x.rows()).map(|i| x.get(i) as f32).collect::<Vec<f32>>(),
            Err(_) => {
                let svd = svd_econ(&j_pos);
                self.svd_solve(&svd, &dx, *current_damping)
            }
        };
        if d_theta.is_empty() {
            return err;
        }

        if let Some(max_d) = self.max_delta_rad {
            let max_abs = d_theta.iter().map(|&t| t.abs()).fold(0.0f32, f32::max);
            if max_abs > max_d && max_abs > 1e-9 {
                let scale = max_d / max_abs;
                for dt in &mut d_theta {
                    *dt *= scale;
                }
            }
        }

        let dof_mapping = self.armature.path_to_dfs_dof_mapping(&path);
        let theta = self.armature.pack();
        let mut best_err_new = err;
        let mut best_theta_new = theta.clone();
        let mut theta_new = theta.clone();
        const ALPHAS: [f32; 4] = [1.0, 0.5, 0.25, 0.125];
        for alpha in ALPHAS {
            theta_new.copy_from_slice(&theta);
            apply_joint_step(
                self.armature.tree(),
                &path,
                &dof_mapping,
                alpha,
                &d_theta,
                &mut theta_new,
            );
            self.armature.unpack(&theta_new);
            self.armature.update_kinematics();
            let ee_new = self.armature.end_effector_position(self.end_effector_idx);
            let dx_new = self.delta_to_target(&ee_new);
            let err_new = self.norm3(dx_new.get(0), dx_new.get(1), dx_new.get(2));
            if err_new < err {
                *current_damping = self.damping;
                return err_new;
            }
            if err_new < best_err_new {
                best_err_new = err_new;
                best_theta_new.copy_from_slice(&theta_new);
            }
        }
        *current_damping = (*current_damping * DAMPING_INCREASE_FACTOR).min(MAX_DAMPING);
        self.armature.unpack(&best_theta_new);
        self.armature.update_kinematics();
        best_err_new
    }

    fn delta_to_target(&self, ee: &Vector3f) -> Vector3f {
        let mut d = Vector3f::with_capacity(3);
        d.set(0, self.target.get(0) - ee.get(0));
        d.set(1, self.target.get(1) - ee.get(1));
        d.set(2, self.target.get(2) - ee.get(2));
        d
    }

    fn svd_solve(&self, svd: &mathlib::SvdEcon, dx: &Vector3f, damping: f64) -> Vec<f32> {
        let u = svd.u();
        let v = svd.v();
        let sigma = svd.sigma();
        let k = sigma.rows();
        let n = v.rows();
        let mut b = vec![0.0f64; k];
        for i in 0..3.min(k) {
            for (j, b_j) in b.iter_mut().enumerate().take(k) {
                *b_j += u.get(i, j) * dx.get(i) as f64;
            }
        }
        let mut y = vec![0.0f64; k];
        for (i, y_i) in y.iter_mut().enumerate().take(k) {
            let s = sigma.get(i);
            let s_sq = s * s;
            let denom = s_sq + damping;
            *y_i = if denom > 1e-20 {
                (s * b[i]) / denom
            } else {
                0.0
            };
        }
        let mut d_theta = vec![0.0f32; n];
        for (i, d_i) in d_theta.iter_mut().enumerate().take(n) {
            for (j, y_j) in y.iter().enumerate().take(k) {
                *d_i += (v.get(i, j) * y_j) as f32;
            }
        }
        d_theta
    }

    fn norm3(&self, x: f32, y: f32, z: f32) -> f32 {
        (x * x + y * y + z * z).sqrt()
    }
}
