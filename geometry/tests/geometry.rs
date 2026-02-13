//! Integration tests for geometry crate.

use geometry::{GeometryError, TriMesh};

#[test]
fn tri_mesh_from_positions_and_indices() {
    let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    let indices = vec![[0, 1, 2]];
    let mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
    assert_eq!(mesh.num_vertices(), 3);
    assert_eq!(mesh.num_triangles(), 1);
}

#[test]
fn tri_mesh_empty_positions_error() {
    let result = TriMesh::from_positions_and_indices(vec![], vec![[0, 1, 2]]);
    assert!(matches!(result, Err(GeometryError::EmptyInput)));
}

#[test]
fn tri_mesh_index_out_of_bounds_error() {
    let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
    let indices = vec![[0, 1, 5]]; // 5 out of bounds
    let result = TriMesh::from_positions_and_indices(positions, indices);
    assert!(matches!(result, Err(GeometryError::InvalidTopology(_))));
}

#[test]
fn tri_mesh_compute_normals() {
    let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    let indices = vec![[0, 1, 2]];
    let mut mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
    mesh.compute_normals();
    assert!(mesh.normals.is_some());
    let normals = mesh.normals.as_ref().unwrap();
    assert_eq!(normals.len(), 3);
    // Face normal should be (0, 0, 1) for CCW triangle in xy plane
    assert!((normals[0][2] - 1.0).abs() < 1e-5);
}

#[test]
fn tri_mesh_to_render_vertices() {
    let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    let indices = vec![[0, 1, 2]];
    let mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
    let (pos, uv, color, idx) = mesh.to_render_vertices();
    assert_eq!(pos.len(), 3);
    assert_eq!(uv.len(), 3);
    assert_eq!(color.len(), 3);
    assert_eq!(idx.len(), 3);
}
