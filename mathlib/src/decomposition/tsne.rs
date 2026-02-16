//! t-SNE (t-distributed Stochastic Neighbor Embedding) for dimensionality reduction.
//!
//! Maps high-dimensional data to 2D or 3D for visualization. Uses only mathlib primitives
//! (no external crates). Data layout: rows = samples, cols = features.
//!
//! # Algorithm
//!
//! 1. Compute pairwise squared distances.
//! 2. Binary search σ per point to achieve target perplexity.
//! 3. Build symmetric joint probability P.
//! 4. Initialize embedding via PCA or random.
//! 5. Gradient descent on KL(P||Q) with Student-t kernel, momentum, and early exaggeration.

use super::pca;
use crate::distance::squared_euclidean_rows;
use crate::matrix::Matrix;
use crate::types::Storage;
use std::f64::consts::LN_2;
use tracing::{debug, info};

/// Options for t-SNE.
#[derive(Clone, Debug)]
pub struct TsneOptions {
    /// Number of output dimensions (typically 2 or 3).
    pub n_components: usize,
    /// Perplexity (typically 5..50); roughly the number of nearest neighbors.
    pub perplexity: f64,
    /// Maximum optimization iterations (default 1000).
    pub max_iters: usize,
    /// Random seed for reproducibility (None = use default).
    pub seed: Option<u64>,
    /// Use PCA for initialization (true) or random (false). Default true.
    pub init_pca: bool,
    /// Early exaggeration multiplier for first ~100 iters (default 4.0).
    pub early_exaggeration: f64,
    /// Momentum after early exaggeration phase (default 0.8).
    pub momentum: f64,
}

impl Default for TsneOptions {
    fn default() -> Self {
        Self {
            n_components: 2,
            perplexity: 30.0,
            max_iters: 1000,
            seed: None,
            init_pca: true,
            early_exaggeration: 4.0,
            momentum: 0.8,
        }
    }
}

/// Deterministic RNG (xorshift64) for reproducible t-SNE without rand dependency.
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

/// Compute Shannon entropy H(P) = -sum p log2(p) for conditional distribution.
fn entropy_log2(probs: &[f64]) -> f64 {
    let mut h = 0.0;
    for &p in probs {
        if p > 1e-20 {
            h -= p * (p.ln() / LN_2);
        }
    }
    h
}

/// Binary search for σ such that perplexity = 2^H(P).
fn find_sigma(sq_dists: &[f64], exclude_self: usize, target_log_perp: f64) -> f64 {
    let n = sq_dists.len();
    let mut sigma_lo = 1e-10_f64;
    let mut sigma_hi = 1e4_f64;
    let mut sigma = 1.0;

    for _ in 0..50 {
        sigma = f64::midpoint(sigma_lo, sigma_hi);
        let sigma_sq = sigma * sigma;

        let mut probs = vec![0.0; n];
        let mut sum = 0.0;
        for (j, &d) in sq_dists.iter().enumerate() {
            if j == exclude_self {
                continue;
            }
            let p = (-d / (2.0 * sigma_sq)).exp();
            probs[j] = p;
            sum += p;
        }
        if sum < 1e-20 {
            sigma_lo = sigma;
            continue;
        }
        for p in &mut probs {
            *p /= sum;
        }
        let h = entropy_log2(&probs);
        if h < target_log_perp {
            sigma_hi = sigma;
        } else {
            sigma_lo = sigma;
        }
    }
    sigma
}

