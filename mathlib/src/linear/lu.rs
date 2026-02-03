//! LU decomposition with partial pivoting: P A = L U.
//! Used for solving general square linear systems.

use crate::matrix::Matrix;
use crate::types::Storage;
use crate::vector::Vector;
use std::fmt;
use tracing::debug;

/// Error from LU decomposition.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LuError {
    /// Matrix is not square.
    NotSquare,
    /// Matrix is singular (zero pivot encountered).
    Singular,
}

impl std::error::Error for LuError {}

impl fmt::Display for LuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LuError::NotSquare => write!(f, "matrix is not square"),
            LuError::Singular => write!(f, "matrix is singular"),
        }
    }
}

/// LU decomposition with row pivoting. Stores combined L (unit diagonal) and U in one matrix
/// plus pivot indices: P A = L U, so A = P^T L U.
#[derive(Clone, Debug)]
#[allow(clippy::struct_field_names)] // `lu` is standard naming for the LU factor matrix
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lu {
    /// Combined LU matrix: lower part is L (unit diagonal implicit), upper part is U.
    pub(crate) lu: Matrix<f64>,
    /// Row pivot: pivot[i] = row index that was swapped to row i (0-based).
    pub(crate) pivot: Vec<usize>,
    /// Sign of permutation: (-1)^(number of row swaps).
    pub(crate) sign: i8,
}

impl Lu {
    /// Compute LU decomposition with partial pivoting. A must be square.
    ///
    /// # Errors
    ///
    /// Returns `LuError::NotSquare` if `a` is not square. Returns `LuError::Singular`
    /// if a zero pivot is encountered (matrix is singular).
    #[must_use = "this `Result` may be an `Err` that should be handled"]
    pub fn new(a: &Matrix<f64>) -> Result<Self, LuError> {
        let rows = a.rows();
        let cols = a.cols();
        debug!(rows, cols, "lu");
        let n = rows;
        if cols != n {
            return Err(LuError::NotSquare);
        }
        let mut lu = Matrix::with_storage(n, n, Storage::Column);
        for i in 0..n {
            for j in 0..n {
                lu.set(i, j, a.get(i, j));
            }
        }
        let mut pivot: Vec<usize> = (0..n).collect();
        let mut sign: i8 = 1;

        for k in 0..n {
            // Find pivot in column k
            let mut max_row = k;
            let mut max_val = lu.get(k, k).abs();
            for i in (k + 1)..n {
                let v = lu.get(i, k).abs();
                if v > max_val {
                    max_val = v;
                    max_row = i;
                }
            }
            if max_val == 0.0 {
                return Err(LuError::Singular);
            }
            if max_row != k {
                sign = -sign;
                pivot.swap(k, max_row);
                for j in 0..n {
                    let tmp = lu.get(k, j);
                    lu.set(k, j, lu.get(max_row, j));
                    lu.set(max_row, j, tmp);
                }
            }
            let pivot_val = lu.get(k, k);
            for i in (k + 1)..n {
                let factor = lu.get(i, k) / pivot_val;
                lu.set(i, k, factor);
                for j in (k + 1)..n {
                    let v = lu.get(i, j) - factor * lu.get(k, j);
                    lu.set(i, j, v);
                }
            }
        }
        Ok(Lu { lu, pivot, sign })
    }

    /// Determinant of the original matrix: det(A) = sign * prod(diag(U)).
    #[must_use]
    pub fn determinant(&self) -> f64 {
        let n = self.size();
        let mut prod = f64::from(self.sign);
        for i in 0..n {
            prod *= self.lu.get(i, i);
        }
        prod
    }

    /// Number of rows/columns.
    #[inline]
    pub fn size(&self) -> usize {
        self.lu.rows()
    }

    /// Combined LU matrix (L unit lower, U upper).
    pub fn lu(&self) -> &Matrix<f64> {
        &self.lu
    }

