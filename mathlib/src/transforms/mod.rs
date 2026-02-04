//! Signal transforms: FFT, DCT, wavelets, convolution, windows.
//!
//! All implementations are pure Rust with no external dependencies.
//! Compatible with wasm32. Uses `crate::cpu::dot_f64` in convolution when available
//! (SIMD/parallel via features).

mod complex;
mod convolution;
mod dct;
mod fft;
mod wavelets;
mod windows;

pub use complex::Complex64;
pub use convolution::{conv_1d, conv_1d_same, conv_2d};
pub use dct::{dct2_forward, dct2_inverse};
pub use fft::{fft_forward, fft_forward_real, fft_inverse};
pub use wavelets::{dwt_haar_forward, dwt_haar_inverse};
pub use windows::{apply_window, apply_window_in_place, blackman, hamming, hann, tukey};

/// Errors for transform operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransformsError {
    /// Input length is not a power of two (required for FFT).
    LengthNotPowerOfTwo(usize),
    /// Empty input provided.
    EmptyInput,
}

impl std::fmt::Display for TransformsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LengthNotPowerOfTwo(n) => write!(f, "length {} is not a power of two", n),
            Self::EmptyInput => write!(f, "empty input"),
        }
    }
}

impl std::error::Error for TransformsError {}
