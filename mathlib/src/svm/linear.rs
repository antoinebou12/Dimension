//! Linear SVM (binary classification) via dual formulation and SMO.
//!
//! Data: `X` rows = samples, cols = features. Labels `y`: ±1.

use crate::matrix::Matrix;
use crate::vector::Vector;
use std::fmt;
use tracing::debug;

/// Options for training a linear SVM.
#[derive(Clone, Debug)]
pub struct SvmOptions {
    /// Regularization parameter C (upper bound on `α_i`). Larger = less margin, fewer support vectors.
    pub c: f64,
    /// Maximum SMO iterations.
    pub max_iters: u32,
    /// Tolerance for KKT conditions (stop when violation < tol).
    pub tol: f64,
}

impl Default for SvmOptions {
    fn default() -> Self {
        Self {
            c: 1.0,
            max_iters: 10_000,
            tol: 1e-3,
        }
    }
}

/// Trained linear SVM: weight vector and bias for prediction `sign(w·x + b)`.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SvmResult {
    /// Weight vector (`n_features`).
    pub(crate) w: Vector<f64>,
    /// Bias term.
    pub(crate) b: f64,
}

impl SvmResult {
    /// Weight vector (same dimension as feature rows of training data).
    #[inline]
    pub fn weights(&self) -> &Vector<f64> {
        &self.w
    }

    /// Bias term.
    #[inline]
    pub fn bias(&self) -> f64 {
        self.b
    }

    /// Predict label for one sample: +1 or -1. `x` is a row of features (length `n_features`).
    #[must_use]
    pub fn predict_row(&self, x: &[f64]) -> f64 {
        let mut sum = self.b;
        for (i, &v) in x.iter().enumerate() {
            sum += self.w.get(i) * v;
        }
        if sum >= 0.0 { 1.0 } else { -1.0 }
    }

    /// Predict label for one sample given as matrix row index. `x` must have same number of cols as training data.
    #[must_use]
    pub fn predict_sample(&self, x: &Matrix<f64>, row: usize) -> f64 {
        let n = x.cols();
        let mut sum = self.b;
        for j in 0..n {
            sum += self.w.get(j) * x.get(row, j);
        }
        if sum >= 0.0 { 1.0 } else { -1.0 }
    }

    /// Predict labels for all rows of `x`. Returns a vector of ±1.
    #[must_use]
    pub fn predict(&self, x: &Matrix<f64>) -> Vec<f64> {
        (0..x.rows()).map(|i| self.predict_sample(x, i)).collect()
    }
}

/// Error from SVM training.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SvmError {
    /// Number of labels does not match number of samples.
    LabelLength,
    /// No samples or no features.
    EmptyData,
    /// Only one class present in labels (need both +1 and -1).
    SingleClass,
}

impl std::error::Error for SvmError {}

impl fmt::Display for SvmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SvmError::LabelLength => write!(f, "labels length does not match number of samples"),
            SvmError::EmptyData => write!(f, "empty data or zero features"),
            SvmError::SingleClass => write!(f, "only one class in labels (need both +1 and -1)"),
        }
    }
}

/// Dot product between row `i` and row `j` of matrix `x`.
fn row_dot(x: &Matrix<f64>, i: usize, j: usize) -> f64 {
    let n = x.cols();
    let mut sum = 0.0;
    for c in 0..n {
        sum += x.get(i, c) * x.get(j, c);
    }
    sum
}

/// Run linear SVM (binary classification). Labels in `y` are converted to ±1 (positive → 1, else -1).
/// Returns weight vector and bias for prediction `sign(w·x + b)`.
///
/// # Errors
///
/// Returns [`SvmError::LabelLength`] if `y.len() != x.rows()`.
/// Returns [`SvmError::EmptyData`] if no samples or no features.
/// Returns [`SvmError::SingleClass`] if all labels become the same after mapping to ±1.
#[allow(
    clippy::cast_precision_loss,
    clippy::similar_names,
    clippy::too_many_lines
)]
pub fn svm(x: &Matrix<f64>, y: &[f64], options: Option<SvmOptions>) -> Result<SvmResult, SvmError> {
    let opts = options.unwrap_or_default();
    let n = x.rows();
    let n_features = x.cols();
    if y.len() != n {
        return Err(SvmError::LabelLength);
    }
    if n == 0 || n_features == 0 {
        return Err(SvmError::EmptyData);
    }

    // Normalize labels to ±1
    let y: Vec<f64> = y
        .iter()
        .map(|&v| if v > 0.0 { 1.0 } else { -1.0 })
        .collect();
    let n_pos = y.iter().filter(|&&v| v > 0.0).count();
    if n_pos == 0 || n_pos == n {
        return Err(SvmError::SingleClass);
    }

    debug!(n, n_features, c = opts.c, "svm fit");

    // Linear kernel matrix k[i][j] = x_i · x_j
    let mut k = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            k[i * n + j] = row_dot(x, i, j);
        }
    }

    // Dual: min (1/2) α^T Q α - 1^T α,  Q_ij = y_i y_j K_ij,  s.t. 0 ≤ α ≤ C, Σ α_i y_i = 0
    let mut alpha = vec![0.0_f64; n];
    let mut b = 0.0_f64;

    // f_scores(i) = Σ_j α_j y_j k(i,j)  =>  f(x_i) = f_scores(i) + b
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
                // Select j != i (simplified: take best among a few random or first violation)
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

    // Reconstruct w = Σ α_i y_i x_i
    let mut w = Vector::with_capacity(n_features);
    w.resize(n_features);
    w.set_zero();
    for i in 0..n {
        if alpha[i] > 1e-12 {
            for j in 0..n_features {
                w.set(j, w.get(j) + alpha[i] * y[i] * x.get(i, j));
            }
        }
    }

    Ok(SvmResult { w, b })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::Storage;

    #[test]
    fn row_dot_symmetric_and_diagonal() {
        let mut x = Matrix::with_storage(2, 3, Storage::Column);
        x.set(0, 0, 1.0);
        x.set(0, 1, 2.0);
        x.set(0, 2, 3.0);
        x.set(1, 0, 4.0);
        x.set(1, 1, 5.0);
        x.set(1, 2, 6.0);
        let d01 = row_dot(&x, 0, 1);
        let d10 = row_dot(&x, 1, 0);
        assert!((d01 - d10).abs() < 1e-12, "row_dot symmetric");
        assert!((d01 - (1.0 * 4.0 + 2.0 * 5.0 + 3.0 * 6.0)).abs() < 1e-12);
        let d00 = row_dot(&x, 0, 0);
        assert!(d00 >= 0.0 && (d00 - (1.0 + 4.0 + 9.0)).abs() < 1e-12);
    }
}
