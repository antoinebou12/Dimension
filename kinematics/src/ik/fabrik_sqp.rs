//! FABRIK + SQP hybrid IK solver.
//!
//! Based on Xu et al., "A Combined Inverse Kinematics Algorithm Using FABRIK with Optimization"
//! (arXiv:2209.02532). Runs FABRIK for a limited number of iterations; if it does not converge
//! within the switch index `nl`, falls back to projected gradient descent with joint limits,
//! using the FABRIK output as initial seed. Improves stability and convergence under high
//! error constraints (εtol ≈ 10⁻⁶).

use mathlib::cg::vector3;
use mathlib::math3d::transform_vector;
use mathlib::{Matrix, Storage, Vector3f};

use super::chain::build_geometric_jacobian;
use super::fabrik::FabrikIk;
use crate::armature::{Armature, JointVariant};

const DEFAULT_NL: usize = 10;
const DEFAULT_TOLERANCE: f32 = 1e-5;
const DEFAULT_MAX_ITERS_FALLBACK: usize = 50;
const LOOSE_MIN: f32 = -std::f32::consts::PI;
const LOOSE_MAX: f32 = std::f32::consts::PI;

/// FABRIK + SQP hybrid IK solver for position-only chains.
///
/// Phase 1: Run FABRIK for at most `nl` iterations. If convergence (err ≤ εtol), done.
/// Phase 2: Otherwise, run projected gradient descent with joint limits using FABRIK output
/// as initial seed.
pub struct FabrikSqpIk<'a> {
    armature: &'a mut Armature,
    end_effector_idx: usize,
    target: Vector3f,
    nl: usize,
    tolerance: f32,
    max_iters_fallback: usize,
}

impl<'a> FabrikSqpIk<'a> {
    /// Creates a FABRIK+SQP solver.
    pub fn new(armature: &'a mut Armature, end_effector_idx: usize, target: Vector3f) -> Self {
        Self {
            armature,
            end_effector_idx,
            target,
            nl: DEFAULT_NL,
            tolerance: DEFAULT_TOLERANCE,
            max_iters_fallback: DEFAULT_MAX_ITERS_FALLBACK,
        }
    }

    /// Sets the FABRIK switch index (max FABRIK iterations before fallback). Paper: 5 for 2D, 15 for 3D.
    #[must_use]
    pub fn with_nl(mut self, nl: usize) -> Self {
        self.nl = nl;
        self
    }

    /// Sets position tolerance for convergence.
    #[must_use]
    pub fn with_tolerance(mut self, tol: f32) -> Self {
        self.tolerance = tol;
        self
    }

    /// Sets maximum iterations for the fallback optimization phase.
    #[must_use]
    pub fn with_max_iters_fallback(mut self, n: usize) -> Self {
        self.max_iters_fallback = n;
        self
    }

    /// Runs the hybrid solver; updates armature joint angles. Returns final error.
    pub fn solve(&mut self) -> f32 {
        let path = self.armature.path_to(self.end_effector_idx);
        if path.len() < 2 {
            return self.error();
        }

        // Phase 1: FABRIK with nl iterations
        let err_fabrik = FabrikIk::new(self.armature, self.end_effector_idx, self.target.clone())
            .with_max_iters(self.nl)
            .with_tolerance(self.tolerance)
            .solve();

        if err_fabrik <= self.tolerance {
            return err_fabrik;
        }

        // Phase 2: Projected gradient descent with joint limits
        self.run_fallback_optimization(&path)
    }

