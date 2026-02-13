# render

2D/3D rendering engine using wgpu and mathlib. WASM-first, platform-separated, GPU/CPU.

Full architecture and main types: [docs/render.md](../docs/render.md).

## Build and run

**Native:**
```bash
cargo run -p render-demo --example render_native
```
Or from repo root: `just run-render`

**WASM web:**
```bash
just render-wasm         # build + serve; open http://localhost:3000/wasm-demo/
```
Or: `just demo-render` (alias). The render WASM build requires **wasm-bindgen** on PATH (e.g. `cargo install wasm-bindgen-cli`). **Rust (stable)**; see `rust-toolchain.toml`. On **Linux/WSL**, a C compiler (**clang** or **build-essential**) may be needed for other build steps; e.g. `sudo apt update && sudo apt install clang`. **wasm-opt** (binaryen) is optional—if not installed, the demo still runs but the WASM is not optimized. Plain compile: `just build-render-wasm`

**Troubleshooting:** If `just render-wasm` reports "wasm-bindgen not found", install it with `cargo install wasm-bindgen-cli`. If you see "Missing manifest in toolchain 'stable'" (Windows) or rustup rename/conflict errors (Linux), repair the stable toolchain: `rustup toolchain uninstall stable` then `rustup install stable`. On WSL with a shared Windows home dir, use Rust from one environment only to avoid toolchain conflicts.

## Architecture

- **Platform**: `wasm` (canvas, request_animation_frame) and `native` (winit, forte for async init)
- **Scene**: mathlib `Tree` for hierarchical world; `Transform`, `Primitive`, `NodeData`
- **Backend**: wgpu only; mathlib for CPU math (MVP, world matrix, orthographic camera)

## Examples (render-demo crate)

See [render/demo/README.md](demo/README.md). Key examples:

- `render_native` — native window with 3D scene (Cube, Tetrahedron, Cylinder), Scene panel, gizmo, picking.
- `curves_native` — line segment, Bézier, Hermite, B-spline with gizmo.
- `aabb2d_native` — 2D collision demo: two circles (Lissajous + fixed), AABB outlines, intersection/inclusion colors. Run: `just run-render-aabb2d` or `cargo run -p render-demo --example aabb2d_native`.
- `render_wasm` — WASM demo with 3D scene. See [render/demo/wasm-demo/index.html](demo/wasm-demo/index.html).
- `sdl3_quad` — native via SDL3 (`cargo run -p render-demo --example sdl3_quad --features sdl3`).

## Tests

```bash
cargo test --test native
cargo test --test shader   # WGSL parse/validate (naga, wgpu's frontend; no GPU)
cargo test --target wasm32-unknown-unknown --test wasm
```
