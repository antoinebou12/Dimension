//! Integration tests for conjugate gradient.

use mathlib::structure::Storage;
use mathlib::{Matrix, Vector};
use mathlib::{NonlinearCgOptions, nonlinear_cg, solve_cg};

fn make_vector(data: &[f64]) -> Vector<f64> {
    let mut v = Vector::with_capacity(data.len());
    v.resize(data.len());
    for (i, &val) in data.iter().enumerate() {
        v.set(i, val);
    }
    v
}

fn make_spd_2x2() -> Matrix<f64> {
    let mut a = Matrix::with_storage(2, 2, Storage::Column);
    a.set(0, 0, 4.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 1.0);
    a.set(1, 1, 3.0);
    a
}

#[test]
fn solve_cg_integration() {
    let a = make_spd_2x2();
    let b = make_vector(&[1.0, 2.0]);
    let x = solve_cg(&a, &b, 1e-12, 10).unwrap();
    let ax = &a * &x;
    let r = &b - &ax;
    let r_norm_sq = r.data().iter().map(|v| v * v).sum::<f64>();
    assert!(r_norm_sq < 1e-20);
}

#[test]
fn nonlinear_cg_integration() {
    let x0 = vec![2.0_f64, 2.0];
    let cost = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
    let gradient = |x: &[f64], g: &mut [f64]| {
        g[0] = 2.0 * x[0];
        g[1] = 2.0 * x[1];
    };
    let opts = NonlinearCgOptions {
        max_iters: 300,
        tol: 1e-9,
        ..Default::default()
    };
    let result = nonlinear_cg(&x0, cost, gradient, &opts);
    assert!(result.cost < 1e-5);
}