/// Run t-SNE on `data` (rows = samples, cols = features).
/// Returns embedding matrix (`n_samples` × `n_components`).
#[allow(
    clippy::cast_precision_loss,
    clippy::doc_markdown,
    clippy::needless_range_loop,
    clippy::too_many_lines
)]
pub fn tsne(data: &Matrix<f64>, options: &TsneOptions) -> Matrix<f64> {
    let (n_samples, n_features) = (data.rows(), data.cols());
    let n_comp = options.n_components;
    debug!(
        n_samples,
        n_features,
        n_components = n_comp,
        perplexity = options.perplexity,
        "tsne"
    );

    assert!(
        n_samples >= 2 && n_features >= 1,
        "t-SNE requires at least 2 samples and 1 feature"
    );
    assert!(
        options.perplexity >= 1.0 && options.perplexity < (n_samples - 1) as f64,
        "perplexity should be in [1, n-1)"
    );

    let target_log_perp = options.perplexity.ln() / LN_2;

    // 1. Pairwise squared distances
    let mut sq_dists = vec![vec![0.0; n_samples]; n_samples];
    for i in 0..n_samples {
        for j in (i + 1)..n_samples {
            let d = squared_euclidean_rows(data, i, j);
            sq_dists[i][j] = d;
            sq_dists[j][i] = d;
        }
    }

    // 2. Binary search σ per point, compute p(j|i), symmetrize to p_ij
    let mut p_sym = vec![vec![0.0; n_samples]; n_samples];
    let n_f = n_samples as f64;

    for i in 0..n_samples {
        let sigma = find_sigma(&sq_dists[i], i, target_log_perp);
        let sigma_sq = sigma * sigma;
        let mut row_sum = 0.0;
        for j in 0..n_samples {
            if i == j {
                continue;
            }
            let val = (-sq_dists[i][j] / (2.0 * sigma_sq)).exp();
            p_sym[i][j] = val;
            row_sum += val;
        }
        if row_sum > 1e-20 {
            for j in 0..n_samples {
                if i != j {
                    p_sym[i][j] /= row_sum;
                }
            }
        }
    }

    // Symmetrize: p_ij = (p(j|i) + p(i|j)) / (2n)
    for i in 0..n_samples {
        for j in (i + 1)..n_samples {
            let p = (p_sym[i][j] + p_sym[j][i]) / (2.0 * n_f);
            p_sym[i][j] = p;
            p_sym[j][i] = p;
        }
    }

    // 3. Initialize Y
    let mut y = Matrix::with_storage(n_samples, n_comp, Storage::Column);
    if options.init_pca && n_features >= n_comp {
        let pca_result = pca::pca(data, Some(n_comp));
        let mean = pca_result.mean();
        let components = pca_result.components();
        for i in 0..n_samples {
            for c in 0..n_comp {
                let mut sum = 0.0;
                for k in 0..n_features {
                    sum += (data.get(i, k) - mean.get(k)) * components.get(k, c);
                }
                y.set(i, c, sum);
            }
        }
    } else {
        let seed = options.seed.unwrap_or(0x1234_5678);
        let mut rng = XorShift64::new(seed);
        for i in 0..n_samples {
            for c in 0..n_comp {
                y.set(i, c, 1e-4 * (rng.uniform01() - 0.5));
            }
        }
    }

    // 4. Gradient descent with momentum and early exaggeration
    let early_iters = options.max_iters / 4;
    let mut velocity: Vec<Vec<f64>> = vec![vec![0.0; n_comp]; n_samples];
    let lr = 200.0;
    let min_gain = 0.01;

    for iter in 0..options.max_iters {
        let mult = if iter < early_iters {
            options.early_exaggeration
        } else {
            1.0
        };
        let mom = if iter < early_iters {
            0.5
        } else {
            options.momentum
        };

        // Compute Q and gradient
        let mut q_sum = 0.0;
        let mut q_ij = vec![vec![0.0; n_samples]; n_samples];
        for i in 0..n_samples {
            for j in (i + 1)..n_samples {
                let mut d_sq = 0.0;
                for c in 0..n_comp {
                    let dy = y.get(i, c) - y.get(j, c);
                    d_sq += dy * dy;
                }
                let q = 1.0 / (1.0 + d_sq);
                q_ij[i][j] = q;
                q_ij[j][i] = q;
                q_sum += 2.0 * q;
            }
        }
        let q_sum = q_sum.max(1e-12);

        let mut grad = vec![vec![0.0; n_comp]; n_samples];
        for i in 0..n_samples {
            for j in 0..n_samples {
                if i == j {
                    continue;
                }
                let p = p_sym[i][j] * mult;
                let q = q_ij[i][j] / q_sum;
                let pq = (p - q) * q_ij[i][j];
                for c in 0..n_comp {
                    let dy = y.get(i, c) - y.get(j, c);
                    grad[i][c] += pq * dy;
                }
            }
        }
        for i in 0..n_samples {
            for c in 0..n_comp {
                grad[i][c] *= 4.0;
            }
        }

        // Update with momentum
        for i in 0..n_samples {
            for c in 0..n_comp {
                let g = grad[i][c];
                let v_old = velocity[i][c];
                let v_abs = velocity[i][c].abs();
                let gain = if (g > 0.0) == (v_old > 0.0) {
                    (0.8_f64 + v_abs).min(1.0)
                } else {
                    (0.2_f64 + v_abs).min(1.0)
                };
                let gain = gain.max(min_gain);
                velocity[i][c] = mom * v_old - lr * gain * g;
                let new_val = y.get(i, c) + velocity[i][c];
                y.set(i, c, new_val);
            }
        }
    }

    info!(n_samples, n_components = n_comp, "tsne ok");
    y
}
