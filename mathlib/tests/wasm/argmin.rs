//! Integration tests for wasm argmin bindings (line search, PSO behavior).
//! Line search wasm API requires a JS cost callback; we test the native backtracking here.

#![cfg(feature = "wasm")]

use mathlib::argmin::linesearch::{self, LineSearchOptions};

#[test]
fn wasm_demo_linesearch_backtracking_quadratic() {
    let x = vec![1.0, 1.0];
    let d = vec![-1.0, -1.0];
    let f = 2.0;
    let g_dot_d = -2.0;
    let cost = |pt: &[f64]| pt[0] * pt[0] + pt[1] * pt[1];
    let mut x_plus_alpha_d = x.clone();
    let alpha = linesearch::backtracking(
        &x,
        &d,
        f,
        g_dot_d,
        &cost,
        &LineSearchOptions::default(),
        &mut x_plus_alpha_d,
    );
    assert!(alpha > 0.0 && alpha <= 1.0);
    let new_cost = cost(&x_plus_alpha_d);
    assert!(new_cost <= f, "Armijo: cost should decrease or stay");
}
