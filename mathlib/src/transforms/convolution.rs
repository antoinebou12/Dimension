//! 1D and 2D convolution.
//!
//! WASM compatible. Optional parallel outer loop when `parallel` feature enabled (non-wasm).

use crate::matrix::Matrix;
use crate::structure::Storage;

/// Full 1D convolution: output length = `signal.len()` + `kernel.len()` - 1.
///
/// (signal * kernel)[n] = `sum_m` signal[m] * kernel[n-m]. Zero-pads at boundaries.
#[must_use]
pub fn conv_1d(signal: &[f64], kernel: &[f64]) -> Vec<f64> {
    if signal.is_empty() || kernel.is_empty() {
        return Vec::new();
    }
    let out_len = signal.len() + kernel.len() - 1;
    let kl = kernel.len();
    let sl = signal.len();
    let mut out = vec![0.0; out_len];

    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    {
        use par_iter::prelude::*;
        out.par_iter_mut().enumerate().for_each(|(n, o)| {
            let m_start = n.saturating_sub(kl - 1);
            let m_end = (n + 1).min(sl);
            let mut sum = 0.0;
            for m in m_start..m_end {
                sum += signal[m] * kernel[n - m];
            }
            *o = sum;
        });
    }

    #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
    {
        for n in 0..out_len {
            let m_start = n.saturating_sub(kl - 1);
            let m_end = (n + 1).min(sl);
            let mut sum = 0.0;
            for m in m_start..m_end {
                sum += signal[m] * kernel[n - m];
            }
            out[n] = sum;
        }
    }

    out
}

/// Same-length 1D convolution: output length = `signal.len()`, centered.
#[must_use]
pub fn conv_1d_same(signal: &[f64], kernel: &[f64]) -> Vec<f64> {
    if signal.is_empty() || kernel.is_empty() {
        return Vec::new();
    }
    let full = conv_1d(signal, kernel);
    let pad = (kernel.len() - 1) / 2;
    full[pad..pad + signal.len()].to_vec()
}

/// 2D convolution with "same" padding (output same size as input).
///
/// Kernel must have odd dimensions.
#[must_use]
pub fn conv_2d(matrix: &Matrix<f64>, kernel: &Matrix<f64>) -> Matrix<f64> {
    let rows = matrix.rows();
    let cols = matrix.cols();
    let kr = kernel.rows();
    let kc = kernel.cols();
    assert!(
        kr % 2 == 1 && kc % 2 == 1,
        "2D convolution requires odd-sized kernel"
    );
    let hr = kr / 2;
    let hc = kc / 2;

    let mut out = Matrix::with_storage(rows, cols, Storage::Column);

    for i in 0..rows {
        for j in 0..cols {
            let mut sum = 0.0;
            for ki in 0..kr {
                for kj in 0..kc {
                    #[allow(
                        clippy::cast_possible_truncation,
                        clippy::cast_possible_wrap,
                        clippy::cast_sign_loss
                    )]
                    let val = {
                        let ri = i as i32 + ki as i32 - hr as i32;
                        let cj = j as i32 + kj as i32 - hc as i32;
                        if ri >= 0 && ri < rows as i32 && cj >= 0 && cj < cols as i32 {
                            matrix.get(ri as usize, cj as usize)
                        } else {
                            0.0
                        }
                    };
                    sum += val * kernel.get(ki, kj);
                }
            }
            out.set(i, j, sum);
        }
    }
    out
}
