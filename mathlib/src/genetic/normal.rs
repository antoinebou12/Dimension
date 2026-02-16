//! In-crate standard normal N(0,1) sampling for the genetic feature.
//!
//! Uses the Marsaglia polar method: transforms two uniforms into two normals
//! with no sin/cos. One sample is returned per call; the method naturally
//! produces two normals per acceptance—we use one and discard the other
//! to keep the API stateless.

use rand::{Rng, RngExt};

/// Sample one draw from the standard normal distribution N(0, 1).
///
/// Uses the Marsaglia polar method. Requires the given RNG to produce
/// uniform f64 values in [0, 1) (e.g. `rand::Rng::random`).
///
/// # Example
///
/// ```
/// # #[cfg(feature = "genetic")] {
/// use rand::rngs::StdRng;
/// use rand::SeedableRng;
/// use mathlib::genetic::normal::sample_standard_normal;
///
/// let mut rng = StdRng::seed_from_u64(42);
/// let z: f64 = sample_standard_normal(&mut rng);
/// # }
/// ```
#[inline]
#[must_use]
pub fn sample_standard_normal<R: Rng + ?Sized>(rng: &mut R) -> f64 {
    loop {
        let u: f64 = 2.0 * rng.random::<f64>() - 1.0;
        let v: f64 = 2.0 * rng.random::<f64>() - 1.0;
        let s: f64 = u * u + v * v;
        if s < 1.0 && s > 0.0 {
            let t: f64 = (-2.0 * s.ln() / s).sqrt();
            return u * t;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn standard_normal_mean_and_variance() {
        let mut rng = StdRng::seed_from_u64(12345);
        let n = 100_000_usize;
        let sum: f64 = (0..n).map(|_| sample_standard_normal(&mut rng)).sum();
        let mean = sum / n as f64;
        let mut sum_sq = 0.0;
        let mut rng2 = StdRng::seed_from_u64(12345);
        for _ in 0..n {
            let z = sample_standard_normal(&mut rng2);
            sum_sq += (z - mean) * (z - mean);
        }
        let variance = sum_sq / (n - 1) as f64;
        assert!(
            mean.abs() < 0.1,
            "sample mean should be near 0, got {}",
            mean
        );
        assert!(
            (variance - 1.0).abs() < 0.1,
            "sample variance should be near 1, got {}",
            variance
        );
    }
}
