//! Spectral window functions for FFT and spectral analysis.
//!
//! Windows reduce spectral leakage when applied to signals before FFT.

use std::f64::consts::PI;

/// Hann (Hanning) window: w[n] = 0.5 * (1 - cos(2πn/(N-1))).
/// For N=1 returns `[1.0]`.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn hann(len: usize) -> Vec<f64> {
    if len == 0 {
        return Vec::new();
    }
    if len == 1 {
        return vec![1.0];
    }
    let scale = 2.0 * PI / (len - 1) as f64;
    (0..len)
        .map(|n| 0.5 * (1.0 - (scale * n as f64).cos()))
        .collect()
}

/// Hamming window: w[n] = 0.54 - 0.46*cos(2πn/(N-1)).
/// For N=1 returns `[1.0]`.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn hamming(len: usize) -> Vec<f64> {
    if len == 0 {
        return Vec::new();
    }
    if len == 1 {
        return vec![1.0];
    }
    let scale = 2.0 * PI / (len - 1) as f64;
    (0..len)
        .map(|n| 0.54 - 0.46 * (scale * n as f64).cos())
        .collect()
}

/// Tukey (tapered cosine) window. `alpha` in [0, 1]: 0 = rectangular, 1 = Hann.
/// Default alpha 0.5 balances flat top and smooth edges. For N=1 returns `[1.0]`.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn tukey(len: usize, alpha: f64) -> Vec<f64> {
    if len == 0 {
        return Vec::new();
    }
    if len == 1 {
        return vec![1.0];
    }
    let n = (len - 1) as f64;
    let alpha = alpha.clamp(0.0, 1.0);
    if alpha <= 0.0 {
        return vec![1.0; len];
    }
    let mut out = vec![1.0; len];
    let half_alpha = alpha / 2.0;
    for (i, out_i) in out.iter_mut().enumerate().take(len) {
        let x = i as f64 / n;
        if x < half_alpha {
            *out_i = 0.5 * (1.0 + (2.0 * PI / alpha * (x - half_alpha)).cos());
        } else if x > 1.0 - half_alpha {
            *out_i = 0.5 * (1.0 + (2.0 * PI / alpha * (x - 1.0 + half_alpha)).cos());
        }
    }
    out
}

/// Blackman window: w[n] = 0.42 - 0.5*cos(2πn/(N-1)) + 0.08*cos(4πn/(N-1)).
/// For N=1 returns `[1.0]`.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn blackman(len: usize) -> Vec<f64> {
    if len == 0 {
        return Vec::new();
    }
    if len == 1 {
        return vec![1.0];
    }
    let scale = 2.0 * PI / (len - 1) as f64;
    (0..len)
        .map(|n| {
            let x = scale * n as f64;
            0.42 - 0.5 * x.cos() + 0.08 * (2.0 * x).cos()
        })
        .collect()
}

/// Applies a window to a signal: `out[i] = signal[i] * window[i]`.
///
/// # Panics
///
/// Panics if lengths differ.
pub fn apply_window(signal: &[f64], window: &[f64], out: &mut [f64]) {
    assert_eq!(signal.len(), window.len());
    assert_eq!(signal.len(), out.len());
    for (i, (&s, &w)) in signal.iter().zip(window.iter()).enumerate() {
        out[i] = s * w;
    }
}

/// Applies a window in place: `signal[i] *= window[i]`.
///
/// # Panics
///
/// Panics if lengths differ.
pub fn apply_window_in_place(signal: &mut [f64], window: &[f64]) {
    assert_eq!(signal.len(), window.len());
    for (s, &w) in signal.iter_mut().zip(window.iter()) {
        *s *= w;
    }
}
