//! Integration tests for stats (covariance).

use mathlib::{Matrix, Storage, covariance};

fn make_matrix(rows: usize, cols: usize, data: &[f64]) -> Matrix<f64> {
    let mut m = Matrix::with_storage(rows, cols, Storage::Column);
    for i in 0..rows {
        for j in 0..cols {
            m.set(i, j, data[i * cols + j]);
        }
    }
    m
}

#[test]
fn covariance_2x2() {
    let data = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
    let cov = covariance(&data);
    assert!((cov.get(0, 0) - 2.0).abs() < 1e-10);
    assert!((cov.get(0, 1) - 2.0).abs() < 1e-10);
    assert!((cov.get(1, 0) - 2.0).abs() < 1e-10);
    assert!((cov.get(1, 1) - 2.0).abs() < 1e-10);
}

#[test]
fn covariance_symmetric() {
    let data = make_matrix(
        5,
        3,
        &[
            1.0, 2.0, 3.0, 4.0, 5.0, 2.0, 4.0, 6.0, 8.0, 10.0, 0.0, 1.0, 0.0, 1.0, 0.0,
        ],
    );
    let cov = covariance(&data);
    for i in 0..3 {
        for j in 0..3 {
            assert!((cov.get(i, j) - cov.get(j, i)).abs() < 1e-10);
        }
    }
}

#[test]
fn covariance_positive_semi_definite() {
    let data = make_matrix(
        5,
        3,
        &[
            1.0, 2.0, 3.0, 4.0, 5.0, 2.0, 4.0, 6.0, 8.0, 10.0, 0.0, 1.0, 0.0, 1.0, 0.0,
        ],
    );
    let cov = covariance(&data);
    let test_vectors: [[f64; 3]; 4] = [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 1.0],
        [2.0, -1.0, 0.5],
    ];
    for x in &test_vectors {
        let mut quad = 0.0;
        for i in 0..3 {
            for j in 0..3 {
                quad += x[i] * cov.get(i, j) * x[j];
            }
        }
        assert!(quad >= -1e-10, "x^T C x = {} should be >= 0", quad);
    }
}
