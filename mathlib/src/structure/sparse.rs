#![allow(clippy::cast_possible_truncation)]

use super::dense_storage::DenseStorageDynamic;
use super::types::Triplet;
use crate::vector::Vector;
use std::fmt;
use std::ops::Mul;

pub trait SparseStorage<T> {
    fn from_triplets(rows: usize, cols: usize, triplets: &[Triplet<T>]) -> Self
    where
        T: Copy + Default;

    fn get(&self, i: usize, j: usize) -> T
    where
        T: Copy + Default;

    fn rows(&self) -> usize;

    fn cols(&self) -> usize;

    fn nnz(&self) -> usize;

    fn mul_vector(&self, x: &Vector<T>) -> Vector<T>
    where
        T: Copy + Default + std::ops::AddAssign + Mul<Output = T>;

    fn mul_vector_transpose(&self, x: &Vector<T>) -> Vector<T>
    where
        T: Copy + Default + std::ops::AddAssign + Mul<Output = T>;
}

#[derive(Clone, Debug)]
pub struct SparseMatrixBase<T> {
    pub(crate) vals: DenseStorageDynamic<T>,
    pub(crate) inner: DenseStorageDynamic<u32>,
    pub(crate) start: DenseStorageDynamic<u32>,
}

impl<T: Default> SparseMatrixBase<T> {
    pub fn new() -> Self {
        Self {
            vals: DenseStorageDynamic::new(),
            inner: DenseStorageDynamic::new(),
            start: DenseStorageDynamic::new(),
        }
    }
}

impl<T: Default + Clone> SparseMatrixBase<T> {
    pub fn with_capacity(outer_size: usize, nnz: usize) -> Self {
        let mut start = DenseStorageDynamic::with_capacity(outer_size + 1);
        start.resize(outer_size + 1);
        let mut vals = DenseStorageDynamic::with_capacity(nnz);
        vals.resize(nnz);
        let mut inner = DenseStorageDynamic::with_capacity(nnz);
        inner.resize(nnz);
        Self { vals, inner, start }
    }

    pub fn set_inner_size(&mut self, nnz: usize) {
        self.vals.resize(nnz);
        self.inner.resize(nnz);
    }

    pub fn set_outer_size(&mut self, outer_size: usize) {
        self.start.resize(outer_size + 1);
    }

    pub fn set_zero(&mut self)
    where
        T: Copy + From<u8>,
    {
        self.vals.set_zero();
        self.inner.resize(0);
        self.start.set_zero();
    }

    pub fn get_inner_size(&self) -> usize {
        self.inner.size()
    }

    pub fn get_outer_size(&self) -> usize {
        self.start.size().saturating_sub(1)
    }

    pub fn values(&self) -> &[T] {
        self.vals.data()
    }

    pub fn outer(&self) -> &[u32] {
        self.start.data()
    }

    pub fn inner(&self) -> &[u32] {
        self.inner.data()
    }
}

impl<T: fmt::Display> fmt::Display for SparseMatrixBase<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SparseMatrixBase vals={} inner={} start={}",
            self.vals,
            self.inner.size(),
            self.start.size()
        )
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for SparseMatrixBase<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("SparseMatrixBase", 3)?;
        s.serialize_field("vals", self.vals.data())?;
        s.serialize_field("inner", self.inner.data())?;
        s.serialize_field("start", self.start.data())?;
        s.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for SparseMatrixBase<T>
where
    T: Copy + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct SparseBaseDe<T> {
            vals: Vec<T>,
            inner: Vec<u32>,
            start: Vec<u32>,
        }
        let d = SparseBaseDe::<T>::deserialize(deserializer)?;
        Ok(Self {
            vals: DenseStorageDynamic::from_slice(&d.vals),
            inner: DenseStorageDynamic::from_slice(&d.inner),
            start: DenseStorageDynamic::from_slice(&d.start),
        })
    }
}

