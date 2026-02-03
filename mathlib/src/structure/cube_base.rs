//! Cube base: 3rd-order tensor storage. Data stored as contiguous slices;
//! within each slice elements are column-major (Armadillo-compatible).

use super::dense_storage::DenseStorageDynamic;
use super::types::Fill;
use std::fmt;

#[derive(Clone, Debug)]
pub struct CubeBase<T> {
    pub(crate) storage: DenseStorageDynamic<T>,
    pub(crate) rows: usize,
    pub(crate) cols: usize,
    pub(crate) slices: usize,
}

impl<T> CubeBase<T> {
    pub fn new() -> Self {
        Self {
            storage: DenseStorageDynamic::new(),
            rows: 0,
            cols: 0,
            slices: 0,
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

    #[inline]
    pub fn slices(&self) -> usize {
        self.slices
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.storage.size()
    }

    #[inline]
    pub fn data(&self) -> &[T] {
        self.storage.data()
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut [T] {
        self.storage.data_mut()
    }

    /// Linear index for (i, j, k): column-major within each slice.
    /// Index = k * (rows * cols) + j * rows + i.
    #[inline]
    pub fn index_at(&self, i: usize, j: usize, k: usize) -> usize {
        debug_assert!(i < self.rows && j < self.cols && k < self.slices);
        k * (self.rows * self.cols) + j * self.rows + i
    }

    pub fn get(&self, i: usize, j: usize, k: usize) -> T
    where
        T: Copy,
    {
        let idx = self.index_at(i, j, k);
        self.storage.data()[idx]
    }

    pub fn set(&mut self, i: usize, j: usize, k: usize, value: T) {
        let idx = self.index_at(i, j, k);
        self.storage.data_mut()[idx] = value;
    }
}

impl<T: Clone + Default> CubeBase<T> {
    pub fn with_dimensions(rows: usize, cols: usize, slices: usize) -> Self {
        let n = rows * cols * slices;
        let mut storage = DenseStorageDynamic::with_capacity(n);
        storage.resize(n);
        Self {
            storage,
            rows,
            cols,
            slices,
        }
    }

    /// Construct with dimensions and initial fill (Zeros, Ones, or None).
    pub fn with_dimensions_fill(rows: usize, cols: usize, slices: usize, fill: Fill) -> Self
    where
        T: Copy + From<u8>,
    {
        let n = rows * cols * slices;
        let mut storage = DenseStorageDynamic::with_capacity(n);
        storage.resize(n);
        let mut cube = Self {
            storage,
            rows,
            cols,
            slices,
        };
        match fill {
            Fill::Zeros => cube.set_zero(),
            Fill::Ones => {
                for x in cube.storage.data_mut() {
                    *x = T::from(1_u8);
                }
            }
            Fill::None => {}
        }
        cube
    }

    pub fn resize(&mut self, rows: usize, cols: usize, slices: usize) {
        let n = rows * cols * slices;
        self.storage.resize(n);
        self.rows = rows;
        self.cols = cols;
        self.slices = slices;
    }

    pub fn set_zero(&mut self)
    where
        T: Copy + From<u8>,
    {
        self.storage.set_zero();
    }
}

impl<T> Default for CubeBase<T> {
    fn default() -> Self {
        Self {
            storage: DenseStorageDynamic::new(),
            rows: 0,
            cols: 0,
            slices: 0,
        }
    }
}

impl<T: fmt::Display> fmt::Display for CubeBase<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}x{} cube", self.rows, self.cols, self.slices)
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for CubeBase<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("CubeBase", 4)?;
        s.serialize_field("rows", &self.rows)?;
        s.serialize_field("cols", &self.cols)?;
        s.serialize_field("slices", &self.slices)?;
        s.serialize_field("data", self.data())?;
        s.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for CubeBase<T>
where
    T: Clone + Copy + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Deserialize, MapAccess, Visitor};
        use std::marker::PhantomData;
        struct CubeBaseVisitor<T>(PhantomData<T>);
        impl<'de, T: Deserialize<'de> + Clone + Copy + Default> Visitor<'de> for CubeBaseVisitor<T> {
            type Value = CubeBase<T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct CubeBase with rows, cols, slices, data")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<CubeBase<T>, A::Error> {
                let mut rows = None;
                let mut cols = None;
                let mut slices = None;
                let mut data = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "rows" => rows = Some(map.next_value()?),
                        "cols" => cols = Some(map.next_value()?),
                        "slices" => slices = Some(map.next_value()?),
                        "data" => data = Some(map.next_value()?),
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                let rows: usize = rows.ok_or_else(|| serde::de::Error::missing_field("rows"))?;
                let cols: usize = cols.ok_or_else(|| serde::de::Error::missing_field("cols"))?;
                let slices: usize =
                    slices.ok_or_else(|| serde::de::Error::missing_field("slices"))?;
                let data: Vec<T> = data.ok_or_else(|| serde::de::Error::missing_field("data"))?;
                let storage = DenseStorageDynamic::from_slice(&data);
                Ok(CubeBase {
                    storage,
                    rows,
                    cols,
                    slices,
                })
            }
        }
        deserializer.deserialize_struct(
            "CubeBase",
            &["rows", "cols", "slices", "data"],
            CubeBaseVisitor::<T>(PhantomData),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_base_new() {
        let c: CubeBase<f64> = CubeBase::new();
        assert_eq!(c.rows(), 0);
        assert_eq!(c.cols(), 0);
        assert_eq!(c.slices(), 0);
        assert_eq!(c.size(), 0);
    }

    #[test]
    fn test_cube_base_with_dimensions() {
        let c: CubeBase<f64> = CubeBase::with_dimensions(2, 3, 4);
        assert_eq!(c.rows(), 2);
        assert_eq!(c.cols(), 3);
        assert_eq!(c.slices(), 4);
        assert_eq!(c.size(), 24);
    }

    #[test]
    fn test_cube_base_index_at() {
        let c: CubeBase<f64> = CubeBase::with_dimensions(2, 3, 2);
        assert_eq!(c.index_at(0, 0, 0), 0);
        assert_eq!(c.index_at(1, 0, 0), 1);
        assert_eq!(c.index_at(0, 1, 0), 2);
        assert_eq!(c.index_at(1, 1, 0), 3);
        assert_eq!(c.index_at(0, 0, 1), 6);
    }

    #[test]
    fn test_cube_base_get_set() {
        let mut c: CubeBase<f64> = CubeBase::with_dimensions(2, 2, 2);
        c.set(0, 0, 0, 1.0);
        c.set(1, 1, 1, 8.0);
        assert!((c.get(0, 0, 0) - 1.0).abs() < 1e-10);
        assert!((c.get(1, 1, 1) - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_cube_base_resize() {
        let mut c: CubeBase<f64> = CubeBase::with_dimensions(2, 2, 2);
        c.resize(3, 4, 2);
        assert_eq!(c.rows(), 3);
        assert_eq!(c.cols(), 4);
        assert_eq!(c.slices(), 2);
        assert_eq!(c.size(), 24);
    }

    #[test]
    fn test_cube_base_set_zero() {
        let mut c: CubeBase<f64> = CubeBase::with_dimensions(2, 2, 2);
        c.set(0, 0, 0, 5.0);
        c.set_zero();
        assert!((c.get(0, 0, 0) - 0.0).abs() < 1e-10);
    }
}
