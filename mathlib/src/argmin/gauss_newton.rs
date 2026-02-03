//! Gauss-Newton for nonlinear least squares: min ||r(x)||².
//!
//! Step from normal equations (J'J) Δ = -J'r; line search on merit (1/2)||r||².
//! Set `RUST_LOG=mathlib=debug` to see iteration logs.

use crate::chol::{CholError, chol};
use crate::matrix::Matrix;
use crate::structure::Storage;
use crate::vector::Vector;
use tracing::debug;

use super::linesearch::{self, LineSearchOptions};

/// Options for Gauss-Newton.
#[derive(Clone, Debug)]
pub struct GaussNewtonOptions {
    /// Maximum iterations (default 100).
    pub max_iters: usize,
    /// Stop when ||J'r|| below this (default 1e-8).
    pub tol: f64,
    /// Line search options.
    pub line_search_options: LineSearchOptions,
}

impl Default for GaussNewtonOptions {
    fn default() -> Self {
        Self {
            max_iters: 100,
            tol: 1e-8,
            line_search_options: LineSearchOptions::default(),
        }
    }
}

/// Result of Gauss-Newton.
#[derive(Clone, Debug)]
pub struct GaussNewtonResult {
    /// Best point found.
    pub x: Vec<f64>,
    /// Residual norm squared at best point (2 * cost).
    pub residual_norm_sq: f64,
    /// Number of iterations.
    pub iterations: usize,
}

fn vec_to_vector(v: &[f64]) -> Vector<f64> {
    let mut out = Vector::with_capacity(v.len());
    out.resize(v.len());
    for (i, &val) in v.iter().enumerate() {
        out.set(i, val);
    }
    out
}

/// Gauss-Newton for nonlinear least squares: minimize (1/2)||r(x)||².
/// Caller provides residual `residual(x)` and Jacobian `jacobian(x)` (J has rows = len(r), cols = len(x)).
///
/// # Errors
///
/// Returns `CholError` if J'J is not positive definite (e.g. singular Jacobian).
#[must_use = "this `Result` may be an `Err` that should be handled"]
pub fn gauss_newton<R, J>(
    x0: &[f64],
    residual: R,
    jacobian: J,
    options: &GaussNewtonOptions,
) -> Result<GaussNewtonResult, CholError>
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

    let mut x_new = vec![0.0_f64; n];
    let mut jt_j = Matrix::with_storage(n, n, Storage::Column);

    let mut iter = 0_usize;
    while iter < options.max_iters {
        let r_cur = residual(&x);
        let cost_val = 0.5 * r_cur.iter().map(|v| v * v).sum::<f64>();
        let j_mat = jacobian(&x);
        assert_eq!(j_mat.rows(), m);
        assert_eq!(j_mat.cols(), n);

        let jt = j_mat.transpose();
        jt.mul_into(&j_mat, &mut jt_j);
        let r_vec = vec_to_vector(&r_cur);
        let jt_r = &jt * &r_vec;
        let grad_norm_sq = crate::cpu::dot_f64(jt_r.data(), jt_r.data());

        if grad_norm_sq <= tol_sq {
            debug!(iter, residual_norm_sq = %(2.0 * cost_val), "gauss_newton converged");
            return Ok(GaussNewtonResult {
                x: x.clone(),
                residual_norm_sq: 2.0 * cost_val,
                iterations: iter,
            });
        }

        let chol_fac = chol(&jt_j)?;
        let mut neg_jt_r = Vector::with_capacity(n);
        neg_jt_r.resize(n);
        for i in 0..n {
            neg_jt_r.set(i, -jt_r.get(i));
        }
        let delta = chol_fac.solve(&neg_jt_r);
        let d: Vec<f64> = (0..n).map(|i| delta.get(i)).collect();
        let g_dot_d = -grad_norm_sq;

        let merit = |x_slice: &[f64]| {
            let r_x = residual(x_slice);
            0.5 * r_x.iter().map(|v| v * v).sum::<f64>()
        };
        let alpha = linesearch::backtracking(
            &x,
            &d,
            cost_val,
            g_dot_d,
            merit,
            &options.line_search_options,
            &mut x_new,
        );

        debug!(iter, cost = %cost_val, grad_norm = %(grad_norm_sq.sqrt()), alpha, "gauss_newton");

        x.copy_from_slice(&x_new);
        iter += 1;
    }

    let cost_val = 0.5 * residual(&x).iter().map(|v| v * v).sum::<f64>();

    Ok(GaussNewtonResult {
        x,
        residual_norm_sq: 2.0 * cost_val,
        iterations: iter,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauss_newton_linear_fit() {
        // Fit y = a*x + b to points (0,0), (1,1), (2,2) => a=1, b=0.
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
        let opts = GaussNewtonOptions::default();
        let result = gauss_newton(&x0, residual, jacobian, &opts).unwrap();
        assert!((result.x[0] - 1.0).abs() < 1e-5);
        assert!(result.x[1].abs() < 1e-5);
    }
}
