//! CPU execution backends: sequential (default), optional parallel (par-iter with chili), optional SIMD (wide).
//!
//! Backend selection: when multiple features are enabled, prefer SIMD over parallel over sequential.
//! Used by `Vector::dot_f64`, scalar `*` `Vector<f64>`, and by code that calls the free functions
//! (`dot_f64`, `set_zero_f64`, `scalar_mul_f64`, `add_f64`, `sub_f64`) directly on slices.

pub mod sequential;

#[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
pub mod parallel;

#[cfg(feature = "simd")]
pub mod simd;

/// Dot product for f64 slices. Dispatches to simd, parallel, or sequential backend.
#[inline]
pub fn dot_f64(a: &[f64], b: &[f64]) -> f64 {
    #[cfg(feature = "simd")]
    return simd::dot_f64(a, b);
    #[cfg(all(
        feature = "parallel",
        not(target_arch = "wasm32"),
        not(feature = "simd")
    ))]
    return parallel::par_dot_f64(a, b);
    #[cfg(not(any(
        feature = "simd",
        all(feature = "parallel", not(target_arch = "wasm32"))
    )))]
    return sequential::dot_f64(a, b);
}

/// Dot product for f32 slices. Dispatches to simd, parallel, or sequential backend.
#[inline]
pub fn dot_f32(a: &[f32], b: &[f32]) -> f32 {
    #[cfg(feature = "simd")]
    return simd::dot_f32(a, b);
    #[cfg(all(
        feature = "parallel",
        not(target_arch = "wasm32"),
        not(feature = "simd")
    ))]
    return parallel::par_dot_f32(a, b);
    #[cfg(not(any(
        feature = "simd",
        all(feature = "parallel", not(target_arch = "wasm32"))
    )))]
    return sequential::dot_f32(a, b);
}

/// Sum of squared differences for f32 slices: `sum_i` (a[i] - b[i])^2.
/// Dispatches to simd, parallel, or sequential backend.
#[inline]
pub fn squared_diff_sum_f32(a: &[f32], b: &[f32]) -> f32 {
    #[cfg(feature = "simd")]
    return simd::squared_diff_sum_f32(a, b);
    #[cfg(all(
        feature = "parallel",
        not(target_arch = "wasm32"),
        not(feature = "simd")
    ))]
    return parallel::par_squared_diff_sum_f32(a, b);
    #[cfg(not(any(
        feature = "simd",
        all(feature = "parallel", not(target_arch = "wasm32"))
    )))]
    return sequential::squared_diff_sum_f32(a, b);
}

/// Element-wise add for f32: out[i] = a[i] + b[i].
/// Dispatches to simd, parallel, or sequential backend.
#[inline]
pub fn add_f32(a: &[f32], b: &[f32], out: &mut [f32]) {
    #[cfg(feature = "simd")]
    simd::add_f32(a, b, out);
    #[cfg(all(
        feature = "parallel",
        not(target_arch = "wasm32"),
        not(feature = "simd")
    ))]
    parallel::par_add_f32(a, b, out);
    #[cfg(not(any(
        feature = "simd",
        all(feature = "parallel", not(target_arch = "wasm32"))
    )))]
    sequential::add_f32(a, b, out);
}

/// Scalar multiply for f32: out[i] = s * x[i].
/// Dispatches to simd, parallel, or sequential backend.
#[inline]
pub fn scalar_mul_f32(s: f32, x: &[f32], out: &mut [f32]) {
    #[cfg(feature = "simd")]
    simd::scalar_mul_f32(s, x, out);
    #[cfg(all(
        feature = "parallel",
        not(target_arch = "wasm32"),
        not(feature = "simd")
    ))]
    parallel::par_scalar_mul_f32(s, x, out);
    #[cfg(not(any(
        feature = "simd",
        all(feature = "parallel", not(target_arch = "wasm32"))
    )))]
    sequential::scalar_mul_f32(s, x, out);
}

/// Matrix-vector product for column-major f32: y = A * x.
/// A is m×n column-major. Dispatches to simd, parallel, or sequential backend.
#[inline]
pub fn matvec_col_major_f32(m: usize, n: usize, a: &[f32], x: &[f32], y: &mut [f32]) {
    #[cfg(feature = "simd")]
    simd::matvec_col_major_f32(m, n, a, x, y);
    #[cfg(all(
        feature = "parallel",
        not(target_arch = "wasm32"),
        not(feature = "simd")
    ))]
    parallel::par_matvec_col_major_f32(m, n, a, x, y);
    #[cfg(not(any(
        feature = "simd",
        all(feature = "parallel", not(target_arch = "wasm32"))
    )))]
    sequential::matvec_col_major_f32(m, n, a, x, y);
}
