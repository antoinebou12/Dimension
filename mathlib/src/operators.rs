//! Arithmetic operators for matrices, vectors, and cubes.
//!
//! This module implements standard Rust arithmetic traits (`Add`, `Sub`, `Mul`) for
//! the core mathlib types. All operations work on references to avoid unnecessary copies.
//!
//! # Supported Operations
//!
//! | Operation | Types | Notes |
//! |-----------|-------|-------|
//! | `A + B` | Matrix + Matrix | Element-wise, same dimensions required |
//! | `A - B` | Matrix - Matrix | Element-wise, same dimensions required |
//! | `A * B` | Matrix × Matrix | Standard matrix multiply, `A.cols == B.rows` |
//! | `s * A` | scalar × Matrix | Element-wise scaling (`f64`, `f32`) |
//! | `A * v` | Matrix × Vector | Matrix-vector product |
//! | `u + v` | Vector + Vector | Element-wise, same length required |
//! | `u - v` | Vector - Vector | Element-wise, same length required |
//! | `u * v` | Vector × Vector | Dot product (returns scalar) |
//! | `s * v` | scalar × Vector | Element-wise scaling |
//!
//! # Usage
//!
//! ```
//! use mathlib::{Matrix, Vector, Storage};
//!
//! let a = Matrix::<f64>::with_storage(2, 2, Storage::Column);
//! let b = Matrix::<f64>::with_storage(2, 2, Storage::Column);
//! let c = &a + &b;  // Matrix addition
//! let d = &a * &b;  // Matrix multiplication
//!
//! let mut u = Vector::<f64>::with_capacity(2);
//! u.data_mut().copy_from_slice(&[1.0, 2.0]);
//! let mut v = Vector::<f64>::with_capacity(2);
//! v.data_mut().copy_from_slice(&[3.0, 4.0]);
//! let dot = u.dot(&v);  // Dot product: 1*3 + 2*4 = 11
//! ```
//!
//! # Panics
//!
//! Operations panic if dimensions are incompatible:
//! - Addition/subtraction: matrices must have same dimensions
//! - Matrix multiplication: `A.cols()` must equal `B.rows()`
//! - Vector operations: vectors must have same length

use crate::cube::Cube;
use crate::matrix::Matrix;
use crate::types::Storage;
use crate::vector::Vector;
use std::ops::{Add, Mul, Sub};

impl<T: Copy + Default + Add<Output = T> + 'static> Add<&Matrix<T>> for &Matrix<T> {
    type Output = Matrix<T>;

    fn add(self, other: &Matrix<T>) -> Matrix<T> {
        assert!(self.rows() == other.rows() && self.cols() == other.cols());
        #[cfg(feature = "gpu")]
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>()
            && (self.rows() * self.cols()) >= crate::gpu::MIN_LEN_GPU_ELEMENTWISE
        {
            let a: &Matrix<f32> = unsafe { &*(self as *const Matrix<T> as *const Matrix<f32>) };
            let b: &Matrix<f32> = unsafe { &*(other as *const Matrix<T> as *const Matrix<f32>) };
            if let Some(gpu_out) = crate::gpu::try_add_f32(a.data(), b.data()) {
                let mut out = Matrix::with_storage(a.rows(), a.cols(), a.storage);
                out.data_mut().copy_from_slice(&gpu_out);
                return unsafe { std::mem::transmute(out) };
            }
        }
        let mut out = Matrix::with_storage(self.rows(), self.cols(), self.storage);
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                out.set(i, j, self.get(i, j) + other.get(i, j));
            }
        }
        out
    }
}

impl<T: Copy + Default + Sub<Output = T> + 'static> Sub<&Matrix<T>> for &Matrix<T> {
    type Output = Matrix<T>;

    fn sub(self, other: &Matrix<T>) -> Matrix<T> {
        assert!(self.rows() == other.rows() && self.cols() == other.cols());
        #[cfg(feature = "gpu")]
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>()
            && (self.rows() * self.cols()) >= crate::gpu::MIN_LEN_GPU_ELEMENTWISE
        {
            let a: &Matrix<f32> = unsafe { &*(self as *const Matrix<T> as *const Matrix<f32>) };
            let b: &Matrix<f32> = unsafe { &*(other as *const Matrix<T> as *const Matrix<f32>) };
            if let Some(gpu_out) = crate::gpu::try_sub_f32(a.data(), b.data()) {
                let mut out = Matrix::with_storage(a.rows(), a.cols(), a.storage);
                out.data_mut().copy_from_slice(&gpu_out);
                return unsafe { std::mem::transmute(out) };
            }
        }
        let mut out = Matrix::with_storage(self.rows(), self.cols(), self.storage);
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                out.set(i, j, self.get(i, j) - other.get(i, j));
            }
        }
        out
    }
}