#[derive(Clone, Debug)]
pub struct SparseMatrixCRS<T> {
    base: SparseMatrixBase<T>,
    rows: usize,
    cols: usize,
}

pub type SparseMatrix<T> = SparseMatrixCRS<T>;

impl<T: Default + Clone> SparseMatrixCRS<T> {
    pub fn new() -> Self {
        Self {
            base: SparseMatrixBase::new(),
            rows: 0,
            cols: 0,
        }
    }

    pub fn with_dimensions(rows: usize, cols: usize) -> Self
    where
        T: Default,
    {
        let mut base = SparseMatrixBase::with_capacity(rows, 0);
        base.set_outer_size(rows);
        Self { base, rows, cols }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Row pointer array (CRS format). Length `rows + 1`; row `i` has nonzeros in `[row_ptr[i], row_ptr[i+1])`.
    pub fn row_ptr(&self) -> &[u32] {
        self.base.outer()
    }

    /// Column indices for each nonzero (CRS format).
    pub fn col_ind(&self) -> &[u32] {
        self.base.inner()
    }

    /// Nonzero values (CRS format).
    pub fn values(&self) -> &[T] {
        self.base.values()
    }

    pub fn get(&self, i: usize, j: usize) -> T
    where
        T: Copy + Default,
    {
        assert!(i < self.rows && j < self.cols);
        let start = self.base.outer();
        let count = (start[i + 1] - start[i]) as usize;
        for k in 0..count {
            let idx = start[i] as usize + k;
            if self.base.inner()[idx] == j as u32 {
                return self.base.values()[idx];
            }
        }
        T::default()
    }

    pub fn set_identity(&mut self)
    where
        T: Copy + Default + From<u8> + From<i32>,
    {
        assert_eq!(self.rows, self.cols);
        self.base.set_zero();
        self.base.set_inner_size(self.rows);
        self.base.set_outer_size(self.rows);
        for i in 0..self.rows {
            self.base.start.data_mut()[i] = i as u32;
            self.base.start.data_mut()[i + 1] = (i + 1) as u32;
            self.base.inner.data_mut()[i] = i as u32;
            self.base.vals.data_mut()[i] = T::from(1);
        }
    }

    pub fn set_from_triplets(&mut self, triplets: &[Triplet<T>])
    where
        T: Copy + Default + From<u8>,
    {
        let n = self.rows;
        let mut count: Vec<u32> = vec![0; n];
        for t in triplets {
            assert!(t.i < n as u32 && t.j < self.cols as u32);
            count[t.i as usize] += 1;
        }
        let mut start = vec![0u32; n + 1];
        for i in 0..n {
            start[i + 1] = start[i] + count[i];
        }
        let nnz = triplets.len();
        self.base.set_inner_size(nnz);
        self.base.set_outer_size(n);
        for (idx, &s) in start.iter().enumerate().take(n + 1) {
            self.base.start.data_mut()[idx] = s;
        }
        count.fill(0);
        for t in triplets {
            let i = t.i as usize;
            let offset = (start[i] + count[i]) as usize;
            self.base.inner.data_mut()[offset] = t.j;
            self.base.vals.data_mut()[offset] = t.val;
            count[i] += 1;
        }
    }

    pub fn transpose(&self) -> SparseMatrixCRS<T>
    where
        T: Copy + Default + From<u8>,
    {
        let mut t = SparseMatrixCRS::with_dimensions(self.cols, self.rows);
        for j in 0..self.cols {
            for i in 0..self.rows {
                let _v = self.get(i, j);
            }
        }
        let nnz = self.base.get_inner_size();
        let mut triplets: Vec<Triplet<T>> = Vec::with_capacity(nnz);
        let start = self.base.outer();
        let inner = self.base.inner();
        let vals = self.base.values();
        for i in 0..self.rows {
            for k in (start[i] as usize)..(start[i + 1] as usize) {
                let j = inner[k];
                triplets.push(Triplet::new(vals[k], j, i as u32));
            }
        }
        t.set_from_triplets(&triplets);
        t
    }
}

impl<T: fmt::Display> fmt::Display for SparseMatrixCRS<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SparseMatrixCRS {}x{} ", self.rows, self.cols)?;
        self.base.fmt(f)
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for SparseMatrixCRS<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("SparseMatrixCRS", 3)?;
        s.serialize_field("rows", &self.rows)?;
        s.serialize_field("cols", &self.cols)?;
        s.serialize_field("base", &self.base)?;
        s.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for SparseMatrixCRS<T>
where
    T: Copy + Default + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Deserialize, MapAccess, Visitor};
        use std::marker::PhantomData;
        struct SparseCRSVisitor<T>(PhantomData<T>);
        impl<'de, T: Deserialize<'de> + Copy + Default + Clone> Visitor<'de> for SparseCRSVisitor<T> {
            type Value = SparseMatrixCRS<T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct SparseMatrixCRS with rows, cols, base")
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<SparseMatrixCRS<T>, A::Error> {
                let mut rows = None;
                let mut cols = None;
                let mut base = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "rows" => rows = Some(map.next_value()?),
                        "cols" => cols = Some(map.next_value()?),
                        "base" => base = Some(map.next_value()?),
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                let rows: usize = rows.ok_or_else(|| serde::de::Error::missing_field("rows"))?;
                let cols: usize = cols.ok_or_else(|| serde::de::Error::missing_field("cols"))?;
                let base: SparseMatrixBase<T> =
                    base.ok_or_else(|| serde::de::Error::missing_field("base"))?;
                Ok(SparseMatrixCRS { base, rows, cols })
            }
        }
        deserializer.deserialize_struct(
            "SparseMatrixCRS",
            &["rows", "cols", "base"],
            SparseCRSVisitor::<T>(PhantomData),
        )
    }
}

