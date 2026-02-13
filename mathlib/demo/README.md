# mathlib demo

Browser demo of mathlib compiled to WebAssembly. See [wasm-demo/](wasm-demo/) for the full demo with domain-based pages (linear, ml, graph, etc.).

## Build and serve

From repo root:

```bash
just wasm-build      # or just wasm-build-gpu for GPU support
just wasm-serve
```

Then open **/wasm-demo/** in your browser. See [wasm-demo/README.md](wasm-demo/README.md) for details.