    fn run_fallback_optimization(&mut self, path: &[usize]) -> f32 {
        let dof_total: usize = path
            .iter()
            .map(|&i| self.armature.tree().nodes[i].data.joint.dof_count())
            .sum();
        if dof_total == 0 {
            return self.error();
        }

        let limits = Self::path_joint_limits(self.armature, path);
        let dof_mapping = self.armature.path_to_dfs_dof_mapping(path);
        let mut theta = self.armature.pack();

        let mut best_err = self.error();
        let mut best_theta = theta.clone();
        let mut alpha = 0.5f32;

        for _ in 0..self.max_iters_fallback {
            self.armature.update_kinematics();
            let ee = self.armature.end_effector_position(self.end_effector_idx);
            let residual = vector3(
                ee.get(0) - self.target.get(0),
                ee.get(1) - self.target.get(1),
                ee.get(2) - self.target.get(2),
            );
            let err = (residual.get(0) * residual.get(0)
                + residual.get(1) * residual.get(1)
                + residual.get(2) * residual.get(2))
            .sqrt();
            if err <= self.tolerance {
                return err;
            }
            if err < best_err {
                best_err = err;
                best_theta.copy_from_slice(&theta);
            }

            // Gradient descent: J^T * (ee - target) gives descent direction for min ||ee - target||²
            let (jacobian_full, _) = build_geometric_jacobian(self.armature, path, &ee);
            let j_pos = Self::jacobian_position_3xn(&jacobian_full);
            let mut d_theta = vec![0.0f64; dof_total];
            for (col, dt) in d_theta.iter_mut().enumerate() {
                let mut sum = 0.0;
                for row in 0..3 {
                    sum += j_pos.get(row, col) * residual.get(row) as f64;
                }
                *dt = sum;
            }

            let max_abs = d_theta
                .iter()
                .map(|&t| t.abs() as f32)
                .fold(0.0f32, f32::max);
            let scale = if max_abs > 0.5 { 0.5 / max_abs } else { 1.0 };

            let mut theta_new = theta.clone();
            let mut path_dof = 0;
            let tree = self.armature.tree();
            for (path_idx, &node_idx) in path.iter().enumerate() {
                let dof = tree.nodes[node_idx].data.joint.dof_count();
                if dof == 0 {
                    continue;
                }
                let dfs_start = dof_mapping[path_dof];
                let joint = &tree.nodes[node_idx].data.joint;
                let is_spherical = matches!(joint, JointVariant::Spherical(_));

                if is_spherical && dof == 3 && path_idx > 0 {
                    let parent_idx = path[path_idx - 1];
                    let parent_world = &tree.nodes[parent_idx].data.world_transform;
                    let parent_r_t = parent_world.transpose();
                    let mut d_world = Vector3f::with_capacity(3);
                    d_world.set(0, (scale * d_theta[path_dof] as f32) * alpha);
                    d_world.set(1, (scale * d_theta[path_dof + 1] as f32) * alpha);
                    d_world.set(2, (scale * d_theta[path_dof + 2] as f32) * alpha);
                    let d_local = transform_vector(&parent_r_t, &d_world);
                    for d in 0..3 {
                        let (min, max) = limits
                            .get(path_dof + d)
                            .copied()
                            .unwrap_or((LOOSE_MIN, LOOSE_MAX));
                        if dfs_start + d < theta_new.len() {
                            theta_new[dfs_start + d] =
                                (theta_new[dfs_start + d] - d_local.get(d)).clamp(min, max);
                        }
                    }
                } else {
                    for d in 0..dof {
                        let (min, max) = limits
                            .get(path_dof + d)
                            .copied()
                            .unwrap_or((LOOSE_MIN, LOOSE_MAX));
                        let step = (scale * d_theta[path_dof + d] as f32) * alpha;
                        if dfs_start + d < theta_new.len() {
                            theta_new[dfs_start + d] =
                                (theta_new[dfs_start + d] - step).clamp(min, max);
                        }
                    }
                }
                path_dof += dof;
            }

            self.armature.unpack(&theta_new);
            self.armature.update_kinematics();
            let err_new = self.error();
            if err_new < best_err {
                best_err = err_new;
                best_theta.copy_from_slice(&theta_new);
            }
            if err_new < err {
                theta = theta_new;
                alpha = (alpha * 1.2).min(1.0);
            } else {
                alpha *= 0.5;
                if alpha < 1e-4 {
                    break;
                }
                self.armature.unpack(&theta);
                self.armature.update_kinematics();
            }
        }

        self.armature.unpack(&best_theta);
        self.armature.update_kinematics();
        self.error()
    }

    fn path_joint_limits(armature: &Armature, path: &[usize]) -> Vec<(f32, f32)> {
        let mut limits = Vec::new();
        for &node_idx in path {
            let joint = &armature.tree().nodes[node_idx].data.joint;
            match joint {
                JointVariant::Revolute(r) => {
                    let (min, max) = match (r.angle_min, r.angle_max) {
                        (Some(lo), Some(hi)) => (lo, hi),
                        _ => (LOOSE_MIN, LOOSE_MAX),
                    };
                    limits.push((min, max));
                }
                JointVariant::Revolute2d(r) => {
                    let (min, max) = match (r.angle_min, r.angle_max) {
                        (Some(lo), Some(hi)) => (lo, hi),
                        _ => (LOOSE_MIN, LOOSE_MAX),
                    };
                    limits.push((min, max));
                }
                JointVariant::Spherical(_) => {
                    limits.push((LOOSE_MIN, LOOSE_MAX));
                    limits.push((LOOSE_MIN, LOOSE_MAX));
                    limits.push((LOOSE_MIN, LOOSE_MAX));
                }
                JointVariant::Prismatic(_) | JointVariant::Prismatic2d(_) => {
                    limits.push((LOOSE_MIN, LOOSE_MAX));
                }
                JointVariant::Fixed(_) | JointVariant::Fixed2d(_) => {}
            }
        }
        limits
    }

    fn jacobian_position_3xn(jacobian: &Matrix<f64>) -> Matrix<f64> {
        let cols = jacobian.cols();
        let mut j_pos = Matrix::with_storage(3, cols, Storage::Column);
        for col in 0..cols {
            for row in 0..3 {
                j_pos.set(row, col, jacobian.get(row, col));
            }
        }
        j_pos
    }

    fn error(&mut self) -> f32 {
        self.armature.update_kinematics();
        let ee = self.armature.end_effector_position(self.end_effector_idx);
        let dx = ee.get(0) - self.target.get(0);
        let dy = ee.get(1) - self.target.get(1);
        let dz = ee.get(2) - self.target.get(2);
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}
