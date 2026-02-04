# Changelog

All notable changes to mathlib will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

#### Parallel Execution Backend
- **BREAKING**: Replaced Rayon with par-iter + chili backend for parallel execution
  - The `parallel` feature now uses par-iter (v2.0) with chili backend instead of Rayon
  - Chili uses "heartbeat scheduling" instead of work-stealing, which provides better performance for workloads with many short-lived tasks
  - API remains identical - all `.par_iter()`, `.into_par_iter()` calls work unchanged
  - Affected modules: graph algorithms (Dijkstra, A*, BFS), clustering (K-means, DBSCAN), optimization (PSO, CMA-ES), statistics (covariance), transforms (convolution)
  - All tests pass with the new backend; parallel correctness verified
  - Performance characteristics may differ from Rayon - heartbeat scheduling reduces synchronization overhead
  - Still not available on `wasm32` target (par-iter with chili does not support wasm32)

### Added

#### GPU (optional `gpu` feature)
- Matrix-vector multiply (A × v), element-wise add/sub for matrices and vectors
- `try_matvec_f32`, `try_add_f32`, `try_sub_f32` in `mathlib::gpu`
- Transparent GPU routing in `Matrix * Vector`, `A + B`, `A - B`
- Large-matrix example: `examples/gpu/gpu_large_matrix.rs`
- GPU benchmarks: matvec, add; matmul sizes 2048, 4096
- WASM demo: large matrix subsection with size selector; vector dot/norm display
- Additional GPU kernels: scale (s * A, s * v), element-wise mul, div, axpy, abs, sqrt
- Sparse SpMV: `try_spmv_f32`; transparent in `SparseMatrixCRS<f32> * Vector<f32>`
- `try_squared_norm_f32` wrapper
- Unit tests and benchmarks for all new GPU ops

#### Examples reorganization
- Examples grouped by domain: `linear/`, `ml/`, `wasm/`, `gpu/`, `optimization/`, `math/`, `graph/`, `transforms/`, `viz/`

#### Core Types
- `Matrix<T>` — Dense matrix with column-major or row-major storage
- `Vector<T>` — Column vector with dot product, norm, resize
- `Cube<T>` — N-dimensional tensor
- `SubMatrix` — Non-owning block view into a matrix
- `Storage` enum (`Column`, `Row`) for storage layout selection

#### Sparse Matrices
- `SparseMatrixCRS` — Compressed Row Storage format
- `SparseMatrixCCS` — Compressed Column Storage format
- `SparseMatrixCDS` — Compressed Diagonal Storage format
- `SparseMatrixBCRS` — Block Compressed Row Storage format
- `SparseMatrixJDS` — Jagged Diagonal Storage format
- `SparseMatrixSKS` — Skyline Storage format

#### Decompositions
- `Svd`, `SvdEcon` — Singular Value Decomposition (Golub-Reinsch)
- `Cholesky` — Cholesky decomposition (A = L L^T)
- `Lu` — LU decomposition with partial pivoting
- `Schur` — Real Schur decomposition
- `Qz` — Generalized Schur (QZ) decomposition
- `Pca` — Principal Component Analysis via SVD

#### Linear Solvers
- `solve(A, b)` — General linear solve for square systems
- `Cholesky::solve(b)` — Solve via Cholesky factor
- `Lu::solve(b)` — Solve via LU factors
- `det(A)` — Determinant via LU decomposition

#### 3D Math
- Type aliases: `Vector3f`, `Vector4f`, `Matrix3f`, `Matrix4f`
- `Point3` — 3D point type (distinct from vectors)
- `Quat4f` — Quaternion with slerp interpolation
- `make_rotation` — 3×3 rotation from Euler angles
- `matrix4f_inverse`, `matrix3f_inverse` — Matrix inversion
- `matrix4_mul_vector3`, `transform_vector` — Transformations

#### Camera and Projection
- `look_at_rh`, `look_at_lh` — View matrix construction
- `new_perspective`, `new_orthographic` — Projection matrices
- `new_translation`, `new_scaling`, `new_rotation_wrt_point`
- `model_view_projection` — Combined MVP matrix
- `screen_to_view_ray` — Unproject screen coordinates
- `transform_point` — Apply transformation to point

#### Clustering
- `kmeans` — K-means clustering
- `dbscan` — DBSCAN density-based clustering

#### Distance Metrics
- `euclidean`, `squared_euclidean` — Euclidean distance
- `manhattan` — L1 distance
- `cosine_similarity`, `cosine_distance` — Cosine metrics
- `minkowski` — Minkowski distance with arbitrary p
- `chebyshev` — L-infinity distance
- Row variants (`*_rows`) for matrix row comparisons

#### Graph Algorithms
- `Graph` — Directed weighted graph (adjacency list)
- `dijkstra` — Single-source shortest paths
- `astar` — A* pathfinding with heuristic
- `dstar_lite`, `DStarLite` — D* Lite for dynamic replanning
- `UnionFind`, `DisjointSet` — Union-find data structures
- `connected_components`, `connected_components_undirected`
- `articulation_points`, `bridges` — Graph analysis
- `greedy_vertex_coloring`, `is_bipartite`

#### Optimization
- `pso` — Particle Swarm Optimization
- `gradient_descent` — Gradient descent with line search
- `nonlinear_cg` — Nonlinear conjugate gradient
- `gauss_newton` — Gauss-Newton for nonlinear least squares
- Line search variants: `armijo`, `wolfe`, `backtracking`
- `CmaEs` — CMA-ES evolution strategy (requires `genetic` feature)

#### Support Vector Machines
- `svm` — Linear SVM binary classification
- `svm_rbf` — SVM with RBF kernel

#### Interpolation and Easing
- `lerp`, `linear` — Linear interpolation
- `hermite`, `bspline` — Spline interpolation
- `ease_in_*`, `ease_out_*`, `ease_in_out_*` — 30+ easing functions
- `Quat4f::slerp` — Spherical linear interpolation

#### Trigonometry
- `sin`, `cos`, `tan`, `atan2` — Trig functions
- `degrees`, `radians` — Angle conversion
- Hyperbolic variants: `sinh`, `cosh`, `tanh`, etc.

#### Procedural Generation
- `perlin_2d` — 2D Perlin noise
- `fbm_2d` — Fractional Brownian motion
- `wave_2d`, `wave_2d_params` — Sinusoidal waves

#### Color
- `Rgb`, `Rgba`, `Hsv` — Color types
- `rgb_to_hex`, `hex_to_rgb`, etc. — Format conversions
- `rgb_to_hsv`, `hsv_to_rgb` — Color space conversions
- `height_to_rgb`, `height_to_rgba` — Elevation palettes

#### WebAssembly
- `WasmMatrix`, `WasmVector` — Matrix/vector bindings for JS
- `WasmMatrix32` — f32 matrix for 3D graphics
- `WasmSvd` — SVD result accessors
- `WasmCg` — Camera/projection helpers
- `WasmPca` — PCA for dimensionality reduction
- `WasmKmeans` — K-means clustering
- `WasmCholesky` — Cholesky decomposition
- `WasmDistance` — Distance metric functions

#### Features
- `serde` — Serialization support for key types
- `parallel` — Rayon-based parallelization (non-WASM)
- `simd` — SIMD acceleration via `wide` crate
- `wasm` — WebAssembly bindings
- `genetic` — CMA-ES optimizer
- `full` — Shorthand for `parallel` + `simd`

[Unreleased]: https://github.com/anthropics/dimension/compare/v0.1.0...HEAD
