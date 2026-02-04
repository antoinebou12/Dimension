//! Statistical functions: covariance, etc.

use crate::cpu;
use crate::matrix::Matrix;
use crate::types::Storage;
use tracing::debug;

/// Compute sample covariance matrix of data (rows = samples, cols = features).
/// Returns `n_features` × `n_features` matrix. Uses Bessel correction (n-1).
///
/// Uses `cpu::dot_f64` for each inner product, which dispatches to simd/parallel
/// backends when those features are enabled.
///
/// # Panics
///
/// Panics if `n_samples < 2`.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn covariance(data: &Matrix<f64>) -> Matrix<f64> {
    let (n_samples, n_features) = (data.rows(), data.cols());
    debug!(rows = n_samples, cols = n_features, "covariance");
    assert!(n_samples >= 2, "covariance requires at least 2 samples");

    let mut centered = Matrix::with_storage(n_samples, n_features, Storage::Column);
    for j in 0..n_features {
        let mut s = 0.0;
        for i in 0..n_samples {
            s += data.get(i, j);
        }
        let m = s / (n_samples as f64);
        for i in 0..n_samples {
            centered.set(i, j, data.get(i, j) - m);
        }
    }

    let scale = 1.0 / ((n_samples - 1) as f64);
    let data = centered.data();

    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    {
        use par_iter::prelude::*;
        let mut cov = Matrix::with_storage(n_features, n_features, Storage::Column);
        cov.set_zero();
        let rows: Vec<(usize, Vec<f64>)> = (0..n_features)
            .into_par_iter()
            .map(|i| {
                let col_i = &data[i * n_samples..(i + 1) * n_samples];
                let row: Vec<f64> = (0..n_features)
                    .map(|j| {
                        let col_j = &data[j * n_samples..(j + 1) * n_samples];
                        cpu::dot_f64(col_i, col_j) * scale
                    })
                    .collect();
                (i, row)
            })
            .collect();
        for (i, row) in rows {
            for (j, &v) in row.iter().enumerate() {
                cov.set(i, j, v);
            }
        }
        cov
    }

    #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
    {
        let mut cov = Matrix::with_storage(n_features, n_features, Storage::Column);
        cov.set_zero();
        for i in 0..n_features {
            let col_i = &data[i * n_samples..(i + 1) * n_samples];
            for j in 0..n_features {
                let col_j = &data[j * n_samples..(j + 1) * n_samples];
                let val = cpu::dot_f64(col_i, col_j);
                cov.set(i, j, val * scale);
            }
        }
        cov
    }
}
