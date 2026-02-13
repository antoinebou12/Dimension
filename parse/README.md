# parse

Multi-format parsers for Dimension: JSON, BJSON, TOON, XML, OBJ, MTL, BVH, glTF, PLY, PNG, JPEG, ZIP, TAR.

## Features

- **Data**: `json`, `bjson`, `toon`, `xml`
- **3D**: `obj`, `mtl`, `bvh`, `gltf`, `ply`
- **Media**: `image` (PNG, JPEG), `archive` (ZIP, TAR)
- **Perf**: `simd`, `parallel` (native only)
- **Integration**: `wasm`, `serde`

## Usage

```toml
[dependencies]
parse = { path = "../parse", features = ["json", "obj"] }
mathlib = { path = "../mathlib" }
```

## WASM

Use `--features wasm` or `--features "wasm simd"`. Do not use `parallel` with wasm32.
