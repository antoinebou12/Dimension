//! Principal Component Analysis via SVD of the centered data matrix.
//!
//! Data layout: rows = samples, cols = features. Column means are subtracted
//! before SVD. Principal components are the right singular vectors (columns of V).

use crate::matrix::Matrix;
use crate::svd::svd_econ;
use crate::types::Storage;
use crate::vector::Vector;
use std::fmt;

/// Result of PCA: mean, components, and explained variance.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pca {
    /// Column means (length = `n_features`).
    pub(crate) mean: Vector<f64>,
    /// Principal components as columns (`n_features` × `n_components`).
    pub(crate) components: Matrix<f64>,
    /// Explained variance per component (length = `n_components`).
    pub(crate) explained_variance: Vector<f64>,
}

impl Pca {
    /// Mean vector (one per feature).
    #[inline]
    pub fn mean(&self) -> &Vector<f64> {
        &self.mean
    }

    /// Principal components matrix (features × components); each column is a PC.
    #[inline]
    pub fn components(&self) -> &Matrix<f64> {
        &self.components
    }

    /// Explained variance for each component (sigma² from SVD, optionally normalized).
    #[inline]
    pub fn explained_variance(&self) -> &Vector<f64> {
        &self.explained_variance
    }

    /// Number of components.
    #[inline]
    pub fn n_components(&self) -> usize {
        self.components.cols()
    }
}

/// Run PCA on data matrix `data` (rows = samples, cols = features).
/// Centers the data, then performs economical SVD. Returns mean, components (columns of V),
/// and explained variance (sigma²). If `n_components` is `Some(k)`, only the first `k`
/// components are retained; otherwise all `min(n_samples, n_features)` components are kept.
#[allow(clippy::cast_precision_loss)]
pub fn pca(data: &Matrix<f64>, n_components: Option<usize>) -> Pca {
    let (n_samples, n_features) = (data.rows(), data.cols());
    assert!(
        n_samples >= 1 && n_features >= 1,
        "PCA requires at least one sample and one feature"
    );

    let mut centered = Matrix::with_storage(n_samples, n_features, Storage::Column);
    let mut mean = Vector::with_capacity(n_features);
    mean.set_zero();

    for j in 0..n_features {
        let mut s = 0.0;
        for i in 0..n_samples {
            s += data.get(i, j);
        }
        let m = s / (n_samples as f64);
        mean.set(j, m);
        for i in 0..n_samples {
            centered.set(i, j, data.get(i, j) - m);
        }
    }

    let econ = svd_econ(&centered);
    let sigma = econ.sigma();
    let v = econ.v();
    let k_full = sigma.rows();
    let k = n_components.map_or(k_full, |n| n.min(k_full));

    let mut components = Matrix::with_storage(n_features, k, Storage::Column);
    let mut explained_variance = Vector::with_capacity(k);
    explained_variance.set_zero();

    let n = n_samples as f64;
    for j in 0..k {
        let s = sigma.get(j);
        explained_variance.set(j, s * s / n);
        for i in 0..n_features {
            components.set(i, j, v.get(i, j));
        }
    }

    Pca {
        mean,
        components,
        explained_variance,
    }
}

impl fmt::Display for Pca {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Pca mean len={} components {}x{} explained_variance len={}",
            self.mean.rows(),
            self.components.rows(),
            self.components.cols(),
            self.explained_variance.rows()
        )
    }
}
