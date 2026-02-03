//! Noise functions for heightmaps and procedural generation.
//!
//! - **Sin/wave**: deterministic sinusoidal wave (`wave_2d`, `wave_2d_params`).
//! - **Perlin**: 2D Perlin gradient noise (`perlin_2d`).
//! - **FBM**: fractional Brownian motion over any 2D noise (`fbm_2d`).

pub mod fbm;
pub mod perlin;
pub mod sin;

pub use fbm::fbm_2d;
pub use perlin::perlin_2d;
pub use sin::{wave_2d, wave_2d_params};
