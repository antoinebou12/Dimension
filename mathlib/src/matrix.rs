//! Dense matrix type with column-major or row-major storage.
//!
//! The [`Matrix<T>`] type is the primary dense matrix representation. It supports:
//! - Column-major (default) or row-major storage via [`Storage`]
//! - Element access with `get(i, j)` / `set(i, j, value)`
//! - Transpose, identity, zero initialization
//! - Block views via [`SubMatrix`]
//! - Arithmetic operations (`+`, `-`, `*`) via [`crate::operators`]
//!
//! # Storage Layout
//!
//! - **Column-major** (`Storage::Column`): Elements stored column-by-column.
//!   Index `(i, j)` maps to `j * rows + i`. This is the default and matches OpenGL conventions.
//! - **Row-major** (`Storage::Row`): Elements stored row-by-row.
//!   Index `(i, j)` maps to `i * cols + j`.
//!
//! # Example
//!
//! ```
//! use mathlib::{Matrix, Storage};
//!
//! let mut m = Matrix::<f64>::with_storage(3, 3, Storage::Column);
//! m.set_identity();
//! assert_eq!(m.get(0, 0), 1.0);
//! assert_eq!(m.get(0, 1), 0.0);
//!
//! let t = m.transpose();
//! assert_eq!(t.rows(), 3);
//! ```

use crate::matrix_base::MatrixBase;
use crate::submatrix::SubMatrix;
use crate::types::Storage;
use std::fmt;
use std::ops::{AddAssign, Index, IndexMut, Mul};

#[derive(Clone, Debug)]
pub struct Matrix<T> {
    pub(crate) base: MatrixBase<T>,
    pub(crate) storage: Storage,
}

impl<T> Matrix<T> {
    #[inline]
    pub fn rows(&self) -> usize {
        self.base.rows()
    }

    #[inline]
    pub fn cols(&self) -> usize {
        self.base.cols()
    }

    /// Storage order: column-major or row-major.
    #[inline]
    pub fn storage(&self) -> Storage {
        self.storage
    }

    #[inline]
    pub fn data(&self) -> &[T] {
        self.base.data()
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut [T] {
        self.base.data_mut()
    }

    #[inline]
    fn col_index(&self, i: usize, j: usize) -> usize {
        debug_assert!(i < self.base.rows() && j < self.base.cols());
        j * self.base.rows() + i
    }

    #[inline]
    fn row_index(&self, i: usize, j: usize) -> usize {
        debug_assert!(i < self.base.rows() && j < self.base.cols());
        i * self.base.cols() + j
    }

    /// Element at row `i`, column `j`.
    ///
    /// # Panics
    ///
    /// In debug builds, panics if `i >= rows()` or `j >= cols()`.
    #[inline]
    pub fn get(&self, i: usize, j: usize) -> T
    where
        T: Copy,
    {
        let idx = match self.storage {
            Storage::Column => self.col_index(i, j),
            Storage::Row => self.row_index(i, j),
        };
        self.base.data()[idx]
    }

    /// Set element at row `i`, column `j`.
    ///
    /// # Panics
    ///
    /// In debug builds, panics if `i >= rows()` or `j >= cols()`.
    #[inline]
    pub fn set(&mut self, i: usize, j: usize, value: T) {
        let idx = match self.storage {
            Storage::Column => self.col_index(i, j),
            Storage::Row => self.row_index(i, j),
        };
        self.base.data_mut()[idx] = value;
    }

    /// Contiguous slice of elements in storage order (column-major when `Storage::Column`).
    /// For 4×4 column-major matrices this matches OpenGL/convention for shaders.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.base.data()
    }
}

impl<T: Clone + Default> Matrix<T> {
    pub fn new() -> Self {
        Self {
            base: MatrixBase::new(),
            storage: Storage::Column,
        }
    }

    pub fn with_dimensions(rows: usize, cols: usize) -> Self
    where
        T: Default,
    {
        Self {
            base: MatrixBase::with_dimensions(rows, cols),
            storage: Storage::Column,
        }
    }

    pub fn with_storage(rows: usize, cols: usize, storage: Storage) -> Self
    where
        T: Default,
    {
        Self {
            base: MatrixBase::with_dimensions(rows, cols),
            storage,
        }
    }

    /// Creates a matrix from a contiguous slice in column-major order.
    /// `data.len()` must equal `rows * cols`.
    pub fn from_vec(data: &[T], rows: usize, cols: usize, storage: Storage) -> Self
    where
        T: Copy,
    {
        assert_eq!(data.len(), rows * cols);
        Self {
            base: MatrixBase {
                storage: crate::structure::DenseStorageDynamic::from_slice(data),
                rows,
                cols,
            },
            storage,
        }
    }

    pub fn resize(&mut self, rows: usize, cols: usize)
    where
        T: Default,
    {
        self.base.resize(rows, cols);
    }

    pub fn set_zero(&mut self)
    where
        T: Copy + From<u8>,
    {
        self.base.set_zero();
    }

    pub fn set_identity(&mut self)
    where
        T: Copy + From<u8> + PartialEq,
    {
        let (r, c) = (self.base.rows(), self.base.cols());
        self.set_zero();
        let min_dim = r.min(c);
        for i in 0..min_dim {
            self[(i, i)] = T::from(1_u8);
        }
    }

    pub fn block(&mut self, i: usize, j: usize, rows: usize, cols: usize) -> SubMatrix<'_, T> {
        SubMatrix::new(self, i, j, rows, cols)
    }
}

