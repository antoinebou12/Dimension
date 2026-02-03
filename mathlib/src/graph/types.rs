//! Graph types and constructors for pathfinding algorithms.
//!
//! Directed weighted graph represented as an adjacency list: for each node index,
//! a list of `(neighbor, weight)` outgoing edges.

/// Node identifier (index into the graph's node set).
pub type NodeId = usize;

/// Edge weight type (non-negative cost).
pub type Weight = f64;

/// Directed edge from `u` to `v` with weight.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Edge {
    /// Source node.
    pub u: NodeId,
    /// Target node.
    pub v: NodeId,
    /// Edge weight.
    pub weight: Weight,
}

/// Directed weighted graph: `n` nodes, `out_edges[u]` = list of `(neighbor, weight)` from node `u`,
/// `in_edges[u]` = list of incoming edges `(v, w)` meaning edge v→u with weight w.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Graph {
    /// Number of nodes (vertices). Node indices are in `0..n`.
    pub n: usize,
    /// For each node `u`, list of outgoing edges `(v, w)` meaning edge u→v with weight w.
    pub out_edges: Vec<Vec<(usize, Weight)>>,
    /// For each node `u`, list of incoming edges `(v, w)` meaning edge v→u with weight w.
    pub in_edges: Vec<Vec<(usize, Weight)>>,
}

impl Graph {
    /// Builds a graph with `n` nodes and no edges.
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self {
            n,
            out_edges: vec![Vec::new(); n],
            in_edges: vec![Vec::new(); n],
        }
    }

    /// Builds a graph from an adjacency list. `out_edges.len()` must equal `n`;
    /// each list contains `(neighbor_index, weight)` with neighbor in `0..n`.
    /// Builds `in_edges` from `out_edges` in one pass.
    ///
    /// # Panics
    /// Panics if `out_edges.len() != n` or if any neighbor index is out of range (debug).
    #[must_use]
    pub fn from_adjacency_list(n: usize, out_edges: Vec<Vec<(usize, Weight)>>) -> Self {
        assert_eq!(out_edges.len(), n, "out_edges length must equal n");
        #[cfg(debug_assertions)]
        for (u, list) in out_edges.iter().enumerate() {
            for &(v, _) in list {
                assert!(v < n, "neighbor {} out of range for node {}", v, u);
            }
        }
        let mut in_edges = vec![Vec::new(); n];
        for (u, list) in out_edges.iter().enumerate() {
            for &(v, w) in list {
                in_edges[v].push((u, w));
            }
        }
        Self {
            n,
            out_edges,
            in_edges,
        }
    }

    /// Adds a directed edge from `u` to `v` with weight `w`.
    ///
    /// # Panics
    /// Panics if `u` or `v` is out of range (debug).
    pub fn add_edge(&mut self, u: usize, v: usize, w: Weight) {
        assert!(u < self.n && v < self.n);
        self.out_edges[u].push((v, w));
        self.in_edges[v].push((u, w));
    }

    /// Adds an undirected edge between `u` and `v` with weight `w` (both u→v and v→u).
    ///
    /// # Panics
    /// Panics if `u` or `v` is out of range (debug).
    pub fn add_undirected_edge(&mut self, u: usize, v: usize, w: Weight) {
        self.add_edge(u, v, w);
        self.add_edge(v, u, w);
    }

    /// Alias for [`add_undirected_edge`](Self::add_undirected_edge).
    pub fn add_edge_undirected(&mut self, u: usize, v: usize, w: Weight) {
        self.add_undirected_edge(u, v, w);
    }

    /// Returns whether this graph is directed (always `true`).
    #[inline]
    #[must_use]
    pub fn is_directed(&self) -> bool {
        true
    }

    /// Returns the number of nodes.
    #[inline]
    #[must_use]
    pub fn num_nodes(&self) -> usize {
        self.n
    }

    /// Returns the total number of directed edges.
    #[must_use]
    pub fn num_edges(&self) -> usize {
        self.out_edges.iter().map(Vec::len).sum()
    }

    /// Out-degree of node `u` (number of outgoing edges).
    #[inline]
    #[must_use]
    pub fn out_degree(&self, u: usize) -> usize {
        assert!(u < self.n);
        self.out_edges[u].len()
    }

    /// In-degree of node `u` (number of incoming edges).
    #[inline]
    #[must_use]
    pub fn in_degree(&self, u: usize) -> usize {
        assert!(u < self.n);
        self.in_edges[u].len()
    }

    /// Returns the outgoing edges for node `u`: slice of `(neighbor, weight)`.
    #[inline]
    #[must_use]
    pub fn neighbors(&self, u: usize) -> &[(usize, Weight)] {
        assert!(u < self.n);
        self.out_edges[u].as_slice()
    }

    /// Returns the incoming edges for node `u`: slice of `(predecessor, weight)`.
    #[inline]
    #[must_use]
    pub fn in_neighbors(&self, u: usize) -> &[(usize, Weight)] {
        assert!(u < self.n);
        self.in_edges[u].as_slice()
    }

    /// Returns whether there is an edge from `u` to `v`.
    #[must_use]
    pub fn is_adjacent(&self, u: usize, v: usize) -> bool {
        assert!(u < self.n && v < self.n);
        self.out_edges[u].iter().any(|&(v_, _)| v_ == v)
    }

    /// Returns an iterator over all directed edges.
    #[inline]
    pub fn edges(&self) -> Edges<'_> {
        Edges {
            graph: self,
            u: 0,
            i: 0,
        }
    }

    /// Returns all directed edges as a vector (each edge u→v appears once).
    #[must_use]
    pub fn edges_vec(&self) -> Vec<Edge> {
        self.edges().collect()
    }
}

/// Iterator over edges of a graph.
pub struct Edges<'a> {
    graph: &'a Graph,
    u: usize,
    i: usize,
}

impl Iterator for Edges<'_> {
    type Item = Edge;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.graph.n;
        while self.u < n {
            let adj = &self.graph.out_edges[self.u];
            if self.i < adj.len() {
                let (v, w) = adj[self.i];
                self.i += 1;
                return Some(Edge {
                    u: self.u,
                    v,
                    weight: w,
                });
            }
            self.u += 1;
            self.i = 0;
        }
        None
    }
}

/// Builds the reverse graph: same nodes, each edge u→v becomes v→u.
#[must_use]
pub fn reverse_graph(graph: &Graph) -> Graph {
    let n = graph.n;
    let mut out_edges = vec![Vec::new(); n];
    let mut in_edges = vec![Vec::new(); n];
    for (u, out_adj) in graph.out_edges.iter().enumerate() {
        for &(v, w) in out_adj {
            out_edges[v].push((u, w));
            in_edges[u].push((v, w));
        }
    }
    Graph {
        n,
        out_edges,
        in_edges,
    }
}
