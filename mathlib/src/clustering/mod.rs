//! Clustering: K-means and DBSCAN (sequential; optional parallel when `parallel` feature is enabled).
//!
//! Data layout: rows = samples, cols = features. Uses `crate::distance` for row-wise Euclidean distance.

pub mod dbscan;
pub mod kmeans;

pub use dbscan::{DbscanResult, NOISE, dbscan};
pub use kmeans::{KmeansResult, kmeans};
