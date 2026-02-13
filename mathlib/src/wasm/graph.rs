//! `WasmGraph`, `WasmDijkstraResult`, `WasmAstarResult`, and `WasmDStarLiteResult` for JavaScript pathfinding.

use wasm_bindgen::prelude::*;

use crate::graph::{
    Graph, astar, bfs, dfs_postorder, dfs_preorder, dijkstra, dsatur_coloring, dstar,
    greedy_vertex_coloring, is_bipartite,
};

use super::matrix::WasmMatrix;

/// Directed weighted graph for pathfinding (Dijkstra, A*).
#[wasm_bindgen]
pub struct WasmGraph {
    inner: Graph,
}

#[wasm_bindgen]
impl WasmGraph {
    /// Create a graph with `n` nodes and no edges.
    #[wasm_bindgen(constructor)]
    pub fn new(n: usize) -> Self {
        Self {
            inner: Graph::new(n),
        }
    }

    /// Build a graph from a flat edge list: `[u0, v0, w0, u1, v1, w1, ...]` (u→v with weight w).
    #[wasm_bindgen(js_name = fromEdges)]
    pub fn from_edges(n: usize, edges: &[f64]) -> Result<WasmGraph, JsError> {
        if !edges.len().is_multiple_of(3) {
            return Err(JsError::new(&format!(
                "Edges length {} must be a multiple of 3",
                edges.len()
            )));
        }
        let mut g = Graph::new(n);
        for chunk in edges.chunks_exact(3) {
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            let u = chunk[0] as usize;
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            let v = chunk[1] as usize;
            let w = chunk[2];
            if u >= n || v >= n {
                return Err(JsError::new(&format!(
                    "Edge ({}, {}) out of range [0, {})",
                    u, v, n
                )));
            }
            if w < 0.0 {
                return Err(JsError::new("Edge weight must be non-negative"));
            }
            g.add_edge(u, v, w);
        }
        Ok(WasmGraph { inner: g })
    }

    /// Build an undirected 4-connected grid graph with `rows * cols` nodes (unit weight).
    /// Node index = `r * cols + c`. Use with [`astarGrid`](Self::astarGrid) for fast pathfinding.
    #[wasm_bindgen(js_name = fromGrid2d)]
    pub fn from_grid_2d(rows: usize, cols: usize) -> Result<WasmGraph, JsError> {
        if rows == 0 || cols == 0 {
            return Err(JsError::new("rows and cols must be positive"));
        }
        let inner = Graph::from_grid_2d(rows, cols);
        Ok(WasmGraph { inner })
    }

    /// Build an undirected 4-connected grid graph with per-edge weights.
    /// `weights` order: horizontal edges row-by-row (`rows * (cols - 1)`), then vertical
    /// edges column-by-column (`(rows - 1) * cols`). Length must equal
    /// `2 * rows * cols - rows - cols`. Use with [`astarGrid`](Self::astarGrid) for pathfinding.
    #[wasm_bindgen(js_name = fromGrid2dEdgeWeights)]
    pub fn from_grid_2d_edge_weights(
        rows: usize,
        cols: usize,
        weights: &[f64],
    ) -> Result<WasmGraph, JsError> {
        if rows == 0 || cols == 0 {
            return Err(JsError::new("rows and cols must be positive"));
        }
        let expected = 2 * rows * cols - rows - cols;
        if weights.len() != expected {
            return Err(JsError::new(&format!(
                "weights length {} must equal 2*rows*cols - rows - cols = {}",
                weights.len(),
                expected
            )));
        }
        if weights.iter().any(|&w| w < 0.0) {
            return Err(JsError::new("edge weights must be non-negative"));
        }
        let inner = Graph::from_grid_2d_edge_weights(rows, cols, weights);
        Ok(WasmGraph { inner })
    }

    /// Add a directed edge from `u` to `v` with weight `w`.
    #[wasm_bindgen(js_name = addEdge)]
    pub fn add_edge(&mut self, u: usize, v: usize, w: f64) -> Result<(), JsError> {
        if u >= self.inner.num_nodes() || v >= self.inner.num_nodes() {
            return Err(JsError::new(&format!(
                "Node indices {} and {} out of range [0, {})",
                u,
                v,
                self.inner.num_nodes()
            )));
        }
        if w < 0.0 {
            return Err(JsError::new("Edge weight must be non-negative"));
        }
        self.inner.add_edge(u, v, w);
        Ok(())
    }

