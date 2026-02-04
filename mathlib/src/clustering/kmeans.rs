//! K-means clustering (sequential; optional parallel assignment when `parallel` feature is enabled).
//!
//! Data layout: rows = samples, cols = features. Centroids are stored as k rows × `n_features`.

use crate::matrix::Matrix;
use crate::types::Storage;
use std::fmt;
use tracing::{debug, info};

/// Result of K-means: cluster label per point and centroid matrix.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KmeansResult {
    /// labels[i] = cluster index (0..k) for row i.
    pub(crate) labels: Vec<usize>,
    /// Centroids: k rows × `n_features` (same layout as data).
    pub(crate) centroids: Matrix<f64>,
}

impl KmeansResult {
    #[inline]
    pub fn labels(&self) -> &[usize] {
        &self.labels
    }

    #[inline]
    pub fn centroids(&self) -> &Matrix<f64> {
        &self.centroids
    }

    #[inline]
    pub fn n_clusters(&self) -> usize {
        self.centroids.rows()
    }
}

/// Run K-means on `data` (rows = samples, cols = features). Uses first k rows as initial centroids.
/// Returns labels and centroids. Stops when assignments do not change or after `max_iters` (None = 300).
#[allow(clippy::cast_precision_loss)]
pub fn kmeans(data: &Matrix<f64>, k: usize, max_iters: Option<u32>) -> KmeansResult {
    let (n_samples, n_features) = (data.rows(), data.cols());
    assert!(n_samples >= k && k >= 1 && n_features >= 1);
    let max_iters = max_iters.unwrap_or(300);
    debug!(n_samples, n_features, k, max_iters, "kmeans");

    // Step 1: Initialize centroids (first k rows).
    let mut centroids = Matrix::with_storage(k, n_features, Storage::Column);
    for i in 0..k {
        for j in 0..n_features {
            centroids.set(i, j, data.get(i, j));
        }
    }

    let eps_sq = 1e-20_f64;
    let mut labels = vec![0usize; n_samples];
    let mut iterations = 0u32;

    for iter in 0..max_iters {
        iterations = iter + 1;
        // Step 2a: Assign each point to nearest centroid (parallel over points when feature enabled).
        let new_labels = assign_points_to_centroids(data, &centroids, n_samples, k, n_features);

        // Check convergence.
        let mut changed = false;
        for (i, &label_i) in labels.iter().enumerate().take(n_samples) {
            if new_labels[i] != label_i {
                changed = true;
                break;
            }
        }
        labels.copy_from_slice(&new_labels);
        if !changed {
            break;
        }

        // Step 2b: Recompute centroids as mean of assigned points.
        let mut counts = vec![0usize; k];
        centroids.set_zero();
        for (i, &c) in labels.iter().enumerate().take(n_samples) {
            counts[c] += 1;
            for j in 0..n_features {
                centroids.set(c, j, centroids.get(c, j) + data.get(i, j));
            }
        }
        for (c, &count) in counts.iter().enumerate().take(k) {
            let n = count as f64;
            if n > eps_sq {
                for j in 0..n_features {
                    centroids.set(c, j, centroids.get(c, j) / n);
                }
            }
        }
    }

    info!(iterations, "kmeans converged");
    KmeansResult { labels, centroids }
}

/// Assign each row to the centroid with smallest squared Euclidean distance.
fn assign_points_to_centroids(
    data: &Matrix<f64>,
    centroids: &Matrix<f64>,
    n_samples: usize,
    k: usize,
    n_features: usize,
) -> Vec<usize> {
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    {
        use par_iter::prelude::*;
        (0..n_samples)
            .into_par_iter()
            .map(|i| {
                let mut best_c = 0usize;
                let mut best_d_sq = squared_dist_row_to_centroid(data, centroids, i, 0, n_features);
                for c in 1..k {
                    let d_sq = squared_dist_row_to_centroid(data, centroids, i, c, n_features);
                    if d_sq < best_d_sq {
                        best_d_sq = d_sq;
                        best_c = c;
                    }
                }
                best_c
            })
            .collect()
    }

    #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
    {
        let mut new_labels = vec![0usize; n_samples];
        for (i, slot) in new_labels.iter_mut().enumerate().take(n_samples) {
            let mut best_c = 0usize;
            let mut best_d_sq = squared_dist_row_to_centroid(data, centroids, i, 0, n_features);
            for c in 1..k {
                let d_sq = squared_dist_row_to_centroid(data, centroids, i, c, n_features);
                if d_sq < best_d_sq {
                    best_d_sq = d_sq;
                    best_c = c;
                }
            }
            *slot = best_c;
        }
        new_labels
    }
}

#[inline]
fn squared_dist_row_to_centroid(
    data: &Matrix<f64>,
    centroids: &Matrix<f64>,
    row: usize,
    centroid: usize,
    n_features: usize,
) -> f64 {
    let mut sum = 0.0;
    for j in 0..n_features {
        let d = data.get(row, j) - centroids.get(centroid, j);
        sum += d * d;
    }
    sum
}

impl fmt::Display for KmeansResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "KmeansResult labels len={} n_clusters={} centroids {}x{}",
            self.labels.len(),
            self.n_clusters(),
            self.centroids.rows(),
            self.centroids.cols()
        )
    }
}
