//! Orthogonal Procrustes and point cloud alignment (Section 7.2.4).
//!
//! Given two point sets A and B (points as rows, shape n_points × n_dims), finds the
//! orthogonal matrix R that minimizes ‖A − B Rᵀ‖_F. Solution: SVD of Aᵀ B = U Σ Vᵀ, then R = U Vᵀ.

use super::svd::svd_econ;
use crate::matrix::Matrix;
use crate::types::Storage;

/// Orthogonal Procrustes: find orthogonal R (n_dims × n_dims) that minimizes ‖A − B Rᵀ‖_F.
///
/// A and B must have the same shape (n_points × n_dims), with points as rows. Returns R such that
/// the aligned source is B Rᵀ (each row of B multiplied by Rᵀ).
#[must_use]
pub fn procrustes_orthogonal(a: &Matrix<f64>, b: &Matrix<f64>) -> Matrix<f64> {
    assert_eq!(a.rows(), b.rows());
    assert_eq!(a.cols(), b.cols());
    let a_t = a.transpose();
    let mut h = Matrix::with_storage(a.cols(), a.cols(), Storage::Column);
    a_t.mul_into(b, &mut h);
    let svd = svd_econ(&h);
    let u = svd.u();
    let v = svd.v();
    // R = U Vᵀ
    let v_t = v.transpose();
    let mut r = Matrix::with_storage(u.rows(), v_t.cols(), Storage::Column);
    u.mul_into(&v_t, &mut r);
    r
}
