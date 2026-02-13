//! Monte Carlo utilities: π estimation and 1D definite integration.
//!
//! Uses a deterministic RNG (`XorShift64`) so results are reproducible for a given
//! seed and no optional dependency is required.

/// Deterministic RNG (xorshift64) for reproducible Monte Carlo without a `rand` dependency.
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

    /// Uniform in [low, high).
    fn uniform_in_range(&mut self, low: f64, high: f64) -> f64 {
        low + self.uniform01() * (high - low)
    }
}

/// Estimates π by Monte Carlo: unit circle in [-1, 1]².
///
/// Samples `n_samples` points uniformly in the square [-1, 1]² and counts
/// how many fall inside the unit circle (x² + y² ≤ 1). The ratio of area
/// circle/square is π/4, so π ≈ 4 × (hits / `n_samples`).
///
/// With the `simd` feature, the inside-circle check is vectorized (4 samples per step).
///
/// # Examples
///
/// ```
/// let pi_est = mathlib::estimate_pi(42, 100_000);
/// assert!((pi_est - std::f64::consts::PI).abs() < 0.05);
/// ```
///
/// # Panics
///
/// Panics if `n_samples` is 0.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn estimate_pi(seed: u64, n_samples: u64) -> f64 {
    assert!(n_samples > 0, "n_samples must be positive");
    let n = n_samples as f64;

    #[cfg(feature = "simd")]
    {
        use wide::f64x4;
        let mut rng = XorShift64::new(seed);
        let mut sum = 0.0_f64;
        let chunks = (n_samples as usize) / 4;
        for _ in 0..chunks {
            let x = f64x4::new([
                rng.uniform_in_range(-1.0, 1.0),
                rng.uniform_in_range(-1.0, 1.0),
                rng.uniform_in_range(-1.0, 1.0),
                rng.uniform_in_range(-1.0, 1.0),
            ]);
            let y = f64x4::new([
                rng.uniform_in_range(-1.0, 1.0),
                rng.uniform_in_range(-1.0, 1.0),
                rng.uniform_in_range(-1.0, 1.0),
                rng.uniform_in_range(-1.0, 1.0),
            ]);
            let d = x * x + y * y;
            let d_arr = d.to_array();
            for v in d_arr {
                if v <= 1.0 {
                    sum += 1.0;
                }
            }
        }
        for _ in (chunks * 4)..(n_samples as usize) {
            let x = rng.uniform_in_range(-1.0, 1.0);
            let y = rng.uniform_in_range(-1.0, 1.0);
            if x * x + y * y <= 1.0 {
                sum += 1.0;
            }
        }
        return 4.0 * sum / n;
    }

    #[cfg(not(feature = "simd"))]
    {
        let mut rng = XorShift64::new(seed);
        let mut inside = 0u64;
        for _ in 0..n_samples {
            let x = rng.uniform_in_range(-1.0, 1.0);
            let y = rng.uniform_in_range(-1.0, 1.0);
            if x * x + y * y <= 1.0 {
                inside += 1;
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let in_f = inside as f64;
        4.0 * in_f / n
    }
}

/// Estimates ∫ₐᵇ f(x) dx by Monte Carlo with uniform sampling in [a, b].
///
/// Returns (b - a) × (1/n) × Σ f(xᵢ) where xᵢ are uniform in [a, b].
/// Same seed and n yield the same result.
///
/// # Examples
///
/// ```
/// // ∫₀¹ x² dx = 1/3
/// let integral = mathlib::integrate_1d(|x| x * x, 0.0, 1.0, 100_000, 123);
/// assert!((integral - 1.0 / 3.0).abs() < 0.02);
/// ```
///
/// # Panics
///
/// Panics if `n_samples` is 0.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn integrate_1d(f: impl Fn(f64) -> f64, a: f64, b: f64, n_samples: u64, seed: u64) -> f64 {
    assert!(n_samples > 0, "n_samples must be positive");
    let mut rng = XorShift64::new(seed);
    let n = n_samples as f64;
    let mut sum = 0.0;
    for _ in 0..n_samples {
        let x = rng.uniform_in_range(a, b);
        sum += f(x);
    }
    (b - a) * (sum / n)
}
