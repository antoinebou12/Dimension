//! A* shortest path with heuristic (sequential; optional parallel behind `parallel` feature).

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::graph::types::{Graph, Weight};
use tracing::debug;

/// Result of A*: path from start to goal (if found), and optionally distances/predecessors.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AStarResult {
    /// Path from start to goal (empty if no path). Includes both start and goal.
    pub path: Vec<usize>,
    /// Distance from start to goal; `f64::INFINITY` if no path.
    pub dist: Weight,
    /// Predecessor on shortest path; `None` for start or unreachable.
    pub prev: Vec<Option<usize>>,
}

/// Runs A* from `start` to `goal` with heuristic `h(u, goal)` (must be admissible).
/// Returns path and distance; if no path, `path` is empty and `dist` is infinity.
/// With `parallel` feature, uses Rayon for neighbor iteration; otherwise sequential.
#[must_use]
pub fn astar<F>(graph: &Graph, start: usize, goal: usize, h: F) -> AStarResult
where
    F: Fn(usize, usize) -> Weight,
{
    let num_nodes = graph.num_nodes();
    debug!(num_nodes, start, goal, "astar");
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    return astar_parallel(graph, start, goal, &h);
    #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
    astar_sequential(graph, start, goal, &h)
}

#[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
fn astar_parallel<F>(graph: &Graph, start: usize, goal: usize, h: &F) -> AStarResult
where
    F: Fn(usize, usize) -> Weight,
{
    use par_iter::prelude::*;
    let n = graph.num_nodes();
    assert!(start < n && goal < n);

    let mut g: Vec<Weight> = vec![Weight::INFINITY; n];
    let mut prev: Vec<Option<usize>> = vec![None; n];
    g[start] = 0.0;

    let f_start = 0.0 + h(start, goal);
    let mut heap: BinaryHeap<Reverse<(u64, usize)>> = BinaryHeap::new();
    heap.push(Reverse((f_start.to_bits(), start)));

    while let Some(Reverse((f_bits, u))) = heap.pop() {
        let f_u = f64::from_bits(f_bits);
        if u == goal {
            let dist = g[goal];
            let path = path_from_prev(&prev, start, goal);
            return AStarResult { path, dist, prev };
        }
        if f_u > g[u] + h(u, goal) {
            continue;
        }
        let g_u = g[u];
        if g_u == Weight::INFINITY {
            continue;
        }
        let neighbors: Vec<(usize, Weight)> = graph.neighbors(u).to_vec();
        let candidates: Vec<(usize, Weight)> =
            neighbors.par_iter().map(|&(v, w)| (v, g_u + w)).collect();
        for (v, g_new) in candidates {
            if g_new < g[v] {
                g[v] = g_new;
                prev[v] = Some(u);
                let f_v = g_new + h(v, goal);
                heap.push(Reverse((f_v.to_bits(), v)));
            }
        }
    }

    AStarResult {
        path: Vec::new(),
        dist: Weight::INFINITY,
        prev,
    }
}

fn astar_sequential<F>(graph: &Graph, start: usize, goal: usize, h: &F) -> AStarResult
where
    F: Fn(usize, usize) -> Weight,
{
    let n = graph.num_nodes();
    assert!(start < n && goal < n);

    let mut g: Vec<Weight> = vec![Weight::INFINITY; n];
    let mut prev: Vec<Option<usize>> = vec![None; n];
    g[start] = 0.0;

    let f_start = 0.0 + h(start, goal);
    let mut heap: BinaryHeap<Reverse<(u64, usize)>> = BinaryHeap::new();
    heap.push(Reverse((f_start.to_bits(), start)));

    while let Some(Reverse((f_bits, u))) = heap.pop() {
        let f_u = f64::from_bits(f_bits);
        if u == goal {
            let dist = g[goal];
            let path = path_from_prev(&prev, start, goal);
            return AStarResult { path, dist, prev };
        }
        if f_u > g[u] + h(u, goal) {
            continue;
        }
        let g_u = g[u];
        if g_u == Weight::INFINITY {
            continue;
        }
        for &(v, w) in graph.neighbors(u) {
            let g_new = g_u + w;
            if g_new < g[v] {
                g[v] = g_new;
                prev[v] = Some(u);
                let f_v = g_new + h(v, goal);
                heap.push(Reverse((f_v.to_bits(), v)));
            }
        }
    }

    AStarResult {
        path: Vec::new(),
        dist: Weight::INFINITY,
        prev,
    }
}

fn path_from_prev(prev: &[Option<usize>], start: usize, goal: usize) -> Vec<usize> {
    let mut path = vec![goal];
    let mut u = goal;
    while let Some(p) = prev[u] {
        path.push(p);
        u = p;
        if u == start {
            break;
        }
    }
    path.reverse();
    path
}
