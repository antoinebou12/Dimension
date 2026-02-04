//! Tests for graph types: adjacency, in_edges, out_degree, in_degree, edges.

use mathlib::Graph;

#[test]
fn add_edge_undirected() {
    let mut g = Graph::new(3);
    g.add_edge_undirected(0, 1, 1.0);
    g.add_edge_undirected(1, 2, 2.0);
    assert_eq!(g.out_degree(0), 1);
    assert_eq!(g.out_degree(1), 2);
    assert_eq!(g.out_degree(2), 1);
    assert_eq!(g.in_degree(0), 1);
    assert_eq!(g.in_degree(1), 2);
    assert_eq!(g.in_degree(2), 1);
}

#[test]
fn out_degree_in_degree() {
    let mut g = Graph::new(4);
    g.add_edge(0, 1, 1.0);
    g.add_edge(0, 2, 1.0);
    g.add_edge(1, 2, 1.0);
    assert_eq!(g.out_degree(0), 2);
    assert_eq!(g.out_degree(1), 1);
    assert_eq!(g.out_degree(2), 0);
    assert_eq!(g.in_degree(0), 0);
    assert_eq!(g.in_degree(1), 1);
    assert_eq!(g.in_degree(2), 2);
}

#[test]
fn in_neighbors() {
    let mut g = Graph::new(3);
    g.add_edge(0, 2, 1.0);
    g.add_edge(1, 2, 2.0);
    let in2 = g.in_neighbors(2);
    assert_eq!(in2.len(), 2);
    let vs: Vec<usize> = in2.iter().map(|&(v, _)| v).collect();
    assert!(vs.contains(&0));
    assert!(vs.contains(&1));
}

#[test]
fn is_adjacent() {
    let mut g = Graph::new(3);
    g.add_edge(0, 1, 1.0);
    g.add_edge(0, 2, 1.0);
    assert!(g.is_adjacent(0, 1));
    assert!(g.is_adjacent(0, 2));
    assert!(!g.is_adjacent(0, 0));
    assert!(!g.is_adjacent(1, 0));
    assert!(!g.is_adjacent(2, 0));
}

#[test]
fn edges_iterator() {
    let mut g = Graph::new(3);
    g.add_edge(0, 1, 1.0);
    g.add_edge(0, 2, 2.0);
    g.add_edge(1, 2, 3.0);
    let edges: Vec<_> = g.edges().collect();
    assert_eq!(edges.len(), 3);
    assert!(
        edges
            .iter()
            .any(|e| e.u == 0 && e.v == 1 && (e.weight - 1.0).abs() < 1e-10)
    );
    assert!(
        edges
            .iter()
            .any(|e| e.u == 0 && e.v == 2 && (e.weight - 2.0).abs() < 1e-10)
    );
    assert!(
        edges
            .iter()
            .any(|e| e.u == 1 && e.v == 2 && (e.weight - 3.0).abs() < 1e-10)
    );
}

#[test]
fn from_adjacency_list_builds_in_edges() {
    let out_edges = vec![vec![(1, 1.0), (2, 2.0)], vec![(2, 3.0)], vec![]];
    let g = Graph::from_adjacency_list(3, out_edges);
    assert_eq!(g.in_degree(0), 0);
    assert_eq!(g.in_degree(1), 1);
    assert_eq!(g.in_degree(2), 2);
    assert_eq!(g.in_neighbors(2).len(), 2);
}

#[test]
fn is_directed() {
    let g = Graph::new(1);
    assert!(g.is_directed());
}
