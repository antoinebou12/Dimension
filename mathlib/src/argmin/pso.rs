//! Particle Swarm Optimization (PSO).
//!
//! Canonical PSO: positions and velocities in a bounded search space, with inertia,
//! cognitive (personal best) and social (global best) terms. Uses an internal
//! deterministic RNG (no optional deps). Cost evaluation and slice math dispatch
//! to SIMD/parallel backends when features are enabled.

use std::f64::consts::LN_2;
use tracing::info;

/// Result of a PSO run.
#[derive(Clone, Debug)]
pub struct PsoResult {
    /// Best position found.
    pub best_position: Vec<f64>,
    /// Cost at best position.
    pub best_cost: f64,
    /// Number of iterations performed.
    pub iterations: u32,
}

/// Optional PSO weights (inertia, cognitive, social). Defaults match standard PSO 2011.
#[derive(Clone, Debug)]
pub struct PsoOptions {
    /// Inertia weight on velocity (default: 1/(2*ln(2))).
    pub inertia: f64,
    /// Cognitive acceleration (default: 0.5 + ln(2)).
    pub cognitive: f64,
    /// Social acceleration (default: 0.5 + ln(2)).
    pub social: f64,
}

impl Default for PsoOptions {
    fn default() -> Self {
        Self {
            inertia: 1.0 / (2.0 * LN_2),
            cognitive: 0.5 + LN_2,
            social: 0.5 + LN_2,
        }
    }
}

/// Single particle: position, velocity, cost, and personal best.
struct Particle {
    position: Vec<f64>,
    velocity: Vec<f64>,
    cost: f64,
    best_position: Vec<f64>,
    best_cost: f64,
}

/// Deterministic RNG (xorshift64) for reproducible PSO without a rand dependency.
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

    /// Uniform [0, 1) from 53-bit fraction.
    #[allow(clippy::cast_precision_loss)]
    fn uniform01(&mut self) -> f64 {
        const INV_2_53: f64 = 1.0 / 9_007_199_254_740_992.0; // 2^53
        (self.next_u64() >> 11) as f64 * INV_2_53
    }

    /// Uniform in [low[i], high[i]) per dimension.
    fn uniform_in_bounds(&mut self, low: &[f64], high: &[f64], out: &mut [f64]) {
        assert_eq!(low.len(), high.len());
        assert_eq!(low.len(), out.len());
        for i in 0..out.len() {
            let u = self.uniform01();
            out[i] = low[i] + u * (high[i] - low[i]);
        }
    }
}

/// Slice ops dispatch (simd > parallel > sequential), same pattern as distance module.
#[inline]
fn add_f64(a: &[f64], b: &[f64], out: &mut [f64]) {
    #[cfg(feature = "simd")]
    return crate::cpu::simd::add_f64(a, b, out);
    #[cfg(all(
        feature = "parallel",
        not(target_arch = "wasm32"),
        not(feature = "simd")
    ))]
    return crate::cpu::parallel::par_add_f64(a, b, out);
    #[cfg(not(any(
        feature = "simd",
        all(feature = "parallel", not(target_arch = "wasm32"))
    )))]
    crate::cpu::sequential::add_f64(a, b, out);
}

#[inline]
fn sub_f64(a: &[f64], b: &[f64], out: &mut [f64]) {
    #[cfg(feature = "simd")]
    return crate::cpu::simd::sub_f64(a, b, out);
    #[cfg(all(
        feature = "parallel",
        not(target_arch = "wasm32"),
        not(feature = "simd")
    ))]
    return crate::cpu::parallel::par_sub_f64(a, b, out);
    #[cfg(not(any(
        feature = "simd",
        all(feature = "parallel", not(target_arch = "wasm32"))
    )))]
    crate::cpu::sequential::sub_f64(a, b, out);
}

#[inline]
fn scalar_mul_f64(s: f64, x: &[f64], out: &mut [f64]) {
    #[cfg(feature = "simd")]
    return crate::cpu::simd::scalar_mul_f64(s, x, out);
    #[cfg(all(
        feature = "parallel",
        not(target_arch = "wasm32"),
        not(feature = "simd")
    ))]
    return crate::cpu::parallel::par_scalar_mul_f64(s, x, out);
    #[cfg(not(any(
        feature = "simd",
        all(feature = "parallel", not(target_arch = "wasm32"))
    )))]
    crate::cpu::sequential::scalar_mul_f64(s, x, out);
}

