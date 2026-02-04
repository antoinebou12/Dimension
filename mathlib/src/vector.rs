//! Vector: matrix with one column (rows x 1).
//!
//! Mirrors C++ Vector.h: dot product, norm, resize.

use crate::matrix_base::MatrixBase;
use std::fmt;
use std::ops::{Div, Index, IndexMut, Mul};

/// Vector (dynamic size): column vector, rows x 1.
#[derive(Clone, Debug)]
pub struct Vector<T> {
    pub(crate) base: MatrixBase<T>,
}

impl<T> Vector<T> {
    pub fn new() -> Self {
        Self {
            base: MatrixBase::new(),
        }
    }

    #[inline]
    pub fn rows(&self) -> usize {
        self.base.rows()
    }

    #[inline]
    pub fn data(&self) -> &[T] {
        self.base.data()
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut [T] {
        self.base.data_mut()
    }

    /// Contiguous slice of elements (alias for `data()`).
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.base.data()
    }
}

impl<T: Clone + Default> Vector<T> {
    pub fn with_capacity(rows: usize) -> Self {
        Self {
            base: MatrixBase::with_dimensions(rows, 1),
        }
    }

    /// Creates a vector by copying the slice.
    pub fn from_slice(data: &[T]) -> Self
    where
        T: Copy,
    {
        let n = data.len();
        let mut base = MatrixBase::with_dimensions(n, 1);
        base.data_mut().copy_from_slice(data);
        Self { base }
    }

    pub fn resize(&mut self, rows: usize) {
        self.base.resize(rows, 1);
    }

    pub fn set_zero(&mut self)
    where
        T: Copy + From<u8>,
    {
        self.base.set_zero();
    }

    /// Element at index `i`.
    ///
    /// # Panics
    ///
    /// Panics if `i >= rows()`.
    pub fn get(&self, i: usize) -> T
    where
        T: Copy,
    {
        assert!(i < self.base.rows());
        self.base.data()[i]
    }

    /// Set element at index `i`.
    ///
    /// # Panics
    ///
    /// Panics if `i >= rows()`.
    pub fn set(&mut self, i: usize, value: T) {
        assert!(i < self.base.rows());
        self.base.data_mut()[i] = value;
    }

    pub fn dot(&self, other: &Vector<T>) -> T
    where
        T: Copy + Default + std::ops::AddAssign + std::ops::Mul<Output = T> + Float + 'static,
    {
        assert_eq!(self.rows(), other.rows());
        #[cfg(feature = "gpu")]
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>() {
            let a: &Vector<f32> = unsafe { &*(self as *const Vector<T> as *const Vector<f32>) };
            let b: &Vector<f32> = unsafe { &*(other as *const Vector<T> as *const Vector<f32>) };
            if let Some(gpu_dot) = crate::gpu::try_dot_f32(a, b) {
                return unsafe { std::mem::transmute_copy(&gpu_dot) };
            }
        }
        let n = self.rows();
        let mut sum = T::default();
        for i in 0..n {
            sum += self.get(i) * other.get(i);
        }
        sum
    }

    pub fn norm(&self) -> T
    where
        T: Copy + Default + std::ops::AddAssign + std::ops::Mul<Output = T> + Float + 'static,
    {
        #[cfg(feature = "gpu")]
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>() {
            let v: &Vector<f32> = unsafe { &*(self as *const Vector<T> as *const Vector<f32>) };
            if let Some(gpu_norm) = crate::gpu::try_norm_f32(v) {
                return unsafe { std::mem::transmute_copy(&gpu_norm) };
            }
        }
        let n = self.dot(self);
        n.sqrt()
    }

    /// Component-wise map.
    pub fn map<F, U>(&self, f: F) -> Vector<U>
    where
        T: Copy,
        F: Fn(T) -> U,
        U: Clone + Default,
    {
        let n = self.rows();
        let mut out = Vector::with_capacity(n);
        for i in 0..n {
            out.set(i, f(self.get(i)));
        }
        out
    }

    /// Component-wise zip and map.
    pub fn zip_map<F, U>(&self, other: &Vector<T>, f: F) -> Vector<U>
    where
        T: Copy,
        F: Fn(T, T) -> U,
        U: Clone + Default,
    {
        assert_eq!(self.rows(), other.rows());
        let n = self.rows();
        let mut out = Vector::with_capacity(n);
        for i in 0..n {
            out.set(i, f(self.get(i), other.get(i)));
        }
        out
    }

