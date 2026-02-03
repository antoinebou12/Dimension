use super::types::Storage;
use crate::matrix::Matrix;
use std::fmt;
use std::ops::AddAssign;

pub struct SubMatrix<'a, T> {
    matrix: &'a mut Matrix<T>,
    i0: usize,
    j0: usize,
    rows: usize,
    cols: usize,
}

impl<'a, T: Clone + Default> SubMatrix<'a, T> {
    pub fn new(matrix: &'a mut Matrix<T>, i0: usize, j0: usize, rows: usize, cols: usize) -> Self {
        assert!(i0 + rows <= matrix.rows());
        assert!(j0 + cols <= matrix.cols());
        Self {
            matrix,
            i0,
            j0,
            rows,
            cols,
        }
    }

    #[inline]
    pub fn rows(&self) -> usize {
        self.rows
    }

    #[inline]
    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn get(&self, i: usize, j: usize) -> T
    where
        T: Copy,
    {
        assert!(i < self.rows && j < self.cols);
        self.matrix.get(self.i0 + i, self.j0 + j)
    }

    pub fn set(&mut self, i: usize, j: usize, value: T) {
        assert!(i < self.rows && j < self.cols);
        self.matrix.set(self.i0 + i, self.j0 + j, value);
    }

    pub fn transpose(&self) -> Matrix<T>
    where
        T: Copy + Default + From<u8>,
    {
        let mut out = Matrix::with_storage(self.cols, self.rows, Storage::Column);
        for i in 0..self.rows {
            for j in 0..self.cols {
                out.set(j, i, self.get(i, j));
            }
        }
        out
    }

    pub fn assign_from(&mut self, other: &Matrix<T>)
    where
        T: Copy,
    {
        assert!(other.rows() == self.rows && other.cols() == self.cols);
        for i in 0..self.rows {
            for j in 0..self.cols {
                self.set(i, j, other.get(i, j));
            }
        }
    }

    pub fn to_matrix(&self) -> Matrix<T>
    where
        T: Copy + Default,
    {
        let mut m = Matrix::with_storage(self.rows, self.cols, self.matrix.storage);
        for i in 0..self.rows {
            for j in 0..self.cols {
                m.set(i, j, self.get(i, j));
            }
        }
        m
    }
}

impl<T: Clone + Copy + Default + fmt::Display> fmt::Display for SubMatrix<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SubMatrix {}x{}[", self.rows, self.cols)?;
        for i in 0..self.rows {
            if i > 0 {
                write!(f, "; ")?;
            }
            for j in 0..self.cols {
                if j > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", self.get(i, j))?;
            }
        }
        write!(f, "]")
    }
}

#[cfg(feature = "serde")]
impl<T: Clone + Copy + Default + serde::Serialize> serde::Serialize for SubMatrix<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut data = Vec::with_capacity(self.rows * self.cols);
        for i in 0..self.rows {
            for j in 0..self.cols {
                data.push(self.get(i, j));
            }
        }
        let mut s = serializer.serialize_struct("SubMatrix", 3)?;
        s.serialize_field("rows", &self.rows)?;
        s.serialize_field("cols", &self.cols)?;
        s.serialize_field("data", &data)?;
        s.end()
    }
}

