//! WASM bindings for transforms: FFT, DCT, wavelets, convolution, windows.

use wasm_bindgen::prelude::*;

use crate::transforms::{
    apply_window, blackman, conv_1d, conv_1d_same, dct2_forward, dct2_inverse, dwt_haar_forward,
    dwt_haar_inverse, fft_forward_real, fft_inverse, hamming, hann, tukey,
};

/// Forward real FFT. Input length must be power of 2.
/// Returns spectrum as [re0, im0, re1, im1, ...] (interleaved).
#[wasm_bindgen(js_name = fftForwardReal)]
pub fn fft_forward_real_wasm(signal: &[f64]) -> Result<Vec<f64>, JsError> {
    let spectrum = fft_forward_real(signal).map_err(|e| JsError::new(&e.to_string()))?;
    let mut out = Vec::with_capacity(spectrum.len() * 2);
    for c in spectrum {
        out.push(c.re);
        out.push(c.im);
    }
    Ok(out)
}

/// Inverse FFT. Input as interleaved [re0, im0, re1, im1, ...]. Returns real part.
#[wasm_bindgen(js_name = fftInverse)]
pub fn fft_inverse_wasm(spectrum: &[f64]) -> Result<Vec<f64>, JsError> {
    if spectrum.len() % 2 != 0 {
        return Err(JsError::new("spectrum must have even length (re,im pairs)"));
    }
    let complex: Vec<crate::transforms::Complex64> = spectrum
        .chunks_exact(2)
        .map(|c| crate::transforms::Complex64::new(c[0], c[1]))
        .collect();
    let inv = fft_inverse(&complex).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(inv.iter().map(|c| c.re).collect())
}

/// DCT-II forward.
#[wasm_bindgen(js_name = dct2Forward)]
pub fn dct2_forward_wasm(signal: &[f64]) -> Result<Vec<f64>, JsError> {
    dct2_forward(signal).map_err(|e| JsError::new(&e.to_string()))
}

/// DCT-III inverse.
#[wasm_bindgen(js_name = dct2Inverse)]
pub fn dct2_inverse_wasm(coeffs: &[f64]) -> Result<Vec<f64>, JsError> {
    dct2_inverse(coeffs).map_err(|e| JsError::new(&e.to_string()))
}

/// Haar DWT forward (even length).
#[wasm_bindgen(js_name = dwtHaarForward)]
pub fn dwt_haar_forward_wasm(signal: &[f64]) -> Vec<f64> {
    dwt_haar_forward(signal)
}

/// Haar DWT inverse.
#[wasm_bindgen(js_name = dwtHaarInverse)]
pub fn dwt_haar_inverse_wasm(coeffs: &[f64]) -> Vec<f64> {
    dwt_haar_inverse(coeffs)
}

/// 1D convolution (full).
#[wasm_bindgen(js_name = conv1d)]
pub fn conv_1d_wasm(signal: &[f64], kernel: &[f64]) -> Vec<f64> {
    conv_1d(signal, kernel)
}

/// 1D convolution same-length.
#[wasm_bindgen(js_name = conv1dSame)]
pub fn conv_1d_same_wasm(signal: &[f64], kernel: &[f64]) -> Vec<f64> {
    conv_1d_same(signal, kernel)
}

/// Hann window.
#[wasm_bindgen(js_name = hann)]
pub fn hann_wasm(len: usize) -> Vec<f64> {
    hann(len)
}

/// Hamming window.
#[wasm_bindgen(js_name = hamming)]
pub fn hamming_wasm(len: usize) -> Vec<f64> {
    hamming(len)
}

/// Blackman window.
#[wasm_bindgen(js_name = blackman)]
pub fn blackman_wasm(len: usize) -> Vec<f64> {
    blackman(len)
}

/// Tukey window. Alpha in [0, 1]: 0=rectangular, 1=Hann.
#[wasm_bindgen(js_name = tukey)]
pub fn tukey_wasm(len: usize, alpha: f64) -> Vec<f64> {
    tukey(len, alpha)
}

/// Apply window to signal. Returns windowed signal (signal[i] * window[i]).
#[wasm_bindgen(js_name = applyWindow)]
pub fn apply_window_wasm(signal: &[f64], window: &[f64]) -> Result<Vec<f64>, JsError> {
    if signal.len() != window.len() {
        return Err(JsError::new("signal and window must have same length"));
    }
    let mut out = vec![0.0; signal.len()];
    apply_window(signal, window, &mut out);
    Ok(out)
}
