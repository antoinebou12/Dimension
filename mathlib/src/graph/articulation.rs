//! Articulation points (cut vertices) and bridges (cut edges).
//!
//! Expects the graph to be used as **undirected**: build with `add_undirected_edge`
//! or symmetric directed edges. A node is an articulation point if its removal
//! increases the number of connected components. A bridge is an edge whose removal
//! increases the number of connected components.

use crate::graph::types::{Edge, Graph, NodeId};

/// Returns the articulation points (cut vertices) of the graph when treated as undirected.
/// Uses DFS with discovery time and low-link value.
#[must_use]
pub fn articulation_points(graph: &Graph) -> Vec<NodeId> {
    fn dfs_ap(
        u: usize,
        graph: &Graph,
        disc: &mut [usize],
        low: &mut [usize],
        parent: &mut [Option<usize>],
        ap: &mut [bool],
        time: &mut usize,
    ) {
        *time += 1;
        disc[u] = *time;
        low[u] = *time;
        let mut children = 0;
        for &(v, _) in graph.neighbors(u) {
            if disc[v] == 0 {
                children += 1;
                parent[v] = Some(u);
                dfs_ap(v, graph, disc, low, parent, ap, time);
                low[u] = low[u].min(low[v]);
                if parent[u].is_none() && children > 1 {
                    ap[u] = true;
                }
                if parent[u].is_some() && low[v] >= disc[u] {
                    ap[u] = true;
                }
            } else if parent[u] != Some(v) {
                low[u] = low[u].min(disc[v]);
            }
        }
    }

    let n = graph.num_nodes();
    if n == 0 {
        return Vec::new();
    }
    let mut disc = vec![0; n];
    let mut low = vec![0; n];
    let mut parent: Vec<Option<usize>> = vec![None; n];
    let mut ap = vec![false; n];
    let mut time = 0;

    for u in 0..n {
        if disc[u] == 0 {
            dfs_ap(
                u,
                graph,
                &mut disc,
                &mut low,
                &mut parent,
                &mut ap,
                &mut time,
            );
        }
    }

    ap.into_iter()
        .enumerate()
        .filter_map(|(u, is_ap)| if is_ap { Some(u) } else { None })
        .collect()
}

/// Returns the bridges (cut edges) of the graph when treated as undirected.
/// An edge is a bridge if its removal increases the number of connected components.
/// Uses the same DFS as articulation points; tree edge (u, v) is a bridge when low[v] > disc[u].
#[must_use]
pub fn bridges(graph: &Graph) -> Vec<Edge> {
    fn dfs_bridges(
        u: usize,
        graph: &Graph,
        disc: &mut [usize],
        low: &mut [usize],
        parent: &mut [Option<usize>],
        result: &mut Vec<Edge>,
        time: &mut usize,
    ) {
        *time += 1;
        disc[u] = *time;
        low[u] = *time;
        for &(v, w) in graph.neighbors(u) {
            if disc[v] == 0 {
                parent[v] = Some(u);
                dfs_bridges(v, graph, disc, low, parent, result, time);
                low[u] = low[u].min(low[v]);
                if low[v] > disc[u] {
                    result.push(Edge { u, v, weight: w });
                }
            } else if parent[u] != Some(v) {
                low[u] = low[u].min(disc[v]);
            }
        }
    }

    let n = graph.num_nodes();
    if n == 0 {
        return Vec::new();
    }
    let mut disc = vec![0; n];
    let mut low = vec![0; n];
    let mut parent: Vec<Option<usize>> = vec![None; n];
    let mut result = Vec::new();
    let mut time = 0;

    for u in 0..n {
        if disc[u] == 0 {
            dfs_bridges(
                u,
                graph,
                &mut disc,
                &mut low,
                &mut parent,
                &mut result,
                &mut time,
            );
        }
    }
    result
}
