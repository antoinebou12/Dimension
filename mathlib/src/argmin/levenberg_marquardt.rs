//! Levenberg-Marquardt for nonlinear least squares (Ch 12).
//!
//! Dampened Gauss-Newton: (J'J + λI) δ = -J'r with adaptive λ.

use crate::chol::{CholError, chol};
use crate::matrix::Matrix;
use crate::structure::Storage;
use crate::vector::Vector;
use tracing::debug;

fn vec_to_vector(v: &[f64]) -> Vector<f64> {
    let mut out = Vector::with_capacity(v.len());
    out.resize(v.len());
    for (i, &val) in v.iter().enumerate() {
        out.set(i, val);
    }
    out
}

/// Options for Levenberg-Marquardt.
#[derive(Clone, Debug)]
pub struct LevenbergMarquardtOptions {
    /// Maximum iterations (default 200).
    pub max_iters: usize,
    /// Stop when ||J'r|| below this (default 1e-8).
    pub tol: f64,
    /// Initial damping λ (default 1e-3).
    pub lambda0: f64,
    /// Factor to increase λ when step rejected (default 10.0).
    pub lambda_up: f64,
    /// Factor to decrease λ when step accepted (default 0.1).
    pub lambda_down: f64,
}

impl Default for LevenbergMarquardtOptions {
    fn default() -> Self {
        Self {
            max_iters: 200,
            tol: 1e-8,
            lambda0: 1e-3,
            lambda_up: 10.0,
            lambda_down: 0.1,
        }
    }
}

/// Result of Levenberg-Marquardt.
#[derive(Clone, Debug)]
pub struct LevenbergMarquardtResult {
    /// Best point found.
    pub x: Vec<f64>,
    /// Residual norm squared at best point (2 * cost).
    pub residual_norm_sq: f64,
    /// Number of iterations.
    pub iterations: usize,
}

/// Levenberg-Marquardt for nonlinear least squares: minimize (1/2)||r(x)||².
///
/// Step: (J'J + λI) δ = -J'r. λ is increased when step increases cost, decreased when it decreases cost.
///
/// # Errors
///
/// Returns `CholError` if J'J + λI fails to factor (should not happen with λ > 0).
#[must_use = "this `Result` may be an `Err` that should be handled"]
pub fn levenberg_marquardt<R, J>(
    x0: &[f64],
    residual: R,
    jacobian: J,
    options: &LevenbergMarquardtOptions,
) -> Result<LevenbergMarquardtResult, CholError>
where
    R: Fn(&[f64]) -> Vec<f64>,
    J: Fn(&[f64]) -> Matrix<f64>,
{
    let n = x0.len();
    let mut x = x0.to_vec();
    let r = residual(&x);
    let m = r.len();
    assert!(m >= n, "need m >= n for least squares");
    let tol_sq = options.tol * options.tol;
    let mut lambda = options.lambda0;

    let mut jt_j = Matrix::with_storage(n, n, Storage::Column);
    let mut jt_j_lam = Matrix::with_storage(n, n, Storage::Column);

    let mut iter = 0_usize;
    while iter < options.max_iters {
        let r_cur = residual(&x);
        let cost_val = 0.5 * r_cur.iter().map(|v| v * v).sum::<f64>();
        let j_mat = jacobian(&x);
        assert_eq!(j_mat.rows(), m);
        assert_eq!(j_mat.cols(), n);

        let jt = j_mat.transpose();
        jt.mul_into(&j_mat, &mut jt_j);

        // J'J + λI
        for i in 0..n {
            for j in 0..n {
                let v = jt_j.get(i, j) + if i == j { lambda } else { 0.0 };
                jt_j_lam.set(i, j, v);
            }
        }

        let r_vec = vec_to_vector(&r_cur);
        let jt_r = &jt * &r_vec;
        let grad_norm_sq = crate::cpu::dot_f64(jt_r.data(), jt_r.data());

        if grad_norm_sq <= tol_sq {
            debug!(
                iter,
                residual_norm_sq = %(2.0 * cost_val),
                "levenberg_marquardt converged"
            );
            return Ok(LevenbergMarquardtResult {
                x: x.clone(),
                residual_norm_sq: 2.0 * cost_val,
                iterations: iter,
            });
        }

        let chol_fac = chol(&jt_j_lam)?;
        let mut neg_jt_r = Vector::with_capacity(n);
        neg_jt_r.resize(n);
        for i in 0..n {
            neg_jt_r.set(i, -jt_r.get(i));
        }
        let delta = chol_fac.solve(&neg_jt_r);
        let d: Vec<f64> = (0..n).map(|i| delta.get(i)).collect();

        let x_trial: Vec<f64> = x.iter().zip(d.iter()).map(|(a, b)| a + b).collect();
        let r_trial = residual(&x_trial);
        let cost_trial = 0.5 * r_trial.iter().map(|v| v * v).sum::<f64>();

        if cost_trial < cost_val {
            x = x_trial;
            lambda = (lambda * options.lambda_down).max(1e-16);
        } else {
            lambda *= options.lambda_up;
        }

        debug!(
            iter,
            cost = %cost_val,
            grad_norm = %(grad_norm_sq.sqrt()),
            lambda,
            "levenberg_marquardt"
        );
        iter += 1;
    }

    let cost_val = 0.5 * residual(&x).iter().map(|v| v * v).sum::<f64>();

    Ok(LevenbergMarquardtResult {
        x,
        residual_norm_sq: 2.0 * cost_val,
        iterations: iter,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lm_linear_fit() {
        let x0 = vec![0.5_f64, 0.5];
        let xs = [0.0_f64, 1.0, 2.0];
        let ys = [0.0_f64, 1.0, 2.0];
        let residual = |x: &[f64]| {
            let (a, b) = (x[0], x[1]);
            xs.iter()
                .zip(ys.iter())
                .map(|(&xi, &yi)| yi - (a * xi + b))
                .collect::<Vec<_>>()
        };
        let jacobian = |x: &[f64]| {
            let _ = x;
            let mut jac = Matrix::with_storage(3, 2, Storage::Column);
            for (i, &xi) in xs.iter().enumerate() {
                jac.set(i, 0, -xi);
                jac.set(i, 1, -1.0);
            }
            jac
        };
        let opts = LevenbergMarquardtOptions::default();
        let result = levenberg_marquardt(&x0, residual, jacobian, &opts).unwrap();
        assert!((result.x[0] - 1.0).abs() < 1e-5);
        assert!(result.x[1].abs() < 1e-5);
    }
}
