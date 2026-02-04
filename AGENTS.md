# AGENTS.md — Project and API summary for LLMs

This repository (**Dimension**) hosts a single Rust crate, **mathlib**, a linear algebra library. All code lives under `mathlib/`. Use this file for project context, module layout, conventions, and where to add or change code.

## Project role

- **mathlib**: Dense and sparse matrices, vectors, N-dimensional cubes, SVD and other decompositions, linear solvers (Cholesky, LU, general solve), 3D math (Matrix3f/4f, Vector3f, inverses, rotation), camera/projection (look-at, perspective, orthographic, MVP), clustering (k-means, DBSCAN), SVM (binary classification), distance metrics, graph pathfinding (Dijkstra, A*, D* Lite), PCA, particle swarm optimization (PSO), quaternions, trig, and optional SIMD/lane utilities.
- **Namespace**: Public API is under the `mathlib` crate (e.g. `mathlib::Matrix`, `mathlib::Vector`). Run `cd mathlib && cargo doc --open` for full API docs.

## Domain-grouped module map

Tests, benches, and examples follow the same domains. Full path table: [docs/domains.md](docs/domains.md).

| Domain | Subcategories | Modules | Key types / functions |
|--------|---------------|---------|------------------------|
| **linear** | storage, solvers, decompositions, simplex | structure, matrix, vector, operators, linear (chol, lu, solve, schur, qz), decomposition, simplex | Matrix, Vector, Storage, Cholesky, Lu, solve, Svd, Pca, simplex_solve |
| **ml** | clustering, svm, distance | clustering, svm, distance | dbscan, kmeans, svm, svm_rbf, euclidean, cosine_similarity |
| **optimisation** | argmin, genetic | argmin, genetic | pso, gradient_descent, nonlinear_cg, CmaEs |
| **graph** | pathfinding, coloring, structure | graph | Graph, dijkstra, astar, dstar_lite, greedy_vertex_coloring |
| **tree** | traversal, structure | graph::tree | Tree, Node, bfs, dfs_preorder, dfs_postorder, dfs_preorder_forest, dfs_postorder_forest |
| **cg** | camera, math3d, quaternion, trig, easing | cg, math3d, quaternion, trig, easing | look_at_rh, new_perspective, Matrix4f, Quat4f, ease_in_out_cubic |
| **noise** | procedural | noise | perlin_2d, fbm_2d, wave_2d |
| **transforms** | spectral, wavelets, convolution, windows | transforms | fft_forward, dct2_forward, dwt_haar_forward, conv_1d, hann |
| **colormap** | types, convert, palette | colormap | Rgb, Hsv, rgb_to_hex, height_to_rgb |
| **stats** | descriptive | stats | covariance |
| **tensor** | N-dimensional | cube | Cube |
| **runtime** | cpu, gpu | cpu, gpu | (backends) |
| **wasm** | bindings | wasm | WasmMatrix, WasmSvd, etc. |

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
| `parallel` | par-iter with chili backend for parallel execution (not available on target `wasm32`). Uses heartbeat scheduling instead of work-stealing for better performance with many short-lived tasks. |
| `simd` | SIMD via `wide` crate (supported on wasm32). Used by `cpu::simd`, distance, operators, PSO, line search; graph algorithms use SIMD indirectly via dependent ops. |
| `full` | Shorthand for `parallel` + `simd` (native only). |
| `wasm` | Build for WebAssembly; exposes `mathlib::wasm` (matrices, vectors, solve, SVD, LU, 3D/camera, clustering, graph pathfinding, PSO, noise). |
| `gpu` | WebGPU/wgpu compute for f32 matmul, matvec, dot, norm, add, sub. |
| `genetic` | CMA-ES and evolution strategies; adds `rand`, `rand_distr`. |

**Wasm builds:** Use `--features wasm` or `--features "wasm simd"` only. The `parallel` feature is not supported on `wasm32` (par-iter with chili does not support wasm32; build script will error). From repo root: `just build-wasm`, `just check-wasm`, `just test-wasm`, `just build-wasm-simd`, etc. (see [justfile](justfile)).

Build with features: `cargo build --features parallel`, `cargo build --features full`, etc. From repo root, `just build-parallel`, `just build-simd`, `just build-full` (see [justfile](justfile)).

## Error types

- `mathlib::CholError` — Cholesky decomposition failure.
- `mathlib::LuError` — LU decomposition failure.
- `mathlib::SolveError` — Linear solve failure.
- `mathlib::SchurError` — Schur decomposition failure.
- `mathlib::QzError` — QZ decomposition failure.
- `mathlib::SimplexError` — Simplex LP failure (inconsistent dimensions, infeasible, unbounded, max iterations).
- `mathlib::SvmError` — SVM training failure (label length, empty data, single class).
- `mathlib::TransformsError` — FFT/DCT errors (`LengthNotPowerOfTwo`, `EmptyInput`).

## Where to add code (by domain)

- **linear**: `src/linear/`, `src/decomposition/`, `src/structure/`, `src/simplex/`; tests `tests/linear/`; benches `benches/linear/`; examples `examples/linear/`.
- **ml**: `src/clustering/`, `src/svm/`, `src/distance/`; tests `tests/ml/`; benches `benches/ml/`; examples `examples/ml/`.
- **optimisation**: `src/argmin/`, `src/genetic/`; tests `tests/optimisation/`; benches `benches/optimisation/`; examples `examples/optimisation/`.
- **graph**: `src/graph/`; tests `tests/graph/`; benches `benches/graph/`; examples `examples/graph/`.
- **tree**: `src/graph/tree/`; tests `tests/tree/`; benches `benches/tree/`; examples `examples/tree/`.
- **cg**: `src/math/cg.rs`, `src/math/math3d.rs`, `src/math/quaternion.rs`, `src/math/trig.rs`, `src/math/easing/`; tests `tests/cg/`; benches `benches/cg/`; examples `examples/cg/`.
- **noise**: `src/noise/`; tests `tests/noise/`; benches `benches/noise/`; examples `examples/viz/` (noise and colormap viz).
- **transforms**: `src/transforms/`; tests `tests/transforms/`; benches `benches/transforms/`; examples `examples/transforms/`.
- **colormap**: `src/colormap/`; tests `tests/colormap/`.
- **stats**: `src/stats.rs`; tests `tests/stats/`.
- **tensor**: `src/cube.rs`; tests `tests/tensor/`.
- **Tests**: One test binary per domain (e.g. `cargo test --test linear`). Integration/example logic: `tests/integration/`. Serde: `tests/serialization/` (feature-gated).
- **Benchmarks**: Domain folders under `benches/` (linear, ml, optimisation, cg, noise, graph, tree, transforms); main harness in `benches/benchmarks.rs`.

## Tools for agents (Cursor / MCP)

- **Sequential Thinking**: Use the Sequential Thinking MCP for step-by-step reasoning on complex changes (refactors, design choices, multi-file edits).
- **Context7**: Use the Context7 MCP to fetch up-to-date docs and examples for Rust crates (e.g. `criterion`, `rayon`, `serde`) when implementing or debugging.

## More documentation

- [docs/DOCS.md](docs/DOCS.md) — Architecture and usage (human-oriented).
- [docs/dev-tools.md](docs/dev-tools.md) — Dev setup, coverage, and optional tools.
- [docs/claude.md](docs/claude.md) — Short project context for Claude and other AI assistants.
- [SECURITY.md](SECURITY.md) — How to report vulnerabilities (do not open public issues).
- **Rustdoc**: `cd mathlib && cargo doc --open` for full API.
