//! Solve systems of linear equations Ax = b.
//!
//! For general square A, uses [LU](crate::lu::Lu) decomposition with partial pivoting.
//! For symmetric positive definite A, prefer [`Cholesky`](crate::chol::Cholesky) for efficiency.

use super::lu::{Lu, LuError};
use crate::matrix::Matrix;
use crate::vector::Vector;
use std::fmt;
use tracing::debug;

/// Error from solving a linear system.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SolveError {
    /// Matrix is not square.
    NotSquare,
    /// Matrix is singular (no unique solution).
    Singular,
}

impl std::error::Error for SolveError {}

impl fmt::Display for SolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolveError::NotSquare => write!(f, "matrix is not square"),
            SolveError::Singular => write!(f, "matrix is singular"),
        }
    }
}

impl From<LuError> for SolveError {
    fn from(e: LuError) -> Self {
        match e {
            LuError::NotSquare => SolveError::NotSquare,
            LuError::Singular => SolveError::Singular,
        }
    }
}

/// Solve Ax = b for general square A using [LU][Lu] decomposition.
/// Returns the solution x, or an error if A is not square or is singular.
///
/// For symmetric positive definite A, use [`Cholesky::new`] and then [`Cholesky::solve`] instead.
///
/// # Errors
///
/// Returns [`SolveError::NotSquare`] if `a` is not square. Returns [`SolveError::Singular`]
/// if `a` is singular (zero pivot during LU decomposition).
#[must_use = "this `Result` may be an `Err` that should be handled"]
pub fn solve(a: &Matrix<f64>, b: &Vector<f64>) -> Result<Vector<f64>, SolveError> {
    debug!(rows = a.rows(), cols = a.cols(), "solve Ax = b");
    let lu = Lu::new(a)?;
    Ok(lu.solve(b))
}
