//! DBSCAN clustering (sequential; optional parallel neighborhood computation when `parallel` feature is enabled).
//!
//! Data layout: rows = samples, cols = features. Noise points have label `NOISE`.

use crate::distance;
use crate::matrix::Matrix;
use std::fmt;
use tracing::{debug, info};

/// Label for noise points (not assigned to any cluster).
pub const NOISE: usize = usize::MAX;

/// Result of DBSCAN: cluster label per point (NOISE for noise).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DbscanResult {
    /// labels[i] = cluster index (0, 1, ...) or NOISE.
    pub(crate) labels: Vec<usize>,
}

impl DbscanResult {
    #[inline]
    pub fn labels(&self) -> &[usize] {
        &self.labels
    }

    #[inline]
    pub fn is_noise(&self, i: usize) -> bool {
        self.labels[i] == NOISE
    }

    /// Number of clusters (max label + 1, excluding NOISE).
    pub fn n_clusters(&self) -> usize {
        self.labels
            .iter()
            .filter(|&&l| l != NOISE)
            .max()
            .map_or(0, |&m| m + 1)
    }
}

/// Run DBSCAN on `data` (rows = samples, cols = features). Points within `eps` (Euclidean) are neighbors.
/// A point is core if its neighborhood has at least `min_pts` points. Returns labels (cluster id or NOISE).
pub fn dbscan(data: &Matrix<f64>, eps: f64, min_pts: usize) -> DbscanResult {
    let (n_samples, n_features) = (data.rows(), data.cols());
    assert!(eps > 0.0 && min_pts >= 1);
    debug!(n_samples, n_features, eps, min_pts, "dbscan");
    let eps_sq = eps * eps;

    // Step 1: For each point, compute neighbors within eps (parallel over points when feature enabled).
    let neighbors = compute_neighbors(data, n_samples, eps_sq);

    // Step 2: Classify core / border / noise.
    let mut is_core = vec![false; n_samples];
    for i in 0..n_samples {
        is_core[i] = neighbors[i].len() >= min_pts;
    }

    // Step 3: Expand clusters (sequential flood-fill).
    let mut labels = vec![NOISE; n_samples];
    let mut cluster_id = 0usize;
    for i in 0..n_samples {
        if labels[i] != NOISE || !is_core[i] {
            continue;
        }
        expand_cluster(&neighbors, &is_core, &mut labels, i, cluster_id);
        cluster_id += 1;
    }

    let result = DbscanResult { labels };
    info!(n_clusters = result.n_clusters(), "dbscan ok");
    result
}

/// Neighbors: for each point, indices of points within eps (including self).
fn compute_neighbors(data: &Matrix<f64>, n_samples: usize, eps_sq: f64) -> Vec<Vec<usize>> {
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    {
        use par_iter::prelude::*;
        (0..n_samples)
            .into_par_iter()
            .map(|i| {
                let mut nb = Vec::with_capacity(64);
                for j in 0..n_samples {
                    if distance::squared_euclidean_rows(data, i, j) <= eps_sq {
                        nb.push(j);
                    }
                }
                nb
            })
            .collect()
    }

    #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
    {
        let mut neighbors = Vec::with_capacity(n_samples);
        for i in 0..n_samples {
            let mut nb = Vec::with_capacity(64);
            for j in 0..n_samples {
                if distance::squared_euclidean_rows(data, i, j) <= eps_sq {
                    nb.push(j);
                }
            }
            neighbors.push(nb);
        }
        neighbors
    }
}

fn expand_cluster(
    neighbors: &[Vec<usize>],
    is_core: &[bool],
    labels: &mut [usize],
    seed: usize,
    cluster_id: usize,
) {
    let mut stack = vec![seed];
    while let Some(i) = stack.pop() {
        if labels[i] != NOISE {
            continue;
        }
        labels[i] = cluster_id;
        for &j in &neighbors[i] {
            if labels[j] == NOISE {
                if is_core[j] {
                    stack.push(j);
                } else {
                    labels[j] = cluster_id;
                }
            }
        }
    }
}

impl fmt::Display for DbscanResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DbscanResult labels len={} n_clusters={}",
            self.labels.len(),
            self.n_clusters()
        )
    }
}