    /// Add an undirected edge between `u` and `v` with weight `w`.
    #[wasm_bindgen(js_name = addEdgeUndirected)]
    pub fn add_edge_undirected(&mut self, u: usize, v: usize, w: f64) -> Result<(), JsError> {
        if u >= self.inner.num_nodes() || v >= self.inner.num_nodes() {
            return Err(JsError::new(&format!(
                "Node indices {} and {} out of range [0, {})",
                u,
                v,
                self.inner.num_nodes()
            )));
        }
        if w < 0.0 {
            return Err(JsError::new("Edge weight must be non-negative"));
        }
        self.inner.add_edge_undirected(u, v, w);
        Ok(())
    }

    /// Run Dijkstra from `source`. Returns distances and predecessors.
    #[wasm_bindgen(js_name = dijkstra)]
    pub fn run_dijkstra(&self, source: usize) -> Result<WasmDijkstraResult, JsError> {
        if source >= self.inner.num_nodes() {
            return Err(JsError::new(&format!(
                "Source {} out of range [0, {})",
                source,
                self.inner.num_nodes()
            )));
        }
        let result = dijkstra(&self.inner, source);
        Ok(WasmDijkstraResult {
            dist: result.dist,
            prev: result.prev,
        })
    }

    /// Run A* from `start` to `goal` with zero heuristic (equivalent to Dijkstra).
    #[wasm_bindgen(js_name = astar)]
    pub fn run_astar(&self, start: usize, goal: usize) -> Result<WasmAstarResult, JsError> {
        let n = self.inner.num_nodes();
        if start >= n || goal >= n {
            return Err(JsError::new(&format!("Start/goal out of range [0, {})", n)));
        }
        let result = astar(&self.inner, start, goal, |_, _| 0.0);
        Ok(WasmAstarResult {
            path: result.path,
            dist: result.dist,
            prev: result.prev,
        })
    }

    /// Run A* from `start` to `goal` with Manhattan heuristic for a grid (node index = row*cols + col).
    /// Use on a graph built with [`fromGrid2d`](Self::fromGrid2d). Much faster than Dijkstra when only one path is needed.
    #[wasm_bindgen(js_name = astarGrid)]
    pub fn astar_grid(
        &self,
        _rows: usize,
        cols: usize,
        start: usize,
        goal: usize,
    ) -> Result<WasmAstarResult, JsError> {
        let n = self.inner.num_nodes();
        if start >= n || goal >= n {
            return Err(JsError::new(&format!("Start/goal out of range [0, {})", n)));
        }
        let result = astar(&self.inner, start, goal, |u, g| {
            let ur = (u / cols) as f64;
            let uc = (u % cols) as f64;
            let gr = (g / cols) as f64;
            let gc = (g % cols) as f64;
            (ur - gr).abs() + (uc - gc).abs()
        });
        Ok(WasmAstarResult {
            path: result.path,
            dist: result.dist,
            prev: result.prev,
        })
    }

    /// Run A* from `start` to `goal` with Euclidean heuristic from node coordinates.
    /// `coords` must have rows = `num_nodes` and cols = 2 or 3 (x,y or x,y,z).
    #[wasm_bindgen(js_name = astarWithCoords)]
    pub fn run_astar_with_coords(
        &self,
        start: usize,
        goal: usize,
        coords: &WasmMatrix,
    ) -> Result<WasmAstarResult, JsError> {
        let n = self.inner.num_nodes();
        if start >= n || goal >= n {
            return Err(JsError::new(&format!("Start/goal out of range [0, {})", n)));
        }
        if coords.rows() != n {
            return Err(JsError::new(&format!(
                "Coordinates rows {} must equal number of nodes {}",
                coords.rows(),
                n
            )));
        }
        let dim = coords.cols();
        if dim != 2 && dim != 3 {
            return Err(JsError::new("Coordinates must have 2 or 3 columns"));
        }
        let result = astar(&self.inner, start, goal, |u, g| {
            let mut sum = 0.0;
            for j in 0..dim {
                let a = coords.inner.get(u, j);
                let b = coords.inner.get(g, j);
                sum += (a - b) * (a - b);
            }
            sum.sqrt()
        });
        Ok(WasmAstarResult {
            path: result.path,
            dist: result.dist,
            prev: result.prev,
        })
    }

