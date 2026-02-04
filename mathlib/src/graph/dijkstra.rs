//! Dijkstra single-source shortest path (sequential; optional parallel behind `parallel` feature).

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::graph::types::{Graph, Weight};
use tracing::debug;

/// Result of Dijkstra: distances from source and predecessor pointers for path reconstruction.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DijkstraResult {
    /// Distance from source to each node; `f64::INFINITY` if unreachable.
    pub dist: Vec<Weight>,
    /// Predecessor on shortest path; `None` for source or unreachable.
    pub prev: Vec<Option<usize>>,
}

/// Runs Dijkstra from `source` on `graph`. Returns distances and predecessors.
/// With `parallel` feature, uses chili for neighbor relaxation; otherwise sequential.
#[must_use]
pub fn dijkstra(graph: &Graph, source: usize) -> DijkstraResult {
    let num_nodes = graph.num_nodes();
    debug!(num_nodes, source, "dijkstra");
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    return dijkstra_parallel(graph, source);
    #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
    dijkstra_sequential(graph, source)
}

#[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
fn dijkstra_parallel(graph: &Graph, source: usize) -> DijkstraResult {
    use par_iter::prelude::*;
    let n = graph.num_nodes();
    assert!(source < n);

    let mut dist: Vec<Weight> = vec![Weight::INFINITY; n];
    let mut prev: Vec<Option<usize>> = vec![None; n];
    dist[source] = 0.0;

    let mut heap: BinaryHeap<Reverse<(u64, usize)>> = BinaryHeap::new();
    heap.push(Reverse((dist[source].to_bits(), source)));

    while let Some(Reverse((bits, u))) = heap.pop() {
        let du = f64::from_bits(bits);
        if du > dist[u] {
            continue;
        }
        let neighbors: Vec<(usize, Weight)> = graph.neighbors(u).to_vec();
        let candidates: Vec<(usize, Weight)> =
            neighbors.par_iter().map(|&(v, w)| (v, du + w)).collect();
        for (v, alt) in candidates {
            if alt < dist[v] {
                dist[v] = alt;
                prev[v] = Some(u);
                heap.push(Reverse((alt.to_bits(), v)));
            }
        }
    }

    DijkstraResult { dist, prev }
}

fn dijkstra_sequential(graph: &Graph, source: usize) -> DijkstraResult {
    let n = graph.num_nodes();
    assert!(source < n);

    let mut dist: Vec<Weight> = vec![Weight::INFINITY; n];
    let mut prev: Vec<Option<usize>> = vec![None; n];
    dist[source] = 0.0;

    // Min-heap: (distance_bits, node). Smaller distance = smaller u64 for non-neg f64.
    let mut heap: BinaryHeap<Reverse<(u64, usize)>> = BinaryHeap::new();
    heap.push(Reverse((dist[source].to_bits(), source)));

    while let Some(Reverse((bits, u))) = heap.pop() {
        let du = f64::from_bits(bits);
        if du > dist[u] {
            continue;
        }
        for &(v, w) in graph.neighbors(u) {
            let alt = du + w;
            if alt < dist[v] {
                dist[v] = alt;
                prev[v] = Some(u);
                heap.push(Reverse((alt.to_bits(), v)));
            }
        }
    }

    DijkstraResult { dist, prev }
}