impl<T: Copy + Default + From<u8> + std::ops::AddAssign + Mul<Output = T> + 'static> Mul<&Matrix<T>>
    for &Matrix<T>
{
    type Output = Matrix<T>;

    fn mul(self, other: &Matrix<T>) -> Matrix<T> {
        assert!(self.cols() == other.rows());
        #[cfg(feature = "gpu")]
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>()
            && (self.rows() * self.cols() * other.cols()) >= crate::gpu::MIN_ELEMENTS_GPU_MATMUL
        {
            let a: &Matrix<f32> = unsafe { &*(self as *const Matrix<T> as *const Matrix<f32>) };
            let b: &Matrix<f32> = unsafe { &*(other as *const Matrix<T> as *const Matrix<f32>) };
            if let Some(gpu_out) = crate::gpu::try_matmul_f32(a, b) {
                return unsafe { std::mem::transmute(gpu_out) };
            }
        }
        let mut out = Matrix::with_storage(self.rows(), other.cols(), Storage::Column);
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
        out
    }
}

// --- scalar * Matrix (concrete types to satisfy orphan rule) ---
impl Mul<&Matrix<f64>> for f64 {
    type Output = Matrix<f64>;

    fn mul(self, rhs: &Matrix<f64>) -> Matrix<f64> {
        let mut out = Matrix::with_storage(rhs.rows(), rhs.cols(), rhs.storage);
        for i in 0..rhs.rows() {
            for j in 0..rhs.cols() {
                out.set(i, j, self * rhs.get(i, j));
            }
        }
        out
    }
}

impl Mul<&Matrix<f32>> for f32 {
    type Output = Matrix<f32>;

    fn mul(self, rhs: &Matrix<f32>) -> Matrix<f32> {
        #[cfg(feature = "gpu")]
        if (rhs.rows() * rhs.cols()) >= crate::gpu::MIN_LEN_GPU_ELEMENTWISE {
            if let Some(gpu_out) = crate::gpu::try_scale_f32(self, rhs.data()) {
                let mut out = Matrix::with_storage(rhs.rows(), rhs.cols(), rhs.storage);
                out.data_mut().copy_from_slice(&gpu_out);
                return out;
            }
        }
        let mut out = Matrix::with_storage(rhs.rows(), rhs.cols(), rhs.storage);
        for i in 0..rhs.rows() {
            for j in 0..rhs.cols() {
                out.set(i, j, self * rhs.get(i, j));
            }
        }
        out
    }
}

// --- Matrix * Vector ---
impl<
    T: Copy
        + Default
        + From<u8>
        + std::ops::Add<Output = T>
        + std::ops::AddAssign
        + Mul<Output = T>
        + 'static,
> Mul<&Vector<T>> for &Matrix<T>
{
    type Output = Vector<T>;

    fn mul(self, v: &Vector<T>) -> Vector<T> {
        assert!(self.cols() == v.rows());
        #[cfg(feature = "gpu")]
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>()
            && (self.rows() * self.cols()) >= crate::gpu::MIN_ELEMENTS_GPU_MATVEC
        {
            let a: &Matrix<f32> = unsafe { &*(self as *const Matrix<T> as *const Matrix<f32>) };
            let b: &Vector<f32> = unsafe { &*(v as *const Vector<T> as *const Vector<f32>) };
            if let Some(gpu_out) = crate::gpu::try_matvec_f32(a, b) {
                return unsafe { std::mem::transmute(gpu_out) };
            }
        }
        #[cfg(feature = "simd")]
        if self.storage == Storage::Column {
            if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f64>() {
                let m = self.rows();
                let n = self.cols();
                let a: &[f64] = unsafe {
                    std::slice::from_raw_parts(self.data().as_ptr() as *const f64, m * n)
                };
                let x: &[f64] =
                    unsafe { std::slice::from_raw_parts(v.data().as_ptr() as *const f64, n) };
                let mut out = Vector::with_capacity(m);
                out.set_zero();
                crate::cpu::simd::matvec_col_major_f64(m, n, a, x, out.data_mut());
                return unsafe { std::mem::transmute(out) };
            }
            if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>() {
                let m = self.rows();
                let n = self.cols();
                let a: &[f32] = unsafe {
                    std::slice::from_raw_parts(self.data().as_ptr() as *const f32, m * n)
                };
                let x: &[f32] =
                    unsafe { std::slice::from_raw_parts(v.data().as_ptr() as *const f32, n) };
                let mut out = Vector::with_capacity(m);
                out.set_zero();
                crate::cpu::simd::matvec_col_major_f32(m, n, a, x, out.data_mut());
                return unsafe { std::mem::transmute(out) };
            }
        }
        let mut out = Vector::with_capacity(self.rows());
        out.set_zero();
        for j in 0..self.cols() {
            for i in 0..self.rows() {
                out.set(i, out.get(i) + self.get(i, j) * v.get(j));
            }
        }
        out
    }
}

