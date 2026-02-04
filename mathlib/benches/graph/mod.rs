//! Graph benchmarks: pathfinding (Dijkstra, A*, D* Lite) and coloring.

mod coloring;
mod pathfinding;

use criterion::criterion_group;

pub fn graph_helpers_random_graph(n: usize, avg_degree: usize) -> mathlib::Graph {
    let mut g = mathlib::Graph::new(n);
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

pub fn graph_helpers_grid(rows: usize, cols: usize) -> mathlib::Graph {
    let n = rows * cols;
    let mut g = mathlib::Graph::new(n);
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

pub fn graph_helpers_random_undirected(n: usize, avg_degree: usize) -> mathlib::Graph {
    let mut g = mathlib::Graph::new(n);
    for u in 0..n {
        let k = (u * 31 + 7) % avg_degree + 1;
        for _ in 0..k {
            let v = (u * 17 + 13) % n;
            let w = 1.0 + (u as f64) * 0.01 + (v as f64) * 0.001;
            if u != v {
                g.add_edge_undirected(u, v, w);
            }
        }
    }
    g
}

criterion_group!(
    benches,
    pathfinding::bench_dijkstra,
    pathfinding::bench_astar,
    pathfinding::bench_dstar,
    coloring::bench_coloring
);
