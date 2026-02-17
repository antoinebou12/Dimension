//! Embedding and vectorization: text, image, point cloud, graph, PDF.
//!
//! This crate provides optional features for:
//! - **Text embedding**: fastembed (e.g. MiniLM)
//! - **Image embedding**: fastembed (e.g. CLIP)
//! - **Graph embedding**: ONNX models for node embeddings
//! - **Point cloud embedding**: user-supplied ONNX (e.g. PointNet)
//! - **PDF**: text extraction (native only)

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

#[cfg(any(
    feature = "embed-text",
    feature = "embed-image",
    feature = "pointcloud",
    feature = "embed-graph",
    feature = "pdf"
))]
pub mod embed;

#[cfg(feature = "embed-graph")]
pub use embed::GraphEmbedding;
#[cfg(feature = "embed-image")]
pub use embed::ImageEmbedding;
#[cfg(feature = "pointcloud")]
pub use embed::PointCloudEmbedding;
#[cfg(feature = "embed-text")]
pub use embed::TextEmbedding;
#[cfg(all(feature = "pdf", not(target_arch = "wasm32")))]
pub use embed::{extract_text, extract_text_pages};
