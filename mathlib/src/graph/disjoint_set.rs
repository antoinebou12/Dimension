//! Disjoint set (union-find) and connected components.
//!
//! For undirected graphs: use `connected_components` after building the graph
//! with `add_undirected_edge` or symmetric directed edges.

use std::collections::HashMap;

use crate::graph::types::{Graph, NodeId};
use tracing::{debug, info};

/// Union-find (disjoint set) with path compression and rank.
#[derive(Clone, Debug)]
pub struct DisjointSet {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl DisjointSet {
    /// Creates a disjoint set with `n` singleton elements (0..n).
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    /// Returns the representative of the set containing `x` (path compression).
    pub fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    /// Unites the sets containing `x` and `y` (by rank).
    pub fn union(&mut self, x: usize, y: usize) {
        let rx = self.find(x);
        let ry = self.find(y);
        if rx == ry {
            return;
        }
        match self.rank[rx].cmp(&self.rank[ry]) {
            std::cmp::Ordering::Less => self.parent[rx] = ry,
            std::cmp::Ordering::Greater => self.parent[ry] = rx,
            std::cmp::Ordering::Equal => {
                self.parent[ry] = rx;
                self.rank[rx] += 1;
            }
        }
    }

    /// Returns whether `x` and `y` are in the same set.
    #[must_use]
    pub fn connected(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }
}

/// Returns the connected components of the graph when treated as undirected.
/// Each edge u→v (and v→u if present) connects u and v.
/// Returns a list of components, each a vector of node ids.
#[must_use]
pub fn connected_components(graph: &Graph) -> Vec<Vec<NodeId>> {
    let n = graph.num_nodes();
    debug!(num_nodes = n, "connected_components");
    let mut ds = DisjointSet::new(n);
    for edge in graph.edges() {
        ds.union(edge.u, edge.v);
    }
    let mut repr_to_nodes: HashMap<usize, Vec<NodeId>> = HashMap::new();
    for u in 0..n {
        let r = ds.find(u);
        repr_to_nodes.entry(r).or_default().push(u);
    }
    let mut components: Vec<Vec<NodeId>> = repr_to_nodes.into_values().collect();
    components.sort_by_key(|c| std::cmp::Reverse(c.len()));
    let n_components = components.len();
    info!(n_components, "connected_components ok");
    components
}
