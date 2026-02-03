//! Benchmarks for A*.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group};
use mathlib::{Graph, astar};

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

criterion_group!(benches, bench_astar);
