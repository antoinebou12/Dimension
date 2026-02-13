//! Benchmarks for geometry operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geometry::TriMesh;

fn bench_tri_mesh_compute_normals(c: &mut Criterion) {
    let positions: Vec<[f32; 3]> = (0..3000)
        .map(|i| {
            let t = i as f32 * 0.1;
            [t.cos(), t.sin(), t * 0.01]
        })
        .collect();
    let indices: Vec<[u32; 3]> = (0..1000).map(|i| [i * 3, i * 3 + 1, i * 3 + 2]).collect();
    let mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
    c.bench_function("tri_mesh_compute_normals_3k_verts", |b| {
        b.iter(|| {
            let mut m = mesh.clone();
            m.compute_normals();
            black_box(m);
        });
    });
}

criterion_group!(benches, bench_tri_mesh_compute_normals);
criterion_main!(benches);
