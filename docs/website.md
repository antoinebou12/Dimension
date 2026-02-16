# Unified website (WASM demos)

The Dimension **unified website** is a single static site that serves all WASM demos (mathlib, render, kinematics, physics, geometry, neural) from one origin. The hub is at `/`; demos live at `/mathlib/`, `/render/`, `/kinematics/`, `/physics/`, `/geometry/`, `/neural/`, and the **embedding demo** at `/neural/embedding/`.

## Build

From **repo root**:

| Command | Description |
|---------|-------------|
| `just website-build` | Build all WASM demos and populate `website/` (mathlib, render, kinematics, physics, geometry, neural). |
| `just website-build-gpu` | Same, but with GPU-enabled mathlib (WebGPU init, async matmul, etc.). |
| `just website-build-simd` | Same as website-build, but render and kinematics demos built with SIMD (can improve frame rate). |

Each build populates the `website/` directory with the built demos. No separate copy step is needed.

## Serve (local)

From **repo root**:

```bash
just website-serve
```

This runs a static server (e.g. `npx serve`) from `website/`, typically on **port 3000**. Open the hub at `http://localhost:3000/` and demos at `/mathlib/`, `/render/`, `/kinematics/`, `/physics/`, `/geometry/`, `/neural/`, `/neural/embedding/`.

**"GPU bindings not available":** That message refers to the **mathlib** Linear/GPU page (WebGPU compute), not the render demo. Use `just website-build-gpu` and then click **"Init GPU"** on the mathlib Linear page.

## Docker

From **repo root**:

```bash
just website-docker-build   # Build image (builds WASM inside container)
just website-docker-run    # Run container (http://localhost:8080)
```

Requires Docker. The image builds all WASM in the container and serves the static site with nginx.

## Publishing (GitHub Pages)

The same `website/` tree is deployed to **GitHub Pages** on push to `main`/`master`. Enable **Settings → Pages → Build and deployment → GitHub Actions**. The workflow builds the full site with `just website-build` and uploads the `website/` directory as the Pages artifact. The live site is available at your repository's GitHub Pages URL (e.g. `https://<owner>.github.io/<repo>/`). See [.github/workflows/pages.yml](../.github/workflows/pages.yml).

For **project** Pages (URL contains `/<repo>/`), the site is served under that path. If you need a base path (e.g. `/<repo>/`) for assets and links, the workflow or a post-build step can be extended to set it; the default workflow assumes the artifact is served at the site root.

## Embedding demo

The **embedding demo** (`/neural/embedding/`) is a top-level entry on the hub. It shows text embeddings reduced to 3D with PCA; you can click points or pick a query index to see similar documents. It uses mathlib for PCA and shares data with the mathlib **Recommendation** demo at `/mathlib/recommendation/`. To regenerate the shared data, run the neural example `precompute_demo_embeddings` (precompute feature) and write output to `mathlib/demo/wasm-demo/recommendation/data.json`. See [neural.md](neural.md) for the neural crate and embedding features.
