//! RBF (Radial Basis Function) interpolation and kernel.
//!
//! Provides the Gaussian kernel K(a,b) = exp(-γ ‖a - b‖²) from squared distance, with optional
//! easing to shape the falloff and variants (e.g. normalized). Used by [`crate::svm::svm_rbf`] and
//! reusable for interpolation or other kernel methods.

use crate::math::easing::{ease_in_out_cubic, ease_out_cubic, linear};

/// Variant of the RBF kernel.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RbfVariant {
    /// Standard Gaussian: K(a,b) = exp(-γ ‖a - b‖²). K(x,x) = 1.
    #[default]
    Gaussian,
    /// Normalized: K(a,b) / sqrt(K(a,a) * K(b,b)). For Gaussian this equals K(a,b) since K(x,x) = 1.
    NormalizedGaussian,
}

/// Built-in easing applied to the raw kernel value (in [0, 1]) to shape the falloff.
#[derive(Clone, Copy, Debug, Default)]
pub enum RbfEasing {
    /// No shaping: use raw exp(-γ d²).
    #[default]
    Linear,
    /// Ease-out cubic: sharper drop near zero distance.
    EaseOutCubic,
    /// Ease-in-out cubic: smooth S-shaped falloff.
    EaseInOutCubic,
}

impl RbfEasing {
    /// Apply this easing to a value `t` in [0, 1].
    #[inline]
    pub fn apply(self, t: f64) -> f64 {
        match self {
            RbfEasing::Linear => linear(t),
            RbfEasing::EaseOutCubic => ease_out_cubic(t),
            RbfEasing::EaseInOutCubic => ease_in_out_cubic(t),
        }
    }
}

/// Gaussian RBF kernel value from squared distance: exp(-γ * `dist_sq`).
///
/// # Example
///
/// ```
/// use mathlib::math::rbf::rbf_kernel;
/// let k = rbf_kernel(0.0, 0.5);
/// assert!((k - 1.0).abs() < 1e-10);
/// let k_far = rbf_kernel(100.0, 0.5);
/// assert!(k_far < 1e-10);
/// ```
#[inline]
#[must_use]
pub fn rbf_kernel(dist_sq: f64, gamma: f64) -> f64 {
    (-gamma * dist_sq).exp()
}

/// RBF kernel with optional easing applied to the raw value (in [0, 1]).
///
/// Computes raw = exp(-γ * `dist_sq`) then returns `easing.apply(raw)` so the falloff curve
/// can be shaped (e.g. ease-out cubic for a sharper drop).
#[inline]
#[must_use]
pub fn rbf_kernel_eased(dist_sq: f64, gamma: f64, easing: RbfEasing) -> f64 {
    let raw = (-gamma * dist_sq).exp();
    easing.apply(raw)
}

/// Normalized RBF kernel: K(a,b) / sqrt(K(a,a) * K(b,b)).
///
/// For Gaussian, K(x,x) = 1 so this returns the same as [`rbf_kernel`](rbf_kernel). For other
/// future bases the denominator would differ from 1.
#[inline]
#[allow(clippy::similar_names)]
#[must_use]
pub fn rbf_kernel_normalized(dist_sq_ab: f64, dist_sq_aa: f64, dist_sq_bb: f64, gamma: f64) -> f64 {
    let k_ab = (-gamma * dist_sq_ab).exp();
    let k_aa = (-gamma * dist_sq_aa).exp();
    let k_bb = (-gamma * dist_sq_bb).exp();
    let denom = (k_aa * k_bb).sqrt();
    if denom <= 0.0 { k_ab } else { k_ab / denom }
}

/// Compute RBF kernel values for a slice of squared distances.
///
/// Writes `out[i] = exp(-gamma * dist_sq[i])` for each element. When the `simd` feature is
/// enabled, dispatches to a SIMD backend when beneficial; otherwise uses a scalar loop.
pub fn rbf_kernel_batch(gamma: f64, dist_sq: &[f64], out: &mut [f64]) {
    assert_eq!(dist_sq.len(), out.len());
    #[cfg(feature = "simd")]
    {
        crate::cpu::simd::rbf_kernel_batch_f64(gamma, dist_sq, out);
    }
    #[cfg(not(feature = "simd"))]
    {
        for (i, &d) in dist_sq.iter().enumerate() {
            out[i] = (-gamma * d).exp();
        }
    }
}

#[cfg(test)]
#[allow(clippy::similar_names)]
mod tests {
    use super::*;

    #[test]
    fn rbf_kernel_zero_distance_is_one() {
        for gamma in [0.1, 0.5, 1.0, 2.0] {
            let k = rbf_kernel(0.0, gamma);
            assert!((k - 1.0).abs() < 1e-10, "gamma={}", gamma);
        }
    }

    #[test]
    fn rbf_kernel_large_distance_near_zero() {
        let k = rbf_kernel(1000.0, 0.5);
        assert!(k < 1e-100);
    }

    #[test]
    fn rbf_kernel_eased_linear_matches_uneased() {
        let dist_sq = 1.0_f64;
        let gamma = 0.5;
        let k = rbf_kernel(dist_sq, gamma);
        let k_eased = rbf_kernel_eased(dist_sq, gamma, RbfEasing::Linear);
        assert!((k - k_eased).abs() < 1e-15);
    }

