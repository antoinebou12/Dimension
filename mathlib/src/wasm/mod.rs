//! WebAssembly bindings for mathlib.
//!
//! Enable with `--features wasm`. Build with:
//! ```bash
//! wasm-pack build --target web --features wasm
//! ```
//!
//! Exposes dense matrices (f64/f32), vectors, linear solve Ax = b, damped least-squares, SVD, PCA, Cholesky, LU,
//! 3D/camera helpers, k-means, DBSCAN, distance metrics, linear SVM, simplex LP solver,
//! PSO optimization, graph pathfinding (Dijkstra), noise (wave, Perlin, FBM),
//! and easing (lerp, distance) for use from JavaScript.
//!
//! # Exported types (index for JS discoverability)
//!
//! - **Matrices / vectors**: [`WasmMatrix`], [`WasmMatrix32`], [`WasmVector`]
//! - **Decompositions**: [`WasmSvd`], [`WasmPca`], [`WasmCholesky`], [`WasmLu`]
//! - **Camera**: [`WasmCg`]
//! - **Clustering**: [`WasmKmeans`], [`WasmDbscan`]
//! - **Distance**: [`WasmDistance`]
//! - **SVM**: [`WasmSvm`], [`WasmSvmResult`]
//! - **Simplex LP**: [`WasmSimplexResult`]
//! - **Graph**: [`WasmGraph`], [`WasmDijkstraResult`], [`WasmAstarResult`], [`WasmDStarLiteResult`]
//! - **Argmin**: [`WasmPsoResult`], `psoMinimize`, `psoMinimizeWithHistory`, `lineSearchBacktracking`
//! - **Noise**: `wave2d`, `wave2dParams`, `perlin2d`, `fbm2dPerlin`
//! - **Colormap**: `heightToRgb` (elevation-style palette)
//! - **Dual quaternion**: [`WasmDualQuat`] (rigid transforms)

mod argmin;
mod camera;
mod clustering;
mod colormap;
mod decomposition;
mod distance;
mod dual_quaternion;
#[cfg(feature = "gpu")]
mod gpu;
mod graph;
mod matrix;
mod monte_carlo;
mod noise;
mod simplex;
mod svm;
mod transforms;
mod vector;

pub use self::transforms::{
    apply_window_wasm, blackman_wasm, conv_1d_same_wasm, conv_1d_wasm, dct2_forward_wasm,
    dct2_inverse_wasm, dwt_haar_forward_wasm, dwt_haar_inverse_wasm, fft_forward_real_wasm,
    fft_inverse_wasm, hamming_wasm, hann_wasm, tukey_wasm,
};
pub use argmin::{
    WasmPsoResult, WasmPsoResultWithHistory, line_search_backtracking, pso_minimize,
    pso_minimize_with_history,
};
pub use camera::WasmCg;
pub use clustering::{WasmDbscan, WasmKmeans, dbscan_noise_label};
pub use colormap::height_to_rgb_wasm;
pub use decomposition::{WasmCholesky, WasmLu, WasmPca};
pub use distance::WasmDistance;
pub use dual_quaternion::WasmDualQuat;
pub use graph::{
    WasmAstarResult, WasmBfsResult, WasmDStarLiteResult, WasmDijkstraResult, WasmGraph,
};
pub use matrix::{WasmMatrix, WasmMatrix32, WasmSvd};
pub use monte_carlo::{estimate_pi_wasm, integrate_x_squared_wasm};
pub use noise::{fbm2d_perlin, perlin2d, wave2d, wave2d_params};
pub use simplex::WasmSimplexResult;
pub use svm::{WasmSvm, WasmSvmRbf, WasmSvmRbfResult, WasmSvmResult};
pub use vector::{WasmVector, dot_f32};

#[cfg(feature = "gpu")]
pub use gpu::{
    add_f32_gpu_async, dot_f32_gpu_async, gpu_available, gpu_last_error, gpu_matmul_available,
    init_gpu_async, matmul_f32_gpu_async, matvec_f32_gpu_async, norm_f32_gpu_async,
    scale_f32_gpu_async,
};
