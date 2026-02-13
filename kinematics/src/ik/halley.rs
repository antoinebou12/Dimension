//! Halley / QuIK-style IK solver for serial chains (6D pose goals).
//!
//! Based on “Fast and Robust Inverse Kinematics of Serial Robots Using Halley’s Method”
//! (S. Lloyd et al., IEEE T-RO 2022). Uses a damped Newton step followed by a
//! third-order correction via the geometric Hessian.

use super::chain::{build_geometric_jacobian, hessian_product, world_position};
use crate::armature::Armature;
use mathlib::{Matrix4f, Vector, clamp_twist, damped_least_squares, pose_twist_error};

/// Halley inverse-kinematics solver for serial chains.
pub struct HalleyIk<'a> {
    armature: &'a mut Armature,
    end_effector_idx: usize,
    target: Matrix4f,
    max_iters: usize,
    tolerance: f32,
    lambda_sq: f64,
    max_linear_step: f32,
    max_angular_step: f32,
    min_step_norm: f32,
}

impl<'a> HalleyIk<'a> {
    /// Creates a Halley IK solver with default tolerances.
    pub fn new(armature: &'a mut Armature, end_effector_idx: usize, target: Matrix4f) -> Self {
        Self {
            armature,
            end_effector_idx,
            target,
            max_iters: 32,
            tolerance: 1e-4,
            lambda_sq: 1e-8,
            max_linear_step: 0.05,
            max_angular_step: 0.25,
            min_step_norm: 1e-6,
        }
    }

    /// Sets maximum iterations.
    #[must_use]
    pub fn with_max_iters(mut self, max_iters: usize) -> Self {
        self.max_iters = max_iters;
        self
    }

    /// Sets convergence tolerance on the 6D twist norm.
    #[must_use]
    pub fn with_tolerance(mut self, tolerance: f32) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// Sets the damping term (λ²) used by the damped least-squares solves.
    #[must_use]
    pub fn with_damping(mut self, lambda_sq: f64) -> Self {
        self.lambda_sq = lambda_sq;
        self
    }

    /// Sets per-iteration twist clamps (meters and radians).
    #[must_use]
    pub fn with_step_limits(mut self, max_linear: f32, max_angular: f32) -> Self {
        self.max_linear_step = max_linear;
        self.max_angular_step = max_angular;
        self
    }

    /// Sets the minimum joint-space step norm before giving up.
    #[must_use]
    pub fn with_min_step(mut self, min_step_norm: f32) -> Self {
        self.min_step_norm = min_step_norm;
        self
    }

    /// Updates the target pose.
    pub fn set_target(&mut self, target: Matrix4f) {
        self.target = target;
    }

    /// Runs Halley iterations, returning the final pose error magnitude.
    pub fn solve(&mut self) -> f32 {
        let mut best_err = f32::MAX;
        let mut best_state = self.armature.pack();

        for _ in 0..self.max_iters {
            self.armature.update_kinematics();
            let current_tf = self.current_transform();
            let error = pose_twist_error(&current_tf, &self.target);
            let err_norm = twist_norm(&error);
            if err_norm < best_err {
                best_err = err_norm;
                best_state = self.armature.pack();
            }
            if err_norm < self.tolerance {
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

            let ee_pos = world_position(&current_tf);
            let (jacobian, link_types) = build_geometric_jacobian(self.armature, &path, &ee_pos);
            if jacobian.cols() == 0 {
                break;
            }

            let mut solve_error = error.clone();
            clamp_twist(
                &mut solve_error,
                self.max_linear_step,
                self.max_angular_step,
            );
            let error_f64 = to_f64(&solve_error);

            let newton_step = match damped_least_squares(&jacobian, &error_f64, self.lambda_sq) {
                Ok(step) => step,
                Err(_) => break,
            };
            if vector_norm(&newton_step) < f64::from(self.min_step_norm) {
                break;
            }

            let mut scaled = newton_step.clone();
            scale_vector(&mut scaled, 0.5);

            let mut halley_matrix = jacobian.clone();
            hessian_product(&jacobian, &scaled, &link_types, &mut halley_matrix);
            let halley_step = damped_least_squares(&halley_matrix, &error_f64, self.lambda_sq)
                .unwrap_or_else(|_| newton_step.clone());

            let final_step = halley_step;

            match self.try_step(&final_step, err_norm) {
                Some(new_err) => {
                    if new_err < best_err {
                        best_err = new_err;
                        best_state = self.armature.pack();
                    }
                }
                None => break,
            }
        }

        self.armature.unpack(&best_state);
        self.armature.update_kinematics();
        best_err
    }

    fn current_transform(&self) -> Matrix4f {
        self.armature.tree().nodes[self.end_effector_idx]
            .data
            .world_transform
            .clone()
    }

    fn try_step(&mut self, delta: &Vector<f64>, current_err: f32) -> Option<f32> {
        let theta = self.armature.pack();
        let mut alpha = 1.0_f32;
        for _ in 0..6 {
            let mut theta_new = theta.clone();
            for i in 0..delta.rows() {
                if i < theta_new.len() {
                    theta_new[i] += alpha * delta.get(i) as f32;
                }
            }
            self.armature.unpack(&theta_new);
            self.armature.update_kinematics();
            let new_tf = self.current_transform();
            let new_err = twist_norm(&pose_twist_error(&new_tf, &self.target));
            if new_err < current_err {
                return Some(new_err);
            }
            alpha *= 0.5;
        }
        self.armature.unpack(&theta);
        self.armature.update_kinematics();
        None
    }
}

fn twist_norm(v: &Vector<f32>) -> f32 {
    let mut sum = 0.0;
    for i in 0..v.rows() {
        let val = v.get(i);
        sum += val * val;
    }
    sum.sqrt()
}

fn to_f64(v: &Vector<f32>) -> Vector<f64> {
    let mut out = Vector::with_capacity(v.rows());
    for i in 0..v.rows() {
        out.set(i, f64::from(v.get(i)));
    }
    out
}

fn vector_norm(v: &Vector<f64>) -> f64 {
    let mut sum = 0.0;
    for i in 0..v.rows() {
        let val = v.get(i);
        sum += val * val;
    }
    sum.sqrt()
}

fn scale_vector(v: &mut Vector<f64>, scale: f64) {
    for i in 0..v.rows() {
        v.set(i, v.get(i) * scale);
    }
}
