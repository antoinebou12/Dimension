//! QR decomposition (Chapter 5): A = Q R.
//!
//! Modified Gram-Schmidt for reduced QR. Q has orthonormal columns; R is upper triangular.

use crate::matrix::Matrix;
use crate::types::Storage;
use crate::vector::Vector;
use std::fmt;

/// Error from QR decomposition.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QrError {
    /// Matrix has zero columns.
    Empty,
    /// Rank deficiency: a column became zero during orthogonalization.
    RankDeficient,
}

impl std::error::Error for QrError {}

impl fmt::Display for QrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QrError::Empty => write!(f, "matrix has no columns"),
            QrError::RankDeficient => write!(f, "matrix is rank deficient"),
        }
    }
}

/// QR decomposition: A = Q R.
///
/// Q is m×n with orthonormal columns; R is n×n upper triangular.
/// For m×n matrix A with m >= n (reduced QR).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Qr {
    /// Orthonormal factor Q (m×n).
    pub q: Matrix<f64>,
    /// Upper triangular factor R (n×n).
    pub r: Matrix<f64>,
}

impl Qr {
    /// Compute QR decomposition via modified Gram-Schmidt. A must have m >= n.
    ///
    /// # Errors
    ///
    /// Returns `QrError::Empty` if A has no columns. Returns `QrError::RankDeficient`
    /// if a column becomes zero during orthogonalization.
    #[must_use = "this `Result` may be an `Err` that should be handled"]
    pub fn new(a: &Matrix<f64>) -> Result<Self, QrError> {
        let m = a.rows();
        let n = a.cols();
        if n == 0 {
            return Err(QrError::Empty);
        }
        if m < n {
            return Err(QrError::RankDeficient);
        }
        let mut q = Matrix::with_storage(m, n, Storage::Column);
        let mut r = Matrix::with_storage(n, n, Storage::Column);
        r.set_zero();

        for j in 0..n {
            for i in 0..m {
                q.set(i, j, a.get(i, j));
            }
            for k in 0..j {
                let dot = (0..m).map(|i| q.get(i, k) * q.get(i, j)).sum::<f64>();
                r.set(k, j, dot);
                for i in 0..m {
                    q.set(i, j, q.get(i, j) - dot * q.get(i, k));
                }
            }
            let norm = (0..m).map(|i| q.get(i, j).powi(2)).sum::<f64>().sqrt();
            if norm < 1e-15 * (m as f64).sqrt() {
                return Err(QrError::RankDeficient);
            }
            r.set(j, j, norm);
            for i in 0..m {
                q.set(i, j, q.get(i, j) / norm);
            }
        }
        Ok(Qr { q, r })
    }

    /// Orthonormal factor Q.
    pub fn q(&self) -> &Matrix<f64> {
        &self.q
    }

    /// Upper triangular factor R.
    pub fn r(&self) -> &Matrix<f64> {
        &self.r
    }

    /// Solve Rx = Q'b for least-squares min ||Ax - b||. Returns x.
    ///
    /// # Panics
    ///
    /// Panics if b.rows() != Q.rows().
    #[must_use]
    pub fn solve(&self, b: &Vector<f64>) -> Vector<f64> {
        let m = self.q.rows();
        let n = self.q.cols();
        assert_eq!(b.rows(), m);
        let mut qtb = Vector::with_capacity(n);
        qtb.set_zero();
        for j in 0..n {
            let mut s = 0.0;
            for i in 0..m {
                s += self.q.get(i, j) * b.get(i);
            }
            qtb.set(j, s);
        }
        // Back-substitute R x = Q'b
        let mut x = Vector::with_capacity(n);
        for i in (0..n).rev() {
            let mut s = qtb.get(i);
            for j in (i + 1)..n {
                s -= self.r.get(i, j) * x.get(j);
            }
            x.set(i, s / self.r.get(i, i));
        }
        x
    }
}

/// Compute QR decomposition of A. Returns Q and R.
#[must_use = "this `Result` may be an `Err` that should be handled"]
pub fn qr(a: &Matrix<f64>) -> Result<Qr, QrError> {
    Qr::new(a)
}
