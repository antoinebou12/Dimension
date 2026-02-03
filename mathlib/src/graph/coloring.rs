//! Vertex coloring: bipartite check and greedy coloring.
//!
//! Graph is treated as undirected for these algorithms (neighbors in both directions).

use std::collections::VecDeque;

use crate::graph::types::Graph;
use tracing::debug;

/// Returns a 2-coloring if the graph is bipartite, or `None` if it contains an odd cycle.
/// Colors are 0 and 1. Uses BFS: alternating colors along edges; conflict implies not bipartite.
#[must_use]
pub fn is_bipartite(graph: &Graph) -> Option<Vec<usize>> {
    let n = graph.num_nodes();
    debug!(num_nodes = n, "is_bipartite");
    let mut color = vec![None; n];
    let mut queue = VecDeque::new();
    for start in 0..n {
        if color[start].is_some() {
            continue;
        }
        color[start] = Some(0);
        queue.push_back(start);
        while let Some(u) = queue.pop_front() {
            let c = color[u].unwrap();
            for &(v, _) in graph.neighbors(u) {
                match color[v] {
                    None => {
                        color[v] = Some(1 - c);
                        queue.push_back(v);
                    }
                    Some(cv) if cv == c => return None,
                    Some(_) => {}
                }
            }
        }
    }
    Some(color.into_iter().map(|c| c.unwrap_or(0)).collect())
}

/// Returns a valid vertex coloring: `colors[u]` is the color index for node `u`.
/// Greedy: process nodes 0..n, assign each the smallest color not used by already-colored neighbors.
#[must_use]
pub fn greedy_vertex_coloring(graph: &Graph) -> Vec<usize> {
    let n = graph.num_nodes();
    debug!(num_nodes = n, "greedy_vertex_coloring");
    let mut colors = vec![0; n];
    for u in 0..n {
        let mut used = vec![false; n];
        for &(v, _) in graph.neighbors(u) {
            if v < u {
                used[colors[v]] = true;
            }
        }
        let mut c = 0;
        while c < n && used[c] {
            c += 1;
        }
        colors[u] = c;
    }
    colors
}
