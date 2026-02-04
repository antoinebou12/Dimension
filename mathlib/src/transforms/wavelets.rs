//! Haar wavelet transform.
//!
//! Single-level discrete wavelet transform (DWT) and inverse.

use std::f64::consts::FRAC_1_SQRT_2;

/// Single-level Haar wavelet forward transform.
///
/// Input length must be even. Output layout: approx coeffs then detail coeffs.
///
/// # Panics
///
/// Panics if input length is odd.
#[must_use]
pub fn dwt_haar_forward(signal: &[f64]) -> Vec<f64> {
    let n = signal.len();
    assert!(n.is_multiple_of(2), "Haar DWT requires even-length input");
    let half = n / 2;
    let mut out = vec![0.0; n];
    for i in 0..half {
        let a = signal[2 * i];
        let b = signal[2 * i + 1];
        out[i] = (a + b) * FRAC_1_SQRT_2;
        out[half + i] = (a - b) * FRAC_1_SQRT_2;
    }
    out
}

/// Single-level Haar wavelet inverse transform.
///
/// Reconstructs signal from [approx, detail]. Input length must be even.
///
/// # Panics
///
/// Panics if input length is odd.
#[must_use]
pub fn dwt_haar_inverse(coeffs: &[f64]) -> Vec<f64> {
    let n = coeffs.len();
    assert!(n.is_multiple_of(2), "Haar IDWT requires even-length input");
    let half = n / 2;
    let mut out = vec![0.0; n];
    for i in 0..half {
        let a = coeffs[i];
        let d = coeffs[half + i];
        out[2 * i] = (a + d) * FRAC_1_SQRT_2;
        out[2 * i + 1] = (a - d) * FRAC_1_SQRT_2;
    }
    out
}
