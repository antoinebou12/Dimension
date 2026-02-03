use mathlib::{Matrix, Vector};

#[test]
fn matrix_identity_scalar() {
    let mut a: Matrix<f64> = Matrix::with_dimensions(6, 6);
    a.set_identity();
    let alpha = 2.5_f64;
    let b: Matrix<f64> = alpha * &a;
    for i in 0..6 {
        assert!((b.get(i, i) - alpha).abs() < 1e-9);
    }
    assert!(b.get(0, 1).abs() < 1e-9);
}

#[test]
fn matrix_matrix_mul() {
    let mut a: Matrix<f64> = Matrix::with_dimensions(6, 6);
    a.set_identity();
    let b: Matrix<f64> = 2.5_f64 * &a;
    let c: Matrix<f64> = &a * &b;
    for i in 0..6 {
        assert!((c.get(i, i) - 2.5).abs() < 1e-9);
    }
}

#[test]
fn matrix_add() {
    let mut a: Matrix<f64> = Matrix::with_dimensions(6, 6);
    a.set_identity();
    let b: Matrix<f64> = 2.5_f64 * &a;
    let a_plus_b: Matrix<f64> = &a + &b;
    for i in 0..6 {
        assert!((a_plus_b.get(i, i) - 3.5).abs() < 1e-9);
    }
}

#[test]
fn matrix_vector_mul() {
    let mut m: Matrix<f64> = Matrix::with_dimensions(5, 5);
    m.set_identity();
    let mut v: Vector<f64> = Vector::with_capacity(5);
    v.set(0, 1.0);
    v.set(1, 2.0);
    v.set(2, 4.0);
    v.set(3, 8.0);
    v.set(4, 16.0);
    let b = &m * &v;
    assert!((b.get(0) - 1.0).abs() < 1e-9);
    assert!((b.get(4) - 16.0).abs() < 1e-9);
}

#[test]
fn vector_ops() {
    let mut v: Vector<f64> = Vector::with_capacity(5);
    v.set(0, 0.1);
    v.set(1, 0.2);
    v.set(2, 0.4);
    v.set(3, 0.8);
    v.set(4, 1.6);
    let v2: Vector<f64> = 4.0_f64 * &v;
    assert!((v2.get(0) - 0.4_f64).abs() < 1e-9);
    let v3: Vector<f64> = &v + &v2;
    assert!((v3.get(0) - 0.5_f64).abs() < 1e-9);
}
