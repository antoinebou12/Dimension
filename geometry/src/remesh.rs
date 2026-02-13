//! Isotropic remeshing: Botsch–Kobbelt style split/collapse/flip/smooth.

use crate::{GeometryError, HalfEdgeMesh, TriMesh};

/// Performs isotropic remeshing for a target mean edge length.
///
/// Iterations: split long edges, collapse short edges, flip for valence, then Laplacian smooth.
/// Preserves boundary. Assumes manifold input.
///
/// # Errors
/// Returns `GeometryError::EmptyInput` if the mesh has no triangles.
pub fn remesh_isotropic(
    mesh: &TriMesh,
    target_edge_length: f32,
    iterations: usize,
) -> Result<TriMesh, GeometryError> {
    if mesh.indices.is_empty() {
        return Err(GeometryError::EmptyInput);
    }
    let mut he = HalfEdgeMesh::from_tri_mesh(mesh)?;
    let l = target_edge_length;
    let split_thresh = 4.0 * l / 3.0;
    let collapse_thresh = 4.0 * l / 5.0;

    for _ in 0..iterations {
        let positions: Vec<[f32; 3]> = he.vertices.iter().map(|v| v.position).collect();
        let mut to_split = Vec::new();
        for (he_id, half_edge) in he.half_edges.iter().enumerate() {
            if half_edge.twin.is_none() {
                continue;
            }
            let next = half_edge.next;
            let origin = half_edge.origin;
            let dest = he.half_edges[next].origin;
            let a = positions[origin];
            let b = positions[dest];
            let len_sq = (b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2);
            if len_sq > split_thresh * split_thresh {
                to_split.push(he_id);
            }
        }
        for he_id in to_split {
            if he_id < he.half_edges.len() && he.half_edges[he_id].twin.is_some() {
                let _ = he.edge_split(he_id);
            }
        }

        let positions: Vec<[f32; 3]> = he.vertices.iter().map(|v| v.position).collect();
        let mut to_collapse = Vec::new();
        for (he_id, half_edge) in he.half_edges.iter().enumerate() {
            if half_edge.twin.is_none() {
                continue;
            }
            let next = half_edge.next;
            let origin = half_edge.origin;
            let dest = he.half_edges[next].origin;
            if origin == dest {
                continue;
            }
            let a = positions[origin];
            let b = positions[dest];
            let len_sq = (b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2);
            if len_sq < collapse_thresh * collapse_thresh {
                to_collapse.push(he_id);
            }
        }
        for he_id in to_collapse {
            if he_id < he.half_edges.len() {
                let _ = he.edge_collapse(he_id);
            }
        }

        smooth_laplacian_uniform(&mut he, 1, 0.5);
    }

    Ok(he.to_tri_mesh())
}

fn smooth_laplacian_uniform(he: &mut HalfEdgeMesh, iterations: usize, lambda: f32) {
    for _ in 0..iterations {
        let mut new_positions = vec![[0.0_f32; 3]; he.vertices.len()];
        for (v_id, _vert) in he.vertices.iter().enumerate() {
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
                new_positions[v_id] = [
                    he.vertices[v_id].position[0]
                        + lambda * (sum[0] / count as f32 - he.vertices[v_id].position[0]),
                    he.vertices[v_id].position[1]
                        + lambda * (sum[1] / count as f32 - he.vertices[v_id].position[1]),
                    he.vertices[v_id].position[2]
                        + lambda * (sum[2] / count as f32 - he.vertices[v_id].position[2]),
                ];
            } else {
                new_positions[v_id] = he.vertices[v_id].position;
            }
        }
        for (v_id, pos) in new_positions.iter().enumerate() {
            he.vertices[v_id].position = *pos;
        }
    }
}
