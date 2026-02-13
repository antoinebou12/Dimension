//! Tests for tetrahedralization.

use geometry::{TetMesh, TriMesh};

#[test]
fn tetrahedralize_grid() {
    let (positions, mesh) = TetMesh::tetrahedralize_grid(2, 2, 2, 1.0, true);
    assert_eq!(positions.len(), 27);
    assert_eq!(mesh.num_tets(), 48);
    assert!(mesh.dm_inv.is_some());
    assert!(mesh.rest_volumes.is_some());
    let vol = mesh.rest_volumes.as_ref().unwrap()[0];
    assert!(vol > 0.0 && vol < 2.0);
}

#[test]
fn radius_ratio_and_dihedral() {
    let (positions, mesh) = TetMesh::tetrahedralize_grid(1, 1, 1, 1.0, false);
    let r = mesh.radius_ratio(&positions, 0);
    assert!(r > 0.0 && r <= 1.0);
    let a = mesh.min_dihedral_angle(&positions, 0);
    assert!(a >= 0.0 && a <= std::f32::consts::PI);
}

#[test]
fn tetrahedralize_surface_cube_like() {
    let positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.5, 1.0, 0.0],
        [0.5, 0.5, 1.0],
    ];
    let indices = vec![[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]];
    let mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
    let result = TetMesh::tetrahedralize_surface(&mesh, [4, 4, 4], 0.2);
    assert!(result.is_ok());
    let (pos, tet_mesh) = result.unwrap();
    assert!(!pos.is_empty());
    assert!(tet_mesh.num_tets() > 0);
}
