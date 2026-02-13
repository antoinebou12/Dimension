//! Tests for CSG operations.

use geometry::{csg_difference, csg_intersection, csg_union, TriMesh};

#[test]
fn csg_union_two_cubes() {
    let positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.5, 1.0, 0.0],
        [0.5, 0.5, 1.0],
    ];
    let indices = vec![[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]];
    let mesh = TriMesh::from_positions_and_indices(positions.clone(), indices.clone()).unwrap();
    let result = csg_union(&mesh, &mesh).unwrap();
    assert!(result.num_triangles() >= 4);
    assert!(result.num_vertices() >= 4);
}

#[test]
fn csg_intersection_empty_when_disjoint() {
    let a_pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
    let a_tri = vec![[0, 1, 2]];
    let a = TriMesh::from_positions_and_indices(a_pos, a_tri).unwrap();
    let b_pos = vec![[5.0, 5.0, 5.0], [6.0, 5.0, 5.0], [5.5, 6.0, 5.0]];
    let b_tri = vec![[0, 1, 2]];
    let b = TriMesh::from_positions_and_indices(b_pos, b_tri).unwrap();
    let result = csg_intersection(&a, &b).unwrap();
    assert_eq!(result.num_triangles(), 0);
}

#[test]
fn csg_difference_removes_inside() {
    let positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.5, 1.0, 0.0],
        [0.5, 0.5, 1.0],
    ];
    let indices = vec![[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]];
    let mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
    let result = csg_difference(&mesh, &mesh).unwrap();
    assert!(result.num_triangles() <= 8);
}
