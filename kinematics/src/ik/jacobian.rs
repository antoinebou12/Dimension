//! Jacobian-based IK using SVD pseudoinverse and line search.

use mathlib::{Matrix, Storage, Vector3f, svd_econ};

use super::chain::build_geometric_jacobian;
use crate::armature::Armature;

/// Jacobian IK solver (SVD + line search).
pub struct JacobianIk<'a> {
    armature: &'a mut Armature,
    end_effector_idx: usize,
    target: Vector3f,
    max_iters: usize,
    sigma_min: f64,
}

impl<'a> JacobianIk<'a> {
    /// Creates a Jacobian IK solver.
    pub fn new(armature: &'a mut Armature, end_effector_idx: usize, target: Vector3f) -> Self {
        Self {
            armature,
            end_effector_idx,
            target,
            max_iters: 50,
            sigma_min: 1e-6,
        }
    }

    /// Sets maximum iterations.
    #[must_use]
    pub fn with_max_iters(mut self, n: usize) -> Self {
        self.max_iters = n;
        self
    }

    /// Performs one or more IK steps; returns final error magnitude.
    ///
    /// Uses a stagnation counter so that when the target moves, the solver
    /// can keep tracking instead of breaking on the first non-improving step.
    pub fn solve(&mut self) -> f32 {
        let mut best_err = f32::MAX;
        let mut stagnant = 0;
        for _ in 0..self.max_iters {
            let err = self.step();
            if err < 1e-5 {
                return err;
            }
            if err < best_err {
                best_err = err;
                stagnant = 0;
            } else {
                stagnant += 1;
                if stagnant >= 3 {
                    break;
                }
            }
        }
        best_err
    }

    /// Performs one IK step; returns error magnitude after the step.
    pub fn step(&mut self) -> f32 {
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

        let j = self.build_jacobian(&path, &ee);
        let j_f64 = self.matrix_f32_to_f64(&j);
        let svd = svd_econ(&j_f64);
        let d_theta = self.svd_solve(&svd, &dx);
        if d_theta.is_empty() {
            return err;
        }

        let dof_mapping = self.armature.path_to_dfs_dof_mapping(&path);
        let theta = self.armature.pack();
        let mut alpha = 1.0f32;
        for _ in 0..8 {
            let mut theta_new = theta.clone();
            for (path_dof_idx, &dt) in d_theta.iter().enumerate() {
                if let Some(&dfs_dof_idx) = dof_mapping.get(path_dof_idx) {
                    if dfs_dof_idx < theta_new.len() {
                        theta_new[dfs_dof_idx] += alpha * dt;
                    }
                }
            }
            self.armature.unpack(&theta_new);
            self.armature.update_kinematics();
            let ee_new = self.armature.end_effector_position(self.end_effector_idx);
            let dx_new = self.delta_to_target(&ee_new);
            let err_new = self.norm3(dx_new.get(0), dx_new.get(1), dx_new.get(2));
            if err_new < err {
                return err_new;
            }
            alpha *= 0.5;
        }
        self.armature.unpack(&theta);
        self.armature.update_kinematics();
        err
    }

    fn delta_to_target(&self, ee: &Vector3f) -> Vector3f {
        let mut d = Vector3f::with_capacity(3);
        d.set(0, self.target.get(0) - ee.get(0));
        d.set(1, self.target.get(1) - ee.get(1));
        d.set(2, self.target.get(2) - ee.get(2));
        d
    }

    fn build_jacobian(&self, path: &[usize], ee_pos: &Vector3f) -> Vec<Vec<f32>> {
        let (jacobian, _) = build_geometric_jacobian(self.armature, path, ee_pos);
        let cols = jacobian.cols();
        let mut j = vec![vec![0.0f32; cols], vec![0.0f32; cols], vec![0.0f32; cols]];
        for col in 0..cols {
            for row in 0..3 {
                j[row][col] = jacobian.get(row, col) as f32;
            }
        }
        j
    }

    fn matrix_f32_to_f64(&self, j: &[Vec<f32>]) -> Matrix<f64> {
        let rows = j.len();
        let cols = if rows > 0 { j[0].len() } else { 0 };
        let mut m = Matrix::with_storage(rows, cols, Storage::Column);
        for (i, row) in j.iter().enumerate().take(rows) {
            for (c, &v) in row.iter().enumerate() {
                m.set(i, c, v as f64);
            }
        }
        m
    }

    fn svd_solve(&self, svd: &mathlib::SvdEcon, dx: &Vector3f) -> Vec<f32> {
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
            *y_i = if s > self.sigma_min { b[i] / s } else { 0.0 };
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
