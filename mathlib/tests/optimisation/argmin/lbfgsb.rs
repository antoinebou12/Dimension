//! Integration tests for L-BFGS-B.

use mathlib::{LbfgsbOptions, lbfgsb};

#[test]
fn lbfgsb_sphere_unconstrained() {
    let x0 = vec![5.0_f64, 5.0];
    let lower = vec![-1e30_f64, -1e30];
    let upper = vec![1e30_f64, 1e30];
    let cost = |x: &[f64]| x.iter().map(|v| v * v).sum::<f64>();
    let gradient = |x: &[f64], g: &mut [f64]| {
        for (i, &v) in x.iter().enumerate() {
            g[i] = 2.0 * v;
        }
    };
    let opts = LbfgsbOptions {
        max_iters: 500,
        tol: 1e-10,
        m: 5,
        ..Default::default()
    };
    let result = lbfgsb(&x0, &lower, &upper, cost, gradient, &opts);
    assert!(result.cost < 1e-6);
    assert!(result.x[0].abs() < 1e-3 && result.x[1].abs() < 1e-3);
}

#[test]
fn lbfgsb_sphere_bounded() {
    let x0 = vec![0.5_f64, 0.5];
    let lower = vec![-1.0_f64, -1.0];
    let upper = vec![1.0_f64, 1.0];
    let cost = |x: &[f64]| x.iter().map(|v| v * v).sum::<f64>();
    let gradient = |x: &[f64], g: &mut [f64]| {
        for (i, &v) in x.iter().enumerate() {
            g[i] = 2.0 * v;
        }
    };
    let opts = LbfgsbOptions {
        max_iters: 200,
        tol: 1e-9,
        m: 5,
        ..Default::default()
    };
    let result = lbfgsb(&x0, &lower, &upper, cost, gradient, &opts);
    assert!(result.cost < 1e-10);
    assert!(result.x[0].abs() < 1e-5 && result.x[1].abs() < 1e-5);
    assert!(result.x[0] >= lower[0] && result.x[0] <= upper[0]);
    assert!(result.x[1] >= lower[1] && result.x[1] <= upper[1]);
}

#[test]
fn lbfgsb_rosenbrock_reduces_cost() {
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
