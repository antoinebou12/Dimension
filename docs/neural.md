# neural

The **neural** crate provides neural inverse kinematics (train and run IK models) and optional **embedding / vectorization** for text, image, point cloud, graph, and PDF.

## Capabilities

- **Neural IK**: Chain config, Burn-based training, ONNX inference; WASM bindings for browser.
- **Text embedding**: MiniLM and other models via the `embed-text` feature (fastembed).
- **Image embedding**: CLIP and other models via the `embed-image` feature (fastembed).
- **Point cloud**: Run a user-supplied ONNX model (e.g. PointNet) with the `pointcloud` feature.
- **Graph**: Run a graph-embedding ONNX model (e.g. from Hugging Face) with the `embed-graph` feature; models are downloaded into the repo via the `download_models` binary (`cargo run -p neural --bin download_models -- --all`).
- **PDF**: Extract text from PDF bytes with the `pdf` feature (native only if pdf-extract does not build for wasm32).

See [neural/README.md](../neural/README.md) for usage, features, and the download binary.

## WASM demos on the website

Build and serve the unified website with `just website-build` and `just website-serve`. See [website.md](website.md) for full instructions.

### Embedding demo

The **embedding demo** is available as a top-level entry on the hub at `/neural/embedding/`. It shows text embeddings in 3D (PCA); click points or pick a query index to see similar documents. Data is shared with the mathlib **Recommendation** demo at `/mathlib/recommendation/`. Regenerate embeddings with the neural example `precompute_demo_embeddings` (precompute feature) and write output to `mathlib/demo/wasm-demo/recommendation/data.json`.

### Neural IK (kinematics)

The **Kinematics** demo (`/kinematics/`) can be built with a **Neural** solver option when built with `just build-kinematics-demo-wasm-neural` (requires an ONNX model at `neural/iknet.onnx` or `NEURAL_IK_ONNX`). The standard `website-build` uses the kinematics demo without the neural feature so the solver dropdown shows FABRIK, Halley, etc.; use the neural build for the Neural IK option.