// --- scalar * Vector (concrete types), f64 uses cpu backends ---
impl Mul<&Vector<f64>> for f64 {
    type Output = Vector<f64>;

    fn mul(self, rhs: &Vector<f64>) -> Vector<f64> {
        let mut out = Vector::with_capacity(rhs.rows());
        #[cfg(feature = "simd")]
        crate::cpu::simd::scalar_mul_f64(self, rhs.data(), out.data_mut());
        #[cfg(all(
            feature = "parallel",
            not(target_arch = "wasm32"),
            not(feature = "simd")
        ))]
        crate::cpu::parallel::par_scalar_mul_f64(self, rhs.data(), out.data_mut());
        #[cfg(not(any(
            feature = "simd",
            all(feature = "parallel", not(target_arch = "wasm32"))
        )))]
        crate::cpu::sequential::scalar_mul_f64(self, rhs.data(), out.data_mut());
        out
    }
}

impl Mul<&Vector<f32>> for f32 {
    type Output = Vector<f32>;

    fn mul(self, rhs: &Vector<f32>) -> Vector<f32> {
        #[cfg(feature = "gpu")]
        if rhs.rows() >= crate::gpu::MIN_LEN_GPU_ELEMENTWISE {
            if let Some(gpu_out) = crate::gpu::try_scale_f32(self, rhs.data()) {
                let mut out = Vector::with_capacity(rhs.rows());
                out.data_mut().copy_from_slice(&gpu_out);
                return out;
            }
        }
        let mut out = Vector::with_capacity(rhs.rows());
        #[cfg(feature = "simd")]
        crate::cpu::simd::scalar_mul_f32(self, rhs.data(), out.data_mut());
        #[cfg(all(
            feature = "parallel",
            not(target_arch = "wasm32"),
            not(feature = "simd")
        ))]
        crate::cpu::parallel::par_scalar_mul_f32(self, rhs.data(), out.data_mut());
        #[cfg(not(any(
            feature = "simd",
            all(feature = "parallel", not(target_arch = "wasm32"))
        )))]
        crate::cpu::sequential::scalar_mul_f32(self, rhs.data(), out.data_mut());
        out
    }
}

impl<T: Copy + Default + Add<Output = T> + 'static> Add<&Vector<T>> for &Vector<T> {
    type Output = Vector<T>;

    fn add(self, other: &Vector<T>) -> Vector<T> {
        assert!(self.rows() == other.rows());
        #[cfg(feature = "gpu")]
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>()
            && self.rows() >= crate::gpu::MIN_LEN_GPU_ELEMENTWISE
        {
            let a: &Vector<f32> = unsafe { &*(self as *const Vector<T> as *const Vector<f32>) };
            let b: &Vector<f32> = unsafe { &*(other as *const Vector<T> as *const Vector<f32>) };
            if let Some(gpu_out) = crate::gpu::try_add_f32(a.data(), b.data()) {
                let mut out = Vector::with_capacity(a.rows());
                out.data_mut().copy_from_slice(&gpu_out);
                return unsafe { std::mem::transmute(out) };
            }
        }
        let mut out = Vector::with_capacity(self.rows());
        for i in 0..self.rows() {
            out.set(i, self.get(i) + other.get(i));
        }
        out
    }
}

impl<T: Copy + Default + Sub<Output = T> + 'static> Sub<&Vector<T>> for &Vector<T> {
    type Output = Vector<T>;

    fn sub(self, other: &Vector<T>) -> Vector<T> {
        assert!(self.rows() == other.rows());
        #[cfg(feature = "gpu")]
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>()
            && self.rows() >= crate::gpu::MIN_LEN_GPU_ELEMENTWISE
        {
            let a: &Vector<f32> = unsafe { &*(self as *const Vector<T> as *const Vector<f32>) };
            let b: &Vector<f32> = unsafe { &*(other as *const Vector<T> as *const Vector<f32>) };
            if let Some(gpu_out) = crate::gpu::try_sub_f32(a.data(), b.data()) {
                let mut out = Vector::with_capacity(a.rows());
                out.data_mut().copy_from_slice(&gpu_out);
                return unsafe { std::mem::transmute(out) };
            }
        }
        let mut out = Vector::with_capacity(self.rows());
        for i in 0..self.rows() {
            out.set(i, self.get(i) - other.get(i));
        }
        out
    }
}

