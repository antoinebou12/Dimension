//! DFS benchmarks.

use criterion::{BenchmarkId, Criterion, black_box};
use mathlib::{dfs_postorder, dfs_preorder};

use super::helpers;

pub fn bench_dfs(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_dfs");
    for n in [100, 500, 1000] {
        let g = helpers::random_undirected(n, 8);
        group.bench_with_input(BenchmarkId::new("preorder", n), &g, |b, graph| {
            b.iter(|| black_box(dfs_preorder(graph, 0)))
        });
        group.bench_with_input(BenchmarkId::new("postorder", n), &g, |b, graph| {
            b.iter(|| black_box(dfs_postorder(graph, 0)))
        });
    }
    group.finish();
}
