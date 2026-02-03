//! Conjugate gradient: linear CG for Ax = b (A SPD) and nonlinear CG for general f.
//!
//! Set `RUST_LOG=mathlib=debug` to see iteration logs.

use crate::matrix::Matrix;
use crate::vector::Vector;
use std::fmt;
use tracing::debug;

use super::linesearch::{self, LineSearchOptions};

/// Error from conjugate gradient.
#[derive(Clone, Debug, PartialEq)]
pub enum CgError {
    /// Matrix is not square.
    NotSquare,
    /// Dimension mismatch between matrix and vector.
    DimensionMismatch,
    /// Maximum iterations exceeded without converging.
    MaxItersExceeded,
}

impl std::error::Error for CgError {}

impl fmt::Display for CgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CgError::NotSquare => write!(f, "matrix is not square"),
            CgError::DimensionMismatch => write!(f, "dimension mismatch"),
            CgError::MaxItersExceeded => write!(f, "maximum iterations exceeded"),
        }
    }
}

/// Linear conjugate gradient for Ax = b with A symmetric positive definite.
/// Returns solution x or error if not converged.
///
/// # Errors
///
/// Returns `CgError::NotSquare` if `a` is not square, `CgError::DimensionMismatch`
/// if dimensions do not match, `CgError::MaxItersExceeded` if not converged within `max_iters`.
#[must_use = "this `Result` may be an `Err` that should be handled"]
pub fn solve_cg(
    a: &Matrix<f64>,
    b: &Vector<f64>,
    tol: f64,
    max_iters: usize,
) -> Result<Vector<f64>, CgError> {
    let n = a.rows();
    if a.cols() != n {
        return Err(CgError::NotSquare);
    }
    if b.rows() != n {
        return Err(CgError::DimensionMismatch);
    }
    let tol_sq = tol * tol;

    let mut x = Vector::with_capacity(n);
    x.resize(n);
    x.set_zero();

    let ax = a * &x;
    let mut r = b - &ax;
    let mut p = r.clone();
    let mut r_dot_r = crate::cpu::dot_f64(r.data(), r.data());

    let mut iter = 0_usize;
    while iter < max_iters {
        if r_dot_r <= tol_sq {
            debug!(iter, "solve_cg converged");
            return Ok(x);
        }
        let ap = a * &p;
        let p_ap = crate::cpu::dot_f64(p.data(), ap.data());
        if p_ap <= 0.0 {
            return Err(CgError::MaxItersExceeded);
        }
        let alpha = r_dot_r / p_ap;
        let alpha_p = alpha * &p;
        x = &x + &alpha_p;
        let step_ap = alpha * &ap;
        r = &r - &step_ap;
        let r_new_dot_r_new = crate::cpu::dot_f64(r.data(), r.data());
        let beta = r_new_dot_r_new / r_dot_r;
        r_dot_r = r_new_dot_r_new;
        p = &r + &(beta * &p);
        debug!(iter, r_dot_r = %r_dot_r, "solve_cg");
        iter += 1;
    }
    Err(CgError::MaxItersExceeded)
}

/// Nonlinear conjugate gradient (Fletcher–Reeves) with line search.
/// Minimizes f(x) with gradient; resets to steepest descent every `n` steps.
#[derive(Clone, Debug)]
pub struct NonlinearCgOptions {
    /// Maximum iterations (default 1000).
    pub max_iters: usize,
    /// Stop when gradient norm below this (default 1e-8).
    pub tol: f64,
    /// Reset to steepest descent every this many steps (default: dim).
    pub reset_interval: Option<usize>,
    /// Line search options.
    pub line_search_options: LineSearchOptions,
}

impl Default for NonlinearCgOptions {
    fn default() -> Self {
        Self {
            max_iters: 1000,
            tol: 1e-8,
            reset_interval: None,
            line_search_options: LineSearchOptions::default(),
        }
    }
}

/// Result of nonlinear conjugate gradient.
#[derive(Clone, Debug)]
pub struct NonlinearCgResult {
    /// Best point found.
    pub x: Vec<f64>,
    /// Cost at best point.
    pub cost: f64,
    /// Number of iterations.
    pub iterations: usize,
}

#[inline]
fn add_f64(a: &[f64], b: &[f64], out: &mut [f64]) {
    #[cfg(feature = "simd")]
    return crate::cpu::simd::add_f64(a, b, out);
    #[cfg(all(feature = "parallel", not(feature = "simd")))]
    return crate::cpu::parallel::par_add_f64(a, b, out);
    #[cfg(not(any(feature = "simd", feature = "parallel")))]
    crate::cpu::sequential::add_f64(a, b, out);
}

