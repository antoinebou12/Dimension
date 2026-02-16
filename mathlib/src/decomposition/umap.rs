//! UMAP (Uniform Manifold Approximation and Projection) for dimensionality reduction.
//!
//! Maps high-dimensional data to 2D or 3D for visualization. Uses only mathlib primitives
//! (no external crates). Data layout: rows = samples, cols = features.
//!
//! # Algorithm
//!
//! 1. Build k-NN graph (brute-force).
//! 2. Compute fuzzy simplicial set via smooth k-NN distances.
//! 3. Symmetrize weights (fuzzy union).
//! 4. Optimize embedding with cross-entropy loss.

use crate::distance::squared_euclidean_rows;
use crate::matrix::Matrix;
use crate::types::Storage;
use std::f64::consts::LN_2;
use tracing::{debug, info};

/// Options for UMAP.
#[derive(Clone, Debug)]
pub struct UmapOptions {
    /// Number of output dimensions (typically 2 or 3).
    pub n_components: usize,
    /// Number of nearest neighbors (default 15).
    pub n_neighbors: usize,
    /// Minimum distance in low-D (0.0..1.0, default 0.1).
    pub min_dist: f64,
    /// Maximum optimization iterations (default 500).
    pub max_iters: usize,
    /// Random seed for reproducibility (None = use default).
    pub seed: Option<u64>,
}

impl Default for UmapOptions {
    fn default() -> Self {
        Self {
            n_components: 2,
            n_neighbors: 15,
            min_dist: 0.1,
            max_iters: 500,
            seed: None,
        }
    }
}

/// Deterministic RNG (xorshift64) for reproducible UMAP.
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let x = self.state;
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        x
    }

    #[allow(clippy::cast_precision_loss)]
    fn uniform01(&mut self) -> f64 {
        const INV_2_53: f64 = 1.0 / 9_007_199_254_740_992.0;
        (self.next_u64() >> 11) as f64 * INV_2_53
    }
}

/// Brute-force k-NN: for each point, return (indices, distances) of k nearest neighbors.
fn k_nearest_neighbors(data: &Matrix<f64>, k: usize) -> Vec<(Vec<usize>, Vec<f64>)> {
    let n = data.rows();
    let k = k.min(n - 1).max(1);

    let mut result = Vec::with_capacity(n);
    for i in 0..n {
        let mut neighbors: Vec<(usize, f64)> = (0..n)
            .filter(|&j| j != i)
            .map(|j| {
                let d_sq = squared_euclidean_rows(data, i, j);
                (j, d_sq.sqrt())
            })
            .collect();

        neighbors.select_nth_unstable_by(k - 1, |a, b| a.1.partial_cmp(&b.1).unwrap());
        neighbors.truncate(k);
        neighbors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let (indices, dists): (Vec<_>, Vec<_>) = neighbors.into_iter().unzip();
        result.push((indices, dists));
    }
    result
}

/// Binary search for `sigma` such that `sum_j exp(-(d_j - rho)/sigma) = log2(k)`.
fn find_sigma(rho: f64, dists: &[f64], target: f64) -> f64 {
    if dists.is_empty() {
        return 1.0;
    }
    let mut sigma_lo = 1e-10_f64;
    let mut sigma_hi = 1e4_f64;

    for _ in 0..50 {
        let sigma = f64::midpoint(sigma_lo, sigma_hi);
        let mut sum = 0.0;
        for &d in dists {
            if d > rho {
                sum += (-(d - rho) / sigma).exp();
            } else {
                sum += 1.0;
            }
        }
        if sum < target {
            sigma_hi = sigma;
        } else {
            sigma_lo = sigma;
        }
    }
    f64::midpoint(sigma_lo, sigma_hi)
}

