//! Coloring benchmarks: greedy, DSatur.

use criterion::{BenchmarkId, Criterion};
use mathlib::{dsatur_coloring, greedy_vertex_coloring};
use std::hint::black_box;

use super::graph_helpers_random_undirected;

pub fn bench_coloring(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_coloring");
    for n in [50, 100, 200] {
        let g = graph_helpers_random_undirected(n, 8);
        group.bench_with_input(BenchmarkId::new("greedy", n), &g, |b, graph| {
            b.iter(|| black_box(greedy_vertex_coloring(graph)))
        });
        group.bench_with_input(BenchmarkId::new("dsatur", n), &g, |b, graph| {
            b.iter(|| black_box(dsatur_coloring(graph)))
        });
    }
    group.finish();
}
