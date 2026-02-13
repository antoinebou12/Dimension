//! Mathematical traits for SIMD and `AoSoA` (Array of Struct of Arrays).
//!
//! Abstracts over scalar vs SIMD lane types so algorithms can be written once
//! and run with f64/f32 or `wide::f64x4`/`f32x4` (when `simd` feature is enabled).

/// Number of scalar elements in one lane (1 for scalar, 4 for f64x4).
pub trait LaneCount {
    /// Number of lanes.
    const LANES: usize;
}

/// The scalar element type for this lane (e.g. f64 for f64 and f64x4).
pub trait LaneScalar {
    /// Scalar type (e.g. f64).
    type Scalar: Copy;
}

/// Lane type that supports horizontal reduce and splat (scalar or SIMD).
pub trait SimdLane: Copy + LaneCount + LaneScalar {
    /// Horizontal sum of lane elements.
    fn reduce_add(self) -> Self::Scalar;
    /// Broadcast scalar to all lanes.
    fn splat(scalar: Self::Scalar) -> Self;
}

// --- Scalar impls ---

impl LaneCount for f64 {
    const LANES: usize = 1;
}

impl LaneScalar for f64 {
    type Scalar = f64;
}

impl SimdLane for f64 {
    fn reduce_add(self) -> Self::Scalar {
        self
    }
    fn splat(scalar: Self::Scalar) -> Self {
        scalar
    }
}

impl LaneCount for f32 {
    const LANES: usize = 1;
}

impl LaneScalar for f32 {
    type Scalar = f32;
}

impl SimdLane for f32 {
    fn reduce_add(self) -> Self::Scalar {
        self
    }
    fn splat(scalar: Self::Scalar) -> Self {
        scalar
    }
}

// --- SIMD impls (wide) when feature "simd" ---

#[cfg(feature = "simd")]
mod wide_impls {
    use super::{LaneCount, LaneScalar, SimdLane};
    use wide::{f32x4, f64x4};

    impl LaneCount for f64x4 {
        const LANES: usize = 4;
    }

    impl LaneScalar for f64x4 {
        type Scalar = f64;
    }

    impl SimdLane for f64x4 {
        fn reduce_add(self) -> Self::Scalar {
            self.reduce_add()
        }
        fn splat(scalar: Self::Scalar) -> Self {
            f64x4::splat(scalar)
        }
    }

    impl LaneCount for f32x4 {
        const LANES: usize = 4;
    }

    impl LaneScalar for f32x4 {
        type Scalar = f32;
    }

    impl SimdLane for f32x4 {
        fn reduce_add(self) -> Self::Scalar {
            self.reduce_add()
        }
        fn splat(scalar: Self::Scalar) -> Self {
            f32x4::splat(scalar)
        }
    }
}

// --- Chunked slice view (AoSoA-friendly, no new storage) ---

/// View `&[f64]` as chunks of 4 for SIMD load/store. Length must be a multiple of 4.
///
/// # Panics
///
/// Panics if `slice.len() % 4 != 0`.
///
/// # Safety
///
/// The `unsafe` block is safe because: the pointer is derived from a valid `&[f64]` slice
/// (non-null, aligned for `f64` and thus for `[f64; 4]`), the length is `slice.len() / 4` so
/// the resulting slice covers exactly the same memory, and the lifetime of the result is
/// tied to the input slice.
#[inline]
pub fn as_f64x4_chunks(slice: &[f64]) -> &[[f64; 4]] {
    assert!(
        slice.len().is_multiple_of(4),
        "slice length must be multiple of 4, got {}",
        slice.len()
    );
    let ptr = slice.as_ptr().cast::<[f64; 4]>();
    let len = slice.len() / 4;
    // SAFETY: ptr and len derived from valid slice; same memory, same lifetime.
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

/// View `&mut [f64]` as mutable chunks of 4 for SIMD load/store. Length must be a multiple of 4.
///
/// # Panics
///
/// Panics if `slice.len() % 4 != 0`.
///
/// # Safety
///
/// The `unsafe` block is safe because: the pointer is derived from a valid `&mut [f64]` slice
/// (non-null, aligned for `f64` and thus for `[f64; 4]`), the length is `slice.len() / 4` so
/// the resulting slice covers exactly the same memory, and the lifetime of the result is
/// tied to the input slice.
#[inline]
pub fn as_f64x4_chunks_mut(slice: &mut [f64]) -> &mut [[f64; 4]] {
    assert!(
        slice.len().is_multiple_of(4),
        "slice length must be multiple of 4, got {}",
        slice.len()
    );
    let ptr = slice.as_mut_ptr().cast::<[f64; 4]>();
    let len = slice.len() / 4;
    // SAFETY: ptr and len derived from valid slice; same memory, same lifetime.
    unsafe { std::slice::from_raw_parts_mut(ptr, len) }
}

/// View `&[f32]` as chunks of 4 for SIMD load/store. Length must be a multiple of 4.
///
/// # Panics
///
/// Panics if `slice.len() % 4 != 0`.
#[inline]
pub fn as_f32x4_chunks(slice: &[f32]) -> &[[f32; 4]] {
    assert!(
        slice.len().is_multiple_of(4),
        "slice length must be multiple of 4, got {}",
        slice.len()
    );
    let ptr = slice.as_ptr().cast::<[f32; 4]>();
    let len = slice.len() / 4;
    // SAFETY: ptr and len derived from valid slice; same memory, same lifetime.
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

/// View `&mut [f32]` as mutable chunks of 4 for SIMD load/store. Length must be a multiple of 4.
///
/// # Panics
///
/// Panics if `slice.len() % 4 != 0`.
#[inline]
pub fn as_f32x4_chunks_mut(slice: &mut [f32]) -> &mut [[f32; 4]] {
    assert!(
        slice.len().is_multiple_of(4),
        "slice length must be multiple of 4, got {}",
        slice.len()
    );
    let ptr = slice.as_mut_ptr().cast::<[f32; 4]>();
    let len = slice.len() / 4;
    // SAFETY: ptr and len derived from valid slice; same memory, same lifetime.
    unsafe { std::slice::from_raw_parts_mut(ptr, len) }
}
