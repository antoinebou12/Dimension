//! L-BFGS-B: limited-memory BFGS with box constraints.
//!
//! Minimizes a smooth function f(x) subject to lower ≤ x ≤ upper using the L-BFGS
//! quasi-Newton approximation and projected search directions. See Byrd et al.,
//! "A Limited Memory Algorithm for Bound Constrained Optimization" (SIAM J. Sci. Comput., 1995).
//!
//! Set `RUST_LOG=mathlib=debug` to see per-iteration cost and gradient norm.

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

/// Options for L-BFGS-B.
#[derive(Clone, Debug)]
pub struct LbfgsbOptions {
    /// Maximum iterations (default 1000).
    pub max_iters: usize,
    /// Stop when gradient norm (in free variables) below this (default 1e-8).
    pub tol: f64,
    /// L-BFGS history size, number of (s, y) pairs (default 10).
    pub m: usize,
    /// Line search parameters (Armijo backtracking).
    pub line_search_options: LineSearchOptions,
}

impl Default for LbfgsbOptions {
    fn default() -> Self {
        Self {
            max_iters: 1000,
            tol: 1e-8,
            m: 10,
            line_search_options: LineSearchOptions::default(),
        }
    }
}

/// Result of L-BFGS-B.
#[derive(Clone, Debug)]
pub struct LbfgsbResult {
    /// Best point found (within bounds).
    pub x: Vec<f64>,
    /// Cost at best point.
    pub cost: f64,
    /// Number of iterations performed.
    pub iterations: usize,
}

/// Project search direction so that moving from x along d does not immediately violate bounds.
/// Zeros d[i] when x[i] is at lower bound and d[i] < 0, or at upper bound and d[i] > 0.
fn project_direction(x: &[f64], d: &mut [f64], lower: &[f64], upper: &[f64]) {
    for i in 0..x.len() {
        if (d[i] < 0.0 && x[i] <= lower[i]) || (d[i] > 0.0 && x[i] >= upper[i]) {
            d[i] = 0.0;
        }
    }
}

/// Clamp x to [lower, upper] in place.
fn clamp_to_bounds(x: &mut [f64], lower: &[f64], upper: &[f64]) {
    for i in 0..x.len() {
        x[i] = x[i].clamp(lower[i], upper[i]);
    }
}

/// L-BFGS two-loop: compute d = -H*g where H approximates the inverse Hessian from history.
/// History: oldest first, `(s, y)` pairs. Scale factor `gamma` from the most recent pair.
fn two_loop(
    g: &[f64],
    history: &[(Vec<f64>, Vec<f64>)],
    gamma: f64,
    d: &mut [f64],
    scratch_q: &mut [f64],
    scratch_alpha: &mut [f64],
) {
    let n = g.len();
    let m = history.len();
    scratch_q.copy_from_slice(g);

    // First loop: i from newest to oldest (m-1 down to 0)
    for i in (0..m).rev() {
        let (s, y) = &history[i];
        let rho = 1.0 / crate::cpu::dot_f64(s, y);
        let dot_sq = crate::cpu::dot_f64(s, scratch_q);
        scratch_alpha[i] = rho * dot_sq;
        for j in 0..n {
            scratch_q[j] -= scratch_alpha[i] * y[j];
        }
    }

    // r = gamma * q
    scalar_mul_f64(gamma, scratch_q, d);

    // Second loop: i from oldest to newest (0 to m-1)
    for i in 0..m {
        let (s, y) = &history[i];
        let rho = 1.0 / crate::cpu::dot_f64(s, y);
        let dot_yr = crate::cpu::dot_f64(y, d);
        let beta = rho * dot_yr;
        for j in 0..n {
            d[j] += s[j] * (scratch_alpha[i] - beta);
        }
    }

    // d = -d (we computed H*g, we want -H*g); use scratch to avoid borrow conflict
    for v in d.iter_mut() {
        *v = -(*v);
    }
}

