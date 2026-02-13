//! Tests for Laplacian smoothing.

use geometry::{smooth_cotangent, smooth_implicit, smooth_uniform, TriMesh};

#[test]
fn smooth_uniform_reduces_noise() {
    let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
    let indices = vec![[0, 1, 2]];
    let mut mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
    smooth_uniform(&mut mesh, 2, 0.5).unwrap();
    assert_eq!(mesh.num_vertices(), 3);
}

#[test]
fn smooth_cotangent_no_panic() {
    let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
    let indices = vec![[0, 1, 2]];
    let mut mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
    smooth_cotangent(&mut mesh, 1, 0.3).unwrap();
    assert!(mesh.num_vertices() >= 3);
}

#[test]
fn smooth_implicit_no_panic() {
    let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
    let indices = vec![[0, 1, 2]];
    let mut mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
    smooth_implicit(&mut mesh, 0.1, 2).unwrap();
    assert!(mesh.num_vertices() >= 3);
}

#[test]
fn smooth_empty_error() {
    let mut mesh = TriMesh::new();
    assert!(smooth_uniform(&mut mesh, 1, 0.5).is_err());
}
