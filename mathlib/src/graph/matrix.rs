//! Matrix representation of graphs and trees.
//!
//! Converts [`Graph`] and [`Tree`] to sparse matrix form (adjacency, optional Laplacian)
//! as triplets or as [`SparseMatrixCRS`] / [`SparseMatrixCCS`]. Constructed matrices are
//! compatible with existing wgpu `SpMV` (`try_spmv_f32`, `try_spmv_ccs_f32`) and SIMD-backed
//! vector ops when the `gpu` and `simd` features are enabled.
#![allow(clippy::cast_possible_truncation)]
//!
//! # Visualization data (data only, no layout)
//!
//! For visualization pipelines, export edge list and matrix data as follows:
//!
//! - **Graph**: [`Graph::num_nodes`](super::types::Graph::num_nodes), [`Graph::edges_vec`](super::types::Graph::edges_vec)
//!   for node count and edge list; [`adjacency_triplets`] or [`adjacency_crs`] for sparse matrix
//!   construction or GPU `SpMV`.
//! - **Tree**: [`tree_adjacency_triplets`] or [`tree_adjacency_crs`] for the parent–child
//!   adjacency matrix; the tree's `nodes` (parent/children) give the connection structure
//!   for drawing. No 2D/3D layout is computed in mathlib; render or external tools consume the data.

use crate::graph::tree::Tree;
use crate::graph::types::{Graph, Weight};
use crate::structure::{SparseMatrixCCS, SparseMatrixCRS, SparseStorage, Triplet};

/// Builds adjacency-matrix triplets for a graph.
///
/// One triplet per directed edge: row = source, column = target, value = weight.
/// Matrix size is `n × n` with `n = graph.num_nodes()`.
///
/// # Examples
///
/// ```
/// use crate::graph::{Graph, adjacency_triplets, adjacency_crs};
/// use crate::SparseStorage;
/// let mut g = Graph::new(3);
/// g.add_edge(0, 1, 2.0);
/// g.add_edge(1, 2, 1.0);
/// let triplets = adjacency_triplets(&g);
/// assert_eq!(triplets.len(), 2);
/// assert_eq!(triplets[0].i, 0);
/// assert_eq!(triplets[0].j, 1);
/// assert!((triplets[0].val - 2.0).abs() < 1e-10);
/// let crs = adjacency_crs(&g);
/// assert_eq!(crs.rows(), 3);
/// assert_eq!(crs.nnz(), 2);
/// ```
#[must_use]
pub fn adjacency_triplets(graph: &Graph) -> Vec<Triplet<Weight>> {
    let n = graph.num_nodes();
    let mut triplets = Vec::with_capacity(graph.num_edges());
    for u in 0..n {
        for &(v, w) in graph.neighbors(u) {
            triplets.push(Triplet::new(w, u as u32, v as u32));
        }
    }
    triplets
}

/// Builds the adjacency matrix of a graph in CRS format.
///
/// The matrix is `n × n` with `n = graph.num_nodes()`. Compatible with
/// [`crate::gpu::try_spmv_f32`] when `T = f32` and the `gpu` feature is enabled.
///
/// # Examples
///
/// ```
/// use mathlib::graph::{Graph, adjacency_crs};
/// use mathlib::structure::SparseStorage;
/// let mut g = Graph::new(2);
/// g.add_edge(0, 1, 1.0);
/// let a = adjacency_crs(&g);
/// assert_eq!(a.rows(), 2);
/// assert_eq!(a.cols(), 2);
/// assert_eq!(a.nnz(), 1);
/// ```
#[must_use]
pub fn adjacency_crs(graph: &Graph) -> SparseMatrixCRS<Weight> {
    let n = graph.num_nodes();
    let triplets = adjacency_triplets(graph);
    SparseStorage::from_triplets(n, n, &triplets)
}

/// Builds the adjacency matrix of a graph in CCS format.
///
/// The matrix is `n × n`. Compatible with [`crate::gpu::try_spmv_ccs_f32`] when
/// `T = f32` and the `gpu` feature is enabled.
#[must_use]
pub fn adjacency_ccs(graph: &Graph) -> SparseMatrixCCS<Weight> {
    let n = graph.num_nodes();
    let triplets = adjacency_triplets(graph);
    SparseStorage::from_triplets(n, n, &triplets)
}

