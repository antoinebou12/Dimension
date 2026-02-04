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
