//! Error types for geometry operations.

use std::fmt;

/// Errors that can occur during geometry processing.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum GeometryError {
    /// Mesh has degenerate triangles or invalid geometry.
    DegenerateMesh(String),
    /// Input is empty where data was expected.
    EmptyInput,
    /// Mesh is not manifold (e.g. non-manifold edges or vertices).
    NonManifold(String),
    /// Invalid topology (e.g. broken half-edge connectivity).
    InvalidTopology(String),
    /// Voxel grid dimensions exceed the allowed maximum.
    VoxelGridTooLarge {
        /// Requested total number of voxels.
        requested: usize,
        /// Maximum allowed.
        max: usize,
    },
    /// Tetrahedralization failed to meet quality threshold.
    TetQualityFailed {
        /// Minimum dihedral angle achieved.
        min_angle: f32,
    },
}

impl fmt::Display for GeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeometryError::DegenerateMesh(msg) => write!(f, "degenerate mesh: {}", msg),
            GeometryError::EmptyInput => write!(f, "empty input"),
            GeometryError::NonManifold(msg) => write!(f, "non-manifold: {}", msg),
            GeometryError::InvalidTopology(msg) => write!(f, "invalid topology: {}", msg),
            GeometryError::VoxelGridTooLarge { requested, max } => {
                write!(
                    f,
                    "voxel grid too large: requested {} cells, max {}",
                    requested, max
                )
            }
            GeometryError::TetQualityFailed { min_angle } => {
                write!(
                    f,
                    "tetrahedralization quality failed: min_angle {}",
                    min_angle
                )
            }
        }
    }
}

impl std::error::Error for GeometryError {}
