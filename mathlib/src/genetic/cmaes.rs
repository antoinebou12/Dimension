//! CMA-ES (Covariance Matrix Adaptation Evolution Strategy).
//!
//! Minimizes a scalar objective over continuous vectors. Uses the existing `cpu` backends
//! (simd / parallel / sequential) for dot products and vector ops; evaluates λ candidates
//! in parallel when the `parallel` feature is enabled.

use crate::chol::{CholError, Cholesky};
use crate::cpu;
use crate::genetic::normal::sample_standard_normal;
use crate::matrix::Matrix;
use crate::types::Storage;
use rand::SeedableRng;
use rand::rngs::StdRng;
use tracing::{debug, info};

/// Result of a CMA-ES run: best solution and its fitness.
#[derive(Clone, Debug)]
pub struct CmaEsResult {
    /// Best solution found (minimizer).
    pub solution: Vec<f64>,
    /// Objective value at `solution` (minimum found).
    pub fitness: f64,
    /// Number of generations executed.
    pub generations: u32,
}

/// Builder for CMA-ES: dimension, initial mean, step size, and optional seed.
#[derive(Clone, Debug)]
pub struct CmaEsBuilder {
    dim: usize,
    mean: Vec<f64>,
    sigma: f64,
    max_generations: u32,
    seed: Option<u64>,
}

impl CmaEsBuilder {
    /// New builder: dimension `dim`, initial mean `mean` (length must be `dim`), initial step size `sigma`.
    pub fn new(dim: usize, mean: Vec<f64>, sigma: f64) -> Self {
        assert_eq!(mean.len(), dim, "mean length must equal dim");
        assert!(sigma > 0.0, "sigma must be positive");
        Self {
            dim,
            mean,
            sigma,
            max_generations: 500,
            seed: None,
        }
    }

    /// Set maximum number of generations (default 500).
    #[must_use]
    pub fn max_generations(mut self, n: u32) -> Self {
        self.max_generations = n;
        self
    }

    /// Set RNG seed for reproducible runs (optional).
    #[must_use]
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Build the CMA-ES optimizer.
    pub fn build(self) -> CmaEs {
        CmaEs::from_builder(self)
    }
}

/// CMA-ES optimizer state. Create via [`CmaEsBuilder`] or [`CmaEs::new`].
pub struct CmaEs {
    n: usize,
    mean: Vec<f64>,
    sigma: f64,
    /// Covariance matrix (n×n, column-major).
    c: Matrix<f64>,
    /// Evolution path for step-size adaptation.
    p_sigma: Vec<f64>,
    /// Evolution path for covariance rank-one update.
    p_c: Vec<f64>,
    /// Population size.
    lambda: usize,
    /// Number of parents (top μ).
    mu: usize,
    /// Recombination weights (length μ), sum = 1.
    weights: Vec<f64>,
    /// Variance effective selection mass.
    mu_eff: f64,
    /// Strategy parameters.
    c_sigma: f64,
    d_sigma: f64,
    c_c: f64,
    c_1: f64,
    c_mu: f64,
    /// E[||N(0,I)||] approximation for step-size update.
    chi_n: f64,
    max_generations: u32,
    rng: rand::rngs::StdRng,
}

impl CmaEs {
    /// New CMA-ES: initial mean (copy), initial step size, optional max generations and seed.
    ///
    /// For more control use [`CmaEsBuilder`].
    #[must_use]
    pub fn new(mean: &[f64], sigma: f64) -> Self {
        let dim = mean.len();
        CmaEsBuilder::new(dim, mean.to_vec(), sigma).build()
    }

    fn from_builder(b: CmaEsBuilder) -> Self {
        let n = b.dim;
        #[allow(clippy::cast_sign_loss)]
        let lambda = 4 + (3.0 * (n as f64).ln()) as usize;
        let lambda = lambda.max(2);
        let mu = lambda / 2;
        let mu = mu.max(1);

        // Weights: w_i ∝ ln(μ+1) - ln(i), i=1..μ, then normalize to sum 1.
        let mut weights: Vec<f64> = (1..=mu)
            .map(|i| ((mu + 1) as f64).ln() - (i as f64).ln())
            .collect();
        let sum_w: f64 = weights.iter().sum();
        for w in &mut weights {
            *w /= sum_w;
        }
        let mu_eff: f64 = 1.0 / weights.iter().map(|w| w * w).sum::<f64>();

        let c_sigma = (mu_eff + 2.0) / (n as f64 + mu_eff + 3.0);
        let d_sigma =
            1.0 + 2.0 * (0.0_f64).max((mu_eff - 1.0) / (n as f64 + 1.0).sqrt() - 1.0) + c_sigma;
        let c_c = 4.0 / (n as f64 + 4.0);
        let c_1 = 2.0 / ((n as f64 + 1.3).powi(2) + mu_eff);
        let c_mu = (1.0 - c_1)
            .min(2.0 * (mu_eff - 2.0 + 1.0 / mu_eff) / ((n as f64 + 2.0).powi(2) + mu_eff));
        let chi_n =
            (n as f64).sqrt() * (1.0 - 1.0 / (4.0 * n as f64) + 1.0 / (21.0 * (n as f64).powi(2)));

        let mut c = Matrix::with_storage(n, n, Storage::Column);
        c.set_zero();
        for i in 0..n {
            c.set(i, i, 1.0);
        }

        let p_sigma = vec![0.0; n];
        let p_c = vec![0.0; n];

        let rng = if let Some(seed) = b.seed {
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::from_rng(&mut rand::rng())
        };

        Self {
            n,
            mean: b.mean,
            sigma: b.sigma,
            c,
            p_sigma,
            p_c,
            lambda,
            mu,
            weights,
            mu_eff,
            c_sigma,
            d_sigma,
            c_c,
            c_1,
            c_mu,
            chi_n,
            max_generations: b.max_generations,
            rng,
        }
    }

