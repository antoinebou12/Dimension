//! Integration tests for Dijkstra.

use mathlib::{Graph, dijkstra};

#[test]
fn dijkstra_single_node() {
    let g = Graph::new(1);
    let res = dijkstra(&g, 0);
    assert_eq!(res.dist.len(), 1);
    assert!((res.dist[0] - 0.0).abs() < 1e-10);
    assert!(res.prev[0].is_none());
}

#[test]
fn dijkstra_two_nodes() {
    let mut g = Graph::new(2);
    g.add_edge(0, 1, 3.0);
    let res = dijkstra(&g, 0);
    assert!((res.dist[0] - 0.0).abs() < 1e-10);
    assert!((res.dist[1] - 3.0).abs() < 1e-10);
    assert_eq!(res.prev[1], Some(0));
}

#[test]
fn dijkstra_small_graph() {
    // 0 -> 1 (1), 0 -> 2 (4), 1 -> 2 (2), 1 -> 3 (6), 2 -> 3 (1). Shortest 0->3 = 0->1->2->3 = 4.
    let mut g = Graph::new(4);
    g.add_edge(0, 1, 1.0);
    g.add_edge(0, 2, 4.0);
    g.add_edge(1, 2, 2.0);
    g.add_edge(1, 3, 6.0);
    g.add_edge(2, 3, 1.0);
    let res = dijkstra(&g, 0);
    assert!((res.dist[0] - 0.0).abs() < 1e-10);
    assert!((res.dist[1] - 1.0).abs() < 1e-10);
    assert!((res.dist[2] - 3.0).abs() < 1e-10);
    assert!((res.dist[3] - 4.0).abs() < 1e-10);
    assert_eq!(res.prev[1], Some(0));
    assert_eq!(res.prev[2], Some(1));
    assert_eq!(res.prev[3], Some(2));
}

#[test]
fn dijkstra_unreachable() {
    let mut g = Graph::new(3);
    g.add_edge(0, 1, 1.0);
    let res = dijkstra(&g, 0);
    assert!(res.dist[2].is_infinite());
    assert!(res.prev[2].is_none());
}
