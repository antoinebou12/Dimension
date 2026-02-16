# mathlib WASM demo

Browser demo of mathlib compiled to WebAssembly. Demos are split into domain-based pages aligned with [mathlib/examples/](../examples/) and [mathlib/tests/wasm/](../tests/wasm/).

**Live demo:** The demo is deployed to GitHub Pages from CI (see [.github/workflows/pages.yml](../../.github/workflows/pages.yml)). Enable **Settings → Pages → GitHub Actions** to get a live URL (e.g. `https://<owner>.github.io/<repo>/`).

## Structure

| Page | Path | Contents |
|------|------|----------|
| Hub | [index.html](index.html) | Landing with links to domain pages |
| Linear | [linear/](linear/) | Vector, Matrix, Storage, Cholesky, SVD, LU; Large matrix (GPU) |
| GPU | [gpu/](gpu/) | WebGPU init, async matmul, dot, matvec (build with wasm-build-gpu) |
| ML | [ml/](ml/) | K-means, PCA, SVM, DBSCAN |
| Optimization | [optimization/](optimization/) | Simplex LP, Line search, PSO, L-BFGS-B |
| Graph | [graph/](graph/) | Dijkstra, A*, D* Lite; vertex coloring; BFS/DFS |
| Distance | [distance/](distance/) | Euclidean, Manhattan, cosine, Chebyshev, Minkowski |
| Monte Carlo | [monte_carlo/](monte_carlo/) | π estimation (scatter); optional ∫x² 1D integration |
| Noise | [noise/](noise/) | Wave 2D, Perlin 2D, FBM Perlin heightmap |
| Transforms | [transforms/](transforms/) | FFT, DCT-2, Haar DWT, convolution |
| Camera | [camera/](camera/) | Perspective and look-at 4×4 matrices |
| Viz | [viz/](viz/) | Procedural heightmap (wave, FBM) |

Shared utilities live in [shared.js](shared.js). Each domain page loads WASM on demand via `initLib()`. A **dark mode** toggle is available on every page (theme is stored in `localStorage`).

## Build

From **repo root**: `just wasm-build` — builds with wasm-pack and copies `pkg/` into `wasm-demo/pkg/`. For GPU (Large matrix, vector dot/norm, matvec): use `just wasm-build-gpu`; then open the Linear page and click **"Init GPU"** first. Async GPU bindings: `matmulF32GpuAsync`, `dotF32GpuAsync`, `normF32GpuAsync`, `matvecF32GpuAsync`. If you see "GPU bindings not available (build with just wasm-build-gpu)", that means the pkg was built without the `gpu` feature — run `just wasm-build-gpu` (or for the unified site, `just website-build-gpu`) and click "Init GPU" in the browser.

From **mathlib** directory:

```bash
wasm-pack build --target web --features wasm
# Copy pkg to wasm-demo/pkg (e.g. cp -r pkg wasm-demo/ when in mathlib, or Copy-Item on Windows)
```

**Windows:** `Copy-Item -Path mathlib\pkg -Destination mathlib\demo\wasm-demo\pkg -Recurse -Force`

## Serve

Browsers need HTTP for ES modules and WASM.

**From mathlib/demo**: `npx serve .` then open **/wasm-demo/** (use the URL shown by the server).

**Or** from mathlib: `npx serve demo` then open **/wasm-demo/** (use the URL shown by the server).

### Load performance (streaming)

The server must serve `.wasm` with `Content-Type: application/wasm`. Most static servers (including `npx serve`) do this by default.
