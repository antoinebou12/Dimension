//! Fractional Brownian motion: octave summation over a base 2D noise.

/// FBM: sum over octaves of `amplitude * noise(frequency * x, frequency * y)`.
///
/// `lacunarity` multiplies frequency each octave; `persistence` multiplies amplitude each octave.
/// Result is normalized by the sum of amplitudes so the output stays bounded.
/// Typical values: lacunarity 2.0, persistence 0.5.
#[inline]
pub fn fbm_2d<F>(x: f64, y: f64, octaves: u32, lacunarity: f64, persistence: f64, noise: F) -> f64
where
    F: Fn(f64, f64) -> f64,
{
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        value += amplitude * noise(x * frequency, y * frequency);
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    value / max_value
}

#[cfg(test)]
mod tests {
    use super::fbm_2d;

    #[test]
    fn fbm_2d_deterministic() {
        let n = |x: f64, y: f64| x + y;
        let a = fbm_2d(1.0, 2.0, 4, 2.0, 0.5, n);
        let b = fbm_2d(1.0, 2.0, 4, 2.0, 0.5, n);
        assert_eq!(a, b);
    }

    #[test]
    fn fbm_2d_bounded_linear_base() {
        let n = |x: f64, y: f64| x + y;
        let v = fbm_2d(0.5, 0.5, 4, 2.0, 0.5, n);
        assert!(v.is_finite(), "fbm_2d should be finite");
    }
}
