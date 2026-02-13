# mathlib domain organization

Quick reference for domain taxonomy and paths. Used by contributors and LLMs.

## Domain taxonomy

| Domain | Description | Src | Tests | Benches | Examples |
|--------|-------------|-----|-------|---------|----------|
| **linear** | Solvers, decompositions, simplex | linear/, decomposition/, simplex/ | tests/linear/ | benches/linear/ | examples/linear/ |
| **structure** | Storage, matrix base, sparse | structure/, matrix, operators | tests/structure/ | — | — |
| **vector** | Column vector | vector | tests/vector/ | — | — |
| **ml** | Clustering, SVM, distance | clustering/, svm/, distance/ | tests/ml/ | benches/ml/ | examples/ml/ |
| **optimisation** | PSO, gradient descent, genetic | argmin/, genetic/ | tests/optimisation/ | benches/optimisation/ | examples/optimisation/ |
| **graph** | Pathfinding, coloring, matrix representation | graph/ | tests/graph/ | benches/graph/ | examples/graph/ |
| **tree** | BFS, DFS, Tree/Node | graph/tree/ | tests/tree/ | benches/tree/ | examples/tree/ |
| **cg** | Camera, 3D, quaternion, easing, curve | math/cg, math3d, quaternion, trig, easing, math/curve | tests/cg/ | benches/cg/ | examples/cg/ |
| **noise** | Procedural noise | noise/ | tests/noise/ | benches/noise/ | examples/viz/ |
| **transforms** | FFT, DCT, wavelets, convolution | transforms/ | tests/transforms/ | benches/transforms/ | examples/transforms/ |
| **colormap** | Color types, palettes | colormap/ | tests/colormap/ | — | examples/viz/ |
| **stats** | Covariance, descriptive | stats | tests/stats/ | — | — |
| **monte_carlo** | π estimation, 1D integration | monte_carlo | tests/monte_carlo/ | benches/monte_carlo/ | examples/monte_carlo/, examples/viz/ (scatter) |
| **tensor** | N-dimensional cube | cube | tests/tensor/ | — | — |
| **hash** | Hashing utilities | hash | tests/hash/ | — | — |
| **runtime** | CPU/GPU backends | cpu, gpu | tests/gpu/, tests/parallel/ | gpu_bench, parallel_comparison | examples/gpu/ |
| **wasm** | WebAssembly bindings | wasm | tests/wasm/ | — | examples/wasm/ |
| **serialization** | Serde tests | — | tests/serialization/ | — | — |
| **integration** | Example compilation | — | tests/integration/ | — | — |
| **parse** | Multi-format parsers | parse/src/ | parse/tests/ | parse/benches/ | — |
| **kinematics** | Joints, armature, IK; demo at kinematics/demo | kinematics | kinematics/tests | kinematics/benches | kinematics/examples |
| **physics** | PBD/XPBD, particles, constraints, contact, rigid; demo at physics/demo | physics | physics/tests | physics/benches | physics/examples |

## Running by domain

```bash
cd mathlib

# Tests
cargo test --test linear
cargo test --test ml
cargo test --test optimisation
cargo test --test graph
cargo test --test cg
cargo test --test noise
cargo test --test transforms

# Benchmarks (filter by group name)
cargo bench -- linear
cargo bench -- ml
cargo bench -- transforms
```

## Test targets

- linear, structure, vector, ml, optimisation, graph, tree, cg
- noise, transforms, colormap, stats, monte_carlo, tensor, hash
- serialization, parallel, performance, integration
- wasm, gpu