// --- Cube element-wise Add / Sub ---
impl<T: Copy + Default + Add<Output = T> + 'static> Add<&Cube<T>> for &Cube<T> {
    type Output = Cube<T>;

    fn add(self, other: &Cube<T>) -> Cube<T> {
        assert!(
            self.rows() == other.rows()
                && self.cols() == other.cols()
                && self.slices() == other.slices()
        );
        #[cfg(feature = "gpu")]
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>()
            && (self.rows() * self.cols() * self.slices()) >= crate::gpu::MIN_LEN_GPU_ELEMENTWISE
        {
            let a = self.data();
            let b = other.data();
            let a_f32: &[f32] =
                unsafe { std::slice::from_raw_parts(a.as_ptr() as *const f32, a.len()) };
            let b_f32: &[f32] =
                unsafe { std::slice::from_raw_parts(b.as_ptr() as *const f32, b.len()) };
            if let Some(gpu_out) = crate::gpu::try_add_f32(a_f32, b_f32) {
                let mut out = Cube::with_dimensions(self.rows(), self.cols(), self.slices());
                out.data_mut().copy_from_slice(&gpu_out);
                return unsafe { std::mem::transmute(out) };
            }
        }
        let mut out = Cube::with_dimensions(self.rows(), self.cols(), self.slices());
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                for k in 0..self.slices() {
                    out.set(i, j, k, self.get(i, j, k) + other.get(i, j, k));
                }
            }
        }
        out
    }
}

impl<T: Copy + Default + Sub<Output = T> + 'static> Sub<&Cube<T>> for &Cube<T> {
    type Output = Cube<T>;

    fn sub(self, other: &Cube<T>) -> Cube<T> {
        assert!(
            self.rows() == other.rows()
                && self.cols() == other.cols()
                && self.slices() == other.slices()
        );
        #[cfg(feature = "gpu")]
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>()
            && (self.rows() * self.cols() * self.slices()) >= crate::gpu::MIN_LEN_GPU_ELEMENTWISE
        {
            let a = self.data();
            let b = other.data();
            let a_f32: &[f32] =
                unsafe { std::slice::from_raw_parts(a.as_ptr() as *const f32, a.len()) };
            let b_f32: &[f32] =
                unsafe { std::slice::from_raw_parts(b.as_ptr() as *const f32, b.len()) };
            if let Some(gpu_out) = crate::gpu::try_sub_f32(a_f32, b_f32) {
                let mut out = Cube::with_dimensions(self.rows(), self.cols(), self.slices());
                out.data_mut().copy_from_slice(&gpu_out);
                return unsafe { std::mem::transmute(out) };
            }
        }
        let mut out = Cube::with_dimensions(self.rows(), self.cols(), self.slices());
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                for k in 0..self.slices() {
                    out.set(i, j, k, self.get(i, j, k) - other.get(i, j, k));
                }
            }
        }
        out
    }
}

// --- scalar * Cube (f32 uses GPU when available, f64 uses CPU) ---
impl Mul<&Cube<f32>> for f32 {
    type Output = Cube<f32>;

    fn mul(self, rhs: &Cube<f32>) -> Cube<f32> {
        #[cfg(feature = "gpu")]
        if (rhs.rows() * rhs.cols() * rhs.slices()) >= crate::gpu::MIN_LEN_GPU_ELEMENTWISE {
            if let Some(gpu_out) = crate::gpu::try_scale_f32(self, rhs.data()) {
                let mut out = Cube::with_dimensions(rhs.rows(), rhs.cols(), rhs.slices());
                out.data_mut().copy_from_slice(&gpu_out);
                return out;
            }
        }
        let mut out = Cube::with_dimensions(rhs.rows(), rhs.cols(), rhs.slices());
        for i in 0..rhs.rows() {
            for j in 0..rhs.cols() {
                for k in 0..rhs.slices() {
                    out.set(i, j, k, self * rhs.get(i, j, k));
                }
            }
        }
        out
    }
}

impl Mul<&Cube<f64>> for f64 {
    type Output = Cube<f64>;

    fn mul(self, rhs: &Cube<f64>) -> Cube<f64> {
        let mut out = Cube::with_dimensions(rhs.rows(), rhs.cols(), rhs.slices());
        for i in 0..rhs.rows() {
            for j in 0..rhs.cols() {
                for k in 0..rhs.slices() {
                    out.set(i, j, k, self * rhs.get(i, j, k));
                }
            }
        }
        out
    }
}