    #[test]
    fn rbf_kernel_normalized_matches_gaussian_when_diagonal_one() {
        // K(a,a) = K(b,b) = 1 for Gaussian, so normalized = K(a,b).
        let dist_sq_ab = 2.0;
        let dist_sq_aa = 0.0;
        let dist_sq_bb = 0.0;
        let gamma = 0.5;
        let k_norm = rbf_kernel_normalized(dist_sq_ab, dist_sq_aa, dist_sq_bb, gamma);
        let k = rbf_kernel(dist_sq_ab, gamma);
        assert!((k_norm - k).abs() < 1e-15);
    }

    #[test]
    fn rbf_kernel_batch_single() {
        let gamma = 0.5;
        let dist_sq = [0.0, 1.0, 2.0];
        let mut out = [0.0; 3];
        rbf_kernel_batch(gamma, &dist_sq, &mut out);
        assert!((out[0] - 1.0).abs() < 1e-10);
        assert!((out[1] - (-0.5_f64).exp()).abs() < 1e-10);
        assert!((out[2] - (-1.0_f64).exp()).abs() < 1e-10);
    }

    #[test]
    fn rbf_kernel_monotonic_in_dist_sq() {
        let gamma = 0.5;
        let mut prev = rbf_kernel(0.0, gamma);
        for dist_sq in [0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 50.0] {
            let k = rbf_kernel(dist_sq, gamma);
            assert!(k < prev, "kernel should decrease as dist_sq increases");
            assert!(k > 0.0 && k <= 1.0);
            prev = k;
        }
    }

    #[test]
    fn rbf_kernel_eased_bounds() {
        let gamma = 0.5;
        for dist_sq in [0.0, 0.5, 1.0, 2.0, 10.0] {
            for easing in [
                RbfEasing::Linear,
                RbfEasing::EaseOutCubic,
                RbfEasing::EaseInOutCubic,
            ] {
                let k = rbf_kernel_eased(dist_sq, gamma, easing);
                assert!(
                    (0.0..=1.0).contains(&k),
                    "eased kernel in [0,1], got {} dist_sq={}",
                    k,
                    dist_sq
                );
            }
        }
    }

    #[test]
    fn rbf_kernel_eased_ease_out_cubic_differs_from_linear_at_mid() {
        let dist_sq = 1.0_f64; // raw = exp(-0.5) in (0,1)
        let gamma = 0.5;
        let k_linear = rbf_kernel_eased(dist_sq, gamma, RbfEasing::Linear);
        let k_ease_out = rbf_kernel_eased(dist_sq, gamma, RbfEasing::EaseOutCubic);
        assert!(
            (k_ease_out - k_linear).abs() > 1e-10,
            "EaseOutCubic should shape the value"
        );
        assert!((k_linear - 1.0).abs() > 1e-10, "sanity: mid value not 1");
    }

    #[test]
    fn rbf_kernel_normalized_scale() {
        // When K_aa and K_bb are not 1, normalized divides by sqrt(K_aa * K_bb).
        let gamma = 1.0;
        let dist_sq_ab = 1.0;
        let dist_sq_aa = 1.0; // K_aa = exp(-1)
        let dist_sq_bb = 1.0; // K_bb = exp(-1)
        let k_norm = rbf_kernel_normalized(dist_sq_ab, dist_sq_aa, dist_sq_bb, gamma);
        let k_raw = rbf_kernel(dist_sq_ab, gamma);
        let denom = (-gamma * dist_sq_aa).exp() * (-gamma * dist_sq_bb).exp();
        let expected = k_raw / denom.sqrt();
        assert!((k_norm - expected).abs() < 1e-15);
        assert!(
            k_norm > k_raw,
            "normalized with same-point < 1 should be larger"
        );
    }

    #[test]
    fn rbf_kernel_batch_matches_scalar() {
        let gamma = 0.25;
        let dist_sq: Vec<f64> = (0..20).map(|i| f64::from(i) * 0.5).collect();
        let mut out = vec![0.0; dist_sq.len()];
        rbf_kernel_batch(gamma, &dist_sq, &mut out);
        for (i, &d) in dist_sq.iter().enumerate() {
            let expected = rbf_kernel(d, gamma);
            assert!(
                (out[i] - expected).abs() < 1e-14,
                "batch[{}] = {} expected {}",
                i,
                out[i],
                expected
            );
        }
    }

    #[test]
    fn rbf_kernel_batch_empty() {
        let gamma = 0.5;
        let dist_sq: [f64; 0] = [];
        let mut out: [f64; 0] = [];
        rbf_kernel_batch(gamma, &dist_sq, &mut out);
    }

    #[test]
    fn rbf_easing_apply_boundaries() {
        assert!((RbfEasing::Linear.apply(0.0) - 0.0).abs() < 1e-15);
        assert!((RbfEasing::Linear.apply(1.0) - 1.0).abs() < 1e-15);
        assert!((RbfEasing::EaseOutCubic.apply(0.0) - 0.0).abs() < 1e-15);
        assert!((RbfEasing::EaseOutCubic.apply(1.0) - 1.0).abs() < 1e-15);
        assert!((RbfEasing::EaseInOutCubic.apply(0.0) - 0.0).abs() < 1e-15);
        assert!((RbfEasing::EaseInOutCubic.apply(1.0) - 1.0).abs() < 1e-15);
    }

    #[test]
    fn rbf_variant_default_is_gaussian() {
        assert_eq!(RbfVariant::default(), RbfVariant::Gaussian);
    }

    #[test]
    fn rbf_easing_default_is_linear() {
        assert!(matches!(RbfEasing::default(), RbfEasing::Linear));
    }
}
