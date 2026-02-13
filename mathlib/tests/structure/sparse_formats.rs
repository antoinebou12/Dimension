//! Tests for sparse matrix formats (CCS, BCRS, CDS, JDS, SKS).

use mathlib::{
    SparseMatrixBCRS, SparseMatrixCCS, SparseMatrixCDS, SparseMatrixJDS, SparseMatrixSKS,
    SparseStorage, Triplet, Vector,
};

fn make_vector(data: &[f64]) -> Vector<f64> {
    let mut v = Vector::with_capacity(data.len());
    for (i, &val) in data.iter().enumerate() {
        v.set(i, val);
    }
    v
}

#[test]
fn sparse_ccs_triplets_get_mul_vector() {
    let triplets = [
        Triplet::new(6.0_f64, 0, 0),
        Triplet::new(1.0_f64, 1, 1),
        Triplet::new(2.5_f64, 2, 1),
        Triplet::new(-0.1_f64, 2, 2),
    ];
    let ccs = SparseMatrixCCS::from_triplets(3, 3, &triplets);
    assert_eq!(ccs.rows(), 3);
    assert_eq!(ccs.cols(), 3);
    assert!(ccs.get(2, 0).abs() < 1e-9);
    assert!((ccs.get(0, 0) - 6.0_f64).abs() < 1e-9);
    assert!((ccs.get(2, 1) - 2.5_f64).abs() < 1e-9);

    let v = make_vector(&[1.0, 2.0, 3.0]);
    let vout = ccs.mul_vector(&v);
    assert!((vout.get(0) - 6.0_f64).abs() < 1e-9);
    assert!((vout.get(1) - 2.0_f64).abs() < 1e-9);
    assert!((vout.get(2) - 4.7_f64).abs() < 1e-9);
}

#[test]
fn sparse_ccs_mul_vector_transpose() {
    let triplets = [
        Triplet::new(1.0_f64, 0, 0),
        Triplet::new(2.0_f64, 1, 0),
        Triplet::new(3.0_f64, 2, 0),
    ];
    let ccs = SparseMatrixCCS::from_triplets(3, 1, &triplets);
    let v = make_vector(&[1.0, 2.0, 3.0]);
    let vout = ccs.mul_vector_transpose(&v);
    assert_eq!(vout.rows(), 1);
    assert!((vout.get(0) - 14.0_f64).abs() < 1e-9);
}

#[test]
fn sparse_cds_triplets_get_mul_vector() {
    // Matrix: [1 0.5 0; 0 2 0.5; 0 0 3]
    let triplets = [
        Triplet::new(1.0_f64, 0, 0),
        Triplet::new(2.0_f64, 1, 1),
        Triplet::new(3.0_f64, 2, 2),
        Triplet::new(0.5_f64, 0, 1),
        Triplet::new(0.5_f64, 1, 2),
    ];
    let cds = SparseMatrixCDS::from_triplets(3, 3, &triplets);
    assert_eq!(cds.rows(), 3);
    assert_eq!(cds.cols(), 3);
    assert!((cds.get(0, 0) - 1.0_f64).abs() < 1e-9);
    assert!((cds.get(1, 1) - 2.0_f64).abs() < 1e-9);
    assert!((cds.get(0, 1) - 0.5_f64).abs() < 1e-9);

    // A * v with v = [1, 2, 3] => [1*1+0.5*2, 2*2+0.5*3, 3*3] = [2, 5.5, 9]
    let v = make_vector(&[1.0_f64, 2.0, 3.0]);
    let vout = cds.mul_vector(&v);
    assert!((vout.get(0) - 2.0_f64).abs() < 1e-9);
    assert!((vout.get(1) - 5.5_f64).abs() < 1e-9);
    assert!((vout.get(2) - 9.0_f64).abs() < 1e-9);
}

#[test]
fn sparse_cds_mul_vector_transpose() {
    let triplets = [Triplet::new(1.0_f64, 0, 0), Triplet::new(2.0_f64, 1, 1)];
    let cds = SparseMatrixCDS::from_triplets(2, 2, &triplets);
    let v = make_vector(&[3.0, 4.0]);
    let vout = cds.mul_vector_transpose(&v);
    assert!((vout.get(0) - 3.0).abs() < 1e-9);
    assert!((vout.get(1) - 8.0).abs() < 1e-9);
}

#[test]
fn sparse_bcrs_from_triplets_get_mul_vector() {
    let triplets = [Triplet::new(1.0_f64, 0, 0), Triplet::new(2.0_f64, 1, 1)];
    let bcrs = SparseMatrixBCRS::from_triplets(2, 2, &triplets);
    assert_eq!(bcrs.rows(), 2);
    assert_eq!(bcrs.cols(), 2);

    let v = make_vector(&[1.0, 1.0]);
    let vout = bcrs.mul_vector(&v);
    assert_eq!(vout.rows(), 2);

    let vout_t = bcrs.mul_vector_transpose(&v);
    assert_eq!(vout_t.rows(), 2);
}

#[test]
fn sparse_jds_from_triplets_get_mul_vector() {
    let triplets = [Triplet::new(1.0_f64, 0, 0), Triplet::new(2.0_f64, 1, 1)];
    let jds = SparseMatrixJDS::from_triplets(2, 2, &triplets);
    assert_eq!(jds.rows(), 2);
    assert_eq!(jds.cols(), 2);
    assert_eq!(jds.nnz(), 0);

    let v = make_vector(&[1.0, 1.0]);
    let vout = jds.mul_vector(&v);
    assert_eq!(vout.rows(), 2);
    assert!(vout.get(0).abs() < 1e-9);
    assert!(vout.get(1).abs() < 1e-9);

    let vout_t = jds.mul_vector_transpose(&v);
    assert_eq!(vout_t.rows(), 2);
}

#[test]
fn sparse_sks_from_triplets_get_mul_vector() {
    let triplets = [
        Triplet::new(1.0_f64, 0, 0),
        Triplet::new(2.0_f64, 1, 0),
        Triplet::new(3.0_f64, 1, 1),
    ];
    let sks = SparseMatrixSKS::from_triplets(2, 2, &triplets);
    assert_eq!(sks.rows(), 2);
    assert_eq!(sks.cols(), 2);

    let v = make_vector(&[1.0, 1.0]);
    let vout = sks.mul_vector(&v);
    assert_eq!(vout.rows(), 2);

    let vout_t = sks.mul_vector_transpose(&v);
    assert_eq!(vout_t.rows(), 2);
}
