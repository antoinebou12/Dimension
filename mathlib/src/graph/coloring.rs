//! Vertex coloring: bipartite check, greedy coloring, and `DSatur`.
//!
//! Graph is treated as undirected for these algorithms (neighbors in both directions).
//!
//! - **Greedy**: Process nodes 0..n, assign smallest valid color. Fast, but may use more colors.
//! - **`DSatur`**: Pick uncolored node with highest saturation (distinct colors among neighbors);
//!   assign smallest valid color. Often uses fewer colors than greedy.
//! - **Bipartite**: Returns 2-coloring if no odd cycle, `None` otherwise.

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

/// Returns a valid vertex coloring using the `DSatur` heuristic.
///
/// Repeatedly picks the uncolored node with highest *saturation* (number of distinct colors
/// among its colored neighbors); assigns the smallest valid color. Ties broken by degree
/// (higher first), then by node index.
///
/// Often yields fewer colors than [`greedy_vertex_coloring`]. Time complexity O(V²) in the
/// worst case.
///
/// # Example
///
/// ```ignore
/// let mut g = Graph::new(4);
/// g.add_edge_undirected(0, 1, 1.0);
/// g.add_edge_undirected(1, 2, 1.0);
/// g.add_edge_undirected(2, 3, 1.0);
/// g.add_edge_undirected(0, 2, 1.0);
/// let colors = dsatur_coloring(&g);
/// assert_eq!(colors.len(), 4);
/// for u in 0..4 {
///     for &(v, _) in g.neighbors(u) { assert_ne!(colors[u], colors[v]); }
/// }
/// ```
#[must_use]
pub fn dsatur_coloring(graph: &Graph) -> Vec<usize> {
    use std::collections::HashSet;
    let n = graph.num_nodes();
    debug!(num_nodes = n, "dsatur_coloring");
    let mut colors = vec![None; n];
    let mut colored_count = 0;
    while colored_count < n {
        let mut best = None; // (saturation, degree, node)
        for u in 0..n {
            if colors[u].is_some() {
                continue;
            }
            let mut used = HashSet::new();
            for &(v, _) in graph.neighbors(u) {
                if let Some(c) = colors[v] {
                    used.insert(c);
                }
            }
            for &(v, _) in graph.in_neighbors(u) {
                if let Some(c) = colors[v] {
                    used.insert(c);
                }
            }
            let saturation = used.len();
            let degree = graph.neighbors(u).len() + graph.in_neighbors(u).len();
            let candidate = (saturation, degree, u);
            if best.is_none_or(|b| candidate > b) {
                best = Some(candidate);
            }
        }
        let u = best.unwrap().2;
        let mut used = vec![false; n];
        for &(v, _) in graph.neighbors(u) {
            if let Some(c) = colors[v] {
                used[c] = true;
            }
        }
        for &(v, _) in graph.in_neighbors(u) {
            if let Some(c) = colors[v] {
                used[c] = true;
            }
        }
        let mut c = 0;
        while c < n && used[c] {
            c += 1;
        }
        colors[u] = Some(c);
        colored_count += 1;
    }
    colors.into_iter().map(|c| c.unwrap()).collect()
}
