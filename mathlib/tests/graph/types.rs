//! Tests for graph types: adjacency, in_edges, out_degree, in_degree, edges,
//! and dense adjacency matrix conversion (to_adjacency_matrix, from_adjacency_matrix).

use mathlib::{Graph, Matrix, Storage};

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

// --- Dense adjacency matrix conversion ---

#[test]
fn graph_to_adjacency_matrix() {
    let mut g = Graph::new(3);
    g.add_edge(0, 1, 2.0);
    g.add_edge(1, 2, 1.0);
    g.add_edge(0, 2, 0.5);
    let a = g.to_adjacency_matrix();
    assert_eq!(a.rows(), 3);
    assert_eq!(a.cols(), 3);
    assert!((a.get(0, 1) - 2.0).abs() < 1e-10);
    assert!((a.get(1, 2) - 1.0).abs() < 1e-10);
    assert!((a.get(0, 2) - 0.5).abs() < 1e-10);
    assert!(a.get(0, 0).abs() < 1e-10);
    assert!(a.get(1, 0).abs() < 1e-10);
}

#[test]
fn from_adjacency_matrix() {
    let mut a = Matrix::with_storage(3, 3, Storage::Column);
    a.set_zero();
    a.set(0, 1, 2.0);
    a.set(1, 2, 1.0);
    a.set(0, 2, 0.5);
    let g = Graph::from_adjacency_matrix(&a);
    assert_eq!(g.num_nodes(), 3);
    assert_eq!(g.num_edges(), 3);
    assert!(g.is_adjacent(0, 1));
    assert!(g.is_adjacent(1, 2));
    assert!(g.is_adjacent(0, 2));
    assert!((g.neighbors(0).iter().find(|&&(v, _)| v == 1).unwrap().1 - 2.0).abs() < 1e-10);
}

#[test]
fn roundtrip_graph_matrix() {
    let mut g = Graph::new(4);
    g.add_edge(0, 1, 1.0);
    g.add_edge(0, 2, 2.0);
    g.add_edge(1, 3, 1.5);
    g.add_edge(2, 3, 0.5);
    let a = g.to_adjacency_matrix();
    let g2 = Graph::from_adjacency_matrix(&a);
    assert_eq!(g2.num_nodes(), g.num_nodes());
    assert_eq!(g2.num_edges(), g.num_edges());
    for edge in g.edges() {
        assert!(g2.is_adjacent(edge.u, edge.v));
        let w = g2
            .neighbors(edge.u)
            .iter()
            .find(|&&(v, _)| v == edge.v)
            .unwrap()
            .1;
        assert!((w - edge.weight).abs() < 1e-10);
    }
}

#[test]
fn roundtrip_undirected_via_symmetric_matrix() {
    let mut g = Graph::new(3);
    g.add_undirected_edge(0, 1, 1.0);
    g.add_undirected_edge(1, 2, 2.0);
    let a = g.to_adjacency_matrix();
    assert!((a.get(0, 1) - 1.0).abs() < 1e-10);
    assert!((a.get(1, 0) - 1.0).abs() < 1e-10);
    let g2 = Graph::from_adjacency_matrix(&a);
    assert_eq!(g2.num_nodes(), 3);
    assert_eq!(g2.num_edges(), 4);
    assert!(g2.is_adjacent(0, 1) && g2.is_adjacent(1, 0));
    assert!(g2.is_adjacent(1, 2) && g2.is_adjacent(2, 1));
}

#[test]
fn empty_graph_adjacency_matrix() {
    let g = Graph::new(0);
    let a = g.to_adjacency_matrix();
    assert_eq!(a.rows(), 0);
    assert_eq!(a.cols(), 0);
    let g2 = Graph::from_adjacency_matrix(&a);
    assert_eq!(g2.num_nodes(), 0);
    assert_eq!(g2.num_edges(), 0);
}
