# mathlib — Architecture and usage

**mathlib** is a Rust linear algebra crate in the Dimension repo. It provides dense and sparse matrices, vectors, SVD decomposition, and 3D math helpers under the `mathlib` namespace.

For AI/LLM context see [docs/claude.md](claude.md) and [AGENTS.md](../AGENTS.md). Domain taxonomy: [docs/domains.md](domains.md). Unified WASM website (build/serve, GitHub Pages): [docs/website.md](website.md).

## Domain organization

The codebase is organized by **domain**: linear (solvers, decompositions, simplex), structure (storage, matrix, sparse), vector, ml (clustering, svm, distance), optimisation (argmin, genetic), graph, tree, cg (camera, math3d, quaternion, trig, easing), noise, transforms, colormap, stats, tensor (Cube). Cross-cutting: runtime (cpu, gpu), wasm. Tests, benches, and examples follow the same domains (e.g. `tests/linear/`, `tests/ml/`, `benches/linear/`, `examples/cg/`). See [domains.md](domains.md) for the full table and paths.

## Documentation standards

Public API documentation follows the Rust skill doc-* rules (e.g. doc-all-public, doc-examples-section, doc-errors-section). See [AGENTS.md](../AGENTS.md#documentation) for the full documentation checklist.

## Overview

- **Namespace:** All public API is under the `mathlib` crate (e.g. `mathlib::Matrix`, `mathlib::Vector`).
- **Purpose:** Dense/sparse matrices, vectors, block views (SubMatrix), N-dimensional cubes (Cube), SVD and other decompositions (Cholesky, LU, Schur, QZ), 3D types (Vector3f, Matrix3f, Matrix4f) with inverses and rotation, camera/projection (cg: look-at, perspective, orthographic, MVP), clustering (dbscan, kmeans), distance metrics (euclidean, cosine, manhattan, etc.), PCA, particle swarm optimization (pso), quaternions (Quat4f), dual quaternions (DualQuat4f) for rigid transforms, trig (degrees/radians, sin/cos/atan2), and optional lane/SIMD utilities.

## Architecture

### Storage

- **Column vs row:** `Storage` enum (`Storage::Column`, `Storage::Row`); constants `COLUMN_STORAGE`, `ROW_STORAGE`.
- **Layout:** Column-major: index `(i, j)` → `j * rows + i`; row-major: `i * cols + j`.

### Matrix multiplication and storage

- **Matrix × matrix:** Entry (AB)_{i,j} = row_i(A)·col_j(B). So column j of AB = A × (column j of B); each column of the product is a matrix–vector product.
- **Matrix × vector:** y = A×v is the linear combination of columns of A: y = v_1·(col_1 of A) + v_2·(col_2 of A) + … . Equivalently, y_i = row i of A · v.
- **Wasm / JS:** mathlib and `WasmMatrix` use column-major storage. `fromArray(rows, cols, data)` expects a flat array in column-major order: `[col_0, col_1, …]`, i.e. index (i, j) → `data[j * rows + i]`.

### Fixed vs dynamic size

- **Fixed:** `DenseStorage<T, const N: usize>` with `[T; N]` (when used via matrix types).
- **Dynamic:** `DenseStorageDynamic<T>` with `Vec<T>`; matrix/vector dimensions can be set at runtime (`with_dimensions`, `resize`).

## Main types

| Type | Role |
|------|------|
| `DenseStorage`, `DenseStorageDynamic`, `DenseStorageTrait` | Dense buffer (fixed size array or `Vec`); trait unifies both. |
| `MatrixBase<T>` | Holds storage and dimensions (rows, cols); `with_dimensions_fill` for Zeros/Ones/None. |
| `Matrix<T>` | Column- or row-major matrix; indexing, `transpose`, `set_identity`, `block()` → `SubMatrix`. |
| `Vector<T>` | Column vector (rows×1); `dot`, `norm`, `resize`. |
| `SubMatrix` | Non-owning view into a matrix block `(i, j, rows, cols)`; get/set, `transpose()` → new Matrix, assign, `AddAssign`, `Into<Matrix>`. |
| `SparseMatrixCRS`, `SparseMatrixBase` | CRS sparse format; triplets, `(i,j)` access, sparse × vector, `set_identity`. |
| `Svd` | Holds U, V, sigma (vector); `decompose()`, `get_u()`, `get_v()`, `get_sigma()`. Source: `decomposition`. |
| `SvdEcon` | Economical SVD: U (m×k), V (n×k), sigma (k) with k = min(m, n); `svd_econ(a)`. |
| `Cholesky` | Cholesky factor L (A = L L^T); `Cholesky::new(a)`, `solve(b)`, `chol(a)`. |
| `Lu` | LU with pivoting; `Lu::new(a)`, `lu.solve(b)`. |
| `solve` | Solve Ax = b for general square A; `solve(a, b)`. |
| Simplex | Linear programming (standard form): `simplex_solve(c, A, b)` → `SimplexResult` (solution vector, objective, status); `SimplexError`, `SimplexStatus`. |
| `Schur` | Real Schur A = Q T Q^T; `schur(a)` (API; full impl planned). |
| `Qz` | Generalised Schur (A, B) → (AA, BB, Q, Z); `qz(a, b)` (API; full impl planned). |
| `Vector3f`, `Matrix3f`, `Matrix4f` | Type aliases for 3D; `matrix4f_inverse`, `matrix3f_inverse`, `matrix4_mul_vector3`, `make_rotation`, `vector3_cross`, `rotation_matrix_to_euler_xyz`, `wrap_angle_to_pi`, `euler_angles_close_to`. |
| `math3d_raw` | Raw array-based 3D math for physics: `[f32; 9]` row-major matrices, `[f32; 3]` vectors, `[f32; 4]` quaternions. `mat3_det`, `mat3_inverse`, `mat3_mul`, `quat_rotate_vec`, `vec3_cross`, `generalized_inverse_mass`, etc. |
| `Cube` | N-dimensional tensor; see `mathlib::cube`. |
| `SparseCube`, `Quadruplet`, `SparseCubeBase` | Sparse 3rd-order tensor in COO format; `from_quadruplets`, `to_dense`, `nnz`. |
| Clustering | `dbscan`, `kmeans`; `DbscanResult`, `KmeansResult`, `NOISE` (noise label). |
| Distance | `euclidean`, `squared_euclidean`, `cosine_similarity`, `cosine_distance`, `manhattan`, `minkowski`, `chebyshev`; row variants `*_rows` (e.g. `euclidean_rows`). |
| Graph (pathfinding) | `Graph`, `Weight`, `dijkstra`, `astar`, `dstar_lite`, `DStarLite`; `DijkstraResult`, `AStarResult`, `DStarLiteResult`. Sequential by default; with `parallel` feature, Dijkstra and A* use par-iter with chili backend for neighbor iteration. Dense adjacency matrix: `Graph::to_adjacency_matrix()` and `Graph::from_adjacency_matrix()` for bidirectional conversion with `Matrix<f64>`. |
| Graph (coloring) | `greedy_vertex_coloring`, `dsatur_coloring`, `is_bipartite`; treats graph as undirected. |
| Graph (tree) | `tree::bfs`, `tree::dfs_preorder`, `tree::dfs_postorder`, `tree::dfs_preorder_forest`, `tree::dfs_postorder_forest`, `BfsResult`; `Tree<T>`, `Node<T>`; BFS/DFS with undirected semantics. `Tree::from_bfs_spanning_tree` builds a tree from a graph. |
| Graph (matrix) | `graph::matrix`: adjacency/Laplacian as sparse matrices. `adjacency_triplets`, `adjacency_crs`, `adjacency_ccs`, `laplacian_triplets`, `laplacian_crs` for [`Graph`]; `tree_adjacency_triplets`, `tree_adjacency_crs` for [`Tree`]. Built CRS/CCS work with existing wgpu SpMV and SIMD when features are enabled. Visualization (data only): use `edges_vec()`, `num_nodes()`, and adjacency triplets for edge list and matrix export. |
| Camera/projection (cg) | `look_at_lh`, `look_at_rh`, `new_perspective`, `new_orthographic`, `model_view_projection`, `transform_point`, `Perspective3`, etc. |
| `Quat4f` | Quaternion (4D); `from_euler_angles`, `to_euler_angles`, `from_rotation_matrix3`; see `mathlib::quaternion`. |
| `DualQuat4f` | Dual quaternion for rigid transforms (rotation + translation). Eight f32 components (real + dual quaternion). Build with `from_rotation_and_translation`, `from_matrix4`; compose with `*`; `transform_point`, `to_matrix4`, `batch_transform_points` (parallel when feature enabled). WASM: `WasmDualQuat`. See `mathlib::dual_quaternion`. |
| easing | `linear`, `lerp`, `ease_in_sine`, `ease_out_cubic`, `ease_in_out_cubic`, etc., `hermite`, `bspline`; `Quat4f::slerp` for spherical interpolation. Parameter `t` in [0, 1]. |
| curve (3D) | `math::curve::linear_curve`, `bezier_curve`, `hermite_curve`, `bspline_curve`; parameter `t` in [0, 1], return `[f32; 3]`. Used by render for line/curve primitives. |
| RBF interpolation | `rbf_kernel`, `rbf_kernel_eased`, `rbf_kernel_normalized`, `rbf_kernel_batch`; `RbfVariant`, `RbfEasing`. Gaussian kernel from squared distance, optional easing for falloff shape; used by `svm_rbf`. With `simd` feature, batch uses SIMD backend. |
| trig | `sin`, `cos`, `sin_scalar`, `cos_scalar`, `tan`, `atan2`, `degrees`, `radians`, hyperbolic variants. |
| noise | `wave_2d`, `wave_2d_params` (sinusoidal), `perlin_2d` (2D Perlin), `fbm_2d` (fractional Brownian motion over a 2D noise). |
| Monte Carlo | `estimate_pi`, `integrate_1d`; π by unit circle in [-1,1]², 1D definite integral by uniform sampling. Deterministic RNG (no extra dependency). |
| colormap | `Rgb`, `Rgba`, `Hsv`; conversions `rgba_to_rgb`, `rgb_to_rgba`, `rgb_to_hsv`, `hsv_to_rgb`, `rgb_to_hex`, `hex_to_rgb`, `rgba_to_hex`, `hex_to_rgba`; palettes `height_to_rgb`, `height_to_rgba` (elevation); presets `scalar_to_rgb_viridis`, `scalar_to_rgb_magma`, `scalar_to_rgb_inferno`, `scalar_to_rgb_plasma`, `scalar_to_rgb_preset` (t in [0,1]). |
| `Pca`, `pca` | Principal component analysis (source: `decomposition`); see `mathlib::pca`. |
| `pso`, `PsoResult`, `PsoOptions` | Particle swarm optimization (source: `aargmin`); see `mathlib::pso`. |
| Transforms | FFT, DCT, Haar wavelets, convolution, spectral windows (`hann`, `hamming`, `blackman`). Pure Rust, wasm-compatible. |
| Lane/SIMD | `LaneCount`, `SimdLane`, `as_f64x4_chunks`, `as_f32x4_chunks` (optional; requires `simd` feature). f32/f64 SIMD for dot, add, sub, scalar_mul, squared_diff_sum, matvec_col_major. |

Additional sparse formats (CCS, CDS, BCRS, JDS, SKS) are available in `sparse_formats`. CCS SpMV uses GPU when available (via `try_spmv_ccs_f32`).

### Direct vs iterative solvers

[`Cholesky`], [`Lu`], and [`solve`] operate on **dense** matrices only. Converting a sparse A to dense and factoring it can produce dense L and U (fill-in), which is not suitable for large sparse systems. For **large sparse symmetric positive definite** systems, use the iterative conjugate gradient method ([`solve_cg_sparse`](crate::argmin::solve_cg_sparse) for [`SparseMatrixCRS`], or [`solve_cg`](crate::argmin::solve_cg) for dense A), which uses only matrix-vector products and avoids fill-in.

### Prelude

Use `mathlib::prelude::*` for common types and functions: `Matrix`, `Vector`, `Cube`, `Storage`, `solve`, `Cholesky`, `Lu`, `Svd`, `SvdEcon`, `Pca`, `DualQuat4f`, and core 3D types (e.g. `Matrix3f`, `Matrix4f`, `Vector3f`, `make_rotation`, `matrix4f_inverse`, `transform_point`). See rustdoc for the full list.

## Operators

Standard Rust traits are implemented in `mathlib::operators`:

- **Add / Sub:** Matrix–matrix, vector–vector, scalar–matrix, scalar–vector.
- **Mul:** Matrix–matrix, matrix–vector, vector–vector (dot), scalar–matrix, scalar–vector.

Use `use mathlib::*` or import the types and rely on `+`, `-`, `*` with references as needed.

## Usage examples

### Matrix and vector

```rust
use mathlib::{Matrix, Vector, Storage};

let mut a = Matrix::<f64>::with_storage(2, 2, Storage::Column);
a.set(0, 0, 1.0);
a.set(1, 1, 1.0);

let v = Vector::from_slice(&[1.0, 2.0]);
let n = v.norm();
let w = &a * &v;
```

### 3D

```rust
use mathlib::{Matrix4f, Vector3f, make_rotation, matrix4f_inverse, matrix4_mul_vector3};

let r = make_rotation(0.1, 0.2, 0.3);
let inv = matrix4f_inverse(&m4);
let v = Vector3f::from_slice(&[1.0, 0.0, 0.0]);
let out = matrix4_mul_vector3(&m4, &v);
```

### SVD (see `mathlib::Svd`)

```rust
use mathlib::{Matrix, Svd};

let m = Matrix::<f64>::from_vec(2, 2, vec![1.0, 0.0, 0.0, 2.0]);
let mut svd = Svd::new(m);
svd.decompose();
let u = svd.get_u();
let sigma = svd.get_sigma();
```

### Clustering (see `mathlib::clustering`)

```rust
use mathlib::{kmeans, dbscan, Matrix};

// K-means: rows = points, cols = dimensions
let data = Matrix::<f64>::from_vec(100, 3, vec![/* ... */]);
let result = kmeans(&data, 5, None); // max_iters None = 300
// result.centroids(), result.labels()

// DBSCAN: rows = points; eps, min_pts
let result = dbscan(&data, 0.5, 5);
// result.labels() (NOISE for outliers)
```

### Distance (see `mathlib::distance`)

```rust
use mathlib::{euclidean, cosine_similarity, euclidean_rows, Vector, Matrix};

let a = Vector::from_slice(&[1.0, 0.0, 0.0]);
let b = Vector::from_slice(&[0.0, 1.0, 0.0]);
let d = euclidean(&a, &b);
let sim = cosine_similarity(&a, &b);

// Row-wise: distance between row i and row j of a matrix
let m = Matrix::<f64>::from_vec(10, 3, vec![/* ... */]);
let d_01 = euclidean_rows(&m, 0, 1);
```

### Graph pathfinding (see `mathlib::graph`)

```rust
use mathlib::{Graph, dijkstra, astar, dstar_lite, DStarLite};

// Build a directed weighted graph (adjacency list)
let mut g = Graph::new(4);
g.add_edge(0, 1, 1.0);
g.add_edge(1, 2, 2.0);
g.add_edge(2, 3, 1.0);

// Dijkstra: single-source shortest path
let res = dijkstra(&g, 0);
// res.dist[i], res.prev[i]

// A*: shortest path from start to goal with heuristic
let h = |u: usize, goal: usize| 0.0_f64; // or an admissible heuristic
let res = astar(&g, 0, 3, h);
// res.path, res.dist

// D* Lite style: replan after edge updates
let mut g2 = Graph::new(3);
g2.add_edge(0, 1, 1.0);
g2.add_edge(1, 2, 1.0);
let mut dl = DStarLite::new(&mut g2, 0, 2);
let path1 = dl.replan();
dl.update_edge(0, 1, 10.0);
let path2 = dl.replan();
```

With the `parallel` feature, `dijkstra` and `astar` use Rayon for neighbor iteration.

### PSO (see `mathlib::pso`)

```rust
use mathlib::{pso, PsoOptions, PsoResult};

// Minimize sphere: sum of squares
let cost = |x: &[f64]| x.iter().map(|v| v * v).sum::<f64>();
let low = vec![-5.0, -5.0, -5.0];
let high = vec![5.0, 5.0, 5.0];
let result = pso((low, high), 40, cost, 100, Some(PsoOptions::default()));
// result.best_position, result.best_cost, result.iterations
```

With the `parallel` feature, cost evaluation over particles is parallelized; slice math (add/sub/scalar_mul) uses SIMD when the `simd` feature is enabled.

### Easing (see `mathlib::easing`)

```rust
use mathlib::easing::{linear, lerp, ease_in_out_cubic, hermite, bspline};
use mathlib::cg::vector3;
use mathlib::Quat4f;

// Scalar easing: t in [0, 1]
let eased = ease_in_out_cubic(0.5);  // smooth start and end

// Linear interpolation between values
let mid = lerp(0.0, 100.0, 0.5);  // 50.0

// Hermite: p0, p1, tangent at 0, tangent at 1, t
let v = hermite(0.0, 1.0, 0.0, 0.0, 0.5);  // linear segment

// B-spline: 4 control points, t in [0, 1]
let pts = [0.0f64, 1.0, 2.0, 3.0];
let pt = bspline(&pts, 0.5);

// Quaternion slerp (spherical interpolation)
let axis = vector3(0.0, 1.0, 0.0);
let q0 = Quat4f::from_axis_angle(&axis, 0.0);
let q1 = Quat4f::from_axis_angle(&axis, std::f32::consts::FRAC_PI_2);
let q_mid = q0.slerp(&q1, 0.5);
```

Run `cargo run -p mathlib --example easing` for a sample output.

### Genetic (CMA-ES) — see `mathlib::genetic`

```rust
use mathlib::{CmaEsBuilder, CmaEsResult};

// Minimize sphere: Σ xᵢ²
fn sphere(x: &[f64]) -> f64 { x.iter().map(|v| v * v).sum() }
let dim = 6;
let mean = vec![1.0; dim];
let mut opt = CmaEsBuilder::new(dim, mean, 0.3).max_generations(150).seed(42).build();
let result: CmaEsResult = opt.optimize(sphere);
// result.solution, result.fitness, result.generations
```

Requires the `genetic` feature. Run with `cargo run -p mathlib -F genetic --example cmaes`.

### Transforms (see `mathlib::transforms`)

```rust
use mathlib::{fft_forward, fft_forward_real, fft_inverse, dct2_forward, dct2_inverse};
use mathlib::{dwt_haar_forward, dwt_haar_inverse, conv_1d, conv_1d_same, conv_2d};
use mathlib::{hann, hamming, blackman, tukey, apply_window, Complex64, Matrix, Storage};

// FFT: power-of-2 length required
let signal: Vec<f64> = (0..256).map(|i| (i as f64 * 0.1).sin()).collect();
let spectrum = fft_forward_real(&signal)?;
let reconstructed = fft_inverse(&spectrum)?;

// DCT-II
let coeffs = dct2_forward(&signal)?;
let restored = dct2_inverse(&coeffs)?;

// Haar wavelets (even length)
let coeffs = dwt_haar_forward(&signal);
let restored = dwt_haar_inverse(&coeffs);

// Convolution
let out = conv_1d(&signal, &[1.0, 1.0, 1.0]);
let out_same = conv_1d_same(&signal, &[1.0, 0.0, 1.0]);

// Spectral windows (reduce leakage before FFT)
let w = hann(256);
let w_tukey = tukey(256, 0.5); // alpha 0.5 = tapered cosine
let mut windowed = vec![0.0; 256];
apply_window(&signal, &w, &mut windowed);
let spectrum = fft_forward_real(&windowed)?;
```

All transforms are pure Rust with no external dependencies. Compatible with wasm32. Convolution uses parallel iteration when the `parallel` feature is enabled (native only).

## Logging

The library uses the [`tracing`] facade: it emits events (`debug!`, `info!`, `warn!`) but **does not** initialize a global subscriber. Your application should set up a subscriber, for example:

```rust
tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();
```

Then set `RUST_LOG` (e.g. `RUST_LOG=info,mathlib=debug`) to control verbosity. For file rotation or non-blocking output, use `tracing-appender` on the application side. For WASM, use a wasm-compatible tracing layer if you want logs in the browser.

## Features and WASM

- **Native:** Use `parallel`, `simd`, or `full` (parallel + simd) for faster CPU backends. The `parallel` feature uses par-iter with chili backend (heartbeat scheduling) and is not available on target `wasm32`.
- **Wasm:** Build with `--features wasm` or `--features "wasm simd"` only. Combining `wasm` with `parallel` will fail at build time (build script error). The `wasm` module is split into submodules (`matrix`, `vector`, `decomposition`, `camera`, `clustering`, `distance`, `svm`, `simplex`, `graph`, `argmin`, `noise`, `transforms`); see `mathlib::wasm`. Exports include: matrices/vectors, SVD, PCA, Cholesky, LU, camera, K-means, DBSCAN, distance, SVM, simplex, graph (Dijkstra, A*, D* Lite, coloring, BFS/DFS), PSO (`psoMinimize` with JS cost callback), line search (`lineSearchBacktracking`), noise (`wave2d`, `wave2dParams`, `perlin2d`, `fbm2dPerlin`), and transforms (FFT, DCT, wavelets, convolution, windows). Optional **GPU** (`--features "wasm gpu"`): WebGPU init for f32 matmul, matvec, dot, norm, add, sub, scale, mul, axpy, abs, sqrt, div, sparse SpMV; exposes `initGpuAsync` and `gpuAvailable` in the demo. See `examples/gpu/gpu_large_matrix.rs`. From repo root: `just build-wasm`, `just check-wasm`, `just test-wasm`, `just build-wasm-simd`. See [docs/wasm.md](wasm.md) and [AGENTS.md](../AGENTS.md) for the full feature table.

### WASM and browser demo

A browser demo in [mathlib/demo/wasm-demo/](../mathlib/demo/wasm-demo/) showcases vectors, matrix multiply (with optional GPU backend when built with `wasm gpu`), K-means, PCA, SVM, distance metrics, Cholesky, SVD, simplex LP, camera matrices, DBSCAN, LU solve, graph pathfinding (Dijkstra, A*, D* Lite), line search, PSO, and noise. The demo is built in CI and deployed to **GitHub Pages** on push to `main`/`master`; enable **Settings → Pages → GitHub Actions** to get a live URL. **Build** (from repo root): `just wasm-build` — builds with wasm-pack and copies `pkg/` into `wasm-demo/pkg/`. For GPU support use `just wasm-build-gpu`. **Serve**: from repo root run `just wasm-serve` (or from `mathlib/` run `npx serve .`), then open **/wasm-demo/** (use the URL shown by the server) in a browser. For full instructions and Windows copy notes, see [mathlib/demo/wasm-demo/README.md](../mathlib/demo/wasm-demo/README.md) and [wasm.md](wasm.md).

## Performance notes

- **Column-major** is the default; use it for consistency with column-wise operations and benchmarks.
- **Benchmarks** live in `mathlib/benches/` (e.g. matrix–vector product, matrix addition). Run from the crate root: `cd mathlib && cargo bench`. For GPU vs CPU comparison and **threshold guidance** (matmul, matvec, dot, norm, add, scale, mul, axpy, abs, sqrt, div, spmv): `cargo bench --features gpu --bench gpu`; see [mathlib/benches/gpu_bench.rs](../mathlib/benches/gpu_bench.rs) for sizes and how to tune `ExecutorThresholds`. In the wasm demo, small sizes favor CPU; for crossover guidance see [wasm.md](wasm.md) and the gpu bench.
- See Rustdoc (`cargo doc --open` in `mathlib/`) for full API details.
