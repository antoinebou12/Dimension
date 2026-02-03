//! Cube: 3rd-order tensor (quasi-3D matrix). Data stored as contiguous slices;
//! within each slice elements are column-major (Armadillo-compatible).
//!
//! Use `Cube<f64>` (cube/dcube) or `Cube<f32>` (fcube) for typical types.

use crate::structure::{CubeBase, CubeSlice};
use crate::types::Fill;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Cube<T> {
    pub(crate) base: CubeBase<T>,
}

impl<T> Cube<T> {
    #[inline]
    pub fn rows(&self) -> usize {
        self.base.rows()
    }

    #[inline]
    pub fn cols(&self) -> usize {
        self.base.cols()
    }

    #[inline]
    pub fn slices(&self) -> usize {
        self.base.slices()
    }

    #[inline]
    pub fn data(&self) -> &[T] {
        self.base.data()
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut [T] {
        self.base.data_mut()
    }

    pub fn get(&self, i: usize, j: usize, k: usize) -> T
    where
        T: Copy,
    {
        self.base.get(i, j, k)
    }

    pub fn set(&mut self, i: usize, j: usize, k: usize, value: T) {
        self.base.set(i, j, k, value);
    }

    /// View over the k-th slice (matrix). Each slice is rows×cols, column-major.
    pub fn slice(&mut self, k: usize) -> CubeSlice<'_, T> {
        CubeSlice::new(&mut self.base, k)
    }
}

impl<T: Clone + Default> Cube<T> {
    pub fn new() -> Self {
        Self {
            base: CubeBase::new(),
        }
    }

    pub fn with_dimensions(rows: usize, cols: usize, slices: usize) -> Self
    where
        T: Default,
    {
        Self {
            base: CubeBase::with_dimensions(rows, cols, slices),
        }
    }

    /// Construct with dimensions and initial fill (Zeros, Ones, or None).
    pub fn with_dimensions_fill(rows: usize, cols: usize, slices: usize, fill: Fill) -> Self
    where
        T: Copy + Default + From<u8>,
    {
        Self {
            base: CubeBase::with_dimensions_fill(rows, cols, slices, fill),
        }
    }

    pub fn resize(&mut self, rows: usize, cols: usize, slices: usize)
    where
        T: Default,
    {
        self.base.resize(rows, cols, slices);
    }

    pub fn set_zero(&mut self)
    where
        T: Copy + From<u8>,
    {
        self.base.set_zero();
    }
}

impl<T> Default for Cube<T> {
    fn default() -> Self {
        Self {
            base: CubeBase::new(),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Cube<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.base.fmt(f)
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for Cube<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.base.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Cube<T>
where
    T: Clone + Copy + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let base = CubeBase::deserialize(deserializer)?;
        Ok(Cube { base })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_new() {
        let c: Cube<f64> = Cube::new();
        assert_eq!(c.rows(), 0);
        assert_eq!(c.cols(), 0);
        assert_eq!(c.slices(), 0);
    }

    #[test]
    fn test_cube_with_dimensions() {
        let c: Cube<f64> = Cube::with_dimensions(2, 3, 4);
        assert_eq!(c.rows(), 2);
        assert_eq!(c.cols(), 3);
        assert_eq!(c.slices(), 4);
    }

    #[test]
    fn test_cube_get_set() {
        let mut c: Cube<f64> = Cube::with_dimensions(2, 2, 2);
        c.set(0, 0, 0, 1.0);
        c.set(1, 1, 1, 8.0);
        assert!((c.get(0, 0, 0) - 1.0).abs() < 1e-10);
        assert!((c.get(1, 1, 1) - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_cube_slice_view() {
        let mut c: Cube<f64> = Cube::with_dimensions(2, 2, 2);
        c.set(0, 1, 0, 5.0);
        let mut view = c.slice(0);
        assert!((view.get(0, 1) - 5.0).abs() < 1e-10);
        view.set(1, 0, 7.0);
        assert!((c.get(1, 0, 0) - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_cube_set_zero() {
        let mut c: Cube<f64> = Cube::with_dimensions(2, 2, 2);
        c.set(0, 0, 0, 3.0);
        c.set_zero();
        assert!((c.get(0, 0, 0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_cube_resize() {
        let mut c: Cube<f64> = Cube::with_dimensions(2, 2, 2);
        c.resize(3, 4, 2);
        assert_eq!(c.rows(), 3);
        assert_eq!(c.cols(), 4);
        assert_eq!(c.slices(), 2);
    }
}
