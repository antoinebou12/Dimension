# mathlib WASM demo

Browser demo of mathlib compiled to WebAssembly. Demos are split into domain-based pages aligned with [mathlib/examples/](../examples/) and [mathlib/tests/wasm/](../tests/wasm/).

**Live demo:** The demo is deployed to GitHub Pages from CI (see [.github/workflows/pages.yml](../../.github/workflows/pages.yml)). Enable **Settings → Pages → GitHub Actions** to get a live URL (e.g. `https://<owner>.github.io/<repo>/`).

## Structure

| Page | Path | Contents |
|------|------|----------|
| Hub | [index.html](index.html) | Landing with links to domain pages |
| Linear | [linear/](linear/) | Vector, Matrix, Storage, Cholesky, SVD, LU; Large matrix (GPU) |
| ML | [ml/](ml/) | K-means, PCA, SVM, DBSCAN |
| Optimization | [optimization/](optimization/) | Simplex LP, Line search, PSO |
| Graph | [graph/](graph/) | Dijkstra, A*, D* Lite; vertex coloring; BFS/DFS |
| Distance | [distance/](distance/) | Euclidean, Manhattan, cosine, Chebyshev, Minkowski |
| Noise | [noise/](noise/) | Wave 2D, Perlin 2D, FBM Perlin heightmap |
| Transforms | [transforms/](transforms/) | FFT, DCT-2, Haar DWT, convolution |
| Camera | [camera/](camera/) | Perspective and look-at 4×4 matrices |
| Viz | [viz/](viz/) | Procedural heightmap (wave, FBM) |

Shared utilities live in [shared.js](shared.js). Each domain page loads WASM on demand via `initLib()`.

## Build

From **repo root**: `just wasm-build` — builds with wasm-pack and copies `pkg/` into `wasm-demo/pkg/`. For GPU (Large matrix, vector dot/norm): use `just wasm-build-gpu`; click "Init GPU" in the Linear page first.

From **mathlib** directory:

```bash
wasm-pack build --target web --features wasm
# Copy pkg to wasm-demo/pkg (e.g. cp -r pkg ../wasm-demo/ or Copy-Item on Windows)
```

**Windows:** `Copy-Item -Path mathlib\pkg -Destination mathlib\wasm-demo\pkg -Recurse -Force`

## Serve

Browsers need HTTP for ES modules and WASM.

**From mathlib**: `npx serve .` then open **/wasm-demo/** (use the URL shown by the server).

**Or** from mathlib: `npx serve wasm-demo` then open `/` (use the URL shown by the server).

### Load performance (streaming)

The server must serve `.wasm` with `Content-Type: application/wasm`. Most static servers (including `npx serve`) do this by default.
