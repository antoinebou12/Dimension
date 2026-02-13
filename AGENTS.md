# AGENTS.md — Project and API summary for LLMs

This repository (**Dimension**) hosts **mathlib** (linear algebra) and **render** (2D/3D GPU rendering). Use this file for project context, module layout, conventions, and where to add or change code.

## Project role

- **mathlib**: Dense and sparse matrices, vectors, N-dimensional cubes, SVD and other decompositions, linear solvers (Cholesky, LU, general solve), 3D math (Matrix3f/4f, Vector3f, inverses, rotation), camera/projection (look-at, perspective, orthographic, MVP), clustering (k-means, DBSCAN), SVM (binary classification), distance metrics, graph pathfinding (Dijkstra, A*, D* Lite), PCA, particle swarm optimization (PSO), quaternions, trig, and optional SIMD/lane utilities.
- **render**: WASM-first 2D/3D render engine using wgpu; platform (wasm canvas / native winit), scene (mathlib Tree, World, Transform, Primitive), backend (wgpu GPU-first). Uses mathlib and **collision** for CPU math and culling (Aabb, Frustum, ray_*, BspTree). Optional simd/parallel features via mathlib.
- **collision**: AABB, ray (ray_aabb, ray_triangle, ray_segment), frustum, BSP, BVH, sphere, OBB, capsule. Depends on mathlib; used by render for culling and picking. See [docs/collision.md](docs/collision.md).
- **kinematics**: Forward and inverse kinematics crate; joints (fixed, revolute, prismatic, spherical), armature (Tree), Jacobian and FABRIK IK. Uses mathlib Tree, SVD, rotation helpers. WASM-capable.
- **physics**: PBD/XPBD simulation (particles, constraints, contact, rigid shape-matching). Uses mathlib (graph coloring, SVD, 3D) and **collision** for detection. Optional simd/parallel/serde/wasm. Demo: physics-demo (winit + WASM). See [docs/physics.md](docs/physics.md).
- **geometry**: Mesh processing (half-edge, voxelization, tetrahedralization, CSG, remeshing, smoothing). Uses mathlib and **collision**; optional parse for I/O. Demo: geometry-demo (winit + WASM). See [docs/geometry.md](docs/geometry.md).
- **Namespace**: Public API is under the `mathlib` crate (e.g. `mathlib::Matrix`, `mathlib::Vector`) and `render` crate (e.g. `render::Engine`, `render::World`). Run `cd mathlib && cargo doc --open` or `cd render && cargo doc --open` for full API docs.

### Crate versions

- **mathlib**: version from [mathlib/Cargo.toml](mathlib/Cargo.toml)
- **render**: version from [render/Cargo.toml](render/Cargo.toml)
- Agents should refer to Cargo.toml for authoritative version; crate roots expose it in docs. When publishing render to crates.io, mathlib must be published first; then switch render's dependency from `path = "../mathlib"` to `version = "0.1"` (or matching).

## Domain-grouped module map

Tests, benches, and examples follow the same domains. Full path table: [docs/domains.md](docs/domains.md).

