//! mathlib — Linear algebra: dense and sparse matrices, vectors, SVD, 3D math.
//!
//! Version: 0.1.0
//!
//! See the repository `docs/DOCS.md` for architecture and usage.
//!
//! # Logging
//!
//! The library emits events via [`tracing`] (`debug!`, `info!`, `warn!`, etc.). The library
//! **does not** initialize a global subscriber. Applications should set up a subscriber, for
//! example:
//!
//! ```ignore
//! tracing_subscriber::fmt()
//!     .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
//!     .init();
//! ```
//!
//! Then set `RUST_LOG` (e.g. `RUST_LOG=info,mathlib=debug`) to control verbosity.

#![deny(rust_2018_idioms)]
#![deny(clippy::correctness)]
#![warn(missing_docs)]
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::cargo,
    clippy::undocumented_unsafe_blocks
)]
#![allow(
    clippy::many_single_char_names,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::multiple_crate_versions,
    clippy::uninlined_format_args,
    clippy::needless_lifetimes,
    clippy::new_without_default,
    clippy::return_self_not_must_use,
    clippy::unreadable_literal,
    clippy::excessive_precision,
    clippy::doc_markdown,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::manual_midpoint
)]
#![allow(missing_docs)]

pub mod cpu;
pub mod executor;
pub mod lane;
pub mod structure;

/// Common imports for convenient glob importing.
///
/// # Usage
///
/// ```
/// use mathlib::prelude::*;
/// ```
///
/// # Contents
///
/// This prelude re-exports: [`Matrix`], [`Vector`], [`Cube`], [`Storage`], [`solve`],
/// [`Cholesky`], [`Lu`], [`Svd`], [`SvdEcon`], [`Pca`], core 3D types, and [`linear`] / [`ease_in_out_cubic`].
pub mod prelude {
    pub use crate::cg::{
        Perspective3, from_euler_angles, from_homogeneous, from_scaled_axis, look_at_lh,
        look_at_rh, matrix4_extract_rotation_quat, new_nonuniform_scaling, new_orthographic,
        new_perspective, new_perspective_wgpu, new_rotation_wrt_point, new_scaling,
        new_translation, transform_point, vector3, vector4_from_point, vector4_from_vector,
    };
    pub use crate::chol::{Cholesky, chol};
    pub use crate::cube::Cube;
    pub use crate::dual_quaternion::DualQuat4f;
    pub use crate::easing::{ease_in_out_cubic, linear};
    pub use crate::lu::Lu;
    pub use crate::math::twist::{clamp_twist, pose_twist_error};
    pub use crate::math3d::{
        Matrix3f, Matrix4f, OrthonormalBasis, Point3, Vector3f, Vector4f, center,
        euler_angles_close_to, from_homogeneous_point, from_homogeneous_vector, make_rotation,
        matrix3f_inverse, matrix4_mul_vector3, matrix4f_inverse, point_to_homogeneous,
        rotation_matrix_to_euler_xyz, transform_vector, vec3_cross_f64, vector_to_homogeneous,
        vector3_cross, wrap_angle_to_pi,
    };
    pub use crate::matrix::Matrix;
    pub use crate::solve::{damped_least_squares, solve};
    pub use crate::stats::covariance;
    pub use crate::structure::{COLUMN_STORAGE, ROW_STORAGE, Storage};
    pub use crate::vector::Vector;
    pub use crate::{Pca, pca};
    pub use crate::{Svd, SvdEcon, pinv, svd_econ};
}

pub use structure::{
    COLUMN_STORAGE, CubeBase, CubeSlice, DenseStorage, DenseStorageDynamic, DenseStorageTrait,
    Dynamic, Fill, MatrixBase, Quadruplet, ROW_STORAGE, SparseCube, SparseCubeBase, SparseMatrix,
    SparseMatrixBCRS, SparseMatrixBase, SparseMatrixCCS, SparseMatrixCDS, SparseMatrixCRS,
    SparseMatrixJDS, SparseMatrixSKS, SparseStorage, Storage, SubMatrix, Triplet,
};
pub use structure::{
    cube_base, cube_slice, dense_storage, matrix_base, sparse, sparse_cube, sparse_formats,
    submatrix, types,
};

