# neural

The **neural** crate provides embedding and vectorization for text, image, point cloud, graph, and PDF.

## Capabilities

- **Text embedding**: MiniLM and other models via the `embed-text` feature (fastembed).
- **Image embedding**: CLIP and other models via the `embed-image` feature (fastembed).
- **Point cloud**: Run a user-supplied ONNX model (e.g. PointNet) with the `pointcloud` feature.
- **Graph**: Run a graph-embedding ONNX model (e.g. from Hugging Face) with the `embed-graph` feature; models are downloaded into the repo via the `download_models` binary (`cargo run -p neural --bin download_models -- --all`).
- **PDF**: Extract text from PDF bytes with the `pdf` feature (native only).

See [neural/README.md](../neural/README.md) for usage and features.

## Embedding demo on the website

Build and serve the unified website with `just website-build` and `just website-serve`. See [website.md](website.md) for full instructions.

The **embedding demo** is available at `/neural/embedding/` and links to the mathlib Recommendation demo at `/mathlib/recommendation/` for the interactive 3D scatter of text embeddings. Regenerate embeddings with the neural example `precompute_demo_embeddings` (precompute feature) and write output to `mathlib/demo/wasm-demo/recommendation/data.json`.
