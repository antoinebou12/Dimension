# WASM build and usage

This page covers building mathlib for WebAssembly, running tests, and optional GPU compute.

## Live demo

The wasm-demo is built in CI and deployed to GitHub Pages on every push to `main`/`master`. Once **Settings → Pages → Build and deployment** is set to **GitHub Actions**, the live demo is available at your repository's GitHub Pages URL (e.g. `https://<owner>.github.io/<repo>/`). See [dev-tools.md](dev-tools.md) and [mathlib/wasm-demo/README.md](../mathlib/wasm-demo/README.md).

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
- **WASM:** The demo and JS must call `initGpuAsync()` (returns a Promise) and await it before relying on GPU. The synchronous `*` operator and dot/norm still use CPU on wasm (GPU readback is async there). The demo shows a "Backend: CPU" / "Backend: GPU" label and an "Init GPU" button when built with `wasm gpu`.

Build with GPU for the web demo:

```bash
cd mathlib && wasm-pack build --target web --features "wasm gpu"
```

Then copy `pkg/` to `wasm-demo/pkg/` and serve. The demo will show the GPU status and init button only if the build includes the gpu feature.

To benchmark CPU vs GPU on native:

```bash
cd mathlib && cargo bench --features gpu --bench gpu
```

This runs separate groups for matmul, matvec, add, scale, mul, axpy, abs, sqrt, div, spmv, dot, and norm (each with `cpu_` and `gpu_` variants). The demo includes a "Large matrix" subsection (sizes 64, 256, 512) for matmul timing when built with GPU.

## Troubleshooting

- **"parallel feature not supported for target wasm32"** — Use `--features wasm` or `--features "wasm simd"` only; omit `parallel` and `full` for wasm32.
- **Demo shows "Cannot load pkg/mathlib.js"** — Run `just wasm-build` from the repo root (or `wasm-pack build --target web --features wasm` in `mathlib/`) and ensure `mathlib/wasm-demo/pkg/` exists.
- **GPU init fails in browser** — WebGPU requires a secure context (HTTPS or localhost) and a supporting browser (Chrome 113+, Edge 113+, etc.). The demo falls back to CPU if GPU is unavailable.
