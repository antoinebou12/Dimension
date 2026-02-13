//! Example: isotropic remeshing (remesh module will fill in).

use geometry::TriMesh;

fn main() {
    let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
    let indices = vec![[0, 1, 2]];
    let mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
    println!("Input mesh: {} vertices", mesh.num_vertices());
    // When remesh module is ready: remesh_isotropic(&mesh, target_edge_length, iterations)
}
