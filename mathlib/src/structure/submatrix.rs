use super::types::Storage;
use crate::matrix::Matrix;
use std::any::TypeId;
use std::fmt;
use std::ops::AddAssign;

/// Dispatch `add_f64` to simd, parallel, or sequential (same as svd/operators).
#[inline]
fn submatrix_add_f64(a: &[f64], b: &[f64], out: &mut [f64]) {
    #[cfg(feature = "simd")]
    return crate::cpu::simd::add_f64(a, b, out);
    #[cfg(all(
        feature = "parallel",
        not(target_arch = "wasm32"),
        not(feature = "simd")
    ))]
    return crate::cpu::parallel::par_add_f64(a, b, out);
    #[cfg(not(any(
        feature = "simd",
        all(feature = "parallel", not(target_arch = "wasm32"))
    )))]
    crate::cpu::sequential::add_f64(a, b, out);
}

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
        for j in 0..self.rows {
            for i in 0..self.cols {
                out.set(i, j, self.get(j, i));
            }
        }
        out
    }

    pub fn assign_from(&mut self, other: &Matrix<T>)
    where
        T: Copy,
    {
        assert!(other.rows() == self.rows && other.cols() == self.cols);
        if self.matrix.storage == Storage::Column && other.storage == Storage::Column {
            let nrows_parent = self.matrix.rows();
            let data_mut = self.matrix.data_mut();
            let other_data = other.data();
            for j in 0..self.cols {
                let tgt = &mut data_mut[(self.j0 + j) * nrows_parent + self.i0
                    ..(self.j0 + j) * nrows_parent + self.i0 + self.rows];
                let src = &other_data[j * self.rows..][..self.rows];
                tgt.copy_from_slice(src);
            }
        } else if self.matrix.storage == Storage::Row && other.storage == Storage::Row {
            for i in 0..self.rows {
                for j in 0..self.cols {
                    self.set(i, j, other.get(i, j));
                }
            }
        } else {
            for j in 0..self.cols {
                for i in 0..self.rows {
                    self.set(i, j, other.get(i, j));
                }
            }
        }
    }

    pub fn to_matrix(&self) -> Matrix<T>
    where
        T: Copy + Default,
    {
        let mut m = Matrix::with_storage(self.rows, self.cols, self.matrix.storage);
        match self.matrix.storage {
            Storage::Column => {
                for j in 0..self.cols {
                    for i in 0..self.rows {
                        m.set(i, j, self.get(i, j));
                    }
                }
            }
            Storage::Row => {
                for i in 0..self.rows {
                    for j in 0..self.cols {
                        m.set(i, j, self.get(i, j));
                    }
                }
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

impl<T: Clone + Copy + Default + std::ops::Add<Output = T> + 'static> AddAssign<&Matrix<T>>
    for SubMatrix<'_, T>
{
    fn add_assign(&mut self, rhs: &Matrix<T>) {
        assert!(rhs.rows() == self.rows && rhs.cols() == self.cols);
        if TypeId::of::<T>() == TypeId::of::<f64>()
            && self.matrix.storage == Storage::Column
            && rhs.storage == Storage::Column
        {
            let nrows_parent = self.matrix.rows();
            let mut scratch = vec![0.0_f64; self.rows];
            let (data_mut_ptr, other_data_ptr) = (
                self.matrix.data_mut().as_mut_ptr().cast::<f64>(),
                rhs.data().as_ptr().cast::<f64>(),
            );
            for j in 0..self.cols {
                // SAFETY: T is f64 (checked by TypeId), so data pointers are valid f64 slices.
                let tgt = unsafe {
                    std::slice::from_raw_parts_mut(
                        data_mut_ptr.add((self.j0 + j) * nrows_parent + self.i0),
                        self.rows,
                    )
                };
                // SAFETY: same as above; rhs is Matrix<f64>, slice length is self.rows.
                let src = unsafe {
                    std::slice::from_raw_parts(other_data_ptr.add(j * self.rows), self.rows)
                };
                submatrix_add_f64(tgt, src, &mut scratch);
                tgt.copy_from_slice(&scratch);
            }
        } else {
            for j in 0..self.cols {
                for i in 0..self.rows {
                    self.set(i, j, self.get(i, j) + rhs.get(i, j));
                }
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

    #[test]
    fn test_submatrix_single_element_block() {
        let mut m = make_matrix(3, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
        let sub = m.block(1, 1, 1, 1);
        assert_eq!(sub.rows(), 1);
        assert_eq!(sub.cols(), 1);
        assert!((sub.get(0, 0) - 5.0).abs() < 1e-10);
        {
            let mut sub = m.block(1, 1, 1, 1);
            sub.set(0, 0, -5.0);
        }
        assert!((m.get(1, 1) - (-5.0)).abs() < 1e-10);
    }

    #[test]
    fn test_submatrix_full_matrix_block() {
        let data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let mut m = make_matrix(2, 3, &data);
        let sub = m.block(0, 0, 2, 3);
        assert_eq!(sub.rows(), 2);
        assert_eq!(sub.cols(), 3);
        for i in 0..2 {
            for j in 0..3 {
                assert!((sub.get(i, j) - data[i * 3 + j]).abs() < 1e-10);
            }
        }
        let extracted = sub.to_matrix();
        for i in 0..2 {
            for j in 0..3 {
                assert!((extracted.get(i, j) - data[i * 3 + j]).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_submatrix_row_major_storage() {
        use crate::Storage;
        use crate::matrix::Matrix;
        let mut m = Matrix::with_storage(3, 3, Storage::Row);
        for i in 0..3 {
            for j in 0..3 {
                m.set(i, j, f64::from((i * 3 + j) as u32) + 1.0);
            }
        }
        let sub = m.block(1, 1, 2, 2);
        assert!((sub.get(0, 0) - 5.0).abs() < 1e-10);
        assert!((sub.get(0, 1) - 6.0).abs() < 1e-10);
        assert!((sub.get(1, 0) - 8.0).abs() < 1e-10);
        assert!((sub.get(1, 1) - 9.0).abs() < 1e-10);
        {
            let mut sub = m.block(1, 1, 2, 2);
            sub.set(0, 0, 100.0);
        }
        assert!((m.get(1, 1) - 100.0).abs() < 1e-10);
        let sub = m.block(0, 0, 2, 2);
        let out = sub.to_matrix();
        assert_eq!(out.rows(), 2);
        assert_eq!(out.cols(), 2);
        assert!((out.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((out.get(1, 0) - 4.0).abs() < 1e-10);
        assert!((out.get(1, 1) - 100.0).abs() < 1e-10);
    }

    #[test]
    #[should_panic(expected = "assertion")]
    fn test_submatrix_block_bounds_rows() {
        let mut m = make_matrix(4, 4, &[0.0; 16]);
        let _ = m.block(2, 0, 3, 2);
    }

    #[test]
    #[should_panic(expected = "assertion")]
    fn test_submatrix_block_bounds_cols() {
        let mut m = make_matrix(4, 4, &[0.0; 16]);
        let _ = m.block(0, 2, 2, 3);
    }
}
