//! Example: CSG union of two shapes (csg module will fill in).

use geometry::TriMesh;

fn main() {
    let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
    let indices = vec![[0, 1, 2]];
    let mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
    println!("Mesh A: {} triangles", mesh.num_triangles());
    // When csg module is ready: csg_union(&mesh_a, &mesh_b)
}
