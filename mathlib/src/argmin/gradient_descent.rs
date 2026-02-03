//! Gradient descent with line search (backtracking, Armijo, Wolfe).
//!
//! Set `RUST_LOG=mathlib=debug` to see per-iteration cost, gradient norm, and step size.

use super::linesearch::{self, LineSearchOptions};
use tracing::debug;

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

/// Line search variant for gradient descent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineSearchVariant {
    /// Backtracking until Armijo holds.
    Backtracking,
    /// Armijo condition (same as backtracking).
    Armijo,
    /// Strong Wolfe (Armijo + curvature).
    Wolfe,
}

/// Options for gradient descent.
#[derive(Clone, Debug)]
pub struct GradientDescentOptions {
    /// Maximum iterations (default 1000).
    pub max_iters: usize,
    /// Stop when gradient norm below this (default 1e-8).
    pub tol: f64,
    /// Line search variant (default Backtracking).
    pub line_search: LineSearchVariant,
    /// Line search parameters.
    pub line_search_options: LineSearchOptions,
}

impl Default for GradientDescentOptions {
    fn default() -> Self {
        Self {
            max_iters: 1000,
            tol: 1e-8,
            line_search: LineSearchVariant::Backtracking,
            line_search_options: LineSearchOptions::default(),
        }
    }
}

/// Result of gradient descent.
#[derive(Clone, Debug)]
pub struct GradientDescentResult {
    /// Best point found.
    pub x: Vec<f64>,
    /// Cost at best point.
    pub cost: f64,
    /// Number of iterations performed.
    pub iterations: usize,
}

/// Gradient descent: minimize `cost` with gradient `gradient`, starting at `x0`.
/// Direction d = -g; step size from selected line search.
#[must_use]
pub fn gradient_descent<F, G>(
    x0: &[f64],
    cost: F,
    gradient: G,
    options: &GradientDescentOptions,
) -> GradientDescentResult
where
    F: Fn(&[f64]) -> f64,
    G: Fn(&[f64], &mut [f64]),
{
    let n = x0.len();
    let mut x = x0.to_vec();
    let mut g = vec![0.0_f64; n];
    gradient(&x, &mut g);
    let mut cost_val = cost(&x);
    let mut grad_norm_sq = crate::cpu::dot_f64(&g, &g);
    let tol_sq = options.tol * options.tol;

    let mut d = vec![0.0_f64; n];
    let mut x_new = vec![0.0_f64; n];
    let mut g_trial = vec![0.0_f64; n];

    let mut iter = 0_usize;
    while iter < options.max_iters {
        if grad_norm_sq <= tol_sq {
            debug!(iter, cost = %cost_val, grad_norm_sq = %grad_norm_sq, "gradient_descent converged");
            break;
        }
        // d = -g
        scalar_mul_f64(-1.0, &g, &mut d);
        let g_dot_d = -grad_norm_sq;

        let alpha = match options.line_search {
            LineSearchVariant::Backtracking | LineSearchVariant::Armijo => {
                linesearch::backtracking(
                    &x,
                    &d,
                    cost_val,
                    g_dot_d,
                    &cost,
                    &options.line_search_options,
                    &mut x_new,
                )
            }
            LineSearchVariant::Wolfe => linesearch::wolfe(
                &x,
                &d,
                cost_val,
                g_dot_d,
                &cost,
                &gradient,
                &options.line_search_options,
                &mut x_new,
                &mut g_trial,
            ),
        };

        debug!(iter, cost = %cost_val, grad_norm = %(grad_norm_sq.sqrt()), alpha, "gradient_descent");

        x.copy_from_slice(&x_new);
        cost_val = cost(&x);
        gradient(&x, &mut g);
        grad_norm_sq = crate::cpu::dot_f64(&g, &g);
        iter += 1;
    }

    GradientDescentResult {
        x,
        cost: cost_val,
        iterations: iter,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_descent_sphere() {
        let x0 = vec![3.0_f64, 4.0];
        let cost = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
        let gradient = |x: &[f64], g: &mut [f64]| {
            g[0] = 2.0 * x[0];
            g[1] = 2.0 * x[1];
        };
        let opts = GradientDescentOptions {
            max_iters: 500,
            tol: 1e-10,
            ..Default::default()
        };
        let result = gradient_descent(&x0, cost, gradient, &opts);
        assert!(result.cost < 1e-6);
        assert!(result.x[0].abs() < 1e-3 && result.x[1].abs() < 1e-3);
    }

    #[test]
    fn gradient_descent_quadratic() {
        // min (x0-2)^2 + (x1+1)^2
        let x0 = vec![0.0_f64, 0.0];
        let cost = |x: &[f64]| (x[0] - 2.0).powi(2) + (x[1] + 1.0).powi(2);
        let gradient = |x: &[f64], g: &mut [f64]| {
            g[0] = 2.0 * (x[0] - 2.0);
            g[1] = 2.0 * (x[1] + 1.0);
        };
        let opts = GradientDescentOptions {
            max_iters: 1000,
            tol: 1e-9,
            ..Default::default()
        };
        let result = gradient_descent(&x0, cost, gradient, &opts);
        assert!((result.x[0] - 2.0).abs() < 1e-4);
        assert!((result.x[1] + 1.0).abs() < 1e-4);
    }
}
