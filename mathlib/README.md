# mathlib

Linear algebra library: dense/sparse matrices, vectors, SVD, 3D math, clustering, graph algorithms, and more. Parallel execution uses par-iter with chili backend (heartbeat scheduling). See the [repository root](../) and [docs](https://docs.rs/mathlib) for more.

For the browser demo (vectors, matrices, clustering, solvers, pathfinding, PSO, noise, etc. via WebAssembly), see [demo/wasm-demo/](demo/wasm-demo/). Run `just wasm-build` from the repo root then `just wasm-serve` (or see demo/wasm-demo/README.md). Optional GPU compute (WebGPU) for f32 matmul, matvec, dot, norm, add, sub: build with `--features "wasm gpu"` and use `initGpuAsync()` in the demo. GPU example: `cargo run --example gpu_large_matrix --features gpu`. Examples organized by domain: `linear/`, `ml/`, `wasm/`, `gpu/`, `optimization/`, etc. See [demo/wasm-demo/README.md](demo/wasm-demo/README.md).

## License

MIT
