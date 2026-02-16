//! Neural inverse kinematics: train and run neural IK models for serial chains.
//!
//! This crate provides:
//! - **Chain config**: Describe a kinematic chain (DOF, workspace) for data generation.
//! - **Neural IK model**: MLP that maps target position (and optionally current joint state) → joint angles.
//! - **Dataset**: Generate (target_pos, theta) pairs using the kinematics crate (Halley IK or FK).
//! - **Training**: Burn-based training; optional wgpu backend.
//! - **ONNX**: Load and run ONNX models for inference (optional `ort`); export path documented.
//! - **WASM / wgpu**: Inference and training backends compatible with WASM and wgpu.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod chain_config;
pub mod utils;

#[cfg(feature = "train")]
pub mod dataset;
#[cfg(feature = "train")]
pub mod model;
#[cfg(feature = "train")]
pub mod training;

#[cfg(feature = "onnx")]
pub mod onnx;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(any(
    feature = "embed-text",
    feature = "embed-image",
    feature = "pointcloud",
    feature = "embed-graph",
    feature = "pdf"
))]
pub mod embed;

pub use chain_config::ChainConfig;
pub use utils::{
    denormalize_joints, denormalize_position, normalize_joints, normalize_position, JointLimits,
};

#[cfg(feature = "train")]
pub use dataset::IkDataset;
#[cfg(feature = "train")]
pub use model::{NeuralIkConfig, NeuralIkModel};
#[cfg(feature = "train")]
pub use training::train_neural_ik;

#[cfg(feature = "onnx")]
pub use onnx::OnnxIkSession;

#[cfg(feature = "wasm")]
pub use wasm::NeuralIkWasm;

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
