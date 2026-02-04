# Claude Context for mathlib

Quick reference for Claude and other AI assistants working with the mathlib Rust linear algebra library.

## Quick Reference

| Item | Value |
|------|-------|
| Language | Rust (edition 2021) |
| Crate path | `mathlib/` |
| Storage order | Column-major by default |
| Scalar types | `f64` (primary), `f32` (3D/graphics) |
| Feature flags | `serde`, `parallel`, `simd`, `wasm`, `genetic` |

## Domain Map

Tests, benches, and examples are organized by domain. See [domains.md](domains.md) for paths.

| Domain | Description | Key modules / types |
|--------|-------------|---------------------|
| **linear** | Solvers, decompositions, simplex, structure | structure, matrix, vector, operators, linear (chol, lu, solve, schur, qz), decomposition, simplex |
| **ml** | Clustering, SVM, distance | clustering, svm, distance |
| **optimisation** | Argmin (PSO, GD, CG, etc.), genetic (CMA-ES) | argmin, genetic |
| **graph** | Pathfinding, coloring, structure | graph |
| **tree** | BFS, DFS | graph::tree |
| **cg** | Camera, math3d, quaternion, trig, easing | math/cg, math3d, quaternion, trig, easing |
| **noise** | Procedural noise | noise |
| **transforms** | FFT, DCT, wavelets, convolution, windows | transforms |
| **colormap** | Color types and conversions | colormap |
| **stats** | Descriptive (e.g. covariance) | stats |
| **tensor** | N-dimensional (Cube) | cube |
| **runtime** | CPU (sequential, parallel, simd), GPU | cpu, gpu |
| **wasm** | WebAssembly bindings | wasm |

## Key Types

### Matrix and Vector
```rust
use mathlib::{Matrix, Vector, Storage};

// Column-major matrix (default)
let mut a = Matrix::<f64>::with_storage(3, 3, Storage::Column);
a.set(0, 0, 1.0);
let v = Vector::from_slice(&[1.0, 2.0, 3.0]);
let w = &a * &v;  // matrix-vector product
```

### 3D Types
```rust
use mathlib::{Matrix4f, Vector3f, Point3, Quat4f};
use mathlib::{make_rotation, matrix4f_inverse, matrix4_mul_vector3};
use mathlib::cg::{look_at_rh, new_perspective};

// All 3D types use f32
let rotation = make_rotation(0.1, 0.2, 0.3);  // 3x3 rotation
let view = look_at_rh(&eye, &target, &up);    // 4x4 view matrix
let proj = new_perspective(aspect, fov, near, far);
```

### Decompositions
```rust
use mathlib::{svd_econ, solve, Cholesky, Lu, pca};

let svd = svd_econ(&matrix);  // U, V, sigma
let x = solve(&a, &b)?;       // Ax = b
let chol = Cholesky::new(&spd_matrix)?;
let lu = Lu::new(&square_matrix)?;
let pca_result = pca(&data, Some(3));  // top 3 components
```

### Clustering & Distance
```rust
use mathlib::{kmeans, dbscan, euclidean, cosine_similarity};

let result = kmeans(&data, 5, None);  // 5 clusters
let labels = dbscan(&data, 0.5, 5);   // eps=0.5, min_pts=5
let dist = euclidean(&v1, &v2);
```

## Common Patterns

### Storage Layout
- **Column-major (default)**: `(i, j)` → `j * rows + i`
- **Row-major**: `(i, j)` → `i * cols + j`
- 4×4 matrices in column-major match OpenGL/shader conventions

### Index Convention
- `get(i, j)` / `set(i, j, val)` — row `i`, column `j` (0-indexed)
- Vectors are column vectors with `rows()` elements

### Error Handling
- Decompositions return `Result<T, Error>` (e.g., `CholError`, `SolveError`)
- `solve()` panics on non-square matrices; returns error on singular

### Feature Combinations
```bash
# Native (fastest)
cargo build --features "parallel simd"

# WASM (no parallel, simd optional)
cargo build --target wasm32-unknown-unknown --features wasm

# Serialization
cargo build --features serde
```

## Build Commands

```bash
cd mathlib

cargo build                    # Build library
cargo test                     # Run tests
cargo bench                    # Run benchmarks
cargo doc --open               # Generate and view docs
cargo clippy --all-targets     # Lint

# WASM
wasm-pack build --target web --features wasm

# With features
cargo test --features "parallel simd"
cargo test --features serde
```

Or use `just` from repo root:
```bash
just build          # cargo build
just test           # cargo test
just bench          # cargo bench
just check-wasm     # WASM check
```

## Numerical Notes

### Precision
- Default scalar type is `f64` for numerical stability
- 3D graphics types use `f32` (matches GPU precision)
- SVD uses Golub-Reinsch algorithm

### Stability Considerations
- Cholesky requires symmetric positive definite input
- LU uses partial pivoting
- PCA centers data before SVD (subtracts column means)
- Clustering uses squared Euclidean internally for efficiency

### Special Values
- `NOISE` constant (`usize::MAX`) marks noise points in DBSCAN
- Quaternion `slerp` handles near-identity interpolation