| Domain | Subcategories | Modules | Key types / functions |
|--------|---------------|---------|------------------------|
| **linear** | storage, solvers, decompositions, simplex | structure, matrix, vector, operators, linear (chol, lu, solve, schur, qz), decomposition, simplex | Matrix, Vector, Storage, Cholesky, Lu, solve, Svd, Pca, simplex_solve |
| **ml** | clustering, svm, distance | clustering, svm, distance | dbscan, kmeans, svm, svm_rbf, euclidean, cosine_similarity |
| **optimisation** | argmin, genetic | argmin, genetic | pso, gradient_descent, nonlinear_cg, CmaEs |
| **graph** | pathfinding, coloring, structure, matrix | graph | Graph, dijkstra, astar, dstar_lite, greedy_vertex_coloring; Graph::to_adjacency_matrix, Graph::from_adjacency_matrix (dense); graph::matrix: adjacency_triplets, adjacency_crs, adjacency_ccs, laplacian_triplets, laplacian_crs, tree_adjacency_triplets, tree_adjacency_crs |
| **tree** | traversal, structure | graph::tree | Tree, Node, bfs, dfs_preorder, dfs_postorder, dfs_preorder_forest, dfs_postorder_forest |
| **cg** | camera, math3d, math3d_raw, quaternion, dual quaternion, trig, easing, rbf | cg, math3d, math3d_raw, quaternion, dual_quaternion, trig, easing, math::rbf | look_at_rh, new_perspective, Matrix4f, Quat4f, DualQuat4f, mat3_det, quat_rotate_vec, ease_in_out_cubic, rbf_kernel, RbfVariant |
| **noise** | procedural | noise | perlin_2d, fbm_2d, wave_2d |
| **transforms** | spectral, wavelets, convolution, windows | transforms | fft_forward, dct2_forward, dwt_haar_forward, conv_1d, hann |
| **colormap** | types, convert, palette | colormap | Rgb, Hsv, rgb_to_hex, height_to_rgb |
| **stats** | descriptive | stats | covariance |
| **monte_carlo** | π estimation, 1D integration | monte_carlo | estimate_pi, integrate_1d |
| **tensor** | N-dimensional, sparse | cube, structure::sparse_cube | Cube, SparseCube, Quadruplet |
| **runtime** | cpu, gpu, executor | cpu, gpu, executor | CpuExecutor, GpuExecutor, AutoExecutor, ExecutorThresholds; GPU fallback (log + CPU on op failure). Cube GPU add/sub/scale; CCS SpMV via try_spmv_ccs_f32. |
| **wasm** | bindings | wasm | WasmMatrix, WasmSvd, WasmPca; svdEconAsync; with gpu: WasmPca.transformF32GpuAsync (SVD/PCA fit stay CPU-only). |
| **render** | platform, scene, backend, cull, spatial, gizmo, pick, view_mode, ui | render | Engine, World, GpuRenderer, Transform, Primitive, Camera, FrameStats; cull re-exports collision (Frustum, Aabb, ray_aabb) + primitive_aabb, world_aabb; spatial re-exports collision::BspTree; gizmo (GizmoMode, overlay); pick_entity; ViewMode (solid, wireframe, vertex points); Slider spring (update_ui_springs). Backend uses frustum culling and instancing by (mesh, material). Stats overlay; build_stats_panel, Label. |
| **collision** | aabb, ray, frustum, bsp, bvh, sphere, obb, capsule | collision | Aabb, Frustum, ray_aabb, ray_triangle, ray_segment, ray_sphere, ray_obb, ray_capsule; BspTree, BvhTree; Sphere, Obb, Capsule. Tests `collision/tests/`; benches `collision/benches/`; example `collision/examples/frustum_culling.rs`. See [docs/collision.md](docs/collision.md). |
| **parse** | data, 3D, image, archive | parse | json, bjson, toon, xml, obj, mtl, bvh, gltf, ply, image (png/jpeg), archive (zip/tar). Uses mathlib for Vertex, Mesh, Material. |
| **kinematics** | joints, armature, IK | kinematics | Armature, JointData, JointVariant; joints (fixed, revolute, prismatic, spherical); JacobianIk, FabrikIk. See [kinematics/AGENTS.md](kinematics/AGENTS.md). |
| **physics** | integration, constraints, contact, rigid, spatial, islands | physics | Particle, PhysicsState, Constraint, PbdIntegrator, ContactConstraint, ShapeMatchingConstraint; spatial (broad_phase_pairs, build_bvh); islands (compute_islands, constraint_colors); solver (step_pbd). See [docs/physics.md](docs/physics.md). |
| **geometry** | mesh, half-edge, voxel, tet, csg, remesh, smooth | geometry | TriMesh, HalfEdgeMesh, VoxelGrid, TetMesh; voxelize_mesh, flood_fill, marching_cubes; csg_union, csg_intersection, csg_difference; remesh_isotropic; smooth_uniform, smooth_cotangent, smooth_implicit. See [docs/geometry.md](docs/geometry.md). |

