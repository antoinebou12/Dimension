//! Shared graph helpers for tree benchmarks.

use mathlib::Graph;

/// Random undirected graph with `n` nodes and approx `avg_degree` edges per node.
pub fn random_undirected(n: usize, avg_degree: usize) -> Graph {
    let mut g = Graph::new(n);
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

/// Grid graph (undirected) with `rows` x `cols` nodes.
pub fn grid(rows: usize, cols: usize) -> Graph {
    let n = rows * cols;
    let mut g = Graph::new(n);
    for r in 0..rows {
        for c in 0..cols {
            let u = r * cols + c;
            if c + 1 < cols {
                g.add_edge_undirected(u, r * cols + (c + 1), 1.0);
            }
            if c > 0 {
                g.add_edge_undirected(u, r * cols + (c - 1), 1.0);
            }
            if r + 1 < rows {
                g.add_edge_undirected(u, (r + 1) * cols + c, 1.0);
            }
            if r > 0 {
                g.add_edge_undirected(u, (r - 1) * cols + c, 1.0);
            }
        }
    }
    g
}
