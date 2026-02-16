//! Matrix norms (Section 7.2.3).
//!
//! - **Frobenius norm**: ‖A‖_F = sqrt(sum_ij A_ij²). Uses SIMD when the `simd` feature is enabled.
//! - **Spectral norm** (matrix 2-norm): largest singular value; computed via SVD.

use crate::cpu;
use crate::matrix::Matrix;
use crate::svd_econ;

/// Frobenius norm of a dense matrix: ‖A‖_F = sqrt(sum_ij A_ij²).
///
/// Uses the CPU backend (SIMD when the `simd` feature is enabled) for the sum of squares.
#[must_use]
pub fn frobenius_norm_f64(a: &Matrix<f64>) -> f64 {
    let data = a.data();
    if data.is_empty() {
        return 0.0;
    }
    let sum_sq = cpu::squared_sum_f64(data);
    sum_sq.sqrt()
}

/// Frobenius norm of a dense f32 matrix: ‖A‖_F = sqrt(sum_ij A_ij²).
#[must_use]
pub fn frobenius_norm_f32(a: &Matrix<f32>) -> f32 {
    let data = a.data();
    if data.is_empty() {
        return 0.0;
    }
    let sum_sq = cpu::squared_sum_f32(data);
    sum_sq.sqrt()
}

/// Spectral norm (matrix 2-norm) of A: largest singular value.
///
/// Computed via economical SVD. For the zero matrix or empty matrix, returns 0.0.
#[must_use]
pub fn spectral_norm_f64(a: &Matrix<f64>) -> f64 {
    if a.rows() == 0 || a.cols() == 0 {
        return 0.0;
    }
    let svd = svd_econ(a);
    let sigma = svd.sigma();
    if sigma.rows() == 0 {
        0.0
    } else {
        sigma.get(0).abs()
    }
}
