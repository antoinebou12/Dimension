use mathlib::{SparseMatrix, Triplet, Vector};

#[test]
fn sparse_matrix_triplets() {
    let mut a: SparseMatrix<f64> = SparseMatrix::with_dimensions(3, 3);
    let triplets = [
        Triplet::new(1.0_f64, 1, 1),
        Triplet::new(2.5_f64, 2, 1),
        Triplet::new(-0.1_f64, 2, 2),
        Triplet::new(6.0_f64, 0, 0),
    ];
    a.set_from_triplets(&triplets);
    assert!(a.get(2, 0).abs() < 1e-9);
    assert!((a.get(0, 0) - 6.0_f64).abs() < 1e-9);
    assert!((a.get(2, 1) - 2.5_f64).abs() < 1e-9);
}

#[test]
fn sparse_matrix_vector() {
    let mut a: SparseMatrix<f64> = SparseMatrix::with_dimensions(3, 3);
    let triplets = [
        Triplet::new(1.0_f64, 1, 1),
        Triplet::new(2.5_f64, 2, 1),
        Triplet::new(-0.1_f64, 2, 2),
        Triplet::new(6.0_f64, 0, 0),
    ];
    a.set_from_triplets(&triplets);
    let mut v: Vector<f64> = Vector::with_capacity(3);
    v.set(0, 1.0_f64);
    v.set(1, 2.0_f64);
    v.set(2, 3.0_f64);
    let vout = &a * &v;
    assert!((vout.get(0) - 6.0_f64).abs() < 1e-9);
    assert!((vout.get(1) - 2.0_f64).abs() < 1e-9);
    assert!((vout.get(2) - 4.7_f64).abs() < 1e-9);
}

#[test]
fn sparse_matrix_identity() {
    let mut a: SparseMatrix<f64> = SparseMatrix::with_dimensions(3, 3);
    a.set_identity();
    assert!((a.get(0, 0) - 1.0_f64).abs() < 1e-9);
    assert!(a.get(2, 1).abs() < 1e-9);
    let mut v: Vector<f64> = Vector::with_capacity(3);
    v.set(0, 1.0_f64);
    v.set(1, 2.0_f64);
    v.set(2, 3.0_f64);
    let vout = &a * &v;
    assert!((vout.get(0) - 1.0_f64).abs() < 1e-9);
    assert!((vout.get(1) - 2.0_f64).abs() < 1e-9);
    assert!((vout.get(2) - 3.0_f64).abs() < 1e-9);
}