/// L-BFGS-B: minimize `cost` with gradient `gradient` over box [lower, upper], starting at `x0`.
/// Use large positive (e.g. 1e30) for upper and large negative for lower to effectively leave a dimension unconstrained.
#[must_use]
pub fn lbfgsb<F, G>(
    x0: &[f64],
    lower: &[f64],
    upper: &[f64],
    cost: F,
    gradient: G,
    options: &LbfgsbOptions,
) -> LbfgsbResult
where
    F: Fn(&[f64]) -> f64,
    G: Fn(&[f64], &mut [f64]),
{
    let n = x0.len();
    assert_eq!(lower.len(), n);
    assert_eq!(upper.len(), n);

    let mut x = x0.to_vec();
    clamp_to_bounds(&mut x, lower, upper);

    let mut g = vec![0.0_f64; n];
    gradient(&x, &mut g);
    let mut cost_val = cost(&x);

    let tol_sq = options.tol * options.tol;
    let m = options.m.min(n).max(1);

    // History: list of (s, y), oldest first. Max length m.
    let mut history: Vec<(Vec<f64>, Vec<f64>)> = Vec::with_capacity(m);

    let mut d = vec![0.0_f64; n];
    let mut x_new = vec![0.0_f64; n];
    let mut g_new = vec![0.0_f64; n];
    let mut scratch_q = vec![0.0_f64; n];
    let mut scratch_alpha = vec![0.0_f64; m];

    let mut iter = 0_usize;

    while iter < options.max_iters {
        // Gradient norm (full; for convergence check)
        let grad_norm_sq = crate::cpu::dot_f64(&g, &g);
        if grad_norm_sq <= tol_sq {
            debug!(iter, cost = %cost_val, "lbfgsb converged");
            break;
        }

        // Search direction: -H*g via two-loop, or -g if no history
        if history.is_empty() {
            scalar_mul_f64(-1.0, &g, &mut d);
        } else {
            let (s_last, y_last) = history.last().unwrap();
            let sy = crate::cpu::dot_f64(s_last, y_last);
            let yy = crate::cpu::dot_f64(y_last, y_last);
            let gamma = if yy > 1e-14 { sy / yy } else { 1.0 };
            two_loop(
                &g,
                &history,
                gamma,
                &mut d,
                &mut scratch_q,
                &mut scratch_alpha,
            );
        }

        project_direction(&x, &mut d, lower, upper);

        let d_norm_sq = crate::cpu::dot_f64(&d, &d);
        if d_norm_sq <= 1e-20 {
            debug!(iter, "lbfgsb: zero direction (likely at bound)");
            break;
        }

        let g_dot_d = crate::cpu::dot_f64(&g, &d);
        if g_dot_d >= 0.0 {
            // Not a descent direction; fall back to steepest descent
            scalar_mul_f64(-1.0, &g, &mut d);
            project_direction(&x, &mut d, lower, upper);
        }

        let g_dot_d = crate::cpu::dot_f64(&g, &d);
        let alpha = linesearch::backtracking(
            &x,
            &d,
            cost_val,
            g_dot_d,
            &cost,
            &options.line_search_options,
            &mut x_new,
        );

        clamp_to_bounds(&mut x_new, lower, upper);
        let cost_new = cost(&x_new);
        gradient(&x_new, &mut g_new);

        // Update history: s = x_new - x, y = g_new - g
        let mut s = vec![0.0_f64; n];
        let mut y = vec![0.0_f64; n];
        for i in 0..n {
            s[i] = x_new[i] - x[i];
            y[i] = g_new[i] - g[i];
        }
        let sy = crate::cpu::dot_f64(&s, &y);
        if sy > 1e-14 {
            if history.len() >= m {
                history.remove(0);
            }
            history.push((s, y));
        }

        x.copy_from_slice(&x_new);
        g.copy_from_slice(&g_new);
        cost_val = cost_new;
        iter += 1;

        debug!(
            iter,
            cost = %cost_val,
            grad_norm = %(grad_norm_sq.sqrt()),
            alpha,
            "lbfgsb"
        );
    }

    LbfgsbResult {
        x,
        cost: cost_val,
        iterations: iter,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere_cost(x: &[f64]) -> f64 {
        x.iter().map(|v| v * v).sum()
    }

    fn sphere_gradient(x: &[f64], g: &mut [f64]) {
        for (i, &v) in x.iter().enumerate() {
            g[i] = 2.0 * v;
        }
    }

    #[test]
    fn lbfgsb_sphere_unconstrained() {
        let x0 = vec![5.0_f64, 5.0];
        let lower = vec![-1e30_f64, -1e30];
        let upper = vec![1e30_f64, 1e30];
        let opts = LbfgsbOptions {
            max_iters: 500,
            tol: 1e-10,
            m: 5,
            ..Default::default()
        };
        let result = lbfgsb(&x0, &lower, &upper, sphere_cost, sphere_gradient, &opts);
        assert!(result.cost < 1e-6);
        assert!(result.x[0].abs() < 1e-3 && result.x[1].abs() < 1e-3);
    }

    #[test]
    fn lbfgsb_sphere_bounded() {
        let x0 = vec![0.5_f64, 0.5];
        let lower = vec![-1.0_f64, -1.0];
        let upper = vec![1.0_f64, 1.0];
        let opts = LbfgsbOptions {
            max_iters: 200,
            tol: 1e-9,
            m: 5,
            ..Default::default()
        };
        let result = lbfgsb(&x0, &lower, &upper, sphere_cost, sphere_gradient, &opts);
        assert!(result.cost < 1e-10);
        assert!(result.x[0].abs() < 1e-5 && result.x[1].abs() < 1e-5);
        assert!(result.x[0] >= lower[0] && result.x[0] <= upper[0]);
        assert!(result.x[1] >= lower[1] && result.x[1] <= upper[1]);
    }

    #[test]
    fn lbfgsb_rosenbrock() {
        let rosenbrock = |x: &[f64]| (1.0 - x[0]).powi(2) + 100.0 * (x[1] - x[0] * x[0]).powi(2);
        let rosenbrock_grad = |x: &[f64], g: &mut [f64]| {
            g[0] = -2.0 * (1.0 - x[0]) - 400.0 * x[0] * (x[1] - x[0] * x[0]);
            g[1] = 200.0 * (x[1] - x[0] * x[0]);
        };
        let x0 = vec![0.0_f64, 0.0];
        let lower = vec![-5.0_f64, -5.0];
        let upper = vec![5.0_f64, 5.0];
        let opts = LbfgsbOptions {
            max_iters: 2000,
            tol: 1e-6,
            m: 10,
            ..Default::default()
        };
        let result = lbfgsb(&x0, &lower, &upper, rosenbrock, rosenbrock_grad, &opts);
        assert!(result.cost < 0.1);
        assert!(result.iterations > 0);
    }
}
