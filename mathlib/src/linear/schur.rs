//! Schur decomposition: A = Q T Q^T with T real upper quasi-triangular.
//! API is defined; full implementation (Francis QR iteration) is left for a follow-up.

use crate::matrix::Matrix;
use std::fmt;
use tracing::debug;

/// Error from Schur decomposition.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SchurError {
    /// Matrix is not square.
    NotSquare,
    /// Full real Schur decomposition not yet implemented.
    Unimplemented,
}

impl std::error::Error for SchurError {}

impl fmt::Display for SchurError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchurError::NotSquare => write!(f, "matrix is not square"),
            SchurError::Unimplemented => write!(f, "Schur decomposition not yet implemented"),
        }
    }
}

/// Real Schur decomposition: A = Q T Q^T with Q orthogonal and T upper quasi-triangular.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Schur {
    /// Orthogonal matrix Q.
    pub(crate) q: Matrix<f64>,
    /// Upper quasi-triangular T (1×1 and 2×2 blocks on diagonal).
    pub(crate) t: Matrix<f64>,
}

impl Schur {
    /// Orthogonal factor Q.
    pub fn q(&self) -> &Matrix<f64> {
        &self.q
    }

    /// Upper quasi-triangular factor T.
    pub fn t(&self) -> &Matrix<f64> {
        &self.t
    }

    /// Deprecated. Use [`q()`](Self::q) instead.
    #[deprecated(since = "0.1.0", note = "use `q()` instead")]
    pub fn get_q(&self) -> &Matrix<f64> {
        self.q()
    }

    /// Deprecated. Use [`t()`](Self::t) instead.
    #[deprecated(since = "0.1.0", note = "use `t()` instead")]
    pub fn get_t(&self) -> &Matrix<f64> {
        self.t()
    }
}

/// Compute real Schur decomposition A = Q T Q^T.
/// Currently returns `Err(SchurError::Unimplemented)`; full Francis double-shift QR iteration to be added.
///
/// # Errors
///
/// Returns `SchurError::NotSquare` if `a` is not square. Returns `SchurError::Unimplemented`
/// until the full implementation is added.
#[must_use = "this `Result` may be an `Err` that should be handled"]
pub fn schur(a: &Matrix<f64>) -> Result<Schur, SchurError> {
    let rows = a.rows();
    let cols = a.cols();
    debug!(rows, cols, "schur");
    Err(SchurError::Unimplemented)
}
