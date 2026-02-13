use std::fmt;
use std::ops::{Index, IndexMut};

/// Common trait for dense storage backends (fixed-size array or dynamic Vec).
pub trait DenseStorageTrait<T> {
    /// Returns the number of elements.
    fn size(&self) -> usize;
    /// Returns a slice of the stored data.
    fn data(&self) -> &[T];
    /// Returns a mutable slice of the stored data.
    fn data_mut(&mut self) -> &mut [T];
    /// Sets all elements to zero (via `T::from(0)`).
    fn set_zero(&mut self)
    where
        T: Copy + From<u8>;
}

#[derive(Clone, Debug)]
pub struct DenseStorage<T, const N: usize> {
    data: [T; N],
}

impl<T: Copy + Default, const N: usize> Default for DenseStorage<T, N> {
    fn default() -> Self {
        Self {
            data: [T::default(); N],
        }
    }
}

impl<T, const N: usize> DenseStorage<T, N> {
    pub fn new() -> Self
    where
        T: Copy + Default,
    {
        Self {
            data: [T::default(); N],
        }
    }

    /// Create storage from the first `N` elements of `data`.
    ///
    /// # Panics
    ///
    /// Panics if `data.len() < N`.
    ///
    /// # Safety
    ///
    /// The `unsafe { std::mem::zeroed() }` for `[T; N]` is safe here because the array is
    /// immediately overwritten by `copy_from_slice`; no zeroed `T` value is ever observed.
    /// Callers must ensure `T` is valid to copy from `data`.
    pub fn from_slice(data: &[T]) -> Self
    where
        T: Copy,
    {
        assert!(data.len() >= N);
        // SAFETY: array is overwritten by copy_from_slice before any read.
        let mut arr: [T; N] = [unsafe { std::mem::zeroed() }; N];
        arr.copy_from_slice(&data[..N]);
        Self { data: arr }
    }

    #[inline]
    pub fn size(&self) -> usize {
        N
    }

    pub fn resize(&mut self, _size: usize) {}

    pub fn set_zero(&mut self)
    where
        T: Copy + From<u8>,
    {
        for x in &mut self.data {
            *x = T::from(0);
        }
    }

    #[inline]
    pub fn data(&self) -> &[T] {
        &self.data
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}

impl<T: Copy + From<u8>, const N: usize> DenseStorageTrait<T> for DenseStorage<T, N> {
    #[inline]
    fn size(&self) -> usize {
        N
    }

    #[inline]
    fn data(&self) -> &[T] {
        &self.data
    }

    #[inline]
    fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    fn set_zero(&mut self) {
        for x in &mut self.data {
            *x = T::from(0);
        }
    }
}

impl<T, const N: usize> Index<usize> for DenseStorage<T, N> {
    type Output = T;

    fn index(&self, i: usize) -> &Self::Output {
        assert!(i < N);
        &self.data[i]
    }
}

impl<T, const N: usize> IndexMut<usize> for DenseStorage<T, N> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        assert!(i < N);
        &mut self.data[i]
    }
}

impl<T: fmt::Display, const N: usize> fmt::Display for DenseStorage<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, x) in self.data.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{x}")?;
        }
        write!(f, "]")
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize, const N: usize> serde::Serialize for DenseStorage<T, N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.data.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>, const N: usize> serde::Deserialize<'de>
    for DenseStorage<T, N>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let v: Vec<T> = serde::Deserialize::deserialize(deserializer)?;
        let arr: [T; N] = v.try_into().map_err(|e: Vec<T>| {
            D::Error::invalid_length(e.len(), &format!("array of length {N}").as_str())
        })?;
        Ok(Self { data: arr })
    }
}

#[derive(Clone, Debug)]
pub struct DenseStorageDynamic<T> {
    data: Vec<T>,
}

impl<T> DenseStorageDynamic<T> {
    /// Creates an empty dynamic storage.
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Creates storage with the given capacity, filled with `T::default()`.
    pub fn with_capacity(size: usize) -> Self
    where
        T: Default + Clone,
    {
        let mut data = Vec::with_capacity(size);
        data.resize(size, T::default());
        Self { data }
    }

    /// Creates storage by copying the slice.
    pub fn from_slice(data: &[T]) -> Self
    where
        T: Copy,
    {
        Self {
            data: data.to_vec(),
        }
    }

    /// Returns the number of elements.
    #[inline]
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Resizes storage to `size`, filling new slots with `T::default()`.
    pub fn resize(&mut self, size: usize)
    where
        T: Default + Clone,
    {
        self.data.resize(size, T::default());
    }