impl<
    T: Copy
        + Default
        + From<u8>
        + std::ops::Add<Output = T>
        + std::ops::AddAssign
        + std::ops::Add
        + Mul<Output = T>
        + 'static,
> Mul<&Vector<T>> for &SparseMatrixCRS<T>
{
    type Output = Vector<T>;

    fn mul(self, v: &Vector<T>) -> Vector<T> {
        assert!(self.cols() == v.rows());
        #[cfg(feature = "gpu")]
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>() {
            let a: &SparseMatrixCRS<f32> =
                unsafe { &*(self as *const SparseMatrixCRS<T> as *const SparseMatrixCRS<f32>) };
            let b: &Vector<f32> = unsafe { &*(v as *const Vector<T> as *const Vector<f32>) };
            if let Some(gpu_out) = crate::gpu::try_spmv_f32(a, b) {
                return unsafe { std::mem::transmute(gpu_out) };
            }
        }
        let mut out = Vector::with_capacity(self.rows());
        out.set_zero();
        let start = self.base.outer();
        let inner = self.base.inner();
        let vals = self.base.values();
        for i in 0..self.rows {
            for k in (start[i] as usize)..(start[i + 1] as usize) {
                let j = inner[k] as usize;
                out.set(i, out.get(i) + vals[k] * v.get(j));
            }
        }
        out
    }
}

