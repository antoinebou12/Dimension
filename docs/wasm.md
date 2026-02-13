# WASM build and usage

This page covers building mathlib for WebAssembly, running tests, and optional GPU compute.

## Live demo

The wasm-demo is built in CI and deployed to GitHub Pages on every push to `main`/`master`. Once **Settings → Pages → Build and deployment** is set to **GitHub Actions**, the live demo is available at your repository's GitHub Pages URL (e.g. `https://<owner>.github.io/<repo>/`). See [dev-tools.md](dev-tools.md) and [mathlib/demo/wasm-demo/README.md](../mathlib/demo/wasm-demo/README.md).

## Build matrix

| Build | Command | Notes |
|-------|---------|--------|
| wasm32 (no bindings) | `cargo build --target wasm32-unknown-unknown` | Library only, no JS exports |
| wasm32 + bindings | `cargo build --target wasm32-unknown-unknown --features wasm` | Exposes `mathlib::wasm` and cdylib for wasm-pack |
| wasm32 + SIMD | `cargo build --target wasm32-unknown-unknown --features "wasm simd"` | Faster CPU path where supported |
| wasm32 + GPU | `cargo build --target wasm32-unknown-unknown --features "wasm gpu"` | Adds WebGPU init and `gpuAvailable` / `initGpuAsync` for the demo |

Do **not** use the `parallel` feature with wasm32 (Rayon is not available); the build script will error.

From the repo root you can use:

- `just build-wasm` — build with wasm feature
- `just check-wasm` — check only
- `just test-wasm` — runs the wasm **test target** (native host, not in Node)
- `just build-wasm-simd` — wasm + SIMD

## Running WASM tests (Node)

Integration tests under `mathlib/tests/wasm/` are compiled and run on the **native** host by default:

```bash
cd mathlib && cargo test --features wasm wasm
```

To run the same tests in a Node.js environment (wasm-bindgen-test), you would use wasm-pack's test runner. The project currently runs the wasm integration tests as native tests that exercise the same Rust API as the wasm bindings.

## GPU compute (optional)

When built with the `gpu` feature, mathlib can use WebGPU (via wgpu) for f32 matrix multiplication, matrix-vector product, vector dot product, norm, element-wise add/sub/mul/div, scalar scale, axpy, abs, sqrt, and sparse SpMV on the GPU.

- **Native:** Call `mathlib::gpu::init_blocking()` once before using `Matrix<f32> * Matrix<f32>`, `Matrix<f32> * Vector<f32>`, `s * Matrix<f32>`, `s * Vector<f32>`, `SparseMatrixCRS<f32> * Vector<f32>`, `Vector<f32>::dot`, `Vector<f32>::norm`, `A + B`, or `A - B`. If initialization succeeds, those operations will use the GPU when applicable (matmul and matvec require column-major; dot/norm work for vectors up to 65536 elements).
- **WASM:** The demo and JS must call `initGpuAsync()` (returns a Promise) and await it before relying on GPU. Async GPU bindings include `matmulF32GpuAsync`, `dotF32GpuAsync`, `normF32GpuAsync`, `matvecF32GpuAsync`, and **PCA transform** via `WasmPca.prototype.transformF32GpuAsync(data: WasmMatrix32)` (returns a Promise resolving to the projected matrix or `null` if GPU is unavailable; fall back to sync `transform()` with f64 data). SVD and PCA **fit** remain CPU-only; full GPU SVD is future work. The synchronous `*` operator and dot/norm use CPU on wasm; use the async variants for GPU-accelerated results. The demo shows a "Backend: CPU" / "Backend: GPU" label and an "Init GPU" button when built with `wasm gpu`.

Build with GPU for the web demo:

```bash
cd mathlib && wasm-pack build --target web --features "wasm gpu"
```

Then copy `pkg/` to `wasm-demo/pkg/` and serve. The demo will show the GPU status and init button only if the build includes the gpu feature.

**Async SVD (wasm feature):** `WasmMatrix.prototype.svdEconAsync()` returns a Promise that resolves to the same result as sync `svdEcon()`. Use it for loading states; for very large matrices, run sync `svdEcon()` in a Web Worker.

To benchmark CPU vs GPU on native:

```bash
cd mathlib && cargo bench --features gpu --bench gpu
```

This runs separate groups for matmul, matvec, add, scale, mul, axpy, abs, sqrt, div, spmv, dot, and norm (each with `cpu_` and `gpu_` variants). The demo includes a "Large matrix" subsection (sizes 64, 256, 512) for matmul timing when built with GPU. The demo's "Basic f32 CPU vs GPU" section uses default sizes (100k vector, 256×256 matvec) where CPU typically wins; for GPU to show an advantage, use larger sizes (e.g. 1M–2M elements for vectors; matvec crossover is around ~2M elements per bench guidance). See `ExecutorThresholds` in the crate for threshold-based selection.

## Prerequisites for building the web demo

`just wasm-build` and `just wasm-build-gpu` use **wasm-pack**. Install it if missing:

```bash
cargo install wasm-pack
```

Alternatively, use `just wasm-build-manual` to build with `cargo` and `wasm-bindgen` only (no wasm-pack).

## Troubleshooting

- **"wasm-pack: No such file or directory"** — Install wasm-pack: `cargo install wasm-pack`. Or use `just wasm-build-manual` (cargo + wasm-bindgen only).
- **wasm-opt "invalid code after misc prefix" / "error parsing wasm"** — The system binaryen (wasm-opt) may be older than the WASM produced by Rust. mathlib disables wasm-opt by default (`wasm-opt = false` in `Cargo.toml`). To use wasm-opt, install a recent binaryen and set `wasm-opt = ["-Oz"]` in `[package.metadata.wasm-pack.profile.release]`.
- **"parallel feature not supported for target wasm32"** — Use `--features wasm` or `--features "wasm simd"` only; omit `parallel` and `full` for wasm32.
- **Demo shows "Cannot load pkg/mathlib.js"** — Run `just wasm-build` from the repo root (or `wasm-pack build --target web --features wasm` in `mathlib/`) and ensure `mathlib/demo/wasm-demo/pkg/` exists.
- **GPU init fails in browser** — WebGPU requires a secure context (HTTPS or localhost) and a supporting browser (Chrome 113+, Edge 113+, etc.). The demo falls back to CPU if GPU is unavailable.

For manual verification of WASM demos in a browser, you can use the [Playwright MCP server](https://github.com/microsoft/playwright-mcp) (see [e2e/README.md](../e2e/README.md#manual-wasm-verification-with-playwright-mcp)).
