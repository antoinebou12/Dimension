//! WebAssembly bindings for geometry operations.
//!
//! Enable with `--features wasm`. Exposes voxelize, marching cubes, CSG union,
//! isotropic remeshing, and Laplacian smoothing to JavaScript.

use crate::{
    csg_union, flood_fill, marching_cubes, remesh_isotropic, smooth_uniform, voxelize_mesh, TriMesh,
};
use wasm_bindgen::prelude::*;

/// Result mesh for JS: positions (xyz per vertex) and indices (3 per triangle).
#[wasm_bindgen]
pub struct WasmTriMesh {
    positions: Vec<f32>,
    indices: Vec<u32>,
}

#[wasm_bindgen]
impl WasmTriMesh {
    /// Flat positions (x,y,z per vertex).
    #[wasm_bindgen(getter)]
    pub fn positions(&self) -> Vec<f32> {
        self.positions.clone()
    }

    /// Flat triangle indices (3 per face).
    #[wasm_bindgen(getter)]
    pub fn indices(&self) -> Vec<u32> {
        self.indices.clone()
    }
}

fn tri_mesh_from_flat(positions: &[f32], indices: &[u32]) -> Result<TriMesh, JsError> {
    if positions.len() % 3 != 0 {
        return Err(JsError::new("positions length must be multiple of 3"));
    }
    if indices.len() % 3 != 0 {
        return Err(JsError::new("indices length must be multiple of 3"));
    }
    let n = positions.len() / 3;
    let mut pos_vec = Vec::with_capacity(n);
    for i in 0..n {
        pos_vec.push([positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]]);
    }
    let mut idx_vec = Vec::with_capacity(indices.len() / 3);
    for i in (0..indices.len()).step_by(3) {
        idx_vec.push([indices[i], indices[i + 1], indices[i + 2]]);
    }
    TriMesh::from_positions_and_indices(pos_vec, idx_vec).map_err(|e| JsError::new(&e.to_string()))
}

fn tri_mesh_to_wasm(m: TriMesh) -> WasmTriMesh {
    let positions: Vec<f32> = m
        .positions
        .iter()
        .flat_map(|p| [p[0], p[1], p[2]])
        .collect();
    let indices: Vec<u32> = m.indices.iter().flat_map(|t| [t[0], t[1], t[2]]).collect();
    WasmTriMesh { positions, indices }
}

/// Voxelizes a mesh and returns the isosurface as a triangle mesh.
#[wasm_bindgen(js_name = voxelizeAndExtract)]
pub fn voxelize_and_extract(
    positions: &[f32],
    indices: &[u32],
    res_x: usize,
    res_y: usize,
    res_z: usize,
    padding: f32,
    do_flood_fill: bool,
) -> Result<WasmTriMesh, JsError> {
    let mesh = tri_mesh_from_flat(positions, indices)?;
    let resolution = [res_x, res_y, res_z];
    let mut grid =
        voxelize_mesh(&mesh, resolution, padding).map_err(|e| JsError::new(&e.to_string()))?;
    if do_flood_fill {
        flood_fill(&mut grid);
    }
    let out = marching_cubes(&grid);
    Ok(tri_mesh_to_wasm(out))
}

/// CSG union of two meshes.
#[wasm_bindgen(js_name = csgUnion)]
pub fn wasm_csg_union(
    a_positions: &[f32],
    a_indices: &[u32],
    b_positions: &[f32],
    b_indices: &[u32],
) -> Result<WasmTriMesh, JsError> {
    let a = tri_mesh_from_flat(a_positions, a_indices)?;
    let b = tri_mesh_from_flat(b_positions, b_indices)?;
    let out = csg_union(&a, &b).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(tri_mesh_to_wasm(out))
}

/// Isotropic remeshing.
#[wasm_bindgen(js_name = remeshIsotropic)]
pub fn wasm_remesh_isotropic(
    positions: &[f32],
    indices: &[u32],
    target_edge_length: f32,
    iterations: usize,
) -> Result<WasmTriMesh, JsError> {
    let mesh = tri_mesh_from_flat(positions, indices)?;
    let out = remesh_isotropic(&mesh, target_edge_length, iterations)
        .map_err(|e| JsError::new(&e.to_string()))?;
    Ok(tri_mesh_to_wasm(out))
}

/// Uniform Laplacian smoothing (in-place; returns new mesh).
#[wasm_bindgen(js_name = smoothUniform)]
pub fn wasm_smooth_uniform(
    positions: &[f32],
    indices: &[u32],
    iterations: usize,
    lambda: f32,
) -> Result<WasmTriMesh, JsError> {
    let mut mesh = tri_mesh_from_flat(positions, indices)?;
    smooth_uniform(&mut mesh, iterations, lambda).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(tri_mesh_to_wasm(mesh))
}