## Prelude

`mathlib::prelude::*` re-exports: `Matrix`, `Vector`, `Cube`, `Storage`, `solve`, `Cholesky`, `Lu`, `Svd`, `SvdEcon`, `Pca`, and core 3D types (e.g. `Matrix3f`, `Matrix4f`, `Vector3f`, `make_rotation`, `matrix4f_inverse`, `transform_point`), plus selected cg and math3d functions. See rustdoc for the full list.

## Conventions

- **Style**: Run `cargo fmt` and `cargo clippy` inside `mathlib/`. See [CONTRIBUTING.md](CONTRIBUTING.md).
- **Logging**: Use `tracing` for logging; do not initialize a subscriber in the library.
- **Tests**: Add or update tests when changing behavior; integration tests live in `mathlib/tests/`.
- **Docs**: Add or update doc comments for public API; update `docs/DOCS.md` when changing main types or usage.
- **Panic vs Result**: Indexing (e.g. `get`/`set` on matrices) may panic in debug if out of bounds. Solvers and decompositions return `Result` (`CholError`, `LuError`, `SolveError`, `SchurError`, `QzError`).

### Documentation

Documentation must follow Rust API guidelines as in the project Rust skill. These conventions apply to **both mathlib and render**:

- **doc-all-public** — Document all public items with `///`.
- **doc-module-inner** — Use `//!` for module- and crate-level docs.
- **doc-examples-section** — Include `# Examples` with runnable code where helpful; use `?` in examples, not `.unwrap()` (doc-question-mark).
- **doc-errors-section** — Include `# Errors` for fallible functions.
- **doc-panics-section** — Include `# Panics` for panicking functions.
- **doc-safety-section** — Include `# Safety` for `unsafe` functions.
- **doc-hidden-setup** — Use `# ` prefix to hide setup code in examples.
- **doc-intra-links** — Use intra-doc links: `[Matrix]`, `[Self::method]`.
- **doc-link-types** — Link related types and functions in docs.
- **doc-cargo-metadata** — Ensure `Cargo.toml` has full metadata (repository, documentation, readme, keywords, categories).

Reference: `.cursor/skills/rust-skills` or `/rust-skills` when writing or reviewing docs.

## Features

| Feature | Purpose |
|---------|---------|
| `default` | No optional deps. |
| `serde` | Serde support for selected types (e.g. `Triplet`). |
| `parallel` | par-iter with chili backend for parallel execution (not available on target `wasm32`). Uses heartbeat scheduling instead of work-stealing for better performance with many short-lived tasks. |
| `simd` | SIMD via `wide` crate (supported on wasm32). f64 and f32 ops: dot, add, sub, scalar_mul, squared_diff_sum, matvec_col_major, rbf_kernel_batch_f64. Used by distance, operators, PSO, line search, RBF batch; graph algorithms use SIMD indirectly via dependent ops. |
| `full` | Shorthand for `parallel` + `simd` (native only). |
| `wasm` | Build for WebAssembly; exposes `mathlib::wasm` (matrices, vectors, solve, SVD, LU, 3D/camera, clustering, graph pathfinding, PSO, noise). |
| `gpu` | WebGPU/wgpu compute for f32 matmul (tiled), matvec, dot, norm, add, sub, scale, axpy, Cube add/sub/scale, CRS SpMV, CCS SpMV (try_spmv_ccs_f32). With wasm: WasmPca.transformF32GpuAsync for GPU-accelerated PCA transform. Executor API (CpuExecutor, GpuExecutor, AutoExecutor) and threshold-based selection; fallback to CPU on failure. GPU benchmarks and threshold guidance: `cargo bench --features gpu --bench gpu`. |
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
- `geometry::GeometryError` — Mesh/voxel/tet errors (DegenerateMesh, EmptyInput, NonManifold, InvalidTopology, VoxelGridTooLarge, TetQualityFailed).

## Where to add code (by domain)

