//! Benchmark comparing parallel vs sequential performance.
//!
//! Run with: cargo bench --bench parallel_comparison
//! With parallel: cargo bench --bench parallel_comparison --features parallel
//! With full: cargo bench --bench parallel_comparison --features full

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use mathlib::prelude::*;
use mathlib::{clustering, distance, graph::*, stats};

fn bench_kmeans(c: &mut Criterion) {
    let mut group = c.benchmark_group("kmeans");

    for size in [100, 500, 1000].iter() {
        let kmeans_vec: Vec<f64> = (0..*size * 2)
            .map(|i| (i as f64 * 0.1).sin() * 10.0)
            .collect();
        let data = Matrix::from_vec(&kmeans_vec, *size, 2, Storage::Column);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let result = clustering::kmeans(black_box(&data), 5, Some(20));
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_dbscan(c: &mut Criterion) {
    let mut group = c.benchmark_group("dbscan");

    for size in [100, 200, 500].iter() {
        let dbscan_vec: Vec<f64> = (0..*size * 2)
            .map(|i| (i as f64 * 0.1).sin() * 10.0)
            .collect();
        let data = Matrix::from_vec(&dbscan_vec, *size, 2, Storage::Column);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let result = clustering::dbscan(black_box(&data), 2.0, 3);
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_dijkstra(c: &mut Criterion) {
    let mut group = c.benchmark_group("dijkstra");

    for nodes in [100, 500, 1000].iter() {
        let mut g = Graph::new(*nodes);
        // Create a connected graph with random edges
        for i in 0..*nodes - 1 {
            g.add_edge(i, i + 1, ((i * 7) % 10 + 1) as f64);
            if i % 10 == 0 && i + 10 < *nodes {
                g.add_edge(i, i + 10, ((i * 3) % 5 + 1) as f64);
            }
        }

        group.bench_with_input(BenchmarkId::from_parameter(nodes), nodes, |b, _| {
            b.iter(|| {
                let result = dijkstra(black_box(&g), 0);
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_astar(c: &mut Criterion) {
    let mut group = c.benchmark_group("astar");

    for nodes in [100, 500, 1000].iter() {
        let mut g = Graph::new(*nodes);
        for i in 0..*nodes - 1 {
            g.add_edge(i, i + 1, ((i * 7) % 10 + 1) as f64);
            if i % 10 == 0 && i + 10 < *nodes {
                g.add_edge(i, i + 10, ((i * 3) % 5 + 1) as f64);
            }
        }
        let target = *nodes - 1;

        group.bench_with_input(BenchmarkId::from_parameter(nodes), nodes, |b, _| {
            b.iter(|| {
                let result = astar(black_box(&g), 0, target, |_, _| 0.0);
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_bfs(c: &mut Criterion) {
    let mut group = c.benchmark_group("bfs");

    for nodes in [100, 500, 1000].iter() {
        let mut g = Graph::new(*nodes);
        for i in 0..*nodes - 1 {
            g.add_edge_undirected(i, i + 1, 1.0);
            if i % 10 == 0 && i + 10 < *nodes {
                g.add_edge_undirected(i, i + 10, 1.0);
            }
        }

        group.bench_with_input(BenchmarkId::from_parameter(nodes), nodes, |b, _| {
            b.iter(|| {
                let result = bfs(black_box(&g), 0);
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_covariance(c: &mut Criterion) {
    let mut group = c.benchmark_group("covariance");

    for size in [50, 100, 200].iter() {
        let cov_vec: Vec<f64> = (0..*size * *size).map(|i| (i as f64 * 0.1).sin()).collect();
        let data = Matrix::from_vec(&cov_vec, *size, *size, Storage::Column);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let result = stats::covariance(black_box(&data));
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_pso(c: &mut Criterion) {
    use mathlib::argmin::{PsoOptions, pso};
    let mut group = c.benchmark_group("pso");

    for dims in [2, 5, 10].iter() {
        let cost = |x: &[f64]| x.iter().map(|&v| v * v).sum::<f64>();
        let low = vec![-10.0; *dims];
        let high = vec![10.0; *dims];
        let bounds = (low, high);

        group.bench_with_input(BenchmarkId::from_parameter(dims), dims, |b, _| {
            b.iter(|| {
                let result = pso(
                    black_box(bounds.clone()),
                    30,
                    black_box(cost),
                    50,
                    Some(PsoOptions::default()),
                );
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_convolution(c: &mut Criterion) {
    use mathlib::transforms::conv_1d;
    let mut group = c.benchmark_group("convolution");

    for size in [100, 500, 1000].iter() {
        let signal: Vec<f64> = (0..*size).map(|i| (i as f64 * 0.1).sin()).collect();
        let kernel = vec![0.25, 0.5, 0.25];

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let result = conv_1d(black_box(&signal), black_box(&kernel));
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_vector_dot(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_dot");

    for size in [100, 1000, 10000].iter() {
        let a_vec: Vec<f64> = (0..*size).map(|i| i as f64).collect();
        let b_vec: Vec<f64> = (0..*size).map(|i| (i * 2) as f64).collect();
        let a = Vector::from_slice(&a_vec);
        let b = Vector::from_slice(&b_vec);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |ben, _| {
            ben.iter(|| {
                let result = black_box(&a).dot(black_box(&b));
                black_box(result);
            });
        });
    }
    group.finish();
}

fn bench_distance_euclidean(c: &mut Criterion) {
    let mut group = c.benchmark_group("distance_euclidean");

    for size in [100, 500, 1000].iter() {
        let dist_vec: Vec<f64> = (0..*size * 10).map(|i| i as f64).collect();
        let data = Matrix::from_vec(&dist_vec, *size, 10, Storage::Column);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let result = distance::euclidean_rows(black_box(&data), 0, *size / 2);
                black_box(result);
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_kmeans,
    bench_dbscan,
    bench_dijkstra,
    bench_astar,
    bench_bfs,
    bench_covariance,
    bench_pso,
    bench_convolution,
    bench_vector_dot,
    bench_distance_euclidean
);
criterion_main!(benches);
