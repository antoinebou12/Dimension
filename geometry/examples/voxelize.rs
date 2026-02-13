//! Example: create a simple mesh and show voxelization pipeline (voxel module will fill in).

use geometry::TriMesh;

fn main() {
    let positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.5, 1.0, 0.0],
        [0.5, 0.5, 1.0],
    ];
    let indices = vec![[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]];
    let mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
    println!(
        "Mesh: {} vertices, {} triangles",
        mesh.num_vertices(),
        mesh.num_triangles()
    );
    // When voxel module is ready: voxelize_mesh(&mesh, resolution), flood_fill(), marching_cubes()
}
