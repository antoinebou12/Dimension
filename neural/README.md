# neural

Embedding and vectorization for text, image, point cloud, graph, and PDF.

## Features

- **Text** (`embed-text`): `TextEmbedding::try_new_minilm()` and `embed(&[&str])` via [fastembed](https://crates.io/crates/fastembed).
- **Image** (`embed-image`): `ImageEmbedding::try_new_default()` and `embed_paths` / `embed_bytes` via fastembed.
- **Point cloud** (`pointcloud`, requires `onnx`): `PointCloudEmbedding::from_onnx_path(path)` — run a user-supplied ONNX model (e.g. PointNet) on N×C points.
- **Graph** (`embed-graph`, requires `onnx`): `GraphEmbedding::from_onnx_path(path)` — run ONNX on node features for node embeddings. A model can be downloaded into `neural/models/` via the download script.
- **PDF** (`pdf`): `extract_text(bytes)` and `extract_text_pages(bytes)` via [pdf-extract](https://crates.io/crates/pdf-extract) (native only).

## Usage

### Downloading ONNX models

From repo root:

```bash
cargo run -p neural --bin download_models -- --all
```

Or: `just download-neural-models`. This writes graph (and optionally other) ONNX models into `neural/models/`. See `neural/models/README.md`.

### Precomputing embeddings for the WASM recommendation demo

To regenerate the recommendation demo data (MiniLM text embeddings):

```bash
cargo run -p neural --example precompute_demo_embeddings --features precompute -- mathlib/demo/wasm-demo/recommendation/data.json
```

## Crate layout

| Module | Description |
|--------|-------------|
| `embed` | Text, image, point cloud, graph embeddings; PDF text extraction (optional features). |

## Features

- `embed-text`: Text embeddings (fastembed).
- `embed-image`: Image embeddings (fastembed).
- `pdf`: PDF text extraction (pdf-extract).
- `pointcloud`: Point cloud ONNX runner (depends on `onnx`).
- `embed-graph`: Graph ONNX runner (depends on `onnx`).
- `onnx`: ONNX runtime via `ort`.
- `precompute`: Embed-text + serde_json for the precompute_demo_embeddings example.

## License

MIT
