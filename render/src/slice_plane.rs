//! Slice plane overlay: optional quad with checker pattern and lighting.

use mathlib::math3d::Matrix4f;

/// Slice plane parameters for the overlay quad.
#[derive(Clone, Debug)]
pub struct SlicePlane {
    /// Object (model) matrix for the plane (position and orientation).
    pub object_matrix: Matrix4f,
    /// Length scale for the checker pattern and lighting.
    pub length_scale: f32,
    /// Base color (RGB).
    pub color: [f32; 3],
    /// Grid line color (RGB).
    pub grid_line_color: [f32; 3],
    /// Alpha/transparency (0 = transparent, 1 = opaque).
    pub transparency: f32,
}

impl Default for SlicePlane {
    fn default() -> Self {
        Self {
            object_matrix: mathlib::cg::matrix4f_identity(),
            length_scale: 1.0,
            color: [0.9, 0.9, 0.95],
            grid_line_color: [0.7, 0.7, 0.75],
            transparency: 0.5,
        }
    }
}
