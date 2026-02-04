//! BFS benchmarks.

use criterion::{BenchmarkId, Criterion, black_box};
use mathlib::bfs;

use super::helpers;

pub fn bench_bfs(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_bfs");
    for n in [100, 500, 1000] {
        let g = helpers::random_undirected(n, 8);
        group.bench_with_input(BenchmarkId::new("single_source", n), &g, |b, graph| {
            b.iter(|| black_box(bfs(graph, 0)))
        });
    }
    let g = helpers::grid(20, 20);
    group.bench_with_input(BenchmarkId::new("grid_20x20", 400), &g, |b, graph| {
        b.iter(|| black_box(bfs(graph, 0)))
    });
    group.finish();
}
