use super::dense_storage::DenseStorageDynamic;
use super::types::Fill;
use std::fmt;

/// Base type for dense matrix storage (rows, cols, flat data).
#[derive(Clone, Debug)]
pub struct MatrixBase<T> {
    pub(crate) storage: DenseStorageDynamic<T>,
    pub(crate) rows: usize,
    pub(crate) cols: usize,
}

impl<T> MatrixBase<T> {
    pub fn new() -> Self {
        Self {
            storage: DenseStorageDynamic::new(),
            rows: 0,
            cols: 0,
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
}

impl<T: Clone + Default> MatrixBase<T> {
    /// Creates a matrix with the given dimensions, filled with `T::default()`.
    pub fn with_dimensions(rows: usize, cols: usize) -> Self {
        let n = rows * cols;
        let mut storage = DenseStorageDynamic::with_capacity(n);
        storage.resize(n);
        Self {
            storage,
            rows,
            cols,
        }
    }

    /// Construct with dimensions and initial fill (Zeros, Ones, or None).
    pub fn with_dimensions_fill(rows: usize, cols: usize, fill: Fill) -> Self
    where
        T: Copy + From<u8>,
    {
        let n = rows * cols;
        let mut storage = DenseStorageDynamic::with_capacity(n);
        storage.resize(n);
        let mut base = Self {
            storage,
            rows,
            cols,
        };
        match fill {
            Fill::Zeros => base.set_zero(),
            Fill::Ones => {
                for x in base.storage.data_mut() {
                    *x = T::from(1_u8);
                }
            }
            Fill::None => {}
        }
        base
    }

    /// Resizes the matrix to the given dimensions.
    pub fn resize(&mut self, rows: usize, cols: usize) {
        let n = rows * cols;
        self.storage.resize(n);
        self.rows = rows;
        self.cols = cols;
    }

    /// Sets all elements to zero.
    pub fn set_zero(&mut self)
    where
        T: Copy + From<u8>,
    {
        self.storage.set_zero();
    }
}

impl<T> Default for MatrixBase<T> {
    fn default() -> Self {
        Self {
            storage: DenseStorageDynamic::new(),
            rows: 0,
            cols: 0,
        }
    }
}

impl<T: fmt::Display> fmt::Display for MatrixBase<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (r, c) = (self.rows(), self.cols());
        write!(f, "{}x{}[", r, c)?;
        for i in 0..r {
            if i > 0 {
                write!(f, "; ")?;
            }
            for j in 0..c {
                if j > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", self.data()[i * c + j])?;
            }
        }
        write!(f, "]")
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for MatrixBase<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("MatrixBase", 3)?;
        s.serialize_field("rows", &self.rows)?;
        s.serialize_field("cols", &self.cols)?;
        s.serialize_field("data", self.data())?;
        s.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for MatrixBase<T>
where
    T: Clone + Copy + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Deserialize, MapAccess, Visitor};
        use std::marker::PhantomData;
        struct MatrixBaseVisitor<T>(PhantomData<T>);
        impl<'de, T: Deserialize<'de> + Clone + Copy + Default> Visitor<'de> for MatrixBaseVisitor<T> {
            type Value = MatrixBase<T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct MatrixBase with rows, cols, data")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<MatrixBase<T>, A::Error> {
                let mut rows = None;
                let mut cols = None;
                let mut data = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "rows" => rows = Some(map.next_value()?),
                        "cols" => cols = Some(map.next_value()?),
                        "data" => data = Some(map.next_value()?),
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                let rows: usize = rows.ok_or_else(|| serde::de::Error::missing_field("rows"))?;
                let cols: usize = cols.ok_or_else(|| serde::de::Error::missing_field("cols"))?;
                let data: Vec<T> = data.ok_or_else(|| serde::de::Error::missing_field("data"))?;
                let storage = DenseStorageDynamic::from_slice(&data);
                Ok(MatrixBase {
                    storage,
                    rows,
                    cols,
                })
            }
        }
        deserializer.deserialize_struct(
            "MatrixBase",
            &["rows", "cols", "data"],
            MatrixBaseVisitor::<T>(PhantomData),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::types::Fill;

    #[test]
    fn test_matrix_base_new() {
        let m: MatrixBase<f64> = MatrixBase::new();
        assert_eq!(m.rows(), 0);
        assert_eq!(m.cols(), 0);
        assert_eq!(m.size(), 0);
    }

    #[test]
    fn test_matrix_base_with_dimensions() {
        let m: MatrixBase<f64> = MatrixBase::with_dimensions(3, 4);
        assert_eq!(m.rows(), 3);
        assert_eq!(m.cols(), 4);
        assert_eq!(m.size(), 12);
    }

    #[test]
    fn test_matrix_base_with_dimensions_fill() {
        let m_zeros: MatrixBase<f64> = MatrixBase::with_dimensions_fill(2, 2, Fill::Zeros);
        assert_eq!(m_zeros.rows(), 2);
        assert_eq!(m_zeros.cols(), 2);
        assert!((m_zeros.data()[0] - 0.0).abs() < 1e-10);
        let m_ones: MatrixBase<f64> = MatrixBase::with_dimensions_fill(2, 2, Fill::Ones);
        assert!((m_ones.data()[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_matrix_base_resize() {
        let mut m: MatrixBase<f64> = MatrixBase::with_dimensions(2, 2);
        assert_eq!(m.size(), 4);
        m.resize(5, 3);
        assert_eq!(m.rows(), 5);
        assert_eq!(m.cols(), 3);
        assert_eq!(m.size(), 15);
    }

    #[test]
    fn test_matrix_base_set_zero() {
        let mut m: MatrixBase<f64> = MatrixBase::with_dimensions(2, 2);
        m.data_mut()[0] = 5.0;
        m.data_mut()[1] = 3.0;
        m.set_zero();
        for val in m.data() {
            assert!((*val - 0.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_matrix_base_data_access() {
        let mut m: MatrixBase<f64> = MatrixBase::with_dimensions(2, 3);
        m.data_mut()[0] = 1.0;
        m.data_mut()[5] = 6.0;
        assert!((m.data()[0] - 1.0).abs() < 1e-10);
        assert!((m.data()[5] - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_matrix_base_default() {
        let m: MatrixBase<f64> = MatrixBase::default();
        assert_eq!(m.rows(), 0);
        assert_eq!(m.cols(), 0);
    }

    #[test]
    fn test_matrix_base_display() {
        let mut m: MatrixBase<i32> = MatrixBase::with_dimensions(2, 2);
        m.data_mut()[0] = 1;
        m.data_mut()[1] = 2;
        m.data_mut()[2] = 3;
        m.data_mut()[3] = 4;
        let display = format!("{}", m);
        assert!(display.contains("2x2"));
        assert!(display.contains('1'));
    }
}