/// Builds Laplacian triplets for an undirected graph.
///
/// Uses combinatorial Laplacian `L = D - A` where `D` is the degree matrix (diagonal
/// of row-sums of edge weights) and `A` is the adjacency matrix. For undirected
/// graphs, each edge should appear in both directions (e.g. built with
/// [`Graph::add_undirected_edge`](super::types::Graph::add_undirected_edge)).
///
/// # Panics
///
/// Never panics; returns empty triplets if the graph is empty.
#[must_use]
pub fn laplacian_triplets(graph: &Graph) -> Vec<Triplet<Weight>> {
    let n = graph.num_nodes();
    if n == 0 {
        return Vec::new();
    }
    let mut degree: Vec<Weight> = vec![0.0; n];
    let mut triplets = Vec::with_capacity(2 * graph.num_edges() + n);
    for (u, deg) in degree.iter_mut().enumerate() {
        for &(v, w) in graph.neighbors(u) {
            *deg += w;
            triplets.push(Triplet::new(-w, u as u32, v as u32));
        }
    }
    for (i, &d) in degree.iter().enumerate() {
        if d != 0.0 {
            triplets.push(Triplet::new(d, i as u32, i as u32));
        }
    }
    triplets
}

/// Builds the combinatorial Laplacian of an undirected graph in CRS format.
///
/// See [`laplacian_triplets`] for the definition.
#[must_use]
pub fn laplacian_crs(graph: &Graph) -> SparseMatrixCRS<Weight> {
    let n = graph.num_nodes();
    let triplets = laplacian_triplets(graph);
    SparseStorage::from_triplets(n, n, &triplets)
}

/// Builds adjacency-matrix triplets for a tree.
///
/// One triplet per parent→child edge with weight 1.0. Matrix size is `n × n` where
/// `n = tree.num_nodes()`. Non-tree nodes (indices beyond the root's tree) have no
/// entries.
///
/// # Examples
///
/// ```
/// use crate::graph::{Tree, tree_adjacency_triplets, tree_adjacency_crs};
/// use crate::SparseStorage;
/// let mut t: Tree<()> = Tree::new(0);
/// t.add_child(0, 1);
/// t.add_child(0, 2);
/// let triplets = tree_adjacency_triplets(&t);
/// assert_eq!(triplets.len(), 2);
/// let crs = tree_adjacency_crs(&t);
/// assert_eq!(crs.nnz(), 2);
/// ```
#[must_use]
pub fn tree_adjacency_triplets<T>(tree: &Tree<T>) -> Vec<Triplet<Weight>> {
    let n = tree.nodes.len();
    let mut triplets = Vec::new();
    for (child, node) in tree.nodes.iter().enumerate() {
        if let Some(parent) = node.parent
            && parent < n
            && child < n
        {
            triplets.push(Triplet::new(1.0, parent as u32, child as u32));
        }
    }
    triplets
}

/// Builds the adjacency matrix of a tree in CRS format.
///
/// Matrix is `n × n` with one entry per parent→child edge (value 1.0).
#[must_use]
pub fn tree_adjacency_crs<T>(tree: &Tree<T>) -> SparseMatrixCRS<Weight> {
    let n = tree.nodes.len();
    let triplets = tree_adjacency_triplets(tree);
    SparseStorage::from_triplets(n, n, &triplets)
}

/// Builds triplets for the 2D discrete Laplacian on a rectangular grid (5-point stencil).
///
/// Matrix size is `(rows * cols) × (rows * cols)`. Each interior point has diagonal entry
/// `-4` and `1` for its four neighbors (up, down, left, right); boundary points have
/// diagonal `-(number of in-grid neighbors)` and `1` for each such neighbor. The result
/// has O(p) nonzeros for p = rows × cols, suitable for image grids or PDE discretizations.
///
/// # Examples
///
/// ```
/// use crate::graph::laplacian_2d_grid_triplets;
/// use crate::structure::{SparseMatrixCRS, SparseStorage, Triplet};
/// let t = laplacian_2d_grid_triplets(2, 2);
/// assert_eq!(t.len(), 12); // 4 diag + 8 off-diag (each edge counted once per endpoint)
/// let n = 4;
/// let crs: SparseMatrixCRS<f64> = SparseStorage::from_triplets(n, n, &t);
/// assert_eq!(crs.rows(), 4);
/// assert_eq!(crs.cols(), 4);
/// ```
#[must_use]
pub fn laplacian_2d_grid_triplets(rows: usize, cols: usize) -> Vec<Triplet<f64>> {
    let n = rows * cols;
    if n == 0 {
        return Vec::new();
    }
    let mut triplets = Vec::with_capacity(5 * n);
    for r in 0..rows {
        for c in 0..cols {
            let i = r * cols + c;
            let mut count = 0_u32;
            if r > 0 {
                triplets.push(Triplet::new(1.0_f64, i as u32, (i - cols) as u32));
                count += 1;
            }
            if r + 1 < rows {
                triplets.push(Triplet::new(1.0_f64, i as u32, (i + cols) as u32));
                count += 1;
            }
            if c > 0 {
                triplets.push(Triplet::new(1.0_f64, i as u32, (i - 1) as u32));
                count += 1;
            }
            if c + 1 < cols {
                triplets.push(Triplet::new(1.0_f64, i as u32, (i + 1) as u32));
                count += 1;
            }
            triplets.push(Triplet::new(-f64::from(count), i as u32, i as u32));
        }
    }
    triplets
}

