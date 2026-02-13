//! WASM bindings for Monte Carlo (π estimation and 1D integration).

use wasm_bindgen::prelude::*;

/// Estimates π by Monte Carlo. With `simd` feature, uses vectorized path.
#[wasm_bindgen(js_name = estimatePi)]
pub fn estimate_pi_wasm(seed: u64, n_samples: u64) -> f64 {
    crate::estimate_pi(seed, n_samples)
}

/// Integrates ∫ₐᵇ x² dx by Monte Carlo (uniform sampling). For demo use; ∫₀¹ x² dx = 1/3.
#[wasm_bindgen(js_name = integrateXSquared)]
pub fn integrate_x_squared_wasm(a: f64, b: f64, n_samples: u64, seed: u64) -> f64 {
    crate::integrate_1d(|x| x * x, a, b, n_samples, seed)
}