    /// Number of nodes.
    #[wasm_bindgen(js_name = numNodes)]
    pub fn num_nodes(&self) -> usize {
        self.inner.num_nodes()
    }

    /// Total number of directed edges.
    #[wasm_bindgen(js_name = numEdges)]
    pub fn num_edges(&self) -> usize {
        self.inner.num_edges()
    }

    /// Run D* Lite (one-shot replan) from `start` to `goal`. Mutates the graph internally.
    #[wasm_bindgen(js_name = dstarLite)]
    pub fn dstar_lite(
        &mut self,
        start: usize,
        goal: usize,
    ) -> Result<WasmDStarLiteResult, JsError> {
        let n = self.inner.num_nodes();
        if start >= n || goal >= n {
            return Err(JsError::new(&format!("Start/goal out of range [0, {})", n)));
        }
        let result = dstar::dstar_lite(&mut self.inner, start, goal);
        Ok(WasmDStarLiteResult {
            path: result.path,
            dist: result.dist,
        })
    }

    /// Greedy vertex coloring. Returns array of color indices (one per node).
    #[wasm_bindgen(js_name = greedyVertexColoring)]
    pub fn greedy_vertex_coloring(&self) -> Vec<usize> {
        greedy_vertex_coloring(&self.inner)
    }

    /// DSatur vertex coloring. Returns array of color indices (one per node).
    #[wasm_bindgen(js_name = dsaturColoring)]
    pub fn dsatur_coloring(&self) -> Vec<usize> {
        dsatur_coloring(&self.inner)
    }

    /// Returns 2-coloring if bipartite, or `null` if graph has odd cycle.
    #[wasm_bindgen(js_name = isBipartite)]
    pub fn is_bipartite(&self) -> Option<Vec<usize>> {
        is_bipartite(&self.inner)
    }

    /// BFS from source. Treats graph as undirected.
    #[wasm_bindgen(js_name = bfs)]
    pub fn run_bfs(&self, source: usize) -> Result<WasmBfsResult, JsError> {
        if source >= self.inner.num_nodes() {
            return Err(JsError::new(&format!(
                "Source {} out of range [0, {})",
                source,
                self.inner.num_nodes()
            )));
        }
        let result = bfs(&self.inner, source);
        Ok(WasmBfsResult {
            order: result.order,
            depth: result.depth,
        })
    }

    /// DFS preorder from source. Treats graph as undirected.
    #[wasm_bindgen(js_name = dfsPreorder)]
    pub fn dfs_preorder(&self, source: usize) -> Result<Vec<usize>, JsError> {
        if source >= self.inner.num_nodes() {
            return Err(JsError::new(&format!(
                "Source {} out of range [0, {})",
                source,
                self.inner.num_nodes()
            )));
        }
        Ok(dfs_preorder(&self.inner, source))
    }

    /// DFS postorder from source. Treats graph as undirected.
    #[wasm_bindgen(js_name = dfsPostorder)]
    pub fn dfs_postorder(&self, source: usize) -> Result<Vec<usize>, JsError> {
        if source >= self.inner.num_nodes() {
            return Err(JsError::new(&format!(
                "Source {} out of range [0, {})",
                source,
                self.inner.num_nodes()
            )));
        }
        Ok(dfs_postorder(&self.inner, source))
    }
}

/// Result of Dijkstra: distances and predecessors.
#[wasm_bindgen]
pub struct WasmDijkstraResult {
    dist: Vec<f64>,
    prev: Vec<Option<usize>>,
}

#[wasm_bindgen]
impl WasmDijkstraResult {
    /// Distance from source to each node (`Infinity` if unreachable).
    #[wasm_bindgen(js_name = getDistances)]
    pub fn get_distances(&self) -> Vec<f64> {
        self.dist.clone()
    }

