//! Integration tests for Gauss-Newton.

use mathlib::{GaussNewtonOptions, gauss_newton};

#[test]
fn gauss_newton_integration() {
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
        let mut j = mathlib::Matrix::with_storage(3, 2, mathlib::structure::Storage::Column);
        for (i, &xi) in xs.iter().enumerate() {
            j.set(i, 0, -xi);
            j.set(i, 1, -1.0);
        }
        j
    };
    let opts = GaussNewtonOptions::default();
    let result = gauss_newton(&x0, residual, jacobian, &opts).unwrap();
    assert!((result.x[0] - 1.0).abs() < 1e-4);
    assert!(result.x[1].abs() < 1e-4);
}