impl<T: Clone + Copy + Default + std::ops::Add<Output = T>> AddAssign<&Matrix<T>>
    for SubMatrix<'_, T>
{
    fn add_assign(&mut self, rhs: &Matrix<T>) {
        assert!(rhs.rows() == self.rows && rhs.cols() == self.cols);
        for i in 0..self.rows {
            for j in 0..self.cols {
                self.set(i, j, self.get(i, j) + rhs.get(i, j));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_matrix(rows: usize, cols: usize, data: &[f64]) -> Matrix<f64> {
        let mut m = Matrix::with_dimensions(rows, cols);
        for i in 0..rows {
            for j in 0..cols {
                m.set(i, j, data[i * cols + j]);
            }
        }
        m
    }

    #[test]
    fn test_submatrix_creation() {
        let mut m = make_matrix(
            4,
            4,
            &[
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
                16.0,
            ],
        );
        let sub = m.block(1, 1, 2, 2);
        assert_eq!(sub.rows(), 2);
        assert_eq!(sub.cols(), 2);
    }

    #[test]
    fn test_submatrix_get() {
        let mut m = make_matrix(
            4,
            4,
            &[
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
                16.0,
            ],
        );
        let sub = m.block(1, 1, 2, 2);
        assert!((sub.get(0, 0) - 6.0).abs() < 1e-10);
        assert!((sub.get(0, 1) - 7.0).abs() < 1e-10);
        assert!((sub.get(1, 0) - 10.0).abs() < 1e-10);
        assert!((sub.get(1, 1) - 11.0).abs() < 1e-10);
    }

    #[test]
    fn test_submatrix_set() {
        let mut m = make_matrix(3, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
        {
            let mut sub = m.block(0, 1, 2, 2);
            sub.set(0, 0, 100.0);
            sub.set(1, 1, 200.0);
        }
        assert!((m.get(0, 1) - 100.0).abs() < 1e-10);
        assert!((m.get(1, 2) - 200.0).abs() < 1e-10);
    }

    #[test]
    fn test_submatrix_transpose() {
        let mut m = make_matrix(3, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
        let sub = m.block(0, 0, 2, 3);
        let t = sub.transpose();
        assert_eq!(t.rows(), 3);
        assert_eq!(t.cols(), 2);
        assert!((t.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((t.get(0, 1) - 4.0).abs() < 1e-10);
        assert!((t.get(2, 0) - 3.0).abs() < 1e-10);
        assert!((t.get(2, 1) - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_submatrix_assign_from() {
        let mut m = make_matrix(
            4,
            4,
            &[
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            ],
        );
        let src = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        {
            let mut sub = m.block(1, 1, 2, 2);
            sub.assign_from(&src);
        }
        assert!((m.get(1, 1) - 1.0).abs() < 1e-10);
        assert!((m.get(1, 2) - 2.0).abs() < 1e-10);
        assert!((m.get(2, 1) - 3.0).abs() < 1e-10);
        assert!((m.get(2, 2) - 4.0).abs() < 1e-10);
        assert!((m.get(0, 0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_submatrix_to_matrix() {
        let mut m = make_matrix(3, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
        let sub = m.block(1, 0, 2, 2);
        let extracted = sub.to_matrix();
        assert_eq!(extracted.rows(), 2);
        assert_eq!(extracted.cols(), 2);
        assert!((extracted.get(0, 0) - 4.0).abs() < 1e-10);
        assert!((extracted.get(0, 1) - 5.0).abs() < 1e-10);
        assert!((extracted.get(1, 0) - 7.0).abs() < 1e-10);
        assert!((extracted.get(1, 1) - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_submatrix_add_assign() {
        let mut m = make_matrix(3, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
        let add = make_matrix(2, 2, &[10.0, 20.0, 30.0, 40.0]);
        {
            let mut sub = m.block(0, 0, 2, 2);
            sub += &add;
        }
        assert!((m.get(0, 0) - 11.0).abs() < 1e-10);
        assert!((m.get(0, 1) - 22.0).abs() < 1e-10);
        assert!((m.get(1, 0) - 34.0).abs() < 1e-10);
        assert!((m.get(1, 1) - 45.0).abs() < 1e-10);
        assert!((m.get(2, 2) - 9.0).abs() < 1e-10);
    }

    #[test]
    fn test_submatrix_display() {
        let mut m = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        let sub = m.block(0, 0, 2, 2);
        let display = format!("{}", sub);
        assert!(display.contains("SubMatrix"));
        assert!(display.contains("2x2"));
    }
}
