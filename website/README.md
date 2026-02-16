# Dimension — unified WASM demos website

Single static site that serves all WASM demos (mathlib, render, kinematics, physics, geometry) from one origin.

## Build and serve (local)

From **repo root**:

```bash
just website-build        # Build all WASM demos and populate website/
just website-build-gpu    # Same, but with GPU-enabled mathlib (WebGPU init, async matmul, etc.)
just website-build-simd   # Same as website-build, but render demo built with SIMD (can improve frame rate)
just website-serve        # Serve website/ (e.g. http://localhost:3000)
```

Then open the hub at `/` and demos at `/mathlib/`, `/render/`, `/kinematics/`, `/physics/`, `/geometry/`.

**"GPU bindings not available":** That message refers to the **mathlib** Linear/GPU page (WebGPU compute for matmul, dot, etc.), not the render demo. To fix it, use `just website-build-gpu` instead of `just website-build`, then open the mathlib demo and click **"Init GPU"** on the Linear page.

## Docker

From **repo root**:

```bash
just website-docker-build   # Build image (builds WASM inside container)
just website-docker-run    # Run container (http://localhost:8080)
```

Requires Docker. The image builds all WASM in the container and serves the static site with nginx.
