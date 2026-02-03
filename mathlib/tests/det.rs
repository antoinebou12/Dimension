//! Integration tests for determinant via LU.

use mathlib::{Lu, LuError, Matrix, Storage, det};

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
fn det_2x2_known() {
    // [1 2]  det = 1*4 - 2*3 = -2
    // [3 4]
    let a = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
    let d = det(&a).unwrap();
    assert!((d - (-2.0)).abs() < 1e-10);
}

#[test]
fn det_3x3_known() {
    // Upper triangular [1 2 3; 0 4 5; 0 0 6] -> det = 24
    let a = make_matrix(3, 3, &[1.0, 0.0, 0.0, 2.0, 4.0, 0.0, 3.0, 5.0, 6.0]);
    let d = det(&a).unwrap();
    assert!((d - 24.0).abs() < 1e-10);
}

#[test]
fn det_identity_is_one() {
    let mut a = Matrix::with_storage(4, 4, Storage::Column);
    a.set_identity();
    let d = det(&a).unwrap();
    assert!((d - 1.0).abs() < 1e-10);
}

#[test]
fn det_singular_returns_err() {
    let a = make_matrix(2, 2, &[1.0, 2.0, 1.0, 2.0]);
    assert!(matches!(det(&a), Err(LuError::Singular)));
}

#[test]
fn det_not_square_returns_err() {
    let a = make_matrix(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    assert!(matches!(det(&a), Err(LuError::NotSquare)));
}

#[test]
fn det_ab_eq_det_a_det_b() {
    let a = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
    let b = make_matrix(2, 2, &[5.0, 6.0, 7.0, 8.0]);
    let ab = &a * &b;
    let da = det(&a).unwrap();
    let db = det(&b).unwrap();
    let dab = det(&ab).unwrap();
    assert!((dab - da * db).abs() < 1e-10);
}

#[test]
fn lu_determinant_matches_det() {
    let a = make_matrix(3, 3, &[1.0, 0.0, 0.0, 2.0, 4.0, 0.0, 3.0, 5.0, 6.0]);
    let lu = Lu::new(&a).unwrap();
    let d1 = lu.determinant();
    let d2 = det(&a).unwrap();
    assert!((d1 - d2).abs() < 1e-10);
}
