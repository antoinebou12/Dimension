# Parse crate

Multi-format parser library for the Dimension repository. Supports JSON, BJSON, TOON, XML, OBJ, MTL, BVH, glTF, PLY, PNG, JPEG, ZIP, and TAR.

## Architecture

```
parse crate
├── json, bjson, toon, xml   (data formats)
├── obj, mtl, bvh, gltf, ply (3D formats)
├── image                    (PNG, JPEG)
└── archive                  (ZIP, TAR)
```

All 3D output types use mathlib (`Point3`, `Vector3f`, `Matrix4f`). Shared types: `Vertex`, `Mesh`, `Material`.

## Features

| Feature | Formats |
|---------|---------|
| `json` | JSON |
| `bjson` | BJSON (bjson.org) |
| `toon` | TOON (Token-Oriented Object Notation) |
| `xml` | XML (quick-xml) |
| `obj` | Wavefront OBJ |
| `mtl` | Wavefront MTL |
| `bvh` | BVH (motion capture) |
| `gltf` | glTF / GLB |
| `ply` | PLY |
| `image` | PNG, JPEG |
| `archive` | ZIP, TAR |
| `simd` | SIMD optimizations (wide) |
| `parallel` | Parallel parsing (native only) |
| `wasm` | wasm-bindgen bindings |

## Usage

```rust
// JSON
let v = parse::json::parse(br#"{"a":1}"#)?;

// OBJ
let obj = parse::obj::parse(obj_bytes, None)?;

// BVH
let bvh = parse::bvh::parse(bvh_bytes)?;

// PNG
let img = parse::image::parse_png(png_bytes)?;
```

## WASM

Build with `--features "json,bjson,toon,xml,obj,ply,image,wasm"`. Do not use `parallel` with wasm32.

## Commands

```bash
just build-parse
just test-parse
just bench-parse
just build-parse-wasm
```
