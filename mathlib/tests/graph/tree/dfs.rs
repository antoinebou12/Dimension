//! DFS integration tests.

use mathlib::{Graph, dfs_postorder, dfs_preorder};

#[test]
fn dfs_preorder_path() {
    let mut g = Graph::new(4);
    g.add_edge_undirected(0, 1, 1.0);
    g.add_edge_undirected(1, 2, 1.0);
    g.add_edge_undirected(2, 3, 1.0);
    let order = dfs_preorder(&g, 0);
    assert_eq!(order.len(), 4);
    assert_eq!(order[0], 0);
    assert!(order.contains(&1));
    assert!(order.contains(&2));
    assert!(order.contains(&3));
}

#[test]
fn dfs_preorder_star() {
    let mut g = Graph::new(4);
    g.add_edge_undirected(0, 1, 1.0);
    g.add_edge_undirected(0, 2, 1.0);
    g.add_edge_undirected(0, 3, 1.0);
    let order = dfs_preorder(&g, 0);
    assert_eq!(order.len(), 4);
    assert_eq!(order[0], 0);
}

#[test]
fn dfs_postorder_path() {
    let mut g = Graph::new(4);
    g.add_edge_undirected(0, 1, 1.0);
    g.add_edge_undirected(1, 2, 1.0);
    g.add_edge_undirected(2, 3, 1.0);
    let order = dfs_postorder(&g, 0);
    assert_eq!(order.len(), 4);
    assert_eq!(order[order.len() - 1], 0);
}
