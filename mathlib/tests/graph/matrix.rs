//! Integration tests for graph and tree matrix representation.
//!
//! Builds graphs and trees, obtains adjacency (and Laplacian) triplets and CRS/CCS,
//! and checks dimensions and optional SpMV result.

use mathlib::graph::{
    adjacency_ccs, adjacency_crs, adjacency_triplets, laplacian_crs, laplacian_triplets,
    tree_adjacency_crs, tree_adjacency_triplets,
};
use mathlib::{Graph, SparseStorage, Tree, Vector};

#[test]
fn graph_adjacency_triplets_and_crs() {
    let mut g = Graph::new(4);
    g.add_edge(0, 1, 1.0);
    g.add_edge(1, 2, 2.0);
    g.add_edge(2, 3, 1.0);
    g.add_edge(0, 2, 0.5);
    let triplets = adjacency_triplets(&g);
    assert_eq!(triplets.len(), 4);
    assert_eq!(g.num_nodes(), 4);
    let crs = adjacency_crs(&g);
    assert_eq!(crs.rows(), 4);
    assert_eq!(crs.cols(), 4);
    assert_eq!(crs.nnz(), 4);
    let ccs = adjacency_ccs(&g);
    assert_eq!(ccs.rows(), 4);
    assert_eq!(ccs.cols(), 4);
    assert_eq!(ccs.nnz(), 4);
}

#[test]
fn graph_adjacency_spmv() {
    let mut g = Graph::new(3);
    g.add_edge(0, 1, 1.0);
    g.add_edge(0, 2, 2.0);
    g.add_edge(1, 2, 1.0);
    let a = adjacency_crs(&g);
    let mut x = Vector::with_capacity(3);
    x.set(0, 1.0);
    x.set(1, 0.0);
    x.set(2, 1.0);
    let y = &a * &x;
    assert_eq!(y.rows(), 3);
    // y[0] = A[0,:].x = 1*x[1] + 2*x[2] = 1*0 + 2*1 = 2, y[1] = 1*1 = 1, y[2] = 0
    assert!((y.get(0) - 2.0).abs() < 1e-10);
    assert!((y.get(1) - 1.0).abs() < 1e-10);
    assert!((y.get(2) - 0.0).abs() < 1e-10);
}

#[test]
fn graph_laplacian_undirected() {
    let mut g = Graph::new(3);
    g.add_undirected_edge(0, 1, 1.0);
    g.add_undirected_edge(1, 2, 1.0);
    let triplets = laplacian_triplets(&g);
    assert!(!triplets.is_empty());
    let l = laplacian_crs(&g);
    assert_eq!(l.rows(), 3);
    assert_eq!(l.cols(), 3);
    // L 1 = 0 for Laplacian
    let ones = Vector::from_slice(&[1.0, 1.0, 1.0]);
    let z = &l * &ones;
    assert!(z.get(0).abs() < 1e-10);
    assert!(z.get(1).abs() < 1e-10);
    assert!(z.get(2).abs() < 1e-10);
}

#[test]
fn tree_from_bfs_spanning_tree_then_matrix() {
    let mut g = Graph::new(5);
    g.add_edge(0, 1, 1.0);
    g.add_edge(0, 2, 1.0);
    g.add_edge(1, 3, 1.0);
    g.add_edge(2, 4, 1.0);
    g.add_edge(1, 2, 1.0);
    let tree = Tree::<()>::from_bfs_spanning_tree(&g, 0);
    let triplets = tree_adjacency_triplets(&tree);
    assert_eq!(triplets.len(), 4);
    let crs = tree_adjacency_crs(&tree);
    assert_eq!(crs.rows(), 5);
    assert_eq!(crs.nnz(), 4);
}

#[test]
fn tree_adjacency_spmv() {
    let mut t: Tree<()> = Tree::new(0);
    t.add_child(0, 1);
    t.add_child(0, 2);
    t.add_child(1, 3);
    let a = tree_adjacency_crs(&t);
    let mut x = Vector::with_capacity(4);
    x.set(0, 1.0);
    x.set(1, 2.0);
    x.set(2, 3.0);
    x.set(3, 4.0);
    let y = &a * &x;
    // y = A*x: row 0 has children 1,2 so y[0]=x[1]+x[2]=5; row 1 has child 3 so y[1]=x[3]=4; rows 2,3 have no children
    assert!((y.get(0) - 5.0).abs() < 1e-10);
    assert!((y.get(1) - 4.0).abs() < 1e-10);
    assert!((y.get(2) - 0.0).abs() < 1e-10);
    assert!((y.get(3) - 0.0).abs() < 1e-10);
}
