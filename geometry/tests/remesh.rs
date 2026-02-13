//! Tests for isotropic remeshing.

use geometry::{remesh_isotropic, TriMesh};

#[test]
fn remesh_isotropic_single_triangle() {
    let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
    let indices = vec![[0, 1, 2]];
    let mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
    let result = remesh_isotropic(&mesh, 0.5, 1).unwrap();
    assert!(result.num_vertices() >= 3);
    assert!(result.num_triangles() >= 1);
}

#[test]
fn remesh_isotropic_empty_error() {
    let mesh = TriMesh::new();
    let result = remesh_isotropic(&mesh, 0.5, 1);
    assert!(result.is_err());
}