pub mod clustering;
pub mod colormap;
pub mod cube;
pub mod distance;
#[cfg(feature = "genetic")]
pub mod genetic;
pub mod graph;
pub mod hash;
pub mod linear;
pub mod math;
pub mod noise;
pub use math::{
    cg, curve, dual_quaternion, easing, math3d, math3d_raw, quantize, quaternion, trig,
};
pub mod argmin;
pub mod decomposition;
pub mod differentiation;
#[cfg(feature = "gpu")]
pub mod gpu;
pub mod matrix;
pub mod monte_carlo;
pub mod norms;
pub mod ode;
pub mod operators;
pub mod quadrature;
pub mod rootfinding;
pub mod simplex;
pub mod stats;
pub mod svm;
pub mod transforms;
pub mod vector;

pub use linear::qr::{Qr, QrError, qr};
pub use linear::{chol, lu, qz, schur, solve};

pub use argmin::{
    CgError, GaussNewtonOptions, GaussNewtonResult, GradientDescentOptions, GradientDescentResult,
    LbfgsbOptions, LbfgsbResult, LevenbergMarquardtOptions, LevenbergMarquardtResult,
    LineSearchOptions, LineSearchVariant, NonlinearCgOptions, NonlinearCgResult, PsoOptions,
    PsoResult, armijo, backtracking, gauss_newton, gradient_descent, lbfgsb, levenberg_marquardt,
    muon_step, nonlinear_cg, pso, solve_cg, solve_cg_sparse, wolfe,
};
pub use cg::{
    Perspective3, append_nonuniform_scaling, append_nonuniform_scaling_mut, append_scaling,
    append_scaling_mut, append_translation, append_translation_mut, from_euler_angles,
    from_homogeneous, from_scaled_axis, look_at_lh, look_at_rh, matrix4_extract_rotation_quat,
    matrix4f_identity, matrix4f_to_array, matrix4f_translation, model_view_projection,
    new_nonuniform_scaling, new_orthographic, new_perspective, new_perspective_wgpu,
    new_rotation_wrt_point, new_scaling, new_translation, prepend_nonuniform_scaling,
    prepend_nonuniform_scaling_mut, prepend_scaling, prepend_scaling_mut, prepend_translation,
    prepend_translation_mut, screen_to_view_ray, transform_point, vector3, vector4_from_point,
    vector4_from_vector,
};
pub use chol::{CholError, Cholesky, chol};
pub use clustering::{DbscanResult, KmeansResult, NOISE, dbscan, kmeans};
pub use cube::Cube;
pub use decomposition::pca::{Pca, pca};
pub use decomposition::procrustes_orthogonal;
pub use decomposition::svd::{Svd, SvdEcon, pinv, svd_econ};
pub use decomposition::tsne::{TsneOptions, tsne};
pub use decomposition::umap::{UmapOptions, umap};
pub use differentiation::{default_step, diff_backward, diff_central, diff_forward, diff2_central};
pub use distance::{
    chebyshev, chebyshev_rows, cosine_distance, cosine_similarity, euclidean, euclidean_rows,
    manhattan, manhattan_rows, minkowski, minkowski_rows, squared_euclidean,
    squared_euclidean_rows,
};
pub use dual_quaternion::DualQuat4f;
#[cfg(feature = "genetic")]
pub use genetic::{CmaEs, CmaEsBuilder, CmaEsResult};
pub use graph::disjoint::UnionFind;
pub use graph::disjoint_set::DisjointSet;
pub use graph::dstar::{DStarLite, DStarLiteResult};
pub use graph::{
    AStarResult, BfsResult, DijkstraResult, Edge, Graph, Node, NodeId, Tree, Weight,
    articulation_points, astar, bfs, bridges, connected_components,
    connected_components_undirected, dfs_postorder, dfs_postorder_forest, dfs_preorder,
    dfs_preorder_forest, dijkstra, dsatur_coloring, dstar_lite, greedy_vertex_coloring,
    is_bipartite, path_to_traversal_mapping, reverse_graph,
};
pub use hash::HashableElement;
pub use lane::{
    LaneCount, LaneScalar, SimdLane, as_f32x4_chunks, as_f32x4_chunks_mut, as_f64x4_chunks,
    as_f64x4_chunks_mut,
};
pub use lu::{Lu, LuError, det};
pub use math::twist::{clamp_twist, pose_twist_error};
pub use math3d::{
    Matrix3f, Matrix4f, OrthonormalBasis, Point3, Vector3f, Vector4f, center,
    euler_angles_close_to, from_homogeneous_point, from_homogeneous_vector, make_rotation,
    matrix3f_inverse, matrix4_mul_vector3, matrix4f_inverse, point_to_homogeneous,
    rotation_matrix_to_euler_xyz, transform_vector, vec3_cross_f64, vector_to_homogeneous,
    vector3_cross, wrap_angle_to_pi,
};
pub use matrix::Matrix;
pub use monte_carlo::{estimate_pi, integrate_1d};
pub use norms::{frobenius_norm_f32, frobenius_norm_f64, spectral_norm_f64};
pub use ode::{OdeResult, euler, euler_step, rk4, rk4_step, trapezoidal_step};
pub use quadrature::{gauss_legendre, simpson, trapezoidal};
pub use quantize::{
    decode_half, decode_snorm, decode_unorm, normalize_f32_in_0_1, normalize_f32_in_neg1_pos1,
    pack_4_f32_to_snorm, pack_4_f32_to_unorm, quantize_half, quantize_snorm, quantize_unorm,
    unpack_snorm_to_4_f32, unpack_unorm_to_4_f32,
};
pub use quaternion::Quat4f;
pub use qz::{Qz, QzError, qz};
pub use rootfinding::{RootResult, bisection, brent, newton_1d, secant};
pub use schur::{Schur, SchurError, schur};
pub use simplex::{SimplexError, SimplexResult, SimplexStatus, simplex_solve};
pub use solve::{SolveError, damped_least_squares, solve};
pub use stats::covariance;
pub use svm::{SvmError, SvmOptions, SvmRbfResult, SvmResult, svm, svm_rbf};
pub use transforms::{
    Complex64, TransformsError, apply_window, apply_window_in_place, blackman, conv_1d,
    conv_1d_same, conv_2d, dct2_forward, dct2_inverse, dwt_haar_forward, dwt_haar_inverse,
    fft_forward, fft_forward_real, fft_inverse, hamming, hann, tukey,
};
pub use trig::{
    acos, acosh, asin, asinh, atan, atan2, atanh, cos, cos_scalar, cosh, degrees, radians, sin,
    sin_scalar, sinh, tan, tanh,
};
pub use vector::{Float, RealNumber, Vector};

