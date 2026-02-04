//! Breadth-first search (BFS) for graph traversal.
//!
//! Treats the graph as undirected: traverses both outgoing and incoming edges.
//! Time complexity: O(V + E).
//!
//! With the `parallel` feature (not on wasm32), uses level-synchronous parallel BFS
//! for large graphs; otherwise sequential.

#[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
use std::collections::VecDeque;

use crate::graph::types::Graph;

/// Result of BFS: visit order and depth (hop count from source) per node.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BfsResult {
    /// Visit order: nodes in the order they were first discovered.
    pub order: Vec<usize>,
    /// Depth from source: 0 for source, 1 for its neighbors, etc. Unreachable = `usize::MAX`.
    pub depth: Vec<usize>,
}

/// Runs BFS from `source` on `graph`, treating it as undirected.
#[must_use]
pub fn bfs(graph: &Graph, source: usize) -> BfsResult {
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    return bfs_parallel(graph, source);
    #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
    bfs_sequential(graph, source)
}

#[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
fn bfs_sequential(graph: &Graph, source: usize) -> BfsResult {
    let n = graph.num_nodes();
    if source >= n {
        return BfsResult {
            order: vec![],
            depth: vec![usize::MAX; n],
        };
    }
    let mut depth = vec![usize::MAX; n];
    let mut order = Vec::with_capacity(n);
    let mut queue = VecDeque::new();
    depth[source] = 0;
    order.push(source);
    queue.push_back(source);
    while let Some(u) = queue.pop_front() {
        let d = depth[u];
        for &(v, _) in graph.neighbors(u) {
            if depth[v] == usize::MAX {
                depth[v] = d + 1;
                order.push(v);
                queue.push_back(v);
            }
        }
        for &(v, _) in graph.in_neighbors(u) {
            if depth[v] == usize::MAX {
                depth[v] = d + 1;
                order.push(v);
                queue.push_back(v);
            }
        }
    }
    BfsResult { order, depth }
}

#[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
fn bfs_parallel(graph: &Graph, source: usize) -> BfsResult {
    use par_iter::prelude::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    let n = graph.num_nodes();
    if source >= n {
        return BfsResult {
            order: vec![],
            depth: vec![usize::MAX; n],
        };
    }
    let depth: Vec<AtomicUsize> = (0..n).map(|_| AtomicUsize::new(usize::MAX)).collect();
    depth[source].store(0, Ordering::Relaxed);
    let mut order = vec![source];
    let mut current: Vec<usize> = vec![source];
    while !current.is_empty() {
        let next: Vec<usize> = current
            .par_iter()
            .flat_map(|&u| {
                let d = depth[u].load(Ordering::Relaxed);
                let mut out = Vec::new();
                for &(v, _) in graph.neighbors(u) {
                    if depth[v]
                        .compare_exchange(usize::MAX, d + 1, Ordering::Relaxed, Ordering::Relaxed)
                        .is_ok()
                    {
                        out.push(v);
                    }
                }
                for &(v, _) in graph.in_neighbors(u) {
                    if depth[v]
                        .compare_exchange(usize::MAX, d + 1, Ordering::Relaxed, Ordering::Relaxed)
                        .is_ok()
                    {
                        out.push(v);
                    }
                }
                out
            })
            .collect();
        order.extend_from_slice(&next);
        current = next;
    }
    let depth_vec: Vec<usize> = depth.iter().map(|a| a.load(Ordering::Relaxed)).collect();
    BfsResult {
        order,
        depth: depth_vec,
    }
}
