//! Solve systems of linear equations Ax = b.
//!
//! For general square A, uses [LU](crate::lu::Lu) decomposition with partial pivoting.
//! For symmetric positive definite A, prefer [`Cholesky`](crate::chol::Cholesky) for efficiency.
//! For large sparse systems, iterative methods (e.g. conjugate gradient for SPD: [`solve_cg_sparse`](crate::argmin::solve_cg_sparse)) are preferred over converting to dense and using direct solvers.

use super::chol::{CholError, Cholesky};
use super::lu::{Lu, LuError};
use crate::matrix::Matrix;
use crate::types::Storage;
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

impl From<CholError> for SolveError {
    fn from(e: CholError) -> Self {
        match e {
            CholError::NotSquare => SolveError::NotSquare,
            CholError::NotSPD => SolveError::Singular,
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

/// Damped least-squares solve for (generally non-square) matrices.
///
/// Given `A` (m×n) and `b` (m×1), returns the vector that minimises
/// `||A x - b||² + λ² ||x||²`. Internally this forms the normal equations
/// `(A Aᵀ + λ² I) y = b` and back-substitutes `x = Aᵀ y`, which is the same
/// formulation popularised by `Halley` / `QuIK` inverse kinematics.
///
/// For λ² = 0 this reduces to the normal equations; the system may be
/// ill-conditioned if `A` is rank-deficient. Using λ² > 0 improves stability
/// and reduces the norm of the solution.
#[must_use = "this `Result` may be an `Err` that should be handled"]
pub fn damped_least_squares(
    a: &Matrix<f64>,
    b: &Vector<f64>,
    lambda_sq: f64,
) -> Result<Vector<f64>, SolveError> {
    let rows = a.rows();
    let cols = a.cols();
    assert_eq!(
        b.rows(),
        rows,
        "right-hand side must have the same number of rows as A"
    );

    let a_t = a.transpose();
    let mut normal = Matrix::with_storage(rows, rows, Storage::Column);
    a.mul_into(&a_t, &mut normal);
    if lambda_sq > 0.0 {
        for i in 0..rows {
            normal.set(i, i, normal.get(i, i) + lambda_sq);
        }
    }

    let chol = Cholesky::new(&normal)?;
    let y = chol.solve(b);

    let mut x = Vector::with_capacity(cols);
    x.set_zero();
    for j in 0..cols {
        let mut sum = 0.0;
        for i in 0..rows {
            sum += a.get(i, j) * y.get(i);
        }
        x.set(j, sum);
    }
    Ok(x)
}
