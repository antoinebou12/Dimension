//! Sparse 3rd-order tensor in COO (Coordinate) format.
//!
//! Stores nonzero entries as (i, j, k, val) quadruplets. Efficient for
//! construction and conversion to dense; get/set are O(nnz).

#![allow(clippy::cast_possible_truncation)]

use super::dense_storage::DenseStorageDynamic;
use crate::cube::Cube;
use std::fmt;

/// A sparse tensor entry: value at (i, j, k).
#[derive(Clone, Copy, Debug)]
pub struct Quadruplet<T> {
    /// The value.
    pub val: T,
    /// Row index.
    pub i: u32,
    /// Column index.
    pub j: u32,
    /// Slice index.
    pub k: u32,
}

impl<T> Quadruplet<T> {
    /// Creates a quadruplet (val, i, j, k).
    pub fn new(val: T, i: u32, j: u32, k: u32) -> Self {
        Self { val, i, j, k }
    }
}

impl<T: fmt::Display> fmt::Display for Quadruplet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {}, {})", self.val, self.i, self.j, self.k)
    }
}

/// Base storage for sparse cube COO format.
#[derive(Clone, Debug)]
pub struct SparseCubeBase<T> {
    pub(crate) vals: DenseStorageDynamic<T>,
    pub(crate) row_ind: DenseStorageDynamic<u32>,
    pub(crate) col_ind: DenseStorageDynamic<u32>,
    pub(crate) slice_ind: DenseStorageDynamic<u32>,
}

impl<T> SparseCubeBase<T> {
    /// Creates an empty sparse cube base.
    pub fn new() -> Self {
        Self {
            vals: DenseStorageDynamic::new(),
            row_ind: DenseStorageDynamic::new(),
            col_ind: DenseStorageDynamic::new(),
            slice_ind: DenseStorageDynamic::new(),
        }
    }

    /// Number of nonzero entries.
    #[inline]
    pub fn nnz(&self) -> usize {
        self.vals.size()
    }

    /// Values of nonzero entries.
    #[inline]
    pub fn values(&self) -> &[T] {
        self.vals.data()
    }

    /// Row indices.
    #[inline]
    pub fn row_indices(&self) -> &[u32] {
        self.row_ind.data()
    }

    /// Column indices.
    #[inline]
    pub fn col_indices(&self) -> &[u32] {
        self.col_ind.data()
    }

    /// Slice indices.
    #[inline]
    pub fn slice_indices(&self) -> &[u32] {
        self.slice_ind.data()
    }
}

impl<T> Default for SparseCubeBase<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Sparse 3rd-order tensor in COO format.
#[derive(Clone, Debug)]
pub struct SparseCube<T> {
    base: SparseCubeBase<T>,
    rows: usize,
    cols: usize,
    slices: usize,
}

impl<T> SparseCube<T> {
    /// Number of rows.
    #[inline]
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Number of columns.
    #[inline]
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Number of slices.
    #[inline]
    pub fn slices(&self) -> usize {
        self.slices
    }

    /// Number of nonzero entries.
    #[inline]
    pub fn nnz(&self) -> usize {
        self.base.nnz()
    }
}

impl<T: Default + Clone> SparseCube<T> {
    /// Creates an empty sparse cube.
    pub fn new() -> Self {
        Self {
            base: SparseCubeBase::new(),
            rows: 0,
            cols: 0,
            slices: 0,
        }
    }

    /// Creates a sparse cube with the given dimensions (initially empty).
    pub fn with_dimensions(rows: usize, cols: usize, slices: usize) -> Self {
        Self {
            base: SparseCubeBase::new(),
            rows,
            cols,
            slices,
        }
    }