impl<T> SparseStorage<T> for SparseMatrixCRS<T>
where
    T: Copy
        + Default
        + From<u8>
        + std::ops::Add<Output = T>
        + std::ops::AddAssign
        + Mul<Output = T>
        + 'static,
{
    fn from_triplets(rows: usize, cols: usize, triplets: &[Triplet<T>]) -> Self {
        let mut mat = SparseMatrixCRS::with_dimensions(rows, cols);
        mat.set_from_triplets(triplets);
        mat
    }

    fn get(&self, i: usize, j: usize) -> T {
        SparseMatrixCRS::get(self, i, j)
    }

    fn rows(&self) -> usize {
        self.rows
    }

    fn cols(&self) -> usize {
        self.cols
    }

    fn nnz(&self) -> usize {
        self.base.get_inner_size()
    }

    fn mul_vector(&self, x: &Vector<T>) -> Vector<T> {
        self * x
    }

    fn mul_vector_transpose(&self, x: &Vector<T>) -> Vector<T> {
        assert!(self.rows() == x.rows());
        let mut out = Vector::with_capacity(self.cols());
        out.set_zero();
        let start = self.base.outer();
        let inner = self.base.inner();
        let vals = self.base.values();
        for j in 0..self.rows {
            for k in (start[j] as usize)..(start[j + 1] as usize) {
                let col_idx = inner[k] as usize;
                out.set(col_idx, out.get(col_idx) + vals[k] * x.get(j));
            }
        }
        out
    }
}

