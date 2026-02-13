//! Grid cube overlay: instanced cubes per cell (no geometry shader).

use crate::backend::{GridCubeInstance, GridCubeVertex};

/// Descriptor for a 3D grid of cubes (e.g. volume visualization).
#[derive(Clone, Debug)]
pub struct GridCubeDescriptor {
    /// Resolution (nx, ny, nz).
    pub resolution: [u32; 3],
    /// Cell spacing in world units (sx, sy, sz).
    pub spacing: [f32; 3],
    /// Cube size factor in (0, 1]; 1 = no gap between cells.
    pub cube_size_factor: f32,
}

impl Default for GridCubeDescriptor {
    fn default() -> Self {
        Self {
            resolution: [4, 4, 4],
            spacing: [0.25, 0.25, 0.25],
            cube_size_factor: 0.95,
        }
    }
}

/// Unit cube mesh (positions in [-1, 1]^3) for instanced grid drawing.
#[must_use]
pub fn unit_cube_mesh() -> (Vec<GridCubeVertex>, Vec<u16>) {
    let positions: [[f32; 3]; 8] = [
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
        [1.0, 1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0],
    ];
    let faces: [(u16, u16, u16, u16); 6] = [
        (0, 1, 2, 3),
        (5, 4, 7, 6),
        (4, 0, 3, 7),
        (1, 5, 6, 2),
        (3, 2, 6, 7),
        (4, 5, 1, 0),
    ];
    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);
    for (a, b, c, d) in faces {
        let base = vertices.len() as u16;
        for &vi in &[a, b, c, d] {
            vertices.push(GridCubeVertex {
                position: positions[vi as usize],
            });
        }
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    (vertices, indices)
}

/// Build instance data for drawing the grid cube overlay.
#[must_use]
pub fn build_grid_cube_instances(descriptor: &GridCubeDescriptor) -> Vec<GridCubeInstance> {
    let [nx, ny, nz] = descriptor.resolution;
    let [sx, sy, sz] = descriptor.spacing;
    let mut instances = Vec::with_capacity((nx * ny * nz) as usize);
    for ix in 0..nx {
        for it in 0..ny {
            for iz in 0..nz {
                let cx = (ix as f32 + 0.5) * sx;
                let cy = (it as f32 + 0.5) * sy;
                let cz = (iz as f32 + 0.5) * sz;
                instances.push(GridCubeInstance {
                    cell_position: [cx, cy, cz],
                    _pad: 0.0,
                    cell_index: [ix, it, iz],
                    _pad2: 0,
                });
            }
        }
    }
    instances
}