#[inline]
fn scalar_mul_f64(s: f64, x: &[f64], out: &mut [f64]) {
    #[cfg(feature = "simd")]
    return crate::cpu::simd::scalar_mul_f64(s, x, out);
    #[cfg(all(
        feature = "parallel",
        not(target_arch = "wasm32"),
        not(feature = "simd")
    ))]
    return crate::cpu::parallel::par_scalar_mul_f64(s, x, out);
    #[cfg(not(any(
        feature = "simd",
        all(feature = "parallel", not(target_arch = "wasm32"))
    )))]
    crate::cpu::sequential::scalar_mul_f64(s, x, out);
}

/// Nonlinear conjugate gradient (Fletcher–Reeves) to minimize f(x).
#[must_use]
pub fn nonlinear_cg<F, G>(
    x0: &[f64],
    cost: F,
    gradient: G,
    options: &NonlinearCgOptions,
) -> NonlinearCgResult
where
    F: Fn(&[f64]) -> f64,
    G: Fn(&[f64], &mut [f64]),
{
    let n = x0.len();
    let reset_interval = options.reset_interval.unwrap_or(n);

    let mut x = x0.to_vec();
    let mut g = vec![0.0_f64; n];
    gradient(&x, &mut g);
    let mut cost_val = cost(&x);
    let mut grad_norm_sq = crate::cpu::dot_f64(&g, &g);
    let tol_sq = options.tol * options.tol;

    let mut d = vec![0.0_f64; n];
    scalar_mul_f64(-1.0, &g, &mut d);
    let mut x_new = vec![0.0_f64; n];
    let mut g_trial = vec![0.0_f64; n];
    let mut d_new = vec![0.0_f64; n];

    let mut iter = 0_usize;
    while iter < options.max_iters {
        if grad_norm_sq <= tol_sq {
            debug!(iter, cost = %cost_val, "nonlinear_cg converged");
            break;
        }
        let g_dot_d = -grad_norm_sq;
        let alpha = linesearch::backtracking(
            &x,
            &d,
            cost_val,
            g_dot_d,
            &cost,
            &options.line_search_options,
            &mut x_new,
        );
        debug!(iter, cost = %cost_val, grad_norm = %(grad_norm_sq.sqrt()), alpha, "nonlinear_cg");

        x.copy_from_slice(&x_new);
        cost_val = cost(&x);
        gradient(&x, &mut g);
        let grad_norm_sq_new = crate::cpu::dot_f64(&g, &g);

        let beta = if (iter + 1).is_multiple_of(reset_interval) {
            0.0
        } else {
            grad_norm_sq_new / grad_norm_sq
        };
        scalar_mul_f64(beta, &d, &mut d_new);
        scalar_mul_f64(-1.0, &g, &mut g_trial);
        add_f64(&d_new, &g_trial, &mut d);
        grad_norm_sq = grad_norm_sq_new;
        iter += 1;
    }

    NonlinearCgResult {
        x,
        cost: cost_val,
        iterations: iter,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::Storage;

    fn make_vector(data: &[f64]) -> Vector<f64> {
        let mut v = Vector::with_capacity(data.len());
        v.resize(data.len());
        for (i, &val) in data.iter().enumerate() {
            v.set(i, val);
        }
        v
    }

    fn make_spd_matrix(n: usize, diag: f64, off: f64) -> Matrix<f64> {
        let mut a = Matrix::with_storage(n, n, Storage::Column);
        for i in 0..n {
            for j in 0..n {
                let v = if i == j { diag } else { off };
                a.set(i, j, v);
            }
        }
        a
    }

    #[test]
    fn solve_cg_diagonal() {
        let a = make_spd_matrix(3, 4.0, 0.0);
        let b = make_vector(&[4.0, 8.0, 12.0]);
        let x = solve_cg(&a, &b, 1e-12, 10).unwrap();
        assert!((x.get(0) - 1.0).abs() < 1e-10);
        assert!((x.get(1) - 2.0).abs() < 1e-10);
        assert!((x.get(2) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn solve_cg_max_iters() {
        let a = make_spd_matrix(2, 1.0, 0.0);
        let b = make_vector(&[1.0, 1.0]);
        let result = solve_cg(&a, &b, 1e-20, 1);
        assert!(result.is_err());
    }

    #[test]
    fn nonlinear_cg_sphere() {
        let x0 = vec![3.0_f64, 4.0];
        let cost = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
        let gradient = |x: &[f64], g: &mut [f64]| {
            g[0] = 2.0 * x[0];
            g[1] = 2.0 * x[1];
        };
        let opts = NonlinearCgOptions {
            max_iters: 500,
            tol: 1e-10,
            ..Default::default()
        };
        let result = nonlinear_cg(&x0, cost, gradient, &opts);
        assert!(result.cost < 1e-6);
    }
}