    /// Predecessor on shortest path; `null` for source or unreachable.
    /// Returns -1 for null (JS doesn't have Option<usize>).
    #[wasm_bindgen(js_name = getPredecessors)]
    pub fn get_predecessors(&self) -> Vec<i32> {
        self.prev
            .iter()
            .map(|o| o.map_or(-1, |x| i32::try_from(x).unwrap_or(-1)))
            .collect()
    }

    /// Reconstruct path from source to `target`. Returns array of node indices, or empty if unreachable.
    #[wasm_bindgen(js_name = pathTo)]
    pub fn path_to(&self, target: usize) -> Vec<usize> {
        if target >= self.prev.len() {
            return vec![];
        }
        let mut path = vec![];
        let mut cur = Some(target);
        while let Some(u) = cur {
            path.push(u);
            cur = self.prev[u];
        }
        path.reverse();
        path
    }

    /// Distance from source to `target`; `Infinity` if unreachable.
    #[wasm_bindgen(js_name = distanceTo)]
    pub fn distance_to(&self, target: usize) -> f64 {
        if target >= self.dist.len() {
            return f64::INFINITY;
        }
        self.dist[target]
    }
}

/// Result of A*: path from start to goal, total distance, and predecessors.
#[wasm_bindgen]
pub struct WasmAstarResult {
    path: Vec<usize>,
    dist: f64,
    prev: Vec<Option<usize>>,
}

#[wasm_bindgen]
impl WasmAstarResult {
    /// Path from start to goal (empty if no path). Includes both start and goal.
    #[wasm_bindgen(js_name = getPath)]
    pub fn get_path(&self) -> Vec<usize> {
        self.path.clone()
    }

    /// Total distance from start to goal; `Infinity` if no path.
    #[wasm_bindgen(js_name = getDist)]
    pub fn get_dist(&self) -> f64 {
        self.dist
    }

    /// Predecessor on shortest path; -1 for start or unreachable.
    #[wasm_bindgen(js_name = getPredecessors)]
    pub fn get_predecessors(&self) -> Vec<i32> {
        self.prev
            .iter()
            .map(|o| o.map_or(-1, |x| i32::try_from(x).unwrap_or(-1)))
            .collect()
    }

    /// Reconstruct path from start to `target`. Returns array of node indices, or empty if unreachable.
    #[wasm_bindgen(js_name = pathTo)]
    pub fn path_to(&self, target: usize) -> Vec<usize> {
        if target >= self.prev.len() {
            return vec![];
        }
        let mut path = vec![];
        let mut cur = Some(target);
        while let Some(u) = cur {
            path.push(u);
            cur = self.prev[u];
        }
        path.reverse();
        path
    }
}

/// Result of BFS: visit order and depth per node.
#[wasm_bindgen]
pub struct WasmBfsResult {
    order: Vec<usize>,
    depth: Vec<usize>,
}

#[wasm_bindgen]
impl WasmBfsResult {
    /// Visit order (nodes in discovery order).
    #[wasm_bindgen(js_name = getOrder)]
    pub fn get_order(&self) -> Vec<usize> {
        self.order.clone()
    }

    /// Depth from source (usize::MAX for unreachable).
    #[wasm_bindgen(js_name = getDepth)]
    pub fn get_depth(&self) -> Vec<usize> {
        self.depth.clone()
    }
}

/// Result of D* Lite: path from start to goal and total distance.
#[wasm_bindgen]
pub struct WasmDStarLiteResult {
    path: Vec<usize>,
    dist: f64,
}

#[wasm_bindgen]
impl WasmDStarLiteResult {
    /// Path from start to goal (empty if no path). Includes both start and goal.
    #[wasm_bindgen(js_name = getPath)]
    pub fn get_path(&self) -> Vec<usize> {
        self.path.clone()
    }

    /// Total distance from start to goal; `Infinity` if no path.
    #[wasm_bindgen(js_name = getDist)]
    pub fn get_dist(&self) -> f64 {
        self.dist
    }
}
