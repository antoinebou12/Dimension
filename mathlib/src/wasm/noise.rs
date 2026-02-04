//! Noise functions for JavaScript: wave, Perlin, and FBM (fractional Brownian motion).

use wasm_bindgen::prelude::*;

/// Wave height at (u, v) in [0, 1]². Returns value in [0, 1].
#[wasm_bindgen(js_name = wave2d)]
pub fn wave2d(u: f64, v: f64) -> f64 {
    crate::wave_2d(u, v)
}

/// Wave height with configurable wave numbers (radians per unit).
#[wasm_bindgen(js_name = wave2dParams)]
pub fn wave2d_params(u: f64, v: f64, k1: f64, k2: f64) -> f64 {
    crate::wave_2d_params(u, v, k1, k2)
}

/// 2D Perlin noise at (x, y). Output is approximately in [-1, 1].
#[wasm_bindgen(js_name = perlin2d)]
pub fn perlin2d(x: f64, y: f64) -> f64 {
    crate::perlin_2d(x, y)
}

/// FBM with Perlin base noise. Typical values: lacunarity 2.0, persistence 0.5.
#[wasm_bindgen(js_name = fbm2dPerlin)]
pub fn fbm2d_perlin(x: f64, y: f64, octaves: u32, lacunarity: f64, persistence: f64) -> f64 {
    crate::fbm_2d(x, y, octaves, lacunarity, persistence, crate::perlin_2d)
}
