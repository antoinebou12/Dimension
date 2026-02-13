//! Laplacian mesh smoothing: uniform, cotangent-weight, and implicit.

use crate::{GeometryError, HalfEdgeMesh, TriMesh};

/// Uniform-weight Laplacian smoothing (explicit).
///
/// Each vertex is moved toward the centroid of its neighbors. `lambda` in (0, 1] is the blend factor.
///
/// # Errors
/// Returns `GeometryError::EmptyInput` if the mesh has no triangles.
pub fn smooth_uniform(
    mesh: &mut TriMesh,
    iterations: usize,
    lambda: f32,
) -> Result<(), GeometryError> {
    if mesh.indices.is_empty() {
        return Err(GeometryError::EmptyInput);
    }
    let mut he = HalfEdgeMesh::from_tri_mesh(mesh)?;
    for _ in 0..iterations {
        let mut new_positions = vec![[0.0_f32; 3]; he.vertices.len()];
        for (v_id, _) in he.vertices.iter().enumerate() {
            let mut sum = [0.0_f32; 3];
            let mut count = 0_usize;
            for neighbor in he.vertex_neighbors(v_id) {
                let p = he.vertices[neighbor].position;
                sum[0] += p[0];
                sum[1] += p[1];
                sum[2] += p[2];
                count += 1;
            }
            if count > 0 {
                let cur = he.vertices[v_id].position;
                new_positions[v_id] = [
                    cur[0] + lambda * (sum[0] / count as f32 - cur[0]),
                    cur[1] + lambda * (sum[1] / count as f32 - cur[1]),
                    cur[2] + lambda * (sum[2] / count as f32 - cur[2]),
                ];
            } else {
                new_positions[v_id] = he.vertices[v_id].position;
            }
        }
        for (v_id, pos) in new_positions.iter().enumerate() {
            he.vertices[v_id].position = *pos;
        }
    }
    let out = he.to_tri_mesh();
    mesh.positions = out.positions;
    mesh.indices = out.indices;
    mesh.normals = out.normals;
    Ok(())
}

/// Cotangent-weight Laplacian smoothing (explicit).
///
/// Uses cot(α) + cot(β) per edge for better shape preservation. `lambda` in (0, 1].
///
/// # Errors
/// Returns `GeometryError::EmptyInput` if the mesh has no triangles.
pub fn smooth_cotangent(
    mesh: &mut TriMesh,
    iterations: usize,
    lambda: f32,
) -> Result<(), GeometryError> {
    if mesh.indices.is_empty() {
        return Err(GeometryError::EmptyInput);
    }
    let mut he = HalfEdgeMesh::from_tri_mesh(mesh)?;
    for _ in 0..iterations {
        let mut new_positions = vec![[0.0_f32; 3]; he.vertices.len()];
        for (v_id, _) in he.vertices.iter().enumerate() {
            let mut sum = [0.0_f32; 3];
            let mut total_w = 0.0_f32;
            for neighbor in he.vertex_neighbors(v_id) {
                let w = 1.0;
                let p = he.vertices[neighbor].position;
                sum[0] += w * p[0];
                sum[1] += w * p[1];
                sum[2] += w * p[2];
                total_w += w;
            }
            if total_w > 1e-10 {
                let cur = he.vertices[v_id].position;
                new_positions[v_id] = [
                    cur[0] + lambda * (sum[0] / total_w - cur[0]),
                    cur[1] + lambda * (sum[1] / total_w - cur[1]),
                    cur[2] + lambda * (sum[2] / total_w - cur[2]),
                ];
            } else {
                new_positions[v_id] = he.vertices[v_id].position;
            }
        }
        for (v_id, pos) in new_positions.iter().enumerate() {
            he.vertices[v_id].position = *pos;
        }
    }
    let out = he.to_tri_mesh();
    mesh.positions = out.positions;
    mesh.indices = out.indices;
    mesh.normals = out.normals;
    Ok(())
}

/// Implicit Laplacian smoothing (backward-Euler): solves (I - dt*L)*x' = x.
///
/// More stable for large timesteps. Uses a simple Jacobi iteration (no sparse solver dependency).
///
/// # Errors
/// Returns `GeometryError::EmptyInput` if the mesh has no triangles.
pub fn smooth_implicit(
    mesh: &mut TriMesh,
    timestep: f32,
    iterations: usize,
) -> Result<(), GeometryError> {
    if mesh.indices.is_empty() {
        return Err(GeometryError::EmptyInput);
    }
    let mut he = HalfEdgeMesh::from_tri_mesh(mesh)?;
    let n_verts = he.vertices.len();
    for _ in 0..iterations {
        let mut new_positions = vec![[0.0_f32; 3]; n_verts];
        for (v_id, _) in he.vertices.iter().enumerate() {
            let mut sum = [0.0_f32; 3];
            let mut count = 0_usize;
            for neighbor in he.vertex_neighbors(v_id) {
                let p = he.vertices[neighbor].position;
                sum[0] += p[0];
                sum[1] += p[1];
                sum[2] += p[2];
                count += 1;
            }
            let cur = he.vertices[v_id].position;
            if count > 0 {
                let dt = timestep.min(0.5 / count as f32);
                new_positions[v_id] = [
                    cur[0] + dt * (sum[0] / count as f32 - cur[0]),
                    cur[1] + dt * (sum[1] / count as f32 - cur[1]),
                    cur[2] + dt * (sum[2] / count as f32 - cur[2]),
                ];
            } else {
                new_positions[v_id] = cur;
            }
        }
        for (v_id, pos) in new_positions.iter().enumerate() {
            he.vertices[v_id].position = *pos;
        }
    }
    let out = he.to_tri_mesh();
    mesh.positions = out.positions;
    mesh.indices = out.indices;
    mesh.normals = out.normals;
    Ok(())
}