    /// Deprecated. Use [`lu()`](Self::lu) instead.
    #[deprecated(since = "0.1.0", note = "use `lu()` instead")]
    pub fn get_lu(&self) -> &Matrix<f64> {
        self.lu()
    }

    /// Pivot indices: row i of permuted system is original row pivot[i].
    pub fn pivot(&self) -> &[usize] {
        &self.pivot
    }

    /// Solve A x = b (i.e. P^T L U x = b). Returns x.
    ///
    /// # Panics
    ///
    /// Panics if `b.rows()` is not equal to the matrix dimension.
    #[must_use = "this returns the solution vector and does not modify the inputs"]
    pub fn solve(&self, b: &Vector<f64>) -> Vector<f64> {
        let n = self.size();
        debug!(len = b.rows(), "lu solve");
        assert_eq!(b.rows(), n, "b must have same length as matrix dimension");

        // Permute b: pb[i] = b[pivot[i]]
        let mut pb = Vector::with_capacity(n);
        pb.set_zero();
        for i in 0..n {
            pb.set(i, b.get(self.pivot[i]));
        }

        // Solve L y = pb (forward; L has unit diagonal, stored in lower part of lu)
        let mut y = Vector::with_capacity(n);
        y.set_zero();
        for i in 0..n {
            let mut s = pb.get(i);
            for j in 0..i {
                s -= self.lu.get(i, j) * y.get(j);
            }
            y.set(i, s);
        }

        // Solve U x = y (backward)
        let mut x = Vector::with_capacity(n);
        x.set_zero();
        for i in (0..n).rev() {
            let mut s = y.get(i);
            for j in (i + 1)..n {
                s -= self.lu.get(i, j) * x.get(j);
            }
            let uii = self.lu.get(i, i);
            if uii == 0.0 {
                // Should not happen if LU succeeded
                continue;
            }
            x.set(i, s / uii);
        }
        x
    }
}

/// Compute determinant of a square matrix via LU decomposition.
///
/// # Errors
///
/// Returns `LuError::NotSquare` if `a` is not square. Returns `LuError::Singular`
/// if the matrix is singular (not invertible).
#[must_use = "this `Result` may be an `Err` that should be handled"]
pub fn det(a: &Matrix<f64>) -> Result<f64, LuError> {
    debug!(rows = a.rows(), cols = a.cols(), "det");
    let lu = Lu::new(a)?;
    Ok(lu.determinant())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_matrix(rows: usize, cols: usize, data: &[f64]) -> Matrix<f64> {
        let mut m = Matrix::with_storage(rows, cols, Storage::Column);
        for i in 0..rows {
            for j in 0..cols {
                m.set(i, j, data[i * cols + j]);
            }
        }
        m
    }

    #[test]
    fn test_det_2x2() {
        // [1 2] det = 1*4 - 2*3 = -2
        // [3 4]
        let a = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        let d = det(&a).unwrap();
        assert!((d - (-2.0)).abs() < 1e-10);
    }

    #[test]
    fn test_det_identity() {
        let mut a = Matrix::with_storage(3, 3, Storage::Column);
        a.set_identity();
        let d = det(&a).unwrap();
        assert!((d - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_det_singular() {
        // [1 2]  [1 2]   rows are linearly dependent
        // [1 2]  det = 0
        let a = make_matrix(2, 2, &[1.0, 2.0, 1.0, 2.0]);
        assert!(matches!(det(&a), Err(LuError::Singular)));
    }

    #[test]
    fn test_det_not_square() {
        let a = make_matrix(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert!(matches!(det(&a), Err(LuError::NotSquare)));
    }

    #[test]
    fn test_det_ab_eq_det_a_det_b() {
        let a = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        let b = make_matrix(2, 2, &[5.0, 6.0, 7.0, 8.0]);
        let ab = &a * &b;
        let da = det(&a).unwrap();
        let db = det(&b).unwrap();
        let dab = det(&ab).unwrap();
        assert!((dab - da * db).abs() < 1e-10);
    }
}
