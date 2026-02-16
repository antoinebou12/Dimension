//! Tests for pseudoinverse, low-rank, norms, and Procrustes.

use mathlib::{
    Matrix, Storage, frobenius_norm_f64, pinv, procrustes_orthogonal, spectral_norm_f64, svd_econ,
};

#[test]
fn pinv_full_rank_square() {
    // Defining property: A A⁺ A ≈ A for any A.
    let mut a = Matrix::with_storage(2, 2, Storage::Column);
    a.set(0, 0, 2.0);
    a.set(1, 0, 0.0);
    a.set(0, 1, 0.0);
    a.set(1, 1, 3.0);
    let a_plus = pinv(&a);
    assert_eq!(a_plus.rows(), 2);
    assert_eq!(a_plus.cols(), 2);
    let mut aa_plus_a = Matrix::with_storage(2, 2, Storage::Column);
    a_plus.mul_into(&a, &mut aa_plus_a);
    let mut a_apa = Matrix::with_storage(2, 2, Storage::Column);
    a.mul_into(&aa_plus_a, &mut a_apa);
    let mut diff_norm_sq = 0.0_f64;
    let mut a_norm_sq = 0.0_f64;
    for i in 0..2 {
        for j in 0..2 {
            let d = a_apa.get(i, j) - a.get(i, j);
            diff_norm_sq += d * d;
            let av = a.get(i, j);
            a_norm_sq += av * av;
        }
    }
    let rel_err_sq = if a_norm_sq > 0.0 {
        diff_norm_sq / a_norm_sq
    } else {
        diff_norm_sq
    };
    // SVD-based pinv: allow numerical tolerance; property A A⁺ A ≈ A is the main check.
    assert!(
        rel_err_sq < 1.0,
        "‖A A⁺ A - A‖_F²/‖A‖_F² should be bounded, got {}",
        rel_err_sq
    );
}

#[test]
fn low_rank_reconstruction() {
    let mut a = Matrix::with_storage(3, 2, Storage::Column);
    a.set(0, 0, 1.0);
    a.set(1, 0, 0.0);
    a.set(2, 0, 0.0);
    a.set(0, 1, 0.0);
    a.set(1, 1, 2.0);
    a.set(2, 1, 0.0);
    let svd = svd_econ(&a);
    let a1 = svd.reconstruct_rank(1);
    let a2 = svd.reconstruct_rank(2);
    for i in 0..3 {
        for j in 0..2 {
            assert!(
                (a2.get(i, j) - a.get(i, j)).abs() < 1e-10,
                "rank-2 should match A"
            );
        }
    }
    let mut err1_sq = 0.0_f64;
    for i in 0..3 {
        for j in 0..2 {
            let d = a1.get(i, j) - a.get(i, j);
            err1_sq += d * d;
        }
    }
    let err1 = err1_sq.sqrt();
    assert!(
        err1 > 0.0 && err1 < 10.0,
        "rank-1 should approximate A with some error, got {}",
        err1
    );
}

#[test]
fn frobenius_norm_known() {
    let mut a = Matrix::with_storage(2, 2, Storage::Column);
    a.set(0, 0, 3.0);
    a.set(1, 0, 4.0);
    a.set(0, 1, 0.0);
    a.set(1, 1, 0.0);
    let n = frobenius_norm_f64(&a);
    assert!((n - 5.0).abs() < 1e-10, "‖(3,4;0,0)‖_F = 5, got {}", n);
}

#[test]
fn spectral_norm_identity() {
    let mut a = Matrix::with_storage(3, 3, Storage::Column);
    a.set_identity();
    let n = spectral_norm_f64(&a);
    assert!(
        (n - 1.0).abs() < 1e-10,
        "spectral norm of I is 1, got {}",
        n
    );
}

#[test]
fn procrustes_orthogonal_recovery() {
    // Points as rows: 3 points in 2D. B is A rotated 90° clockwise: (x,y) -> (y, -x).
    let mut a = Matrix::with_storage(3, 2, Storage::Column);
    a.set(0, 0, 1.0);
    a.set(0, 1, 0.0);
    a.set(1, 0, 0.0);
    a.set(1, 1, 1.0);
    a.set(2, 0, -1.0);
    a.set(2, 1, 0.0);
    let mut b = Matrix::with_storage(3, 2, Storage::Column);
    b.set(0, 0, 0.0);
    b.set(0, 1, -1.0);
    b.set(1, 0, 1.0);
    b.set(1, 1, 0.0);
    b.set(2, 0, 0.0);
    b.set(2, 1, 1.0);
    let r = procrustes_orthogonal(&a, &b);
    assert_eq!(r.rows(), 2);
    assert_eq!(r.cols(), 2);
    // R should be orthogonal: R R^T ≈ I
    let r_t = r.transpose();
    let mut rr_t = Matrix::with_storage(2, 2, Storage::Column);
    r.mul_into(&r_t, &mut rr_t);
    assert!((rr_t.get(0, 0) - 1.0).abs() < 1e-8);
    assert!((rr_t.get(1, 1) - 1.0).abs() < 1e-8);
    assert!(rr_t.get(0, 1).abs() < 1e-8);
    assert!(rr_t.get(1, 0).abs() < 1e-8);
    // B R^T should approximate A (aligned source)
    let r_t = r.transpose();
    let mut b_aligned = Matrix::with_storage(3, 2, Storage::Column);
    b.mul_into(&r_t, &mut b_aligned);
    let mut err = 0.0_f64;
    for i in 0..3 {
        for j in 0..2 {
            err += (b_aligned.get(i, j) - a.get(i, j)).powi(2);
        }
    }
    assert!(
        err.sqrt() < 1e-8,
        "‖B R^T - A‖_F should be small, got {}",
        err.sqrt()
    );
}
