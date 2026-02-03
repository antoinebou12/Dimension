//! Graph benchmarks: Dijkstra, A*, D* Lite.
//!
//! Synthetic graphs (random adjacency list, grid); single source and replan timing.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group};
use mathlib::{Graph, astar, dijkstra, dstar_lite};

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

fn grid_graph(rows: usize, cols: usize) -> Graph {
    let n = rows * cols;
    let mut g = Graph::new(n);
    for r in 0..rows {
        for c in 0..cols {
            let u = r * cols + c;
            if c + 1 < cols {
                g.add_edge(u, r * cols + (c + 1), 1.0);
            }
            if c > 0 {
                g.add_edge(u, r * cols + (c - 1), 1.0);
            }
            if r + 1 < rows {
                g.add_edge(u, (r + 1) * cols + c, 1.0);
            }
            if r > 0 {
                g.add_edge(u, (r - 1) * cols + c, 1.0);
            }
        }
    }
    g
}

fn small_grid() -> Graph {
    let mut g = Graph::new(16);
    for i in 0..4 {
        for j in 0..4 {
            let u = i * 4 + j;
            if j + 1 < 4 {
                g.add_edge(u, i * 4 + (j + 1), 1.0);
            }
            if j > 0 {
                g.add_edge(u, i * 4 + (j - 1), 1.0);
            }
            if i + 1 < 4 {
                g.add_edge(u, (i + 1) * 4 + j, 1.0);
            }
            if i > 0 {
                g.add_edge(u, (i - 1) * 4 + j, 1.0);
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

pub fn bench_astar(c: &mut Criterion) {
    let mut group = c.benchmark_group("astar");
    for side in [10, 20, 30] {
        let g = grid_graph(side, side);
        let start = 0;
        let goal = g.num_nodes() - 1;
        let cols = side;
        let h = move |u: usize, g_goal: usize| {
            let ux = (u % cols) as f64;
            let uy = (u / cols) as f64;
            let gx = (g_goal % cols) as f64;
            let gy = (g_goal / cols) as f64;
            (ux - gx).abs() + (uy - gy).abs()
        };
        group.bench_with_input(
            BenchmarkId::new("grid", format!("{}x{}", side, side)),
            &g,
            |b, graph| b.iter(|| black_box(astar(graph, start, goal, h))),
        );
    }
    group.finish();
}

pub fn bench_dstar(c: &mut Criterion) {
    let mut group = c.benchmark_group("dstar");
    group.bench_function("initial_plan", |b| {
        let mut g = small_grid();
        b.iter(|| black_box(dstar_lite(&mut g, 0, 15)))
    });
    group.bench_function("replan_after_update", |b| {
        let mut g = small_grid();
        if let Some(e) = g.out_edges[1].iter_mut().find(|(v, _)| *v == 2) {
            e.1 = 10.0;
        }
        b.iter(|| black_box(dstar_lite(&mut g, 0, 15)))
    });
    group.finish();
}

criterion_group!(benches, bench_dijkstra, bench_astar, bench_dstar);