    /// Run optimization: minimize `objective`. Returns best solution and fitness.
    /// When the `parallel` feature is enabled, `F` must implement `Sync` (e.g. function pointers and `Sync` closures).
    pub fn optimize<F>(&mut self, objective: F) -> CmaEsResult
    where
        F: Fn(&[f64]) -> f64 + Sync,
    {
        let dim = self.n;
        let max_generations = self.max_generations;
        debug!(dim, max_generations, "cmaes optimize");
        let mut generation: u32 = 0;
        let mut best_solution = self.mean.clone();
        let mut best_fitness = objective(&self.mean);

        while generation < self.max_generations {
            // Cholesky of C (with small diagonal if needed for numerical stability).
            let chol = match Cholesky::new(&self.c) {
                Ok(ch) => ch,
                Err(CholError::NotSPD | CholError::NotSquare) => {
                    for i in 0..self.n {
                        let v = self.c.get(i, i);
                        self.c.set(i, i, v + 1e-14);
                    }
                    Cholesky::new(&self.c).expect("Cholesky after regularisation")
                }
            };

            let l = chol.l();
            let mut candidates: Vec<Vec<f64>> = Vec::with_capacity(self.lambda);
            let mut ys: Vec<Vec<f64>> = Vec::with_capacity(self.lambda);

            for _ in 0..self.lambda {
                let z: Vec<f64> = (0..self.n)
                    .map(|_| sample_standard_normal(&mut self.rng))
                    .collect();
                let y = lower_triangular_matvec(l, &z);
                let x: Vec<f64> = weighted_add(1.0, &self.mean, self.sigma, &y);
                ys.push(y);
                candidates.push(x);
            }

            // Evaluate fitness (parallel when feature enabled).
            let fitness_indices: Vec<(usize, f64)> = evaluate_fitness(&candidates, &objective);

            // Sort by fitness (ascending: lower is better).
            let mut order: Vec<usize> = (0..self.lambda).collect();
            order.sort_by(|&a, &b| {
                fitness_indices[a]
                    .1
                    .partial_cmp(&fitness_indices[b].1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let new_best_fitness = fitness_indices[order[0]].1;
            if new_best_fitness < best_fitness {
                best_fitness = new_best_fitness;
                best_solution.clone_from(&candidates[order[0]]);
            }

            // Weighted mean of top μ candidates -> new mean.
            let mut m_new = vec![0.0_f64; self.n];
            for (k, &idx) in order.iter().take(self.mu).enumerate() {
                let w = self.weights[k];
                let cand = &candidates[idx];
                for (i, v) in m_new.iter_mut().enumerate() {
                    *v += w * cand[i];
                }
            }

            // y_w = (m_new - mean) / sigma (weighted sum of top μ y vectors).
            let mut y_w = vec![0.0_f64; self.n];
            for (k, &idx) in order.iter().take(self.mu).enumerate() {
                let w = self.weights[k];
                let y = &ys[idx];
                for (i, v) in y_w.iter_mut().enumerate() {
                    *v += w * y[i];
                }
            }

            // Evolution path p_sigma: uses C^{-1/2} y_w. With C = L L^T, C^{-1/2} y_w = L^{-T} y_w.
            let inv_sqrt_y_w = solve_lt(l, &y_w);
            let coeff_sigma = (self.c_sigma * (2.0 - self.c_sigma) * self.mu_eff).sqrt();
            for (i, v) in self.p_sigma.iter_mut().enumerate() {
                *v = (1.0 - self.c_sigma) * *v + coeff_sigma * inv_sqrt_y_w[i];
            }

            // Step-size update.
            let p_sigma_norm = sqrt_sum_sq(&self.p_sigma);
            self.sigma *= ((self.c_sigma / self.d_sigma) * (p_sigma_norm / self.chi_n - 1.0)).exp();

            // Evolution path p_c.
            let coeff_c = (self.c_c * (2.0 - self.c_c) * self.mu_eff).sqrt();
            for (i, v) in self.p_c.iter_mut().enumerate() {
                *v = (1.0 - self.c_c) * *v + coeff_c * y_w[i];
            }

            // Covariance update: C <- (1-c1-cμ) C + c1 p_c p_c^T + cμ sum w_i y_i y_i^T.
            let scale = 1.0 - self.c_1 - self.c_mu;
            for i in 0..self.n {
                for j in 0..self.n {
                    let mut v = scale * self.c.get(i, j);
                    v += self.c_1 * self.p_c[i] * self.p_c[j];
                    for (k, &idx) in order.iter().take(self.mu).enumerate() {
                        let y = &ys[idx];
                        v += self.c_mu * self.weights[k] * y[i] * y[j];
                    }
                    self.c.set(i, j, v);
                }
            }

            self.mean.clone_from(&m_new);
            generation += 1;
        }

        info!(
            fitness = %best_fitness,
            generations = generation,
            "cmaes completed"
        );
        CmaEsResult {
            solution: best_solution,
            fitness: best_fitness,
            generations: generation,
        }
    }
}

/// Lower triangular L (column-major): out = L * x.
#[allow(clippy::needless_range_loop)]
fn lower_triangular_matvec(l: &Matrix<f64>, x: &[f64]) -> Vec<f64> {
    let n = l.rows();
    assert_eq!(l.cols(), n);
    assert_eq!(x.len(), n);
    let mut out = vec![0.0; n];
    for i in 0..n {
        let mut s = 0.0;
        for j in 0..=i {
            s += l.get(i, j) * x[j];
        }
        out[i] = s;
    }
    out
}

/// Solve L^T x = b (L lower triangular, column-major). Returns x.
#[allow(clippy::needless_range_loop)]
fn solve_lt(l: &Matrix<f64>, b: &[f64]) -> Vec<f64> {
    let n = l.rows();
    assert_eq!(l.cols(), n);
    assert_eq!(b.len(), n);
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let mut s = b[i];
        for j in (i + 1)..n {
            s -= l.get(j, i) * x[j];
        }
        x[i] = s / l.get(i, i);
    }
    x
}

/// out = a_scale * a + sigma * b using cpu backends for vector ops.
fn weighted_add(a_scale: f64, a: &[f64], sigma: f64, b: &[f64]) -> Vec<f64> {
    let n = a.len();
    assert_eq!(b.len(), n);
    let mut out = vec![0.0; n];
    let mut tmp = vec![0.0; n];
    let mut sum = vec![0.0; n];
    #[cfg(feature = "simd")]
    {
        crate::cpu::simd::scalar_mul_f64(a_scale, a, &mut out);
        crate::cpu::simd::scalar_mul_f64(sigma, b, &mut tmp);
        crate::cpu::simd::add_f64(&out, &tmp, &mut sum);
    }
    #[cfg(all(
        feature = "parallel",
        not(target_arch = "wasm32"),
        not(feature = "simd")
    ))]
    {
        crate::cpu::parallel::par_scalar_mul_f64(a_scale, a, &mut out);
        crate::cpu::parallel::par_scalar_mul_f64(sigma, b, &mut tmp);
        crate::cpu::parallel::par_add_f64(&out, &tmp, &mut sum);
    }
    #[cfg(not(any(
        feature = "simd",
        all(feature = "parallel", not(target_arch = "wasm32"))
    )))]
    {
        crate::cpu::sequential::scalar_mul_f64(a_scale, a, &mut out);
        crate::cpu::sequential::scalar_mul_f64(sigma, b, &mut tmp);
        crate::cpu::sequential::add_f64(&out, &tmp, &mut sum);
    }
    out.copy_from_slice(&sum);
    out
}

fn sqrt_sum_sq(x: &[f64]) -> f64 {
    cpu::dot_f64(x, x).sqrt()
}

/// Evaluate objective on each candidate; returns (index, fitness) for each.
fn evaluate_fitness<F>(candidates: &[Vec<f64>], objective: &F) -> Vec<(usize, f64)>
where
    F: Fn(&[f64]) -> f64 + Sync,
{
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    {
        use par_iter::prelude::*;
        (0..candidates.len())
            .into_par_iter()
            .map(|i| (i, objective(&candidates[i])))
            .collect()
    }
    #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
    {
        candidates
            .iter()
            .enumerate()
            .map(|(i, c)| (i, objective(c)))
            .collect()
    }
}
