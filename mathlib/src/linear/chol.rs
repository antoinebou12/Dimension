//! Cholesky decomposition: A = L L^T for symmetric positive definite A.
//! Supports solving Ax = b via forward and backward substitution.

use crate::matrix::Matrix;
use crate::types::Storage;
use crate::vector::Vector;
use std::fmt;
use tracing::debug;

/// Error from Cholesky decomposition.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CholError {
    /// Matrix is not square.
    NotSquare,
    /// Matrix is not symmetric (within tolerance) or not positive definite.
    NotSPD,
}

impl std::error::Error for CholError {}

impl fmt::Display for CholError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CholError::NotSquare => write!(f, "matrix is not square"),
            CholError::NotSPD => write!(f, "matrix is not symmetric positive definite"),
        }
    }
}

/// Cholesky factor L such that A = L L^T (L lower triangular).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cholesky {
    /// Lower triangular factor; only the lower part (i >= j) is defined.
    pub(crate) factor: Matrix<f64>,
}

impl Cholesky {
    /// Compute Cholesky decomposition of A. A must be square and symmetric positive definite.
    ///
    /// # Errors
    ///
    /// Returns `CholError::NotSquare` if `a` is not square. Returns `CholError::NotSPD`
    /// if `a` is not symmetric positive definite (e.g. negative or zero diagonal element).
    #[must_use = "this `Result` may be an `Err` that should be handled"]
    pub fn new(a: &Matrix<f64>) -> Result<Self, CholError> {
        let rows = a.rows();
        let cols = a.cols();
        debug!(rows, cols, "chol");
        let n = rows;
        if cols != n {
            return Err(CholError::NotSquare);
        }
        let mut factor = Matrix::with_storage(n, n, Storage::Column);
        factor.set_zero();

        for i in 0..n {
            for j in 0..=i {
                let mut s = a.get(i, j);
                for k in 0..j {
                    s -= factor.get(i, k) * factor.get(j, k);
                }
                if i == j {
                    if s <= 0.0 {
                        return Err(CholError::NotSPD);
                    }
                    factor.set(i, j, s.sqrt());
                } else {
                    let ljj = factor.get(j, j);
                    if ljj == 0.0 {
                        return Err(CholError::NotSPD);
                    }
                    factor.set(i, j, s / ljj);
                }
            }
        }
        debug!("chol ok");
        Ok(Cholesky { factor })
    }

    /// Number of rows/columns of the factor.
    #[inline]
    pub fn size(&self) -> usize {
        self.factor.rows()
    }

    /// Lower triangular factor L (A = L L^T).
    pub fn l(&self) -> &Matrix<f64> {
        &self.factor
    }

    /// Deprecated. Use [`l()`](Self::l) instead.
    #[deprecated(since = "0.1.0", note = "use `l()` instead")]
    pub fn get_l(&self) -> &Matrix<f64> {
        self.l()
    }

    /// Solve Ax = b where A = L L^T. Returns x.
    ///
    /// # Panics
    ///
    /// Panics if `b.rows()` is not equal to the factor dimension.
    #[must_use = "this returns the solution vector and does not modify the inputs"]
    #[inline]
    pub fn solve(&self, b: &Vector<f64>) -> Vector<f64> {
        let n = self.size();
        debug!(len = b.rows(), "chol solve");
        assert_eq!(b.rows(), n, "b must have same length as matrix dimension");

        // Column-major: (row i, col j) at index j * n + i
        let l = self.factor.as_slice();
        let b_data = b.data();

        // Solve L y = b (forward substitution)
        let mut y = Vector::with_capacity(n);
        y.set_zero();
        let y_data = y.data_mut();
        for i in 0..n {
            let mut s = b_data[i];
            for j in 0..i {
                s -= l[j * n + i] * y_data[j];
            }
            y_data[i] = s / l[i * n + i];
        }

        // Solve L^T x = y (backward substitution)
        // Access column i of L: l[i * n + j] for row j
        let mut x = Vector::with_capacity(n);
        x.set_zero();
        let x_data = x.data_mut();
        for i in (0..n).rev() {
            let mut s = y_data[i];
            for j in (i + 1..n).rev() {
                s -= l[i * n + j] * x_data[j];
            }
            x_data[i] = s / l[i * n + i];
        }
        x
    }
}

/// Compute Cholesky decomposition of A. Convenience wrapper for [`Cholesky::new`].
///
/// # Errors
///
/// Same as [`Cholesky::new`](Cholesky::new).
#[must_use = "this `Result` may be an `Err` that should be handled"]
pub fn chol(a: &Matrix<f64>) -> Result<Cholesky, CholError> {
    Cholesky::new(a)
}
