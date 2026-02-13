//! RBF (Radial Basis Function) kernel SVM for binary classification.
//!
//! Kernel K(a, b) = exp(-γ ‖a - b‖²) via [`crate::math::rbf::rbf_kernel`]. Stores support vectors
//! and dual coefficients; prediction is sign(Σ `α_i` `y_i` `K(sv_i, x)` + b).

use super::linear::{SvmError, SvmOptions};
use crate::distance::squared_euclidean_rows;
use crate::math::rbf::rbf_kernel;
use crate::matrix::Matrix;
use tracing::debug;

/// Trained RBF-kernel SVM: support vectors, dual coefficients, bias, and γ.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SvmRbfResult {
    /// Support vectors: `n_sv` rows × `n_features`.
    pub(crate) support_vectors: Matrix<f64>,
    /// For each support vector, `α_i` * `y_i`.
    pub(crate) alpha_y: Vec<f64>,
    /// Bias term.
    pub(crate) b: f64,
    /// RBF kernel parameter γ.
    pub(crate) gamma: f64,
}

impl SvmRbfResult {
    /// Number of support vectors.
    #[inline]
    pub fn n_support_vectors(&self) -> usize {
        self.alpha_y.len()
    }

    /// Support vectors matrix (`n_sv` × `n_features`).
    #[inline]
    pub fn support_vectors(&self) -> &Matrix<f64> {
        &self.support_vectors
    }

    /// Bias term.
    #[inline]
    pub fn bias(&self) -> f64 {
        self.b
    }

    /// RBF kernel parameter γ.
    #[inline]
    pub fn gamma(&self) -> f64 {
        self.gamma
    }

    /// Predict label for one sample: +1 or -1. `x` is a row of features (length `n_features`).
    #[must_use]
    pub fn predict_row(&self, x: &[f64]) -> f64 {
        let sum = self.decision_row(x);
        if sum >= 0.0 { 1.0 } else { -1.0 }
    }

    /// Predict label for one sample given as matrix row index.
    #[must_use]
    pub fn predict_sample(&self, x: &Matrix<f64>, row: usize) -> f64 {
        let sum = self.decision_sample(x, row);
        if sum >= 0.0 { 1.0 } else { -1.0 }
    }

    /// Predict labels for all rows of `x`. Returns a vector of ±1.
    #[must_use]
    pub fn predict(&self, x: &Matrix<f64>) -> Vec<f64> {
        (0..x.rows()).map(|i| self.predict_sample(x, i)).collect()
    }

    /// Decision function (before sign) for a feature row slice.
    fn decision_row(&self, x: &[f64]) -> f64 {
        let n_sv = self.alpha_y.len();
        let n_features = self.support_vectors.cols();
        let mut sum = self.b;
        for sv in 0..n_sv {
            let mut dist_sq = 0.0;
            for (c, &xc) in x.iter().enumerate().take(n_features) {
                let d = self.support_vectors.get(sv, c) - xc;
                dist_sq += d * d;
            }
            sum += self.alpha_y[sv] * rbf_kernel(dist_sq, self.gamma);
        }
        sum
    }

    /// Decision function (before sign) for a matrix row.
    fn decision_sample(&self, x: &Matrix<f64>, row: usize) -> f64 {
        let n_sv = self.alpha_y.len();
        let n_features = self.support_vectors.cols();
        let mut sum = self.b;
        for sv in 0..n_sv {
            let mut dist_sq = 0.0;
            for c in 0..n_features {
                let d = self.support_vectors.get(sv, c) - x.get(row, c);
                dist_sq += d * d;
            }
            sum += self.alpha_y[sv] * rbf_kernel(dist_sq, self.gamma);
        }
        sum
    }
}