impl<T: Copy> Matrix<T> {
    /// Copy elements from `src` into `self`. Dimensions must match.
    pub fn copy_from(&mut self, src: &Matrix<T>) {
        assert_eq!(self.rows(), src.rows());
        assert_eq!(self.cols(), src.cols());
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                self.set(i, j, src.get(i, j));
            }
        }
    }
}

impl<T: Copy + Default + From<u8> + AddAssign + Mul<Output = T>> Matrix<T> {
    /// Compute `self * other` and write into `out`. `out` must have size `self.rows() x other.cols()`.
    pub fn mul_into(&self, other: &Matrix<T>, out: &mut Matrix<T>) {
        assert_eq!(self.cols(), other.rows());
        assert_eq!(out.rows(), self.rows());
        assert_eq!(out.cols(), other.cols());
        out.set_zero();
        for i in 0..self.rows() {
            for j in 0..other.cols() {
                let mut sum = T::from(0);
                for k in 0..self.cols() {
                    sum += self.get(i, k) * other.get(k, j);
                }
                out.set(i, j, sum);
            }
        }
    }
}

impl<T: Copy + Clone + Default + From<u8> + PartialEq + AddAssign + Mul<Output = T>> Matrix<T> {
    /// Raise this square matrix to the power `exp` in-place. Panics if not square.
    pub fn pow_mut(&mut self, mut exp: u32) {
        assert_eq!(self.rows(), self.cols(), "pow_mut requires a square matrix");
        if exp == 0 {
            self.set_identity();
        } else if exp > 1 {
            let mut x = self.clone();
            let mut workspace = self.clone();

            if exp.is_multiple_of(2) {
                self.set_identity();
            } else {
                exp -= 1;
            }

            loop {
                if exp % 2 == 1 {
                    self.mul_into(&x, &mut workspace);
                    self.copy_from(&workspace);
                }

                exp /= 2;

                if exp == 0 {
                    break;
                }

                x.mul_into(&x, &mut workspace);
                x.copy_from(&workspace);
            }
        }
    }

    /// Raise this square matrix to the power `exp`. Panics if not square.
    #[must_use]
    pub fn pow(&self, exp: u32) -> Matrix<T> {
        let mut result = self.clone();
        result.pow_mut(exp);
        result
    }
}

impl<T: Copy> Index<(usize, usize)> for Matrix<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (i, j) = index;
        let idx = match self.storage {
            Storage::Column => self.col_index(i, j),
            Storage::Row => self.row_index(i, j),
        };
        &self.base.data()[idx]
    }
}

impl<T: Copy> IndexMut<(usize, usize)> for Matrix<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (i, j) = index;
        let idx = match self.storage {
            Storage::Column => self.col_index(i, j),
            Storage::Row => self.row_index(i, j),
        };
        &mut self.base.data_mut()[idx]
    }
}

impl<T: Copy + Default + From<u8>> Matrix<T> {
    #[must_use = "this returns a new matrix and does not modify the input"]
    pub fn transpose(&self) -> Matrix<T> {
        let (r, c) = (self.rows(), self.cols());
        let opposite = match self.storage {
            Storage::Column => Storage::Row,
            Storage::Row => Storage::Column,
        };
        let mut out = Matrix::with_storage(c, r, opposite);
        for i in 0..r {
            for j in 0..c {
                out.set(j, i, self.get(i, j));
            }
        }
        out
    }
}

impl<T: Copy + fmt::Display> fmt::Display for Matrix<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (r, c) = (self.rows(), self.cols());
        write!(f, "{} {}x{}[", self.storage, r, c)?;
        for i in 0..r {
            if i > 0 {
                write!(f, "; ")?;
            }
            for j in 0..c {
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
impl<T: serde::Serialize> serde::Serialize for Matrix<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Matrix", 4)?;
        s.serialize_field("storage", &self.storage)?;
        s.serialize_field("rows", &self.base.rows)?;
        s.serialize_field("cols", &self.base.cols)?;
        s.serialize_field("data", self.base.data())?;
        s.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Matrix<T>
where
    T: Clone + Copy + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Deserialize, MapAccess, Visitor};
        use std::marker::PhantomData;
        struct MatrixVisitor<T>(PhantomData<T>);
        impl<'de, T: Deserialize<'de> + Clone + Copy + Default> Visitor<'de> for MatrixVisitor<T> {
            type Value = Matrix<T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct Matrix with storage, rows, cols, data")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Matrix<T>, A::Error> {
                let mut storage = None;
                let mut rows = None;
                let mut cols = None;
                let mut data = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "storage" => storage = Some(map.next_value()?),
                        "rows" => rows = Some(map.next_value()?),
                        "cols" => cols = Some(map.next_value()?),
                        "data" => data = Some(map.next_value()?),
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                let storage: Storage =
                    storage.ok_or_else(|| serde::de::Error::missing_field("storage"))?;
                let rows: usize = rows.ok_or_else(|| serde::de::Error::missing_field("rows"))?;
                let cols: usize = cols.ok_or_else(|| serde::de::Error::missing_field("cols"))?;
                let data: Vec<T> = data.ok_or_else(|| serde::de::Error::missing_field("data"))?;
                let base = MatrixBase {
                    storage: crate::structure::DenseStorageDynamic::from_slice(&data),
                    rows,
                    cols,
                };
                Ok(Matrix { base, storage })
            }
        }
        deserializer.deserialize_struct(
            "Matrix",
            &["storage", "rows", "cols", "data"],
            MatrixVisitor::<T>(PhantomData),
        )
    }
}
