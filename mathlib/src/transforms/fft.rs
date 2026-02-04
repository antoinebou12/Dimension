//! Fast Fourier Transform (Cooley-Tukey radix-2).
//!
//! Requires power-of-2 input length. Works on wasm32.

use crate::transforms::TransformsError;
use crate::transforms::complex::Complex64;
use std::f64::consts::PI;

/// Forward FFT (complex-to-complex).
///
/// # Errors
///
/// Returns `TransformsError::LengthNotPowerOfTwo` if `in_` length is not a power of 2.
/// Returns `TransformsError::EmptyInput` if input is empty.
#[allow(clippy::missing_panics_doc)]
pub fn fft_forward(in_: &[Complex64]) -> Result<Vec<Complex64>, TransformsError> {
    let n = in_.len();
    if n == 0 {
        return Err(TransformsError::EmptyInput);
    }
    if !n.is_power_of_two() {
        return Err(TransformsError::LengthNotPowerOfTwo(n));
    }
    let mut x: Vec<Complex64> = in_.to_vec();
    fft_forward_in_place(&mut x);
    Ok(x)
}

/// Inverse FFT (complex-to-complex), scaled by 1/N.
///
/// # Errors
///
/// Returns `TransformsError::LengthNotPowerOfTwo` if length is not a power of 2.
/// Returns `TransformsError::EmptyInput` if input is empty.
#[allow(clippy::cast_precision_loss, clippy::missing_panics_doc)]
pub fn fft_inverse(in_: &[Complex64]) -> Result<Vec<Complex64>, TransformsError> {
    let n = in_.len();
    if n == 0 {
        return Err(TransformsError::EmptyInput);
    }
    if !n.is_power_of_two() {
        return Err(TransformsError::LengthNotPowerOfTwo(n));
    }
    let mut x: Vec<Complex64> = in_.iter().map(|c| c.conjugate()).collect();
    fft_forward_in_place(&mut x);
    let scale = 1.0 / n as f64;
    for c in &mut *x {
        *c = c.conjugate() * scale;
    }
    Ok(x)
}

/// Forward FFT for real-valued signal: packs as complex and runs FFT.
/// Output has length N (first N/2+1 bins are unique for real input due to symmetry).
///
/// # Errors
///
/// Returns `TransformsError::LengthNotPowerOfTwo` if length is not a power of 2.
/// Returns `TransformsError::EmptyInput` if input is empty.
pub fn fft_forward_real(signal: &[f64]) -> Result<Vec<Complex64>, TransformsError> {
    let n = signal.len();
    if n == 0 {
        return Err(TransformsError::EmptyInput);
    }
    if !n.is_power_of_two() {
        return Err(TransformsError::LengthNotPowerOfTwo(n));
    }
    let complex: Vec<Complex64> = signal.iter().map(|&re| Complex64::new(re, 0.0)).collect();
    fft_forward(&complex)
}

fn fft_forward_in_place(x: &mut [Complex64]) {
    let n = x.len();
    if n <= 1 {
        return;
    }
    // Bit-reversal permutation (n is power of 2 from caller)
    let bits = n.trailing_zeros();
    for i in 0..n {
        let j = bit_reverse(i, bits);
        if i < j {
            x.swap(i, j);
        }
    }
    // Cooley-Tukey radix-2
    let mut size = 2usize;
    while size <= n {
        let half = size / 2;
        #[allow(clippy::cast_precision_loss)]
        let angle = -2.0 * PI / size as f64;
        for start in (0..n).step_by(size) {
            let w_m = Complex64::exp_i(angle);
            let mut w = Complex64::ONE;
            for j in 0..half {
                let u = x[start + j];
                let t = w * x[start + j + half];
                x[start + j] = u + t;
                x[start + j + half] = u - t;
                w = w * w_m;
            }
        }
        size *= 2;
    }
}

#[inline]
fn bit_reverse(mut x: usize, bits: u32) -> usize {
    let mut r = 0usize;
    for _ in 0..bits {
        r = (r << 1) | (x & 1);
        x >>= 1;
    }
    r
}
