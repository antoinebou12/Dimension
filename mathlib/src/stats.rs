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
        use rayon::prelude::*;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_matrix(rows: usize, cols: usize, data: &[f64]) -> Matrix<f64> {
        let mut m = Matrix::with_storage(rows, cols, Storage::Column);
        for i in 0..rows {
            for j in 0..cols {
                m.set(i, j, data[i * cols + j]);
            }
        }
        m
    }

    #[test]
    fn test_covariance_2x2() {
        // Two samples, two features: [[1, 2], [3, 4]]
        // Mean = [2, 3], centered = [[-1, -1], [1, 1]]
        // cov[0,0] = ((-1)^2 + 1^2)/1 = 2, cov[0,1] = ((-1)*(-1) + 1*1)/1 = 2
        // cov[1,0] = 2, cov[1,1] = 2
        let data = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        let cov = covariance(&data);
        assert!((cov.get(0, 0) - 2.0).abs() < 1e-10);
        assert!((cov.get(0, 1) - 2.0).abs() < 1e-10);
        assert!((cov.get(1, 0) - 2.0).abs() < 1e-10);
        assert!((cov.get(1, 1) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_covariance_symmetric() {
        let data = make_matrix(
            5,
            3,
            &[
                1.0, 2.0, 3.0, 4.0, 5.0, 2.0, 4.0, 6.0, 8.0, 10.0, 0.0, 1.0, 0.0, 1.0, 0.0,
            ],
        );
        let cov = covariance(&data);
        for i in 0..3 {
            for j in 0..3 {
                assert!((cov.get(i, j) - cov.get(j, i)).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_covariance_positive_semi_definite() {
        // For any vector x, x^T C x >= 0 (sample covariance is PSD).
        let data = make_matrix(
            5,
            3,
            &[
                1.0, 2.0, 3.0, 4.0, 5.0, 2.0, 4.0, 6.0, 8.0, 10.0, 0.0, 1.0, 0.0, 1.0, 0.0,
            ],
        );
        let cov = covariance(&data);
        let test_vectors: [[f64; 3]; 4] = [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
            [2.0, -1.0, 0.5],
        ];
        for x in &test_vectors {
            let mut quad = 0.0;
            for i in 0..3 {
                for j in 0..3 {
                    quad += x[i] * cov.get(i, j) * x[j];
                }
            }
            assert!(quad >= -1e-10, "x^T C x = {} should be >= 0", quad);
        }
    }
}
