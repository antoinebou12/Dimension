# mathlib WASM demo — Vector add

Minimal browser example: vector addition using mathlib compiled to WebAssembly. The built `pkg/` lives inside this folder (`wasm-demo/pkg/`) so the demo is self-contained.

## Build

From **repo root**: `just wasm-build` — builds with wasm-pack and copies `pkg/` into `wasm-demo/pkg/`.

From **mathlib** directory:

**Option A — wasm-pack** (if your Cargo supports it):

```bash
wasm-pack build --target web --features wasm
cp -r pkg wasm-demo/
```

**Option B — manual** (if wasm-pack fails with `--artifact-dir` / unstable Cargo). Install `wasm-bindgen-cli` then:

```bash
cargo build --release --target wasm32-unknown-unknown --features wasm
wasm-bindgen target/wasm32-unknown-unknown/release/mathlib.wasm --out-dir pkg --target web --out-name mathlib
cp -r pkg wasm-demo/
```

Or from repo root: `just wasm-build-manual`.

You should have `wasm-demo/pkg/` with `mathlib.js` and `mathlib_bg.wasm`.

## Serve

Browsers need HTTP for ES modules and WASM.

**From mathlib**: `npx serve .` then open **http://localhost:3000/wasm-demo/**.

**Or serve only this folder**: from mathlib run `npx serve wasm-demo` then open **http://localhost:3000/**.