    /// Get element at (i, j, k). Returns `T::default()` if not stored.
    pub fn get(&self, i: usize, j: usize, k: usize) -> T
    where
        T: Copy + Default,
    {
        assert!(i < self.rows && j < self.cols && k < self.slices);
        let ri = self.base.row_indices();
        let ci = self.base.col_indices();
        let si = self.base.slice_indices();
        let vals = self.base.values();
        for idx in 0..self.base.nnz() {
            if ri[idx] == i as u32 && ci[idx] == j as u32 && si[idx] == k as u32 {
                return vals[idx];
            }
        }
        T::default()
    }

    /// Insert or overwrite entry at (i, j, k). If an entry exists, updates it; otherwise appends.
    pub fn set(&mut self, i: usize, j: usize, k: usize, value: T)
    where
        T: Copy + Default + From<u8>,
    {
        assert!(i < self.rows && j < self.cols && k < self.slices);
        let ri = self.base.row_indices();
        let ci = self.base.col_indices();
        let si = self.base.slice_indices();
        for idx in 0..self.base.nnz() {
            if ri[idx] == i as u32 && ci[idx] == j as u32 && si[idx] == k as u32 {
                self.base.vals.data_mut()[idx] = value;
                return;
            }
        }
        self.push(Quadruplet::new(value, i as u32, j as u32, k as u32));
    }

    /// Appends a nonzero entry. Duplicates are allowed (caller should merge if needed).
    pub fn push(&mut self, q: Quadruplet<T>)
    where
        T: Copy + Default + From<u8>,
    {
        assert!(q.i < self.rows as u32 && q.j < self.cols as u32 && q.k < self.slices as u32);
        let n = self.base.nnz();
        self.base.vals.resize(n + 1);
        self.base.row_ind.resize(n + 1);
        self.base.col_ind.resize(n + 1);
        self.base.slice_ind.resize(n + 1);
        self.base.vals.data_mut()[n] = q.val;
        self.base.row_ind.data_mut()[n] = q.i;
        self.base.col_ind.data_mut()[n] = q.j;
        self.base.slice_ind.data_mut()[n] = q.k;
    }

    /// Build from quadruplets. Duplicate (i,j,k) entries are overwritten (last wins).
    pub fn from_quadruplets(
        rows: usize,
        cols: usize,
        slices: usize,
        quadruplets: &[Quadruplet<T>],
    ) -> Self
    where
        T: Copy + Default + From<u8>,
    {
        let mut cube = Self::with_dimensions(rows, cols, slices);
        for &q in quadruplets {
            cube.set(q.i as usize, q.j as usize, q.k as usize, q.val);
        }
        cube
    }

    /// Convert to dense `Cube<T>`.
    pub fn to_dense(&self) -> Cube<T>
    where
        T: Copy + Default + From<u8>,
    {
        let mut c = Cube::with_dimensions(self.rows, self.cols, self.slices);
        c.set_zero();
        let ri = self.base.row_indices();
        let ci = self.base.col_indices();
        let si = self.base.slice_indices();
        let vals = self.base.values();
        for (idx, &v) in vals.iter().enumerate() {
            let i = ri[idx] as usize;
            let j = ci[idx] as usize;
            let k = si[idx] as usize;
            c.set(i, j, k, v);
        }
        c
    }

    /// Convert to quadruplets (copies entries).
    pub fn to_quadruplets(&self) -> Vec<Quadruplet<T>>
    where
        T: Copy,
    {
        let ri = self.base.row_indices();
        let ci = self.base.col_indices();
        let si = self.base.slice_indices();
        let vals = self.base.values();
        ri.iter()
            .zip(ci.iter())
            .zip(si.iter())
            .zip(vals.iter())
            .map(|(((i, j), k), v)| Quadruplet::new(*v, *i, *j, *k))
            .collect()
    }
}

