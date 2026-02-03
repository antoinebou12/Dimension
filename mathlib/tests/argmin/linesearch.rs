//! Integration tests for line search.

use mathlib::{LineSearchOptions, armijo, backtracking, wolfe};

#[test]
fn linesearch_backtracking_integration() {
    let x = [1.0_f64, 0.0];
    let d = [-1.0_f64, 0.0];
    let f = 1.0;
    let g_dot_d = -2.0;
    let cost = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
    let opts = LineSearchOptions::default();
    let mut scratch = [0.0_f64; 2];
    let alpha = backtracking(&x, &d, f, g_dot_d, cost, &opts, &mut scratch);
    assert!(alpha > 0.0 && alpha <= 1.0);
    assert!(cost(&scratch) <= 1.0);
}

#[test]
fn linesearch_armijo_integration() {
    let x = [2.0_f64, 2.0];
    let d = [-1.0_f64, -1.0];
    let f = 8.0;
    let g_dot_d = -8.0;
    let cost = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
    let opts = LineSearchOptions::default();
    let mut scratch = [0.0_f64; 2];
    let alpha = armijo(&x, &d, f, g_dot_d, cost, &opts, &mut scratch);
    assert!(alpha > 0.0);
}

#[test]
fn linesearch_wolfe_integration() {
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
}
