//! Discrete Cosine Transform (DCT-II and DCT-III).
//!
//! DCT-III is the inverse of DCT-II. Orthonormal normalization.

use crate::transforms::TransformsError;
use std::f64::consts::PI;

/// DCT-II: X[k] = `sum_n` x[n] * cos(π k (2n+1) / (2N)).
/// Orthonormal: scale sqrt(2/N) for k>0, sqrt(1/N) for k=0.
#[allow(clippy::missing_panics_doc)]
pub fn dct2_forward(signal: &[f64]) -> Result<Vec<f64>, TransformsError> {
    let n = signal.len();
    if n == 0 {
        return Err(TransformsError::EmptyInput);
    }
    Ok(dct2_forward_direct(signal))
}

/// DCT-III (inverse of DCT-II).
#[allow(clippy::missing_panics_doc)]
pub fn dct2_inverse(coeffs: &[f64]) -> Result<Vec<f64>, TransformsError> {
    let n = coeffs.len();
    if n == 0 {
        return Err(TransformsError::EmptyInput);
    }
    Ok(dct2_inverse_direct(coeffs))
}

#[allow(clippy::cast_precision_loss)]
fn dct2_forward_direct(signal: &[f64]) -> Vec<f64> {
    let n = signal.len();
    let mut out = Vec::with_capacity(n);
    let scale = (2.0 / n as f64).sqrt();
    for k in 0..n {
        let mut sum = 0.0;
        let kf = k as f64;
        for (i, &x) in signal.iter().enumerate() {
            let nf = i as f64;
            sum += x * (PI * kf * (2.0 * nf + 1.0) / (2.0 * n as f64)).cos();
        }
        let factor = if k == 0 { 1.0 / (2.0_f64).sqrt() } else { 1.0 };
        out.push(sum * scale * factor);
    }
    out
}

#[allow(clippy::cast_precision_loss)]
fn dct2_inverse_direct(coeffs: &[f64]) -> Vec<f64> {
    let n = coeffs.len();
    let mut out = Vec::with_capacity(n);
    let scale = (2.0 / n as f64).sqrt();
    for i in 0..n {
        let mut sum = 0.0;
        let nf = i as f64;
        for (k, &x) in coeffs.iter().enumerate() {
            let kf = k as f64;
            let factor = if k == 0 { 1.0 / (2.0_f64).sqrt() } else { 1.0 };
            sum += x * factor * (PI * kf * (2.0 * nf + 1.0) / (2.0 * n as f64)).cos();
        }
        out.push(sum * scale);
    }
    out
}
