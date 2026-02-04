//! Integration tests for LU decomposition (Lu::new, solve, determinant, shape, errors).

use mathlib::{Lu, LuError, Matrix, Storage, Vector};

fn make_2x2() -> Matrix<f64> {
    let mut a = Matrix::with_storage(2, 2, Storage::Column);
    a.set(0, 0, 2.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 1.0);
    a.set(1, 1, 2.0);
    a
}

#[test]
fn lu_new_and_shape() {
    let a = make_2x2();
    let lu = Lu::new(&a).unwrap();
    assert_eq!(lu.size(), 2);
    let lu_mat = lu.lu();
    assert_eq!(lu_mat.rows(), 2);
    assert_eq!(lu_mat.cols(), 2);
    assert_eq!(lu.pivot().len(), 2);
    assert!(lu.sign() == 1 || lu.sign() == -1);
}

#[test]
fn lu_solve() {
    let a = make_2x2();
    let lu = Lu::new(&a).unwrap();
    let mut b = Vector::with_capacity(2);
    b.set(0, 1.0);
    b.set(1, 2.0);
    let x = lu.solve(&b);
    let mut ax = Vector::with_capacity(2);
    ax.set_zero();
    for i in 0..2 {
        let mut s = 0.0;
        for j in 0..2 {
            s += a.get(i, j) * x.get(j);
        }
        ax.set(i, s);
    }
    assert!((ax.get(0) - b.get(0)).abs() < 1e-10);
    assert!((ax.get(1) - b.get(1)).abs() < 1e-10);
}

#[test]
fn lu_determinant() {
    let a = make_2x2();
    let lu = Lu::new(&a).unwrap();
    let det = lu.determinant();
    // det([2 1; 1 2]) = 4 - 1 = 3
    assert!((det - 3.0).abs() < 1e-10);
}

#[test]
fn lu_not_square() {
    let mut a = Matrix::with_storage(2, 3, Storage::Column);
    a.set_zero();
    let err = Lu::new(&a).unwrap_err();
    assert_eq!(err, LuError::NotSquare);
}

#[test]
fn lu_singular() {
    let mut a = Matrix::with_storage(2, 2, Storage::Column);
    a.set(0, 0, 1.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 1.0);
    a.set(1, 1, 1.0);
    let err = Lu::new(&a).unwrap_err();
    assert_eq!(err, LuError::Singular);
}

#[test]
fn lu_3x3_identity() {
    let mut a = Matrix::with_storage(3, 3, Storage::Column);
    a.set_zero();
    a.set(0, 0, 1.0);
    a.set(1, 1, 1.0);
    a.set(2, 2, 1.0);
    let lu = Lu::new(&a).unwrap();
    assert_eq!(lu.size(), 3);
    assert!((lu.determinant() - 1.0).abs() < 1e-10);
}
