//! D* Lite style: replan after edge cost changes (sequential).
//!
//! Provides an API to plan from start to goal, update edges, and replan without full D* Lite
//! internals (replan runs Dijkstra). Suitable for incremental use: `update_edge` then `replan()`.

use crate::graph::dijkstra;
use crate::graph::types::{Graph, Weight};
use tracing::debug;

/// Result of D* Lite replan: path from start to goal (if any), and distance.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DStarLiteResult {
    /// Path from start to goal (empty if no path).
    pub path: Vec<usize>,
    /// Distance from start to goal; `f64::INFINITY` if no path.
    pub dist: Weight,
}

/// D* Lite state: mutable graph plus start/goal; supports replan after edge updates.
pub struct DStarLite<'a> {
    graph: &'a mut Graph,
    start: usize,
    goal: usize,
}

impl<'a> DStarLite<'a> {
    /// Creates state with graph, start and goal. Call `replan()` to get initial path.
    pub fn new(graph: &'a mut Graph, start: usize, goal: usize) -> Self {
        let n = graph.num_nodes();
        assert!(start < n && goal < n);
        Self { graph, start, goal }
    }

    /// Updates edge (u, v) to new cost. Add the edge if missing. Call `replan()` after updates.
    pub fn update_edge(&mut self, u: usize, v: usize, new_cost: Weight) {
        let n = self.graph.num_nodes();
        assert!(u < n && v < n);
        if let Some(edge) = self.graph.out_edges[u].iter_mut().find(|(nv, _)| *nv == v) {
            edge.1 = new_cost;
        } else {
            self.graph.add_edge(u, v, new_cost);
        }
    }

    /// Replans from start to goal (runs Dijkstra), returns path and distance.
    pub fn replan(&mut self) -> DStarLiteResult {
        let res = dijkstra::dijkstra(self.graph, self.start);
        let dist = res.dist[self.goal];
        if !dist.is_finite() {
            return DStarLiteResult {
                path: Vec::new(),
                dist: Weight::INFINITY,
            };
        }
        let path = path_from_prev(&res.prev, self.start, self.goal);
        DStarLiteResult { path, dist }
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

/// One-shot: run replan from start to goal on the given graph. For incremental use, use `DStarLite::new` and call `replan()` / `update_edge()` / `replan()`.
#[must_use]
pub fn dstar_lite(graph: &mut Graph, start: usize, goal: usize) -> DStarLiteResult {
    let num_nodes = graph.num_nodes();
    debug!(num_nodes, start, goal, "dstar_lite");
    let mut dl = DStarLite::new(graph, start, goal);
    dl.replan()
}
