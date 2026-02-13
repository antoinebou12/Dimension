# Dimension — unified WASM demos website

Single static site that serves all WASM demos (mathlib, render, kinematics, physics, geometry) from one origin.

## Build and serve (local)

From **repo root**:

```bash
just website-build    # Build all WASM demos and populate website/
just website-serve    # Serve website/ (e.g. http://localhost:3000)
```

Then open the hub at `/` and demos at `/mathlib/`, `/render/`, `/kinematics/`, `/physics/`, `/geometry/`.

## Docker

From **repo root**:

```bash
just website-docker-build   # Build image (builds WASM inside container)
just website-docker-run    # Run container (http://localhost:8080)
```

Requires Docker. The image builds all WASM in the container and serves the static site with nginx.
