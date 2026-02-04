//! Integration tests for gradient descent.

use mathlib::{GradientDescentOptions, gradient_descent};

#[test]
fn gradient_descent_sphere_integration() {
    let x0 = vec![5.0_f64, 5.0];
    let cost = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
    let gradient = |x: &[f64], g: &mut [f64]| {
        g[0] = 2.0 * x[0];
        g[1] = 2.0 * x[1];
    };
    let opts = GradientDescentOptions {
        max_iters: 500,
        tol: 1e-9,
        ..Default::default()
    };
    let result = gradient_descent(&x0, cost, gradient, &opts);
    assert!(result.cost < 1e-5);
    assert!(result.x[0].abs() < 1e-2 && result.x[1].abs() < 1e-2);
}
