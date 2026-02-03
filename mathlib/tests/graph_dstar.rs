//! Integration tests for D* Lite (replan after edge update).

use mathlib::{DStarLite, Graph, dstar_lite};

#[test]
fn dstar_lite_simple_path() {
    let mut g = Graph::new(3);
    g.add_edge(0, 1, 1.0);
    g.add_edge(1, 2, 1.0);
    let res = dstar_lite(&mut g, 0, 2);
    assert_eq!(res.path, [0, 1, 2]);
    assert!((res.dist - 2.0).abs() < 1e-10);
}

#[test]
fn dstar_lite_no_path() {
    let mut g = Graph::new(3);
    g.add_edge(0, 1, 1.0);
    let res = dstar_lite(&mut g, 0, 2);
    assert!(res.path.is_empty());
    assert!(res.dist.is_infinite());
}

#[test]
fn dstar_lite_update_edge_replan() {
    let mut g = Graph::new(4);
    g.add_edge(0, 1, 1.0);
    g.add_edge(0, 2, 10.0);
    g.add_edge(1, 3, 1.0);
    g.add_edge(2, 3, 1.0);
    let mut dl = DStarLite::new(&mut g, 0, 3);
    let res = dl.replan();
    assert_eq!(res.path, [0, 1, 3]);
    assert!((res.dist - 2.0).abs() < 1e-10);
    dl.update_edge(0, 1, 100.0);
    let res2 = dl.replan();
    assert_eq!(res2.path, [0, 2, 3]);
    assert!((res2.dist - 11.0).abs() < 1e-10);
}