/// Builds the 2D grid Laplacian in CRS format (5-point stencil).
///
/// See [`laplacian_2d_grid_triplets`] for the definition.
#[must_use]
pub fn laplacian_2d_grid_crs(rows: usize, cols: usize) -> SparseMatrixCRS<f64> {
    let n = rows * cols;
    let triplets = laplacian_2d_grid_triplets(rows, cols);
    SparseStorage::from_triplets(n, n, &triplets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::{SparseMatrixCRS, SparseStorage};

    #[test]
    fn graph_adjacency_triplets_small() {
        let mut g = Graph::new(3);
        g.add_edge(0, 1, 2.0);
        g.add_edge(1, 2, 1.0);
        let t = adjacency_triplets(&g);
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].i, 0);
        assert_eq!(t[0].j, 1);
        assert!((t[0].val - 2.0).abs() < 1e-10);
        assert_eq!(t[1].i, 1);
        assert_eq!(t[1].j, 2);
        assert!((t[1].val - 1.0).abs() < 1e-10);
    }

    #[test]
    fn graph_adjacency_crs_dimensions() {
        let mut g = Graph::new(4);
        g.add_edge(0, 1, 1.0);
        g.add_edge(1, 2, 1.0);
        g.add_edge(2, 3, 1.0);
        let crs = adjacency_crs(&g);
        assert_eq!(crs.rows(), 4);
        assert_eq!(crs.cols(), 4);
        assert_eq!(crs.nnz(), 3);
    }

    #[test]
    fn graph_laplacian_undirected() {
        let mut g = Graph::new(3);
        g.add_undirected_edge(0, 1, 1.0);
        g.add_undirected_edge(1, 2, 1.0);
        let l = laplacian_crs(&g);
        assert_eq!(l.rows(), 3);
        assert_eq!(l.cols(), 3);
        // degree 1: (0,0)=1, (1,1)=2, (2,2)=1; off-diag -1
        assert!(l.get(0, 0) > 0.0);
        assert!(l.get(1, 1) > 0.0);
        assert!(l.get(2, 2) > 0.0);
    }

    #[test]
    fn test_tree_adjacency_triplets() {
        let mut t: Tree<()> = Tree::new(0);
        t.add_child(0, 1);
        t.add_child(0, 2);
        t.add_child(1, 3);
        let triplets = super::tree_adjacency_triplets(&t);
        assert_eq!(triplets.len(), 3);
        let crs = super::tree_adjacency_crs(&t);
        assert_eq!(crs.rows(), 4);
        assert_eq!(crs.nnz(), 3);
    }

    #[test]
    fn laplacian_2d_grid_triplets_shape() {
        let t = laplacian_2d_grid_triplets(2, 2);
        assert_eq!(t.len(), 12);
        let crs: SparseMatrixCRS<f64> = SparseStorage::from_triplets(4, 4, &t);
        assert_eq!(crs.rows(), 4);
        assert_eq!(crs.cols(), 4);
    }

    #[test]
    fn laplacian_2d_grid_crs_5pt() {
        let l = laplacian_2d_grid_crs(3, 3);
        assert_eq!(l.rows(), 9);
        assert_eq!(l.cols(), 9);
        // center (1,1) has -4 on diagonal
        let center = 4;
        assert!((l.get(center, center) + 4.0).abs() < 1e-10);
    }
}
