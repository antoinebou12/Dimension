# AGENTS.md — Project and API summary for LLMs

This repository (**Dimension**) hosts a single Rust crate, **mathlib**, a linear algebra library. All code lives under `mathlib/`. Use this file for project context, module layout, conventions, and where to add or change code.

## Project role

- **mathlib**: Dense and sparse matrices, vectors, N-dimensional cubes, SVD and other decompositions, linear solvers (Cholesky, LU, general solve), 3D math (Matrix3f/4f, Vector3f, inverses, rotation), camera/projection (look-at, perspective, orthographic, MVP), clustering (k-means, DBSCAN), SVM (binary classification), distance metrics, graph pathfinding (Dijkstra, A*, D* Lite), PCA, particle swarm optimization (PSO), quaternions, trig, and optional SIMD/lane utilities.
- **Namespace**: Public API is under the `mathlib` crate (e.g. `mathlib::Matrix`, `mathlib::Vector`). Run `cd mathlib && cargo doc --open` for full API docs.

## Module map (from `mathlib/src/lib.rs`)

| Module | Description | Key types / functions |
|--------|-------------|------------------------|
| `structure` | Dense/sparse storage, matrix base, submatrix, sparse formats | `DenseStorage`, `DenseStorageDynamic`, `MatrixBase`, `SubMatrix`, `SparseMatrixCRS`, `SparseMatrixCCS`, etc., `Storage`, `Triplet` |
| `matrix` | Column/row-major matrix | `Matrix<T>` |
| `vector` | Column vector | `Vector<T>`, `Float`, `RealNumber` |
| `operators` | Add, Sub, Mul for matrices/vectors | Trait impls for `+`, `-`, `*` |
| `cube` | N-dimensional tensor | `Cube` |
| `cg` | Camera and projection | `look_at_lh`, `look_at_rh`, `new_perspective`, `new_orthographic`, `model_view_projection`, `transform_point`, `Perspective3`, etc. |
| `chol` | Cholesky decomposition | `Cholesky`, `chol`, `CholError` |
| `lu` | LU with pivoting | `Lu`, `LuError` |
| `solve` | General linear solve Ax = b | `solve`, `SolveError` |
| `decomposition` | SVD and PCA (matrix decompositions) | `Svd`, `SvdEcon`, `svd_econ`, `Pca`, `pca` |
| `schur` | Real Schur decomposition | `Schur`, `schur`, `SchurError` |
| `qz` | Generalised Schur (QZ) | `Qz`, `qz`, `QzError` |
| `math3d` | 3D types and helpers | `Matrix3f`, `Matrix4f`, `Vector3f`, `Vector4f`, `Point3`, `make_rotation`, `matrix4f_inverse`, `matrix4_mul_vector3`, etc. |
| `quaternion` | Quaternion | `Quat4f` |
| `easing` | Easing and interpolation | `linear`, `lerp`, `ease_in_sine`, `ease_out_cubic`, etc., `hermite`, `bspline`; `Quat4f::slerp` |
| `trig` | Trig and degrees/radians | `sin`, `cos`, `sin_scalar`, `cos_scalar`, `tan`, `atan2`, `degrees`, `radians`, etc. |
| `noise` | Noise for heightmaps / procedural | `wave_2d`, `wave_2d_params`, `perlin_2d`, `fbm_2d` |
| `colormap` | Color types and conversions | `Rgb`, `Rgba`, `Hsv`; `rgba_to_rgb`, `rgb_to_rgba`, `rgb_to_hsv`, `hsv_to_rgb`, `rgb_to_hex`, `hex_to_rgb`, `rgba_to_hex`, `hex_to_rgba`; `height_to_rgb`, `height_to_rgba` |
| `clustering` | Clustering algorithms | `dbscan`, `kmeans`, `DbscanResult`, `KmeansResult`, `NOISE` |
| `svm` | Support Vector Machine (binary classification); linear and RBF kernel | `svm`, `svm_rbf`, `SvmResult`, `SvmRbfResult`, `SvmOptions`, `SvmError` |
| `distance` | Distance metrics | `euclidean`, `squared_euclidean`, `cosine_similarity`, `cosine_distance`, `manhattan`, `minkowski`, `chebyshev`, and `*_rows` variants |
| `graph` | Pathfinding and structure | `Graph` (directed), `NodeId`, `Edge`, `Weight`, `add_edge`, `add_edge_undirected`, adjacency (`in_neighbors`, `out_degree`, `in_degree`, `is_adjacent`), `edges()`, `reverse_graph`, `UnionFind`, `connected_components_undirected`, `articulation_points`, `bridges`, `DisjointSet`, `connected_components`, `dijkstra`, `astar`, `dstar_lite`, `is_bipartite`, `greedy_vertex_coloring` (sequential; optional parallel when `parallel` feature) |
| `aargmin` | Particle swarm optimization (PSO) | `pso`, `PsoResult`, `PsoOptions` |
| `genetic` | Evolution strategies (optional) | `CmaEs`, `CmaEsBuilder`, `CmaEsResult` (CMA-ES; requires `genetic` feature) |
| `lane` | SIMD/lane utilities (optional) | `LaneCount`, `SimdLane`, `as_f64x4_chunks`, `as_f64x4_chunks_mut` |
| `cpu` | CPU backend selection | Internal |
| `hash` | Hashing (internal) | Not re-exported at crate root |
| `wasm` | WebAssembly (feature-gated) | `mathlib` with `wasm` feature |

