//! Line search: backtracking, Armijo, and Wolfe conditions.
//!
//! All inner products and vector ops use `crate::cpu::*` so SIMD/parallel
//! apply when features are enabled. Set `RUST_LOG=mathlib=debug` to see
//! alpha and backtrack counts.

use tracing::debug;

/// Options for line search (Armijo/Wolfe/backtracking).
#[derive(Clone, Debug)]
pub struct LineSearchOptions {
    /// Armijo: f(x+αd) ≤ f + c1*α*g·d (default 1e-4).
    pub c1: f64,
    /// Wolfe curvature: g(x+αd)·d ≥ c2*g·d (default 0.9 strong Wolfe).
    pub c2: f64,
    /// Initial step length (default 1.0).
    pub alpha_init: f64,
    /// Backtracking reduction factor: α *= beta (default 0.5).
    pub beta: f64,
    /// Maximum number of backtracks (default 40).
    pub max_backtrack: usize,
}

impl Default for LineSearchOptions {
    fn default() -> Self {
        Self {
            c1: 1e-4,
            c2: 0.9,
            alpha_init: 1.0,
            beta: 0.5,
            max_backtrack: 40,
        }
    }
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

/// Backtracking line search: reduce α until Armijo holds.
/// Returns step length α. Uses caller-provided scratch `x_plus_alpha_d` (length n) for the trial point.
#[must_use]
pub fn backtracking<F>(
    x: &[f64],
    d: &[f64],
    f: f64,
    g_dot_d: f64,
    cost: F,
    options: &LineSearchOptions,
    x_plus_alpha_d: &mut [f64],
) -> f64
where
    F: Fn(&[f64]) -> f64,
{
    let n = x.len();
    assert_eq!(d.len(), n);
    assert_eq!(x_plus_alpha_d.len(), n);
    let mut alpha = options.alpha_init;
    let mut backtracks = 0_usize;
    let mut step = vec![0.0_f64; n];

    loop {
        let armijo_rhs = f + options.c1 * alpha * g_dot_d;
        scalar_mul_f64(alpha, d, &mut step);
        add_f64(x, &step, x_plus_alpha_d);
        let f_new = cost(x_plus_alpha_d);
        if f_new <= armijo_rhs || backtracks >= options.max_backtrack {
            debug!(alpha, backtracks, "backtracking");
            return alpha;
        }
        alpha *= options.beta;
        backtracks += 1;
    }
}

/// Armijo line search: find α such that f(x+αd) ≤ f + c1*α*g·d.
/// Same scratch buffer usage as backtracking.
#[must_use]
pub fn armijo<F>(
    x: &[f64],
    d: &[f64],
    f: f64,
    g_dot_d: f64,
    cost: F,
    options: &LineSearchOptions,
    x_plus_alpha_d: &mut [f64],
) -> f64
where
    F: Fn(&[f64]) -> f64,
{
    backtracking(x, d, f, g_dot_d, cost, options, x_plus_alpha_d)
}

/// Wolfe (strong) line search: Armijo + curvature g(x+αd)·d ≥ c2*g·d.
/// Requires gradient at trial point; `grad` is called to fill gradient at `x_plus_alpha_d`.
/// Scratch: `x_plus_alpha_d` (length n), `g_trial` (length n) for gradient at trial.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn wolfe<F, G>(
    x: &[f64],
    d: &[f64],
    f: f64,
    g_dot_d: f64,
    cost: F,
    grad: G,
    options: &LineSearchOptions,
    x_plus_alpha_d: &mut [f64],
    g_trial: &mut [f64],
) -> f64
where
    F: Fn(&[f64]) -> f64,
    G: Fn(&[f64], &mut [f64]),
{
    let n = x.len();
    assert_eq!(d.len(), n);
    assert_eq!(x_plus_alpha_d.len(), n);
    assert_eq!(g_trial.len(), n);
    let mut alpha = options.alpha_init;
    let mut backtracks = 0_usize;
    let mut step = vec![0.0_f64; n];

    loop {
        scalar_mul_f64(alpha, d, &mut step);
        add_f64(x, &step, x_plus_alpha_d);
        let f_new = cost(x_plus_alpha_d);
        let armijo_rhs = f + options.c1 * alpha * g_dot_d;
        if f_new > armijo_rhs {
            alpha *= options.beta;
            backtracks += 1;
            if backtracks >= options.max_backtrack {
                debug!(alpha, backtracks, "wolfe (armijo fail)");
                return alpha;
            }
            continue;
        }
        grad(x_plus_alpha_d, g_trial);
        let g_trial_dot_d = crate::cpu::dot_f64(g_trial, d);
        if g_trial_dot_d >= options.c2 * g_dot_d {
            debug!(alpha, backtracks, "wolfe");
            return alpha;
        }
        alpha *= options.beta;
        backtracks += 1;
        if backtracks >= options.max_backtrack {
            debug!(alpha, backtracks, "wolfe (max backtracks)");
            return alpha;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linesearch_backtracking_quadratic_1d() {
        // min (x-2)^2, x0=0, d=1 => g_dot_d = -4, optimal alpha = 2.
        let x = [0.0_f64];
        let d = [1.0_f64];
        let f = 4.0; // (0-2)^2
        let g_dot_d = -4.0; // 2*(0-2)*1
        let cost = |x: &[f64]| {
            let t = x[0] - 2.0;
            t * t
        };
        let opts = LineSearchOptions::default();
        let mut scratch = [0.0_f64];
        let alpha = backtracking(&x, &d, f, g_dot_d, cost, &opts, &mut scratch);
        assert!(alpha > 0.0 && alpha <= 1.0);
        assert!(scratch[0] >= 0.0 && scratch[0] <= 2.5);
        let f_new = cost(&scratch);
        assert!(f_new <= 4.0);
    }

    #[test]
    fn linesearch_armijo_sphere() {
        let x = [1.0_f64, 1.0];
        let d = [-1.0_f64, -1.0]; // toward origin
        let f = 2.0; // 1^2+1^2
        let g_dot_d = -2.0; // 2*1*(-1)+2*1*(-1)
        let cost = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
        let opts = LineSearchOptions::default();
        let mut scratch = [0.0_f64; 2];
        let alpha = armijo(&x, &d, f, g_dot_d, cost, &opts, &mut scratch);
        assert!(alpha > 0.0);
        let f_new = cost(&scratch);
        assert!(f_new <= f + opts.c1 * alpha * g_dot_d);
    }

    #[test]
    fn linesearch_wolfe_quadratic() {
        let x = [1.0_f64, 0.0];
        let d = [-1.0_f64, 0.0];
        let f = 1.0;
        let g_dot_d = -2.0;
        let cost = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
        let grad = |x: &[f64], g: &mut [f64]| {
            g[0] = 2.0 * x[0];
            g[1] = 2.0 * x[1];
        };
        let opts = LineSearchOptions::default();
        let mut x_scratch = [0.0_f64; 2];
        let mut g_scratch = [0.0_f64; 2];
        let alpha = wolfe(
            &x,
            &d,
            f,
            g_dot_d,
            cost,
            grad,
            &opts,
            &mut x_scratch,
            &mut g_scratch,
        );
        assert!(alpha > 0.0);
        let f_new = cost(&x_scratch);
        assert!(f_new <= f + opts.c1 * alpha * g_dot_d);
        grad(&x_scratch, &mut g_scratch);
        let gtd = crate::cpu::dot_f64(&g_scratch, &d);
        assert!(gtd >= opts.c2 * g_dot_d - 1e-10);
    }
}