    /// Sets all elements to zero (via `T::from(0)`).
    pub fn set_zero(&mut self)
    where
        T: Copy + From<u8>,
    {
        for x in &mut self.data {
            *x = T::from(0);
        }
    }

    /// Returns a slice of the stored data.
    #[inline]
    pub fn data(&self) -> &[T] {
        &self.data
    }

    /// Returns a mutable slice of the stored data.
    #[inline]
    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}

impl<T: Copy + From<u8>> DenseStorageTrait<T> for DenseStorageDynamic<T> {
    #[inline]
    fn size(&self) -> usize {
        self.data.len()
    }

    #[inline]
    fn data(&self) -> &[T] {
        &self.data
    }

    #[inline]
    fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    fn set_zero(&mut self) {
        for x in &mut self.data {
            *x = T::from(0);
        }
    }
}

impl<T: Default> Default for DenseStorageDynamic<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<usize> for DenseStorageDynamic<T> {
    type Output = T;

    fn index(&self, i: usize) -> &Self::Output {
        assert!(i < self.data.len());
        &self.data[i]
    }
}

impl<T> IndexMut<usize> for DenseStorageDynamic<T> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        assert!(i < self.data.len());
        &mut self.data[i]
    }
}

impl<T: fmt::Display> fmt::Display for DenseStorageDynamic<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, x) in self.data.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{x}")?;
        }
        write!(f, "]")
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for DenseStorageDynamic<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.data.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for DenseStorageDynamic<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data: Vec<T> = serde::Deserialize::deserialize(deserializer)?;
        Ok(Self { data })
    }
}

pub use DenseStorageDynamic as DenseStorageDynamicImpl;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_storage_creation() {
        let s: DenseStorage<f64, 4> = DenseStorage::new();
        assert_eq!(s.size(), 4);
        assert!((s[0] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_fixed_storage_from_slice() {
        let data = [1.0, 2.0, 3.0, 4.0];
        let s: DenseStorage<f64, 4> = DenseStorage::from_slice(&data);
        assert!((s[0] - 1.0).abs() < 1e-10);
        assert!((s[3] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_fixed_storage_index_mut() {
        let mut s: DenseStorage<f64, 3> = DenseStorage::new();
        s[1] = 5.5;
        assert!((s[1] - 5.5).abs() < 1e-10);
    }

    #[test]
    fn test_fixed_storage_set_zero() {
        let mut s: DenseStorage<f64, 3> = DenseStorage::from_slice(&[1.0, 2.0, 3.0]);
        s.set_zero();
        for i in 0..3 {
            assert!((s[i] - 0.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_fixed_storage_data_access() {
        let mut s: DenseStorage<f64, 2> = DenseStorage::new();
        s.data_mut()[0] = 7.0;
        assert!((s.data()[0] - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_dynamic_storage_creation() {
        let s: DenseStorageDynamic<f64> = DenseStorageDynamic::new();
        assert_eq!(s.size(), 0);
    }

    #[test]
    fn test_dynamic_storage_with_capacity() {
        let s: DenseStorageDynamic<f64> = DenseStorageDynamic::with_capacity(5);
        assert_eq!(s.size(), 5);
        assert!((s[0] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_dynamic_storage_resize() {
        let mut s: DenseStorageDynamic<f64> = DenseStorageDynamic::new();
        s.resize(10);
        assert_eq!(s.size(), 10);
        s.resize(3);
        assert_eq!(s.size(), 3);
    }

    #[test]
    fn test_dynamic_storage_from_slice() {
        let s: DenseStorageDynamic<f64> = DenseStorageDynamic::from_slice(&[1.0, 2.0, 3.0]);
        assert_eq!(s.size(), 3);
        assert!((s[1] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_dynamic_storage_set_zero() {
        let mut s: DenseStorageDynamic<f64> = DenseStorageDynamic::from_slice(&[5.0, 6.0]);
        s.set_zero();
        assert!((s[0] - 0.0).abs() < 1e-10);
        assert!((s[1] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_dynamic_storage_display() {
        let s: DenseStorageDynamic<i32> = DenseStorageDynamic::from_slice(&[1, 2, 3]);
        let display = format!("{}", s);
        assert!(display.contains('1'));
        assert!(display.contains('2'));
        assert!(display.contains('3'));
    }

    #[test]
    fn test_fixed_storage_display() {
        let s: DenseStorage<i32, 2> = DenseStorage::from_slice(&[10, 20]);
        let display = format!("{}", s);
        assert!(display.contains("10"));
        assert!(display.contains("20"));
    }
}