/// Run UMAP on `data` (rows = samples, cols = features).
/// Returns embedding matrix (`n_samples` × `n_components`).
#[allow(clippy::cast_precision_loss, clippy::needless_range_loop)]
pub fn umap(data: &Matrix<f64>, options: &UmapOptions) -> Matrix<f64> {
    let (n_samples, n_features) = (data.rows(), data.cols());
    let n_comp = options.n_components;
    let k = options.n_neighbors.min(n_samples - 1).max(1);
    debug!(
        n_samples,
        n_features,
        n_components = n_comp,
        n_neighbors = k,
        "umap"
    );

    assert!(
        n_samples >= 2 && n_features >= 1,
        "UMAP requires at least 2 samples and 1 feature"
    );

    let target_log = (k as f64).ln() / LN_2;

    // 1. k-NN graph
    let knn = k_nearest_neighbors(data, k);

    // 2. Fuzzy simplicial set: weights per edge (i, j) where j is neighbor of i
    // Store in a sparse-ish way: for each (i,j) in knn, we have weight.
    // Use a map or vec of vecs. For grad we need to lookup p_ij for any (i,j).
    // Build symmetric p_ij: collect all edges with weights, symmetrize.
    let mut edge_weights: Vec<Vec<(usize, f64)>> = vec![vec![]; n_samples];

    for i in 0..n_samples {
        let (indices, dists) = &knn[i];
        let rho = dists[0];
        let sigma = find_sigma(rho, dists, target_log);
        for (j, &d) in indices.iter().zip(dists.iter()) {
            let w = if d <= rho {
                1.0
            } else {
                (-(d - rho) / sigma).exp()
            };
            edge_weights[i].push((*j, w));
        }
    }

    // 3. Symmetrize: p_ij = w_ij + w_ji - w_ij*w_ji (fuzzy union)
    let mut p_ij: Vec<Vec<f64>> = vec![vec![0.0; n_samples]; n_samples];
    for i in 0..n_samples {
        for &(j, w_ij) in &edge_weights[i] {
            let w_ji = edge_weights[j]
                .iter()
                .find(|&&(jj, _)| jj == i)
                .map_or(0.0, |&(_, w)| w);
            let p = w_ij + w_ji - w_ij * w_ji;
            p_ij[i][j] = p;
            p_ij[j][i] = p;
        }
    }

    // 4. Initialize embedding randomly
    let seed = options.seed.unwrap_or(0x8765_4321);
    let mut rng = XorShift64::new(seed);
    let mut y = Matrix::with_storage(n_samples, n_comp, Storage::Column);
    for i in 0..n_samples {
        for c in 0..n_comp {
            y.set(i, c, 0.01 * (rng.uniform01() - 0.5));
        }
    }

    // 5. a, b for low-dim curve: q = 1/(1 + a*d^(2b))
    let min_dist = options.min_dist.clamp(0.001, 0.99);
    let a = (1.0 - min_dist) / (min_dist + 1e-6);
    let b = 1.0;

    // 6. Gradient descent
    let lr = 1.0;
    let mut velocity: Vec<Vec<f64>> = vec![vec![0.0; n_comp]; n_samples];

    for iter in 0..options.max_iters {
        let momentum = if iter < 100 { 0.5 } else { 0.8 };

        // Compute q_ij and gradient (only for edges with p_ij > 0)
        let mut grad = vec![vec![0.0_f64; n_comp]; n_samples];

        for i in 0..n_samples {
            for j in (i + 1)..n_samples {
                let p = p_ij[i][j];
                if p < 1e-12 {
                    continue;
                }

                let mut d_sq = 0.0;
                for c in 0..n_comp {
                    let dy = y.get(i, c) - y.get(j, c);
                    d_sq += dy * dy;
                }
                let d_sq = d_sq.max(1e-12);
                let d_2b = d_sq.powf(b);
                let q = 1.0 / (1.0 + a * d_2b);

                let pq = p - q;
                let coeff = 2.0 * a * b * d_2b / (d_sq * (1.0 + a * d_2b)) * pq;

                let (lo, hi) = grad.split_at_mut(j);
                let grad_i = &mut lo[i];
                let grad_j = &mut hi[0];
                for c in 0..n_comp {
                    let dy = y.get(i, c) - y.get(j, c);
                    grad_i[c] += coeff * dy;
                    grad_j[c] -= coeff * dy;
                }
            }
        }

        for i in 0..n_samples {
            for (c, (vel, g)) in velocity[i].iter_mut().zip(grad[i].iter()).enumerate() {
                *vel = momentum * *vel + lr * g;
                y.set(i, c, y.get(i, c) + *vel);
            }
        }
    }

    info!(n_samples, n_components = n_comp, "umap ok");
    y
}
