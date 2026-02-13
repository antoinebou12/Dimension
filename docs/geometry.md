# geometry — Mesh processing for Dimension

**geometry** is a crate in the Dimension repo for mesh processing: half-edge representation, voxelization, tetrahedralization, CSG boolean operations, isotropic remeshing, and Laplacian smoothing. It depends on **mathlib** for linear algebra and **collision** for AABB/BVH; it integrates with **parse** for mesh I/O (STL, MSH) and **render** for the demo.

For project context see [AGENTS.md](../AGENTS.md).

## Layout

| Path | Role |
|------|------|
| `geometry/src/mesh.rs` | [`TriMesh`](geometry/src/mesh.rs): positions, indices, normals; conversions to/from parse and render. |
| `geometry/src/half_edge.rs` | Half-edge mesh: vertices, half-edges, faces; `vertex_neighbors`, `edge_flip`, `edge_split`, `edge_collapse`, `boundary_loops`. |
| `geometry/src/voxel.rs` | [`VoxelGrid`](geometry/src/voxel.rs); `voxelize_mesh`, `flood_fill`, `marching_cubes`. |
| `geometry/src/tet.rs` | [`TetMesh`](geometry/src/tet.rs); `tetrahedralize_grid`, `tetrahedralize_surface`; quality metrics. |
| `geometry/src/csg.rs` | CSG: `csg_union`, `csg_intersection`, `csg_difference` (watertight input). |
| `geometry/src/remesh.rs` | `remesh_isotropic` (split/collapse/flip/smooth). |
| `geometry/src/smooth.rs` | `smooth_uniform`, `smooth_cotangent`, `smooth_implicit`. |
| `geometry/src/error.rs` | [`GeometryError`](geometry/src/error.rs). |
| `geometry/src/wasm.rs` | WASM bindings (feature `wasm`). |

## Dependencies

- **mathlib** — Vectors, 3D math, optional sparse solve for implicit smoothing.
- **collision** — AABB for voxel bounds, BVH for ray queries in voxelization.
- **parse** (optional, feature `parse`) — Mesh I/O; [`TriMesh`] converts to/from `parse::mesh::Mesh`.
- **physics** — Can consume geometry-generated tet meshes via conversion from `geometry::TetMesh`.

## Features

| Feature | Purpose |
|---------|---------|
| `default` | No optional deps. |
| `simd` | SIMD via mathlib (vertex ops, distance). |
| `parallel` | Parallel voxelization, remeshing, marching cubes (not on `wasm32`). |
| `wasm` | WASM bindings (wasm-bindgen). |
| `gpu` | WebGPU compute for voxelization / marching cubes (optional). |
| `parse` | Conversion from `parse::mesh::Mesh` to [`TriMesh`]. |

## Usage

```rust
use geometry::{TriMesh, voxelize_mesh, flood_fill, marching_cubes, remesh_isotropic, smooth_uniform};

// Build a triangle mesh
let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
let indices = vec![[0, 1, 2]];
let mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();

// Voxelize and extract isosurface
let mut grid = voxelize_mesh(&mesh, [32, 32, 32], 0.01).unwrap();
flood_fill(&mut grid);
let out = marching_cubes(&grid);

// Remesh and smooth
let remeshed = remesh_isotropic(&mesh, 0.1, 3).unwrap();
let mut smoothed = remeshed.clone();
smooth_uniform(&mut smoothed, 5, 0.5).unwrap();
```

## Error type

[`GeometryError`]: `DegenerateMesh`, `EmptyInput`, `NonManifold`, `InvalidTopology`, `VoxelGridTooLarge`, `TetQualityFailed`. See `geometry::GeometryError`.

## Tests and build

- **Tests**: `cargo test -p geometry` (unit tests in modules; integration in `geometry/tests/`).
- **Benchmarks**: `cargo bench -p geometry`.
- **Examples**: `geometry/examples/` (voxelize, csg_demo, remesh_demo).
- **Demo**: `geometry/demo/` — native (winit) and WASM demos using the render Engine.

From repo root:

- `just build-geometry` — build geometry crate
- `just test-geometry` — run tests
- `just bench-geometry` — run benchmarks
- `just build-geometry-wasm` — WASM build
- `just run-geometry-demo` — native demo
- `just build-geometry-demo-wasm` — WASM demo build
