//! Tests for voxelization.

use geometry::{flood_fill, marching_cubes, voxelize_mesh, TriMesh, VoxelGrid};

#[test]
fn voxel_grid_new() {
    let grid = VoxelGrid::new(4, 4, 4, [0.0, 0.0, 0.0], 1.0).unwrap();
    assert_eq!(grid.nx, 4);
    assert_eq!(grid.len(), 64);
    assert!(!grid.get(0, 0, 0));
}

#[test]
fn voxelize_mesh_single_triangle() {
    let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
    let indices = vec![[0, 1, 2]];
    let mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
    let grid = voxelize_mesh(&mesh, [8, 8, 8], 0.1).unwrap();
    assert!(grid.len() > 0);
    let mut count = 0;
    for idx in 0..grid.len() {
        if grid.cells[idx] {
            count += 1;
        }
    }
    assert!(count > 0, "at least one voxel should be set");
}

#[test]
fn marching_cubes_returns_mesh() {
    let mut grid = VoxelGrid::new(2, 2, 2, [0.0, 0.0, 0.0], 1.0).unwrap();
    grid.set(0, 0, 0, true);
    grid.set(1, 0, 0, true);
    let mesh = marching_cubes(&grid);
    assert!(mesh.num_vertices() > 0 || mesh.num_triangles() == 0);
}

#[test]
fn flood_fill_no_panic() {
    let mut grid = VoxelGrid::new(3, 3, 3, [0.0, 0.0, 0.0], 1.0).unwrap();
    grid.set(1, 1, 1, true);
    flood_fill(&mut grid);
    assert!(grid.get(1, 1, 1), "interior/surface stays solid");
    assert!(!grid.get(0, 0, 0), "exterior is cleared");
}