/// Run RBF-kernel SVM (binary classification). Labels in `y` are converted to ±1.
/// Returns support vectors, dual coefficients, bias, and γ for prediction.
///
/// # Errors
///
/// Same as [`super::svm`]: [`SvmError::LabelLength`], [`SvmError::EmptyData`], [`SvmError::SingleClass`].
#[allow(
    clippy::cast_precision_loss,
    clippy::similar_names,
    clippy::too_many_lines
)]
pub fn svm_rbf(
    x: &Matrix<f64>,
    y: &[f64],
    gamma: f64,
    options: Option<SvmOptions>,
) -> Result<SvmRbfResult, SvmError> {
    let opts = options.unwrap_or_default();
    let n = x.rows();
    let n_features = x.cols();
    if y.len() != n {
        return Err(SvmError::LabelLength);
    }
    if n == 0 || n_features == 0 {
        return Err(SvmError::EmptyData);
    }

    let y: Vec<f64> = y
        .iter()
        .map(|&v| if v > 0.0 { 1.0 } else { -1.0 })
        .collect();
    let n_pos = y.iter().filter(|&&v| v > 0.0).count();
    if n_pos == 0 || n_pos == n {
        return Err(SvmError::SingleClass);
    }

    debug!(n, n_features, gamma, c = opts.c, "svm_rbf fit");

    // RBF kernel matrix k[i][j] = exp(-gamma * ||x_i - x_j||^2)
    let mut k = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            let dist_sq = squared_euclidean_rows(x, i, j);
            k[i * n + j] = rbf_kernel(dist_sq, gamma);
        }
    }

    let mut alpha = vec![0.0_f64; n];
    let mut b = 0.0_f64;
    let mut f_scores: Vec<f64> = vec![0.0; n];

    let mut iter = 0u32;
    while iter < opts.max_iters {
        iter += 1;
        let mut num_changed = 0u32;

        for i in 0..n {
            let e_i = f_scores[i] + b - y[i];
            let kkt_ok = if y[i] > 0.0 {
                if alpha[i] < opts.c {
                    e_i >= -opts.tol
                } else {
                    e_i <= opts.tol
                }
            } else if alpha[i] > 0.0 {
                e_i <= opts.tol
            } else {
                e_i >= -opts.tol
            };
            if !kkt_ok {
                let mut j = 1 - i.min(1);
                let mut max_delta = 0.0_f64;
                for jj in 0..n {
                    if jj == i {
                        continue;
                    }
                    let e_jj = f_scores[jj] + b - y[jj];
                    let delta = (e_i - e_jj).abs();
                    if delta > max_delta {
                        max_delta = delta;
                        j = jj;
                    }
                }

                let e_j = f_scores[j] + b - y[j];
                let old_ai = alpha[i];
                let old_aj = alpha[j];
                let eta = 2.0 * k[i * n + j] - k[i * n + i] - k[j * n + j];
                if eta >= -1e-12 {
                    continue;
                }
                let mut new_aj = old_aj - y[j] * (e_i - e_j) / eta;
                let (low, high) = if (y[i] - y[j]).abs() < 0.5 {
                    let l = (old_ai + old_aj - opts.c).max(0.0);
                    let h = (old_ai + old_aj).min(opts.c);
                    (l, h)
                } else {
                    let l = (old_aj - old_ai).max(0.0);
                    let h = (opts.c + old_aj - old_ai).min(opts.c);
                    (l, h)
                };
                if low >= high {
                    continue;
                }
                new_aj = new_aj.clamp(low, high);
                let delta_aj = new_aj - old_aj;
                if delta_aj.abs() < 1e-12 {
                    continue;
                }
                let new_ai = old_ai + y[i] * y[j] * (old_aj - new_aj);
                alpha[i] = new_ai;
                alpha[j] = new_aj;

                for idx in 0..n {
                    f_scores[idx] += y[i] * (new_ai - old_ai) * k[i * n + idx]
                        + y[j] * (new_aj - old_aj) * k[j * n + idx];
                }

                let b1 = b
                    - e_i
                    - y[i] * (new_ai - old_ai) * k[i * n + i]
                    - y[j] * (new_aj - old_aj) * k[i * n + j];
                let b2 = b
                    - e_j
                    - y[i] * (new_ai - old_ai) * k[i * n + j]
                    - y[j] * (new_aj - old_aj) * k[j * n + j];
                b = if new_ai > 1e-12 && new_ai < opts.c - 1e-12 {
                    b1
                } else if new_aj > 1e-12 && new_aj < opts.c - 1e-12 {
                    b2
                } else {
                    (b1 + b2) * 0.5
                };
                num_changed += 1;
            }
        }
        if num_changed == 0 {
            break;
        }
    }

    // Collect support vectors (alpha_i > eps) and alpha_y
    let eps = 1e-12_f64;
    let mut support_vectors = Vec::with_capacity(n);
    let mut alpha_y = Vec::with_capacity(n);
    for i in 0..n {
        if alpha[i] > eps {
            for c in 0..n_features {
                support_vectors.push(x.get(i, c));
            }
            alpha_y.push(alpha[i] * y[i]);
        }
    }
    let n_sv = alpha_y.len();
    let mut sv_matrix = Matrix::with_storage(n_sv, n_features, crate::structure::Storage::Column);
    for (sv, chunk) in support_vectors.chunks(n_features).enumerate() {
        for (c, &v) in chunk.iter().enumerate() {
            sv_matrix.set(sv, c, v);
        }
    }

    Ok(SvmRbfResult {
        support_vectors: sv_matrix,
        alpha_y,
        b,
        gamma,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::Storage;

    #[test]
    fn rbf_kernel_diagonal_one() {
        let mut x = Matrix::with_storage(2, 2, Storage::Column);
        x.set(0, 0, 1.0);
        x.set(0, 1, 0.0);
        x.set(1, 0, 0.0);
        x.set(1, 1, 1.0);
        let d00 = squared_euclidean_rows(&x, 0, 0);
        let d11 = squared_euclidean_rows(&x, 1, 1);
        assert!(d00.abs() < 1e-10);
        assert!(d11.abs() < 1e-10);
        let k00 = rbf_kernel(d00, 0.5);
        assert!((k00 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rbf_kernel_symmetric() {
        let mut x = Matrix::with_storage(3, 2, Storage::Column);
        for i in 0..3 {
            for j in 0..2 {
                #[allow(clippy::cast_precision_loss)]
                let val = (i as f64) + (j as f64) * 0.5;
                x.set(i, j, val);
            }
        }
        for i in 0..3 {
            for j in 0..3 {
                let d_ij = squared_euclidean_rows(&x, i, j);
                let d_ji = squared_euclidean_rows(&x, j, i);
                assert!(
                    (d_ij - d_ji).abs() < 1e-12,
                    "squared_euclidean_rows symmetric"
                );
            }
        }
    }

    #[test]
    fn svm_rbf_tiny_fit() {
        let mut x = Matrix::with_storage(4, 2, Storage::Column);
        x.set(0, 0, 0.0);
        x.set(0, 1, 0.0);
        x.set(1, 0, 1.0);
        x.set(1, 1, 0.0);
        x.set(2, 0, 0.0);
        x.set(2, 1, 1.0);
        x.set(3, 0, 1.0);
        x.set(3, 1, 1.0);
        let y = [1.0, 1.0, -1.0, -1.0];
        let result = svm_rbf(&x, &y, 0.5, None).unwrap();
        assert!(result.n_support_vectors() >= 1);
        let pred = result.predict(&x);
        for (i, &label) in y.iter().enumerate() {
            assert!(
                (pred[i] - label).abs() < 1e-10,
                "sample {} predicted {} expected {}",
                i,
                pred[i],
                label
            );
        }
    }
}