/// Clamp position to bounds (element-wise). No cpu backend; simple loop.
fn clamp_f64(x: &[f64], low: &[f64], high: &[f64], out: &mut [f64]) {
    assert_eq!(x.len(), low.len());
    assert_eq!(x.len(), high.len());
    assert_eq!(x.len(), out.len());
    for i in 0..x.len() {
        out[i] = x[i].clamp(low[i], high[i]);
    }
}

/// Run PSO: minimize `cost` over the box `(lower_bound, upper_bound)` with `num_particles`
/// particles for at most `max_iters` iterations. Optional `options` for weights; uses
/// a deterministic internal RNG (seed derived from bounds and `num_particles` for
/// reproducibility; for fixed seed across runs, use a dedicated entry point if added).
///
/// Requires `cost` to be `Sync` when the `parallel` feature is enabled (bulk cost evaluation).
#[must_use]
pub fn pso<F>(
    bounds: (Vec<f64>, Vec<f64>),
    num_particles: usize,
    cost: F,
    max_iters: u32,
    options: Option<PsoOptions>,
) -> PsoResult
where
    F: Fn(&[f64]) -> f64 + Sync,
{
    let (low, high) = bounds;
    let dim = low.len();
    assert_eq!(high.len(), dim);
    assert!(num_particles >= 1);
    assert!(dim >= 1);

    let opts = options.unwrap_or_default();
    let mut rng = XorShift64::new(seed_from_bounds(&low, &high, num_particles));

    // Delta for initial velocity range: velocity in [-delta, delta] per dimension
    let mut delta = vec![0.0_f64; dim];
    for i in 0..dim {
        delta[i] = high[i] - low[i];
    }
    let mut delta_neg = vec![0.0_f64; dim];
    for i in 0..dim {
        delta_neg[i] = -delta[i];
    }

    // Initialize particles: random positions and velocities, then cost
    let mut particles: Vec<Particle> = (0..num_particles)
        .map(|_| {
            let mut position = vec![0.0_f64; dim];
            let mut velocity = vec![0.0_f64; dim];
            rng.uniform_in_bounds(&low, &high, &mut position);
            rng.uniform_in_bounds(&delta_neg, &delta, &mut velocity);
            let c = cost(&position);
            Particle {
                best_position: position.clone(),
                best_cost: c,
                position,
                velocity,
                cost: c,
            }
        })
        .collect();

    // Sort so first is best (lowest cost)
    particles.sort_by(|a, b| {
        a.cost
            .partial_cmp(&b.cost)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut best_position = particles[0].best_position.clone();
    let mut best_cost = particles[0].best_cost;
    let mut iter = 0u32;

    // Scratch buffers for update (avoid per-iteration allocs)
    let mut diff = vec![0.0_f64; dim];
    let mut scaled = vec![0.0_f64; dim];
    let mut new_vel = vec![0.0_f64; dim];
    let mut new_pos = vec![0.0_f64; dim];

    while iter < max_iters {
        for p in &mut particles {
            // v_new = w*v + c1*r1*(pbest - x) + c2*r2*(gbest - x)
            // Momentum: w * v
            scalar_mul_f64(opts.inertia, &p.velocity, &mut new_vel);

            // Cognitive: c1 * r1 * (pbest - x), r1 in [0,1] per dimension
            sub_f64(&p.best_position, &p.position, &mut diff);
            for i in 0..dim {
                scaled[i] = diff[i] * rng.uniform01();
            }
            scalar_mul_f64(opts.cognitive, &scaled, &mut diff);
            add_f64(&new_vel, &diff, &mut scaled);
            new_vel.copy_from_slice(&scaled);

            // Social: c2 * r2 * (gbest - x)
            sub_f64(&best_position, &p.position, &mut diff);
            for i in 0..dim {
                scaled[i] = diff[i] * rng.uniform01();
            }
            scalar_mul_f64(opts.social, &scaled, &mut diff);
            add_f64(&new_vel, &diff, &mut scaled);
            new_vel.copy_from_slice(&scaled);

            p.velocity.copy_from_slice(&new_vel);
            add_f64(&p.position, &p.velocity, &mut new_pos);
            clamp_f64(&new_pos, &low, &high, &mut p.position);
        }

        // Bulk cost evaluation (parallel when feature enabled)
        let positions_vec: Vec<Vec<f64>> = particles.iter().map(|p| p.position.clone()).collect();
        let costs = bulk_cost(&positions_vec, &cost);

        for (p, c) in particles.iter_mut().zip(costs) {
            p.cost = c;
            if c < p.best_cost {
                p.best_position.copy_from_slice(&p.position);
                p.best_cost = c;
                if c < best_cost {
                    best_cost = c;
                    best_position.copy_from_slice(&p.position);
                }
            }
        }

        // Re-sort so global best is first (for consistency; we already track best_position/best_cost)
        particles.sort_by(|a, b| {
            a.cost
                .partial_cmp(&b.cost)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        iter += 1;
    }

    info!(
        iterations = iter,
        best_cost = %best_cost,
        "pso completed"
    );

    PsoResult {
        best_position,
        best_cost,
        iterations: iter,
    }
}

/// Bulk cost evaluation: sequential or parallel (rayon when feature "parallel", not on wasm32).
fn bulk_cost<F>(positions: &[Vec<f64>], cost: &F) -> Vec<f64>
where
    F: Fn(&[f64]) -> f64 + Sync,
{
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    {
        use rayon::prelude::*;
        positions.par_iter().map(|p| cost(p.as_slice())).collect()
    }
    #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
    {
        positions.iter().map(|p| cost(p.as_slice())).collect()
    }
}

/// Seed for internal RNG from bounds and `num_particles` (deterministic).
fn seed_from_bounds(low: &[f64], high: &[f64], num_particles: usize) -> u64 {
    let mut h = num_particles as u64;
    for (i, (&a, &b)) in low.iter().zip(high.iter()).enumerate().take(8) {
        h = h.wrapping_add((i as u64).wrapping_mul(a.to_bits()));
        h = h.wrapping_add(b.to_bits());
    }
    if h == 0 { 1 } else { h }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere_cost(x: &[f64]) -> f64 {
        x.iter().map(|v| v * v).sum()
    }

    #[test]
    fn pso_sphere_small() {
        let dim = 4usize;
        let low = vec![-5.0; dim];
        let high = vec![5.0; dim];
        let result = pso(
            (low, high),
            20,
            sphere_cost,
            100,
            Some(PsoOptions::default()),
        );
        assert!(
            result.best_cost < 1.0,
            "sphere cost should be small, got {}",
            result.best_cost
        );
        for &x in &result.best_position {
            assert!(x.abs() < 2.0, "best position should be near 0");
        }
        assert_eq!(result.iterations, 100);
    }

    #[test]
    fn pso_bounds_respected() {
        let low = vec![-2.0, -3.0];
        let high = vec![2.0, 3.0];
        let result = pso((low.clone(), high.clone()), 10, sphere_cost, 5, None);
        for (i, &x) in result.best_position.iter().enumerate() {
            assert!(
                x >= low[i],
                "position[{}] {} below lower bound {}",
                i,
                x,
                low[i]
            );
            assert!(
                x <= high[i],
                "position[{}] {} above upper bound {}",
                i,
                x,
                high[i]
            );
        }
    }

    #[test]
    fn pso_deterministic() {
        let low = vec![-1.0, -1.0];
        let high = vec![1.0, 1.0];
        let r1 = pso((low.clone(), high.clone()), 8, sphere_cost, 20, None);
        let r2 = pso((low, high), 8, sphere_cost, 20, None);
        assert_eq!(r1.best_cost.to_bits(), r2.best_cost.to_bits());
        assert_eq!(r1.best_position.len(), r2.best_position.len());
        for (a, b) in r1.best_position.iter().zip(r2.best_position.iter()) {
            assert_eq!(a.to_bits(), b.to_bits());
        }
    }

    #[test]
    fn pso_population_size() {
        let low = vec![0.0, 0.0];
        let high = vec![1.0, 1.0];
        let result = pso((low, high), 1, sphere_cost, 3, None);
        assert_eq!(result.best_position.len(), 2);
        let result = pso((vec![0.0, 0.0], vec![1.0, 1.0]), 40, sphere_cost, 2, None);
        assert_eq!(result.best_position.len(), 2);
    }
}
