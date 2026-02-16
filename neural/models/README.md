# ONNX models for neural crate

This directory is populated by the download_models binary. Do not commit large `.onnx` files; add `*.onnx` to `.gitignore` if desired.

## Download

From repo root:

```bash
cargo run -p neural --bin download_models -- --all
```

Or: `just download-neural-models` (see justfile).

## Graph embedding

- **graph**: `model_quantized.onnx` from `vishnun/quantized_knowledge_graph_nlp_onnx` (Hugging Face). Used by `GraphEmbedding::from_onnx_path` when feature `embed-graph` is enabled. Input/output shapes depend on the model; see the crate docs or the model card on Hugging Face.

To use a custom GNN ONNX, place it here and pass the path to `GraphEmbedding::from_onnx_path`.
