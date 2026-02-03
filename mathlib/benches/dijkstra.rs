//! Benchmarks for Dijkstra.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group};
use mathlib::{Graph, dijkstra};

fn random_graph(n: usize, avg_degree: usize) -> Graph {
    let mut g = Graph::new(n);
    for u in 0..n {
        let k = (u * 31 + 7) % avg_degree + 1;
        for _ in 0..k {
            let v = (u * 17 + 13) % n;
            let w = 1.0 + (u as f64) * 0.01 + (v as f64) * 0.001;
            if u != v {
                g.add_edge(u, v, w);
            }
        }
    }
    g
}

pub fn bench_dijkstra(c: &mut Criterion) {
    let mut group = c.benchmark_group("dijkstra");
    for n in [100, 500, 1000] {
        let g = random_graph(n, 8);
        group.bench_with_input(BenchmarkId::new("single_source", n), &g, |b, graph| {
            b.iter(|| black_box(dijkstra(graph, 0)))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_dijkstra);