- **linear**: `src/linear/`, `src/decomposition/`, `src/structure/`, `src/simplex/`; tests `tests/linear/`; benches `benches/linear/`; examples `examples/linear/`.
- **ml**: `src/clustering/`, `src/svm/`, `src/distance/`; tests `tests/ml/`; benches `benches/ml/`; examples `examples/ml/`.
- **optimisation**: `src/argmin/`, `src/genetic/`; tests `tests/optimisation/`; benches `benches/optimisation/`; examples `examples/optimisation/`.
- **graph**: `src/graph/`; tests `tests/graph/`; benches `benches/graph/`; examples `examples/graph/`.
- **tree**: `src/graph/tree/`; tests `tests/tree/`; benches `benches/tree/`; examples `examples/tree/`.
- **cg**: `src/math/cg.rs`, `src/math/math3d.rs`, `src/math/math3d_raw.rs`, `src/math/quaternion.rs`, `src/math/trig.rs`, `src/math/easing/`, `src/math/curve.rs`, `src/math/rbf.rs`; tests `tests/cg/`; benches `benches/cg/`; examples `examples/cg/`. Raw array 3D (`[f32; 9]` matrices, `[f32; 3]` vectors): `src/math/math3d_raw.rs`. RBF interpolation (kernel, easing, variants): `src/math/rbf.rs`. 3D curve evaluation (linear, Bézier, Hermite, B-spline): `src/math/curve.rs`.
- **noise**: `src/noise/`; tests `tests/noise/`; benches `benches/noise/`; examples `examples/viz/` (noise and colormap viz).
- **transforms**: `src/transforms/`; tests `tests/transforms/`; benches `benches/transforms/`; examples `examples/transforms/`.
- **colormap**: `src/colormap/`; tests `tests/colormap/`.
- **stats**: `src/stats.rs`; tests `tests/stats/`.
- **monte_carlo**: `src/monte_carlo.rs`; tests `tests/monte_carlo/`; benches `benches/monte_carlo/`; examples `examples/monte_carlo/`, `examples/viz/` (scatter).
- **tensor**: `src/cube.rs`; tests `tests/tensor/`.
- **Tests**: One test binary per domain (e.g. `cargo test --test linear`). Integration/example logic: `tests/integration/`. Serde: `tests/serialization/` (feature-gated).
- **Benchmarks**: Domain folders under `benches/` (linear, ml, optimisation, cg, noise, graph, tree, transforms); main harness in `benches/benchmarks.rs`.
- **render**: `render/src/platform/` (wasm, native), `render/src/scene/`, `render/src/backend/`, `render/src/engine.rs`, `render/src/gizmo/`, `render/src/pick.rs`, `render/src/view_mode.rs`, `render/src/ui/` (spring in components); line/curve primitives in `scene/components/primitive.rs` (`CurvePoint`, `LineSegment`, `Bezier`, `Hermite`, `BSpline`), mesh in `backend/mesh.rs`, line pipeline in `backend/gpu.rs`; tests `render/tests/` (e.g. `gizmo_pick_view_spring`); examples `render/examples/` (e.g. `gizmo_picking`, `curves_gizmo`); bench `render/benches/simd_vs_scalar.rs`. Build: `just build-render`, `just run-render`, `just build-render-wasm` (see justfile). When adding or changing render code, update [docs/render.md](docs/render.md) and crate/module docs per the Documentation conventions above.
- **collision**: `collision/src/` (aabb, ray, frustum, bsp, bvh, sphere, obb, capsule); tests `collision/tests/`; benches `collision/benches/`; examples `collision/examples/`. Build: `just build-collision`, `just test-collision`, `just bench-collision` (see justfile). See [docs/collision.md](docs/collision.md).
- **parse**: `parse/src/` (json, bjson, toon, xml, obj, mtl, bvh, gltf, ply, image, archive); tests `parse/tests/`; bench `parse/benches/`. Formats are feature-gated. Build: `just build-parse`, `just test-parse`, `just build-parse-wasm` (see justfile). Uses mathlib for 3D types. See [docs/parse.md](docs/parse.md).
- **kinematics**: `kinematics/src/joints/` (fixed, revolute, prismatic, spherical), `kinematics/src/armature.rs`, `kinematics/src/ik/`; tests `kinematics/tests/`; examples `kinematics/examples/`; bench `kinematics/benches/`. Demo app (3D arm + IK, winit + WASM): `kinematics/demo/` (crate `kinematics-demo`). Build: `just build-kinematics`, `just test-kinematics`; demo: `just run-kinematics-demo`, `just build-kinematics-demo-wasm`, `just wasm-kinematics-demo`.
- **physics**: `physics/src/` (particle, constraint, state, solver, integration, contact, rigid, spatial, islands, hooks, serialization); tests `physics/tests/`; benches `physics/benches/`; examples `physics/examples/`. Demo: `physics/demo/` (crate `physics-demo`, winit + WASM). Build: `just build-physics`, `just test-physics`, `just bench-physics`; demo: `just run-physics-demo`, `just build-physics-demo-wasm`, `just wasm-physics-demo`. See [docs/physics.md](docs/physics.md).
- **geometry**: `geometry/src/` (mesh, half_edge, voxel, tet, csg, remesh, smooth, error, wasm); tests `geometry/tests/`; benches `geometry/benches/`; examples `geometry/examples/`. Demo: `geometry/demo/` (crate `geometry-demo`, winit + WASM). Build: `just build-geometry`, `just test-geometry`, `just bench-geometry`; demo: `just run-geometry-demo`, `just build-geometry-demo-wasm`. See [docs/geometry.md](docs/geometry.md).

