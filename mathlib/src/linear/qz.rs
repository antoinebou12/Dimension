//! Generalised Schur (QZ) decomposition: A = Q AA Z^T, B = Q BB Z^T with AA and BB upper.
//! API is defined; full implementation (QZ iteration) is left for a follow-up.

use crate::matrix::Matrix;
use std::fmt;
use tracing::debug;

/// Error from QZ decomposition.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QzError {
    /// A or B is not square, or dimensions differ.
    NotSquare,
    /// Full QZ decomposition not yet implemented.
    Unimplemented,
}

impl std::error::Error for QzError {}

impl fmt::Display for QzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QzError::NotSquare => write!(f, "matrices must be square and same size"),
            QzError::Unimplemented => write!(f, "QZ decomposition not yet implemented"),
        }
    }
}

/// Generalised Schur decomposition: A = Q AA Z^T, B = Q BB Z^T with Q and Z orthogonal.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Qz {
    /// Upper (quasi-)triangular AA.
    pub(crate) aa: Matrix<f64>,
    /// Upper (quasi-)triangular BB.
    pub(crate) bb: Matrix<f64>,
    /// Orthogonal Q.
    pub(crate) q: Matrix<f64>,
    /// Orthogonal Z.
    pub(crate) z: Matrix<f64>,
}

impl Qz {
    /// Upper factor AA (A = Q AA Z^T).
    pub fn aa(&self) -> &Matrix<f64> {
        &self.aa
    }

    /// Upper factor BB (B = Q BB Z^T).
    pub fn bb(&self) -> &Matrix<f64> {
        &self.bb
    }

    /// Orthogonal Q.
    pub fn q(&self) -> &Matrix<f64> {
        &self.q
    }

    /// Orthogonal Z.
    pub fn z(&self) -> &Matrix<f64> {
        &self.z
    }

    /// Deprecated. Use [`aa()`](Self::aa) instead.
    #[deprecated(since = "0.1.0", note = "use `aa()` instead")]
    pub fn get_aa(&self) -> &Matrix<f64> {
        self.aa()
    }

    /// Deprecated. Use [`bb()`](Self::bb) instead.
    #[deprecated(since = "0.1.0", note = "use `bb()` instead")]
    pub fn get_bb(&self) -> &Matrix<f64> {
        self.bb()
    }

    /// Deprecated. Use [`q()`](Self::q) instead.
    #[deprecated(since = "0.1.0", note = "use `q()` instead")]
    pub fn get_q(&self) -> &Matrix<f64> {
        self.q()
    }

    /// Deprecated. Use [`z()`](Self::z) instead.
    #[deprecated(since = "0.1.0", note = "use `z()` instead")]
    pub fn get_z(&self) -> &Matrix<f64> {
        self.z()
    }
}

/// Compute generalised Schur (QZ) decomposition of (A, B).
/// Currently returns `Err(QzError::Unimplemented)`; full QZ iteration to be added.
///
/// # Errors
///
/// Returns `QzError::NotSquare` if `a` or `b` is not square or dimensions differ.
/// Returns `QzError::Unimplemented` until the full implementation is added.
#[must_use = "this `Result` may be an `Err` that should be handled"]
pub fn qz(a: &Matrix<f64>, _b: &Matrix<f64>) -> Result<Qz, QzError> {
    let rows = a.rows();
    let cols = a.cols();
    debug!(rows, cols, "qz");
    Err(QzError::Unimplemented)
}