#[cfg(feature = "gpu")]
pub use executor::{AutoExecutor, GpuExecutor};
pub use executor::{CpuExecutor, Executor, ExecutorThresholds};
#[cfg(feature = "gpu")]
pub use gpu::{GpuConfig, PowerPreference};

pub use colormap::{
    Hsv, Rgb, Rgba, height_to_rgb, height_to_rgba, hex_to_rgb, hex_to_rgba, hsv_to_rgb, rgb_to_hex,
    rgb_to_hsv, rgb_to_rgba, rgba_to_hex, rgba_to_rgb, scalar_to_rgb_inferno, scalar_to_rgb_magma,
    scalar_to_rgb_plasma, scalar_to_rgb_preset, scalar_to_rgb_viridis,
};
pub use easing::{
    bspline, ease_in_back, ease_in_bounce, ease_in_circ, ease_in_cubic, ease_in_elastic,
    ease_in_expo, ease_in_out_back, ease_in_out_bounce, ease_in_out_circ, ease_in_out_cubic,
    ease_in_out_elastic, ease_in_out_expo, ease_in_out_quad, ease_in_out_quart, ease_in_out_quint,
    ease_in_out_sine, ease_in_quad, ease_in_quart, ease_in_quint, ease_in_sine, ease_out_back,
    ease_out_bounce, ease_out_circ, ease_out_cubic, ease_out_elastic, ease_out_expo, ease_out_quad,
    ease_out_quart, ease_out_quint, ease_out_sine, hermite, lerp, linear,
};
pub use math::rbf::{
    RbfEasing, RbfVariant, rbf_kernel, rbf_kernel_batch, rbf_kernel_eased, rbf_kernel_normalized,
};
pub use noise::{fbm_2d, perlin_2d, wave_2d, wave_2d_params};

// Domain re-export modules (optional namespaces; crate root API unchanged).
/// ML domain: clustering, svm, distance, tsne, umap.
pub mod ml {
    pub use crate::decomposition::{tsne, umap};
    pub use crate::{clustering, distance, svm};
}
/// Optimisation domain: argmin, genetic (if feature enabled).
pub mod optimisation {
    pub use crate::argmin;
    #[cfg(feature = "genetic")]
    pub use crate::genetic;
}
/// Tree traversal (BFS, DFS) from graph.
pub mod tree {
    pub use crate::graph::tree::*;
}
/// Tensor domain: Cube and related types.
pub mod tensor {
    pub use crate::cube::*;
}

#[cfg(feature = "wasm")]
pub mod wasm;