## Workflow for complex tasks

- Use **sequential thinking** (e.g. MCP `sequentialthinking`) for multi-step tasks: decompose the problem, plan steps, verify assumptions, refine. For complex render/engine work (gizmo, picking, view modes, UI), use the **render-sequential** skill (`.cursor/skills/render-sequential`).
- Break edits by **domain** using [docs/domains.md](docs/domains.md) (linear, ml, graph, cg, etc.).
- Plan edits before executing; prefer minimal, focused changes.

## Tools for agents (Cursor / MCP)

- **Rust skill**: Use the project Rust skill (e.g. `.cursor/skills/rust-skills` or `/rust-skills`) when writing, reviewing, or refactoring Rust code and when improving documentation (doc-*, api-*, err-* rules).
- **Context7**: Use the Context7 MCP to fetch up-to-date docs and examples for Rust crates (e.g. `criterion`, `rayon`, `serde`) when implementing or debugging.
- **ast-grep** (optional): Use the ast-grep MCP for pattern-based code search when refactoring or adding rules.
- **Playwright MCP**: For manual WASM demo verification, the Playwright MCP server can drive a browser to the served demos; see [e2e/README.md](e2e/README.md#manual-wasm-verification-with-playwright-mcp).

## More documentation

- [docs/DOCS.md](docs/DOCS.md) — mathlib architecture and usage (human-oriented).
- **Unified WASM website**: [website/](website/) — single site for all WASM demos (mathlib, render, kinematics, physics, geometry). From repo root: `just website-build`, `just website-serve`, `just website-docker-build`, `just website-docker-run`. See [website/README.md](website/README.md).
- [docs/render.md](docs/render.md) — render architecture, main types, build, conventions.
- [docs/collision.md](docs/collision.md) — collision crate: AABB, ray, frustum, BSP, BVH, shapes.
- [docs/physics.md](docs/physics.md) — physics crate: PBD/XPBD, particles, constraints, contact, rigid bodies, solver, demo.
- [docs/dev-tools.md](docs/dev-tools.md) — Dev setup, coverage, and optional tools.
- [docs/claude.md](docs/claude.md) — Short project context for Claude and other AI assistants.
- [SECURITY.md](SECURITY.md) — How to report vulnerabilities (do not open public issues).
- **Rustdoc**: `cd mathlib && cargo doc --open` or `cd render && cargo doc --open` for full API.
