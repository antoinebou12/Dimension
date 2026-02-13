//! Graph and tree matrix construction and SpMV benchmarks.
//!
//! Benchmarks adjacency triplet construction, CRS/CCS build from graph, and SpMV.
//! Run with `--features gpu` or `--features simd` to exercise wgpu/SIMD paths.

use criterion::{BenchmarkId, Criterion, black_box};
use mathlib::graph::{adjacency_ccs, adjacency_crs, adjacency_triplets, tree_adjacency_crs};
use mathlib::{Graph, Tree, Vector};

use super::{graph_helpers_grid, graph_helpers_random_graph};

fn small_tree() -> Tree<()> {
    let mut g = Graph::new(64);
    for i in 0..8 {
        for j in 0..8 {
            let u = i * 8 + j;
            if j + 1 < 8 {
                g.add_edge(u, i * 8 + (j + 1), 1.0);
            }
            if i + 1 < 8 {
                g.add_edge(u, (i + 1) * 8 + j, 1.0);
            }
        }
    }
    Tree::from_bfs_spanning_tree(&g, 0)
}

pub fn bench_graph_matrix(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_matrix");

    for n in [100, 500, 2000] {
        let g = graph_helpers_random_graph(n, 8);
        group.bench_with_input(BenchmarkId::new("adjacency_triplets", n), &g, |b, graph| {
            b.iter(|| black_box(adjacency_triplets(graph)))
        });
        group.bench_with_input(BenchmarkId::new("adjacency_crs", n), &g, |b, graph| {
            b.iter(|| black_box(adjacency_crs(graph)))
        });
        group.bench_with_input(BenchmarkId::new("adjacency_ccs", n), &g, |b, graph| {
            b.iter(|| black_box(adjacency_ccs(graph)))
        });
    }

    for side in [10, 20, 40] {
        let g = graph_helpers_grid(side, side);
        let crs = adjacency_crs(&g);
        let mut x = Vector::with_capacity(g.num_nodes());
        for i in 0..g.num_nodes() {
            x.set(i, 1.0 / (i + 1) as f64);
        }
        group.bench_with_input(
            BenchmarkId::new("spmv_crs", side * side),
            (&crs, &x),
            |b, (a, v)| b.iter(|| black_box(a * v)),
        );
        let ccs = adjacency_ccs(&g);
        group.bench_with_input(
            BenchmarkId::new("spmv_ccs", side * side),
            (&ccs, &x),
            |b, (a, v)| b.iter(|| black_box(a * v)),
        );
    }

    group.finish();
}

pub fn bench_tree_matrix(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_tree_matrix");
    let t = small_tree();
    group.bench_function("tree_adjacency_crs", |b| {
        b.iter(|| black_box(tree_adjacency_crs(&t)))
    });
    let crs = tree_adjacency_crs(&t);
    let mut x = Vector::with_capacity(t.nodes.len());
    for i in 0..t.nodes.len() {
        x.set(i, 1.0 / (i + 1) as f64);
    }
    group.bench_function("tree_spmv", |b| b.iter(|| black_box(&crs * &x)));
    group.finish();
}