impl<T> SparseMatrixCRS<T>
where
    T: Copy
        + Default
        + From<u8>
        + std::ops::Add<Output = T>
        + std::ops::AddAssign
        + Mul<Output = T>,
{
    /// Convert to triplets representation.
    pub fn to_triplets(&self) -> Vec<Triplet<T>> {
        let nnz = self.base.get_inner_size();
        let mut triplets = Vec::with_capacity(nnz);
        let start = self.base.outer();
        let inner = self.base.inner();
        let vals = self.base.values();
        for i in 0..self.rows {
            for k in (start[i] as usize)..(start[i + 1] as usize) {
                let j = inner[k];
                triplets.push(Triplet::new(vals[k], i as u32, j));
            }
        }
        triplets
    }

    pub fn from_sparse<S: SparseStorage<T>>(other: &S) -> Self
    where
        T: Copy + Default + 'static,
    {
        let cap = other.rows() * other.cols();
        let mut triplets = Vec::with_capacity(cap);
        for i in 0..other.rows() {
            for j in 0..other.cols() {
                let val = other.get(i, j);
                triplets.push(Triplet::new(val, i as u32, j as u32));
            }
        }
        SparseMatrixCRS::from_triplets(other.rows(), other.cols(), &triplets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vector(data: &[f64]) -> Vector<f64> {
        let mut v = Vector::with_capacity(data.len());
        for (i, &val) in data.iter().enumerate() {
            v.set(i, val);
        }
        v
    }

    #[test]
    fn test_sparse_matrix_base_new() {
        let base: SparseMatrixBase<f64> = SparseMatrixBase::new();
        assert_eq!(base.get_inner_size(), 0);
        assert_eq!(base.get_outer_size(), 0);
    }

    #[test]
    fn test_sparse_matrix_base_with_capacity() {
        let base: SparseMatrixBase<f64> = SparseMatrixBase::with_capacity(5, 10);
        assert_eq!(base.get_outer_size(), 5);
        assert_eq!(base.get_inner_size(), 10);
    }

    #[test]
    fn test_sparse_crs_new() {
        let m: SparseMatrixCRS<f64> = SparseMatrixCRS::new();
        assert_eq!(m.rows(), 0);
        assert_eq!(m.cols(), 0);
    }

    #[test]
    fn test_sparse_crs_with_dimensions() {
        let m: SparseMatrixCRS<f64> = SparseMatrixCRS::with_dimensions(3, 4);
        assert_eq!(m.rows(), 3);
        assert_eq!(m.cols(), 4);
    }

    #[test]
    fn test_sparse_crs_from_triplets() {
        let triplets = vec![
            Triplet::new(1.0_f64, 0, 0),
            Triplet::new(2.0_f64, 0, 2),
            Triplet::new(3.0_f64, 1, 1),
            Triplet::new(4.0_f64, 2, 0),
        ];
        let m = SparseMatrixCRS::from_triplets(3, 3, &triplets);
        assert!((m.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((m.get(0, 2) - 2.0).abs() < 1e-10);
        assert!((m.get(1, 1) - 3.0).abs() < 1e-10);
        assert!((m.get(2, 0) - 4.0).abs() < 1e-10);
        assert!((m.get(0, 1) - 0.0).abs() < 1e-10);
        assert!((m.get(1, 0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_sparse_crs_identity() {
        let mut m: SparseMatrixCRS<f64> = SparseMatrixCRS::with_dimensions(3, 3);
        m.set_identity();
        assert!((m.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((m.get(1, 1) - 1.0).abs() < 1e-10);
        assert!((m.get(2, 2) - 1.0).abs() < 1e-10);
        assert!((m.get(0, 1) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_sparse_crs_mul_vector() {
        let triplets = vec![
            Triplet::new(1.0_f64, 0, 0),
            Triplet::new(2.0_f64, 0, 1),
            Triplet::new(3.0_f64, 1, 0),
            Triplet::new(4.0_f64, 1, 1),
        ];
        let m = SparseMatrixCRS::from_triplets(2, 2, &triplets);
        let v = make_vector(&[1.0, 2.0]);
        let result = &m * &v;
        assert!((result.get(0) - 5.0).abs() < 1e-10);
        assert!((result.get(1) - 11.0).abs() < 1e-10);
    }

    #[test]
    fn test_sparse_crs_mul_vector_transpose() {
        let triplets = vec![
            Triplet::new(1.0_f64, 0, 0),
            Triplet::new(2.0_f64, 0, 1),
            Triplet::new(3.0_f64, 1, 0),
            Triplet::new(4.0_f64, 1, 1),
        ];
        let m = SparseMatrixCRS::from_triplets(2, 2, &triplets);
        let v = make_vector(&[1.0, 2.0]);
        let result = m.mul_vector_transpose(&v);
        assert!((result.get(0) - 7.0).abs() < 1e-10);
        assert!((result.get(1) - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_sparse_crs_to_triplets() {
        let triplets = vec![Triplet::new(1.0_f64, 0, 0), Triplet::new(2.0_f64, 1, 1)];
        let m = SparseMatrixCRS::from_triplets(2, 2, &triplets);
        let recovered = m.to_triplets();
        assert_eq!(recovered.len(), 2);
    }

    #[test]
    fn test_sparse_crs_nnz() {
        let triplets = vec![
            Triplet::new(1.0_f64, 0, 0),
            Triplet::new(2.0_f64, 0, 1),
            Triplet::new(3.0_f64, 1, 0),
        ];
        let m = SparseMatrixCRS::from_triplets(2, 2, &triplets);
        assert_eq!(m.nnz(), 3);
    }

    #[test]
    fn test_sparse_crs_transpose() {
        let triplets = vec![Triplet::new(1.0_f64, 0, 1), Triplet::new(2.0_f64, 1, 0)];
        let m = SparseMatrixCRS::from_triplets(2, 2, &triplets);
        let t = m.transpose();
        assert_eq!(t.rows(), 2);
        assert_eq!(t.cols(), 2);
        assert!((t.get(1, 0) - 1.0).abs() < 1e-10);
        assert!((t.get(0, 1) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_sparse_crs_display() {
        let triplets = vec![Triplet::new(1.0_f64, 0, 0)];
        let m = SparseMatrixCRS::from_triplets(2, 2, &triplets);
        let display = format!("{}", m);
        assert!(display.contains("SparseMatrixCRS"));
        assert!(display.contains("2x2"));
    }

    #[test]
    fn test_sparse_storage_trait() {
        let triplets = vec![Triplet::new(5.0_f64, 0, 0), Triplet::new(6.0_f64, 1, 1)];
        let m: SparseMatrixCRS<f64> = SparseStorage::from_triplets(2, 2, &triplets);
        assert_eq!(SparseStorage::rows(&m), 2);
        assert_eq!(SparseStorage::cols(&m), 2);
        assert_eq!(SparseStorage::nnz(&m), 2);
    }
}
