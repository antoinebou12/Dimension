//! View over a single slice of a cube (k-th matrix slice). Analogous to `SubMatrix` for matrices.

use super::cube_base::CubeBase;
use crate::matrix::Matrix;
use crate::types::Storage;
use std::fmt;

/// View over the k-th slice of a cube. Read/write delegates to the cube's (i, j, `slice_index`).
pub struct CubeSlice<'a, T> {
    base: &'a mut CubeBase<T>,
    slice_index: usize,
}

impl<'a, T> CubeSlice<'a, T> {
    pub fn new(base: &'a mut CubeBase<T>, slice_index: usize) -> Self {
        assert!(slice_index < base.slices());
        Self { base, slice_index }
    }

    #[inline]
    pub fn rows(&self) -> usize {
        self.base.rows()
    }

    #[inline]
    pub fn cols(&self) -> usize {
        self.base.cols()
    }

    #[inline]
    pub fn slice_index(&self) -> usize {
        self.slice_index
    }

    pub fn get(&self, i: usize, j: usize) -> T
    where
        T: Copy,
    {
        assert!(i < self.base.rows() && j < self.base.cols());
        self.base.get(i, j, self.slice_index)
    }

    pub fn set(&mut self, i: usize, j: usize, value: T) {
        assert!(i < self.base.rows() && j < self.base.cols());
        self.base.set(i, j, self.slice_index, value);
    }

    /// Copy this slice into a new matrix (column-major storage).
    pub fn to_matrix(&self) -> Matrix<T>
    where
        T: Copy + Default + From<u8>,
    {
        let mut m = Matrix::with_storage(self.rows(), self.cols(), Storage::Column);
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                m.set(i, j, self.get(i, j));
            }
        }
        m
    }
}

impl<T: Clone + Copy + Default + fmt::Display> fmt::Display for CubeSlice<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CubeSlice {}x{} (slice {})",
            self.rows(),
            self.cols(),
            self.slice_index
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_slice_get_set() {
        let mut base: CubeBase<f64> = CubeBase::with_dimensions(2, 3, 2);
        base.set(0, 1, 0, 5.0);
        base.set(1, 2, 1, 9.0);
        let mut view = CubeSlice::new(&mut base, 0);
        assert!((view.get(0, 1) - 5.0).abs() < 1e-10);
        view.set(1, 0, 7.0);
        assert!((base.get(1, 0, 0) - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_cube_slice_to_matrix() {
        let mut base: CubeBase<f64> = CubeBase::with_dimensions(2, 2, 1);
        base.set(0, 0, 0, 1.0);
        base.set(1, 0, 0, 2.0);
        base.set(0, 1, 0, 3.0);
        base.set(1, 1, 0, 4.0);
        let view = CubeSlice::new(&mut base, 0);
        let m = view.to_matrix();
        assert_eq!(m.rows(), 2);
        assert_eq!(m.cols(), 2);
        assert!((m.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((m.get(1, 1) - 4.0).abs() < 1e-10);
    }
}
