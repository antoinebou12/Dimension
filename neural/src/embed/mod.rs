//! Embedding and vectorization: text, image, point cloud, graph, PDF.
//!
//! Modules are gated by features: `embed-text`, `embed-image`, `pointcloud`, `embed-graph`, `pdf`.

#[cfg(feature = "embed-graph")]
pub mod graph;
#[cfg(feature = "embed-image")]
pub mod image;
#[cfg(all(feature = "pdf", not(target_arch = "wasm32")))]
pub mod pdf;
#[cfg(feature = "pointcloud")]
pub mod pointcloud;
#[cfg(feature = "embed-text")]
pub mod text;

#[cfg(feature = "embed-graph")]
pub use graph::GraphEmbedding;
#[cfg(feature = "embed-image")]
pub use image::ImageEmbedding;
#[cfg(all(feature = "pdf", not(target_arch = "wasm32")))]
pub use pdf::{extract_text, extract_text_pages};
#[cfg(feature = "pointcloud")]
pub use pointcloud::PointCloudEmbedding;
#[cfg(feature = "embed-text")]
pub use text::TextEmbedding;
