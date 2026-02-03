use mathlib::{Matrix, Storage, Vector};

#[test]
fn performance_matrix_vector_correctness() {
    let n = 64;
    let mut a: Matrix<f64> = Matrix::with_storage(n, n, Storage::Column);
    a.set_zero();
    for i in 0..n {
        a.set(i, i, 1.0_f64);
    }
    let mut v: Vector<f64> = Vector::with_capacity(n);
    v.set_zero();
    v.set(0, 1.0_f64);
    let b = &a * &v;
    assert!((b.get(0) - 1.0_f64).abs() < 1e-9);
}

#[test]
fn performance_matrix_add_correctness() {
    let n = 64;
    let mut a: Matrix<f64> = Matrix::with_storage(n, n, Storage::Column);
    a.set_identity();
    let b: Matrix<f64> = &a + &a;
    for i in 0..n {
        assert!((b.get(i, i) - 2.0_f64).abs() < 1e-9);
    }
}