## Prelude

`mathlib::prelude::*` re-exports: `Matrix`, `Vector`, `Cube`, `Storage`, `solve`, `Cholesky`, `Lu`, `Svd`, `SvdEcon`, `Pca`, and core 3D types (e.g. `Matrix3f`, `Matrix4f`, `Vector3f`, `make_rotation`, `matrix4f_inverse`, `transform_point`), plus selected cg and math3d functions. See rustdoc for the full list.

## Conventions

- **Style**: Run `cargo fmt` and `cargo clippy` inside `mathlib/`. See [CONTRIBUTING.md](CONTRIBUTING.md).
- **Logging**: Use `tracing` for logging; do not initialize a subscriber in the library.
- **Tests**: Add or update tests when changing behavior; integration tests live in `mathlib/tests/`.
- **Docs**: Add or update doc comments for public API; update `docs/DOCS.md` when changing main types or usage.
- **Panic vs Result**: Indexing (e.g. `get`/`set` on matrices) may panic in debug if out of bounds. Solvers and decompositions return `Result` (`CholError`, `LuError`, `SolveError`, `SchurError`, `QzError`).

## Features

| Feature | Purpose |
|---------|---------|
| `default` | No optional deps. |
| `serde` | Serde support for selected types (e.g. `Triplet`). |
| `parallel` | Rayon-based parallel backend (not available on target `wasm32`). |
| `simd` | SIMD via `wide` crate (supported on wasm32). |
| `full` | Shorthand for `parallel` + `simd` (native only). |
| `wasm` | Build for WebAssembly; exposes `mathlib::wasm` (matrices, vectors, solve, SVD, 3D/camera helpers). |
| `genetic` | CMA-ES and evolution strategies; adds `rand`, `rand_distr`. |

**Wasm builds:** Use `--features wasm` or `--features "wasm simd"` only. The `parallel` feature is not supported on `wasm32` (build script will error). From repo root: `just build-wasm`, `just check-wasm`, `just test-wasm`, `just build-wasm-simd`, etc. (see [justfile](justfile)).

Build with features: `cargo build --features parallel`, `cargo build --features full`, etc. From repo root, `just build-parallel`, `just build-simd`, `just build-full` (see [justfile](justfile)).

## Error types

- `mathlib::CholError` — Cholesky decomposition failure.
- `mathlib::LuError` — LU decomposition failure.
- `mathlib::SolveError` — Linear solve failure.
- `mathlib::SchurError` — Schur decomposition failure.
- `mathlib::QzError` — QZ decomposition failure.
- `mathlib::SvmError` — SVM training failure (label length, empty data, single class).

## Where to add code

- **New solvers or decompositions**: New module under `mathlib/src/` or under `mathlib/src/decomposition/` for SVD/PCA-style decompositions; then `pub mod` and `pub use` in `lib.rs`. PSO and related optimization live under `mathlib/src/aargmin/`.
- **New storage or matrix/vector types**: Under `mathlib/src/structure/` or extend `matrix.rs`/`vector.rs` as appropriate.
- **New algorithms (e.g. clustering, SVM, distance, graph)**: Under existing `mathlib/src/clustering/`, `mathlib/src/svm/`, `mathlib/src/distance/`, `mathlib/src/graph/`, or new module if unrelated.
- **New 3D or camera helpers**: `mathlib/src/math3d.rs` or `mathlib/src/cg.rs`.
- **Tests**: `mathlib/tests/<name>.rs` for integration tests (or `tests/aargmin/`, `tests/decomposition/` for PSO and SVD/PCA); unit tests next to the code.
- **Examples**: `mathlib/examples/<name>.rs`; run with `cargo run --example <name>`.
- **Benchmarks**: `mathlib/benches/` (PSO under `benches/aargmin/`, PCA under `benches/decomposition/`); run with `cargo bench` in `mathlib/`.

## Tools for agents (Cursor / MCP)

- **Sequential Thinking**: Use the Sequential Thinking MCP for step-by-step reasoning on complex changes (refactors, design choices, multi-file edits).
- **Context7**: Use the Context7 MCP to fetch up-to-date docs and examples for Rust crates (e.g. `criterion`, `rayon`, `serde`) when implementing or debugging.

## More documentation

- [docs/DOCS.md](docs/DOCS.md) — Architecture and usage (human-oriented).
- [docs/dev-tools.md](docs/dev-tools.md) — Dev setup, coverage, and optional tools.
- [docs/claude.md](docs/claude.md) — Short project context for Claude and other AI assistants.
- [SECURITY.md](SECURITY.md) — How to report vulnerabilities (do not open public issues).
- **Rustdoc**: `cd mathlib && cargo doc --open` for full API.
