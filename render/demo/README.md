# render-demo

2D/3D render demo: cube, tetrahedron, cylinder with gizmo and picking.

## Controls

- **Left drag**: Orbit camera
- **Scene panel**: Select entities, change primitives, view modes
- **Scroll**: Zoom

## Run native (winit)

```bash
just run-render
# or
cargo run -p render-demo --example render_native
```

## Run curves demo (native)

```bash
cargo run -p render-demo --example curves_native
```

## Run 2D AABB collision demo (native)

Two circles (one on a Lissajous path, one fixed), AABB outlines, merge/expand, intersection (red/green) and inclusion (blue/magenta) colors. Orthographic camera.

```bash
just run-render-aabb2d
# or
cargo run -p render-demo --example aabb2d_native
```

## Run WASM (browser)

```bash
just wasm-render-build
just wasm-render-serve
```

Then open http://localhost:3000/wasm-demo/ in a browser that supports WebGPU.

**GPU: N/A in the stats overlay:** On WASM, GPU *timing* is disabled (to avoid buffer mapping issues), so the overlay shows "GPU: N/A". This does not mean rendering is CPU-only — the 3D scene still uses wgpu/WebGPU when the browser supports it.

To stop the server: press Ctrl+C. Alternatively, run `just demo-render-bg` to serve in the background (after building), then `just demo-render-stop` to stop it.

## Performance / Troubleshooting

- Use a modern browser with WebGPU enabled (Chrome, Edge, or Firefox with `dom.webgpu.enabled`) for best frame rate; software or limited WebGPU can cause slow FPS.
- For potentially better FPS on CPU-bound work (math, culling), build the render WASM demo with SIMD: `just wasm-render-build-simd`, then `just wasm-render-serve`, and open the same URL.

## E2E tests (Playwright)

From repo root:

```bash
just e2e
```
