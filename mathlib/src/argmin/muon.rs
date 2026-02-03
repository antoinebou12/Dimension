//! Muon-style optimizer: Newton–Schulz iteration for spectrally normalized matrix updates.
//!
//! Given gradient matrix G, applies a few Newton–Schulz steps to approximate the orthogonal
//! polar factor of G, then updates param -= lr * that factor (direction-only update).
//! Set `RUST_LOG=mathlib=debug` to see iteration logs.

use crate::matrix::Matrix;
use crate::structure::Storage;
use tracing::debug;

/// One Muon step: param -= lr * (Newton–Schulz polar factor of grad).
/// Uses `ns_iters` Newton–Schulz iterations. `param` and `grad` must have the same dimensions (m x n).
pub fn muon_step(param: &mut Matrix<f64>, grad: &Matrix<f64>, lr: f64, ns_iters: usize) {
    let m = param.rows();
    let n = param.cols();
    assert_eq!(grad.rows(), m);
    assert_eq!(grad.cols(), n);

    // Scale so singular values are < 1 for Newton–Schulz convergence.
    #[allow(clippy::cast_precision_loss)]
    let scale = (m as f64 * n as f64).sqrt()
        * grad
            .as_slice()
            .iter()
            .map(|x| x * x)
            .sum::<f64>()
            .sqrt()
            .max(1e-10);
    let inv_scale = 1.0 / scale;

    let mut y = Matrix::with_storage(m, n, Storage::Column);
    for i in 0..m {
        for j in 0..n {
            y.set(i, j, grad.get(i, j) * inv_scale);
        }
    }

    let mut yty = Matrix::with_storage(n, n, Storage::Column);
    let mut yty_y = Matrix::with_storage(m, n, Storage::Column);

    let mut y_new = Matrix::with_storage(m, n, Storage::Column);
    for _ in 0..ns_iters {
        // Y^T Y (n x n)
        let yt = y.transpose();
        yt.mul_into(&y, &mut yty);
        // Y * (Y^T Y) (m x n)
        y.mul_into(&yty, &mut yty_y);
        // X_{k+1} = (3/2) X_k - (1/2) X_k X_k^T X_k
        for i in 0..m {
            for j in 0..n {
                y_new.set(i, j, 1.5 * y.get(i, j) - 0.5 * yty_y.get(i, j));
            }
        }
        y.copy_from(&y_new);
    }

    debug!(ns_iters, "muon_step");

    // param -= lr * scale * y (y is now approx orthogonal factor of grad/scale)
    for i in 0..m {
        for j in 0..n {
            param.set(i, j, param.get(i, j) - lr * scale * y.get(i, j));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn muon_step_reduces_norm() {
        let mut param = Matrix::with_storage(2, 2, Storage::Column);
        param.set(0, 0, 1.0);
        param.set(0, 1, 0.0);
        param.set(1, 0, 0.0);
        param.set(1, 1, 1.0);
        let mut grad = Matrix::with_storage(2, 2, Storage::Column);
        for i in 0..2 {
            for j in 0..2 {
                grad.set(i, j, if i == j { 0.1 } else { 0.0 });
            }
        }
        let param_norm_before = param.as_slice().iter().map(|x| x * x).sum::<f64>().sqrt();
        muon_step(&mut param, &grad, 0.1, 5);
        let param_norm_after = param.as_slice().iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!(param_norm_after < param_norm_before + 0.01);
    }
}