    /// Normalized copy: `self / norm(self)`. Returns zero vector if norm is negligible.
    #[must_use]
    pub fn normalize(&self) -> Vector<T>
    where
        T: RealNumber + std::ops::AddAssign + PartialOrd + 'static,
    {
        let n = self.norm();
        let eps = T::from_f64(1e-20);
        if n < eps {
            let zero = T::from_f64(0.0);
            return self.map(|_| zero);
        }
        self.map(|x| x / n)
    }
}

impl<T> Default for Vector<T> {
    fn default() -> Self {
        Self {
            base: MatrixBase::new(),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Vector<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vector({})[", self.rows())?;
        for i in 0..self.rows() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", self.data()[i])?;
        }
        write!(f, "]")
    }
}

impl<T, I> Index<I> for Vector<T>
where
    I: std::slice::SliceIndex<[T]>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.base.data().index(index)
    }
}

impl<T, I> IndexMut<I> for Vector<T>
where
    I: std::slice::SliceIndex<[T]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.base.data_mut().index_mut(index)
    }
}

pub trait Float: Copy {
    fn sqrt(self) -> Self;
}

impl Float for f32 {
    fn sqrt(self) -> Self {
        self.sqrt()
    }
}

impl Float for f64 {
    fn sqrt(self) -> Self {
        self.sqrt()
    }
}

/// Real scalar type with trig and constants (for component-wise vector trig).
pub trait RealNumber: Copy + Default + Float + Mul<Output = Self> + Div<Output = Self> {
    /// Pi.
    fn pi() -> Self;
    /// Convert from f64 (e.g. for 180 in degrees/radians).
    fn from_f64(x: f64) -> Self;
    fn acos(self) -> Self;
    fn asin(self) -> Self;
    fn atan(self) -> Self;
    fn atan2(self, x: Self) -> Self;
    fn cos(self) -> Self;
    fn sin(self) -> Self;
    fn tan(self) -> Self;
    fn cosh(self) -> Self;
    fn sinh(self) -> Self;
    fn tanh(self) -> Self;
    fn acosh(self) -> Self;
    fn asinh(self) -> Self;
    fn atanh(self) -> Self;
}

impl RealNumber for f32 {
    fn pi() -> Self {
        std::f32::consts::PI
    }
    #[allow(clippy::cast_possible_truncation)]
    fn from_f64(x: f64) -> Self {
        x as f32
    }
    fn acos(self) -> Self {
        self.acos()
    }
    fn asin(self) -> Self {
        self.asin()
    }
    fn atan(self) -> Self {
        self.atan()
    }
    fn atan2(self, x: Self) -> Self {
        self.atan2(x)
    }
    fn cos(self) -> Self {
        self.cos()
    }
    fn sin(self) -> Self {
        self.sin()
    }
    fn tan(self) -> Self {
        self.tan()
    }
    fn cosh(self) -> Self {
        self.cosh()
    }
    fn sinh(self) -> Self {
        self.sinh()
    }
    fn tanh(self) -> Self {
        self.tanh()
    }
    fn acosh(self) -> Self {
        self.acosh()
    }
    fn asinh(self) -> Self {
        self.asinh()
    }
    fn atanh(self) -> Self {
        self.atanh()
    }
}

impl RealNumber for f64 {
    fn pi() -> Self {
        std::f64::consts::PI
    }
    fn from_f64(x: f64) -> Self {
        x
    }
    fn acos(self) -> Self {
        self.acos()
    }
    fn asin(self) -> Self {
        self.asin()
    }
    fn atan(self) -> Self {
        self.atan()
    }
    fn atan2(self, x: Self) -> Self {
        self.atan2(x)
    }
    fn cos(self) -> Self {
        self.cos()
    }
    fn sin(self) -> Self {
        self.sin()
    }
    fn tan(self) -> Self {
        self.tan()
    }
    fn cosh(self) -> Self {
        self.cosh()
    }
    fn sinh(self) -> Self {
        self.sinh()
    }
    fn tanh(self) -> Self {
        self.tanh()
    }
    fn acosh(self) -> Self {
        self.acosh()
    }
    fn asinh(self) -> Self {
        self.asinh()
    }
    fn atanh(self) -> Self {
        self.atanh()
    }
}