impl<T> Default for SparseCube<T>
where
    T: Default + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: fmt::Display> fmt::Display for SparseCube<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SparseCube {}x{}x{} nnz={}",
            self.rows,
            self.cols,
            self.slices,
            self.nnz()
        )
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for SparseCube<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("SparseCube", 5)?;
        s.serialize_field("rows", &self.rows)?;
        s.serialize_field("cols", &self.cols)?;
        s.serialize_field("slices", &self.slices)?;
        s.serialize_field("vals", self.base.values())?;
        s.serialize_field(
            "indices",
            &(
                self.base.row_indices().to_vec(),
                self.base.col_indices().to_vec(),
                self.base.slice_indices().to_vec(),
            ),
        )?;
        s.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de> + Copy + Default + From<u8>> serde::Deserialize<'de>
    for SparseCube<T>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct SparseCubeDe<T> {
            rows: usize,
            cols: usize,
            slices: usize,
            vals: Vec<T>,
            indices: (Vec<u32>, Vec<u32>, Vec<u32>),
        }
        let d = SparseCubeDe::<T>::deserialize(deserializer)?;
        let quadruplets: Vec<Quadruplet<T>> = d
            .vals
            .iter()
            .zip(d.indices.0.iter())
            .zip(d.indices.1.iter())
            .zip(d.indices.2.iter())
            .map(|(((v, i), j), k)| Quadruplet::new(*v, *i, *j, *k))
            .collect();
        Ok(Self::from_quadruplets(
            d.rows,
            d.cols,
            d.slices,
            &quadruplets,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparse_cube_new() {
        let c: SparseCube<f64> = SparseCube::new();
        assert_eq!(c.rows(), 0);
        assert_eq!(c.cols(), 0);
        assert_eq!(c.slices(), 0);
        assert_eq!(c.nnz(), 0);
    }

    #[test]
    fn test_sparse_cube_with_dimensions() {
        let c: SparseCube<f64> = SparseCube::with_dimensions(2, 3, 4);
        assert_eq!(c.rows(), 2);
        assert_eq!(c.cols(), 3);
        assert_eq!(c.slices(), 4);
        assert_eq!(c.nnz(), 0);
    }

    #[test]
    fn test_sparse_cube_from_quadruplets() {
        let quads = vec![
            Quadruplet::new(1.0_f64, 0, 0, 0),
            Quadruplet::new(2.0_f64, 0, 1, 0),
            Quadruplet::new(3.0_f64, 1, 1, 1),
        ];
        let c = SparseCube::from_quadruplets(2, 2, 2, &quads);
        assert_eq!(c.nnz(), 3);
        assert!((c.get(0, 0, 0) - 1.0).abs() < 1e-10);
        assert!((c.get(0, 1, 0) - 2.0).abs() < 1e-10);
        assert!((c.get(1, 1, 1) - 3.0).abs() < 1e-10);
        assert!((c.get(0, 1, 1) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_sparse_cube_set() {
        let mut c: SparseCube<f64> = SparseCube::with_dimensions(2, 2, 2);
        c.set(0, 1, 0, 5.0);
        assert!((c.get(0, 1, 0) - 5.0).abs() < 1e-10);
        c.set(0, 1, 0, 7.0);
        assert!((c.get(0, 1, 0) - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_sparse_cube_to_dense() {
        let quads = vec![
            Quadruplet::new(1.0_f64, 0, 0, 0),
            Quadruplet::new(2.0_f64, 1, 1, 0),
        ];
        let sparse = SparseCube::from_quadruplets(2, 2, 1, &quads);
        let dense = sparse.to_dense();
        assert_eq!(dense.rows(), 2);
        assert_eq!(dense.cols(), 2);
        assert_eq!(dense.slices(), 1);
        assert!((dense.get(0, 0, 0) - 1.0).abs() < 1e-10);
        assert!((dense.get(1, 1, 0) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_sparse_cube_to_quadruplets() {
        let quads = vec![Quadruplet::new(3.0_f64, 1, 0, 1)];
        let c = SparseCube::from_quadruplets(2, 2, 2, &quads);
        let out = c.to_quadruplets();
        assert_eq!(out.len(), 1);
        assert!((out[0].val - 3.0).abs() < 1e-10);
        assert_eq!(out[0].i, 1);
        assert_eq!(out[0].j, 0);
        assert_eq!(out[0].k, 1);
    }
}
