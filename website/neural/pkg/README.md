# neural

Neural inverse kinematics: train and run neural IK models for serial chains. Chain-dependent on **N** joint positions in world space; usable from **WASM** and **wgpu**.

## Features

- **Chain config**: Describe DOF, workspace bounds, and joint limits for data generation.
- **Neural IK model**: MLP mapping target position (and optionally current joint state) → joint angles.
- **Dataset**: Generate `(target_pos, theta)` pairs using the `kinematics` crate (Halley IK).
- **Training**: [Burn](https://burn.dev) with NdArray (CPU) or wgpu backend; MSE loss.
- **ONNX**: Load and run ONNX models for inference (optional `ort`); export Burn models to ONNX via Burn’s export or external tools.
- **WASM**: Inference bindings for browser; train on host, run in WASM with ONNX or Burn.

## Usage

### Train (native)

```bash
cargo run -p neural --bin train --features train
```

This builds a small 3-DOF 2D arm, generates 2000 IK samples with Halley IK, and trains an MLP for a few epochs.

### Library

- **Without training**: use `ChainConfig`, `normalize_position`, `denormalize_joints` for data prep; use ONNX or your own model for inference.
- **With `train`**: use `NeuralIkConfig`, `NeuralIkModel`, `IkDataset::generate_with_halley`, `train_neural_ik`.
- **With `onnx`**: use `OnnxIkSession::load_path("model.onnx")` and `predict(&[x,y,z])`.
- **With `wasm`**: use `NeuralIkWasm` for browser-side input buffers and integration with your inference path.

### Export to ONNX

Train with Burn, then export the model to ONNX (e.g. via Burn’s export story or a small Python bridge that loads Burn weights and exports with `torch.onnx.export`). Use the `onnx` feature to load and run the resulting `.onnx` file in Rust or WASM.

## Crate layout

| Module      | Description |
|------------|-------------|
| `chain_config` | Chain DOF, workspace, joint limits. |
| `utils`    | Normalize/denormalize positions and joint angles. |
| `model`    | Neural IK MLP (Burn). |
| `dataset`  | IK dataset and batcher; `generate_with_halley` (native). |
| `training` | `train_neural_ik` loop. |
| `onnx`     | ONNX load/predict (optional `ort`). |
| `wasm`     | WASM bindings for inference. |
| `embed`    | Text, image, point cloud, graph embeddings; PDF text extraction (optional features). |

## Embedding and vectorization

With optional features the crate can:

- **Text** (`embed-text`): `TextEmbedding::try_new_minilm()` and `embed(&[&str])` via [fastembed](https://crates.io/crates/fastembed).
- **Image** (`embed-image`): `ImageEmbedding::try_new_default()` and `embed_paths` / `embed_bytes` via fastembed.
- **Point cloud** (`pointcloud`, requires `onnx`): `PointCloudEmbedding::from_onnx_path(path)` — run a user-supplied ONNX model (e.g. PointNet) on N×C points.
- **Graph** (`embed-graph`, requires `onnx`): `GraphEmbedding::from_onnx_path(path)` — run ONNX on node features for node embeddings. A model can be downloaded into `neural/models/` via the download script.
- **PDF** (`pdf`): `extract_text(bytes)` and `extract_text_pages(bytes)` via [pdf-extract](https://crates.io/crates/pdf-extract) (native only; not built for wasm32 if the dependency fails there).

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

## Features

- `train` (default): Burn + kinematics for training and dataset generation.
- `onnx`: ONNX inference via `ort`.
- `wasm`: wasm-bindgen for browser.
- `wgpu`: Use wgpu backend for training (e.g. `--features "train wgpu"`).
- `embed-text`: Text embeddings (fastembed).
- `embed-image`: Image embeddings (fastembed).
- `pdf`: PDF text extraction (pdf-extract).
- `pointcloud`: Point cloud ONNX runner (depends on `onnx`).
- `embed-graph`: Graph ONNX runner (depends on `onnx`).
- `precompute`: Embed-text + serde_json for the precompute_demo_embeddings example.

## License

MIT
