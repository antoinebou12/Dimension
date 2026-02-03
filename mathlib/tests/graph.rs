//! Integration tests for graph pathfinding: Dijkstra, A*, D* Lite.
//!
//! Small graphs with known shortest paths and distances; grid-like A* with admissible heuristic;
//! D* Lite with one edge weight change and updated path verification.

use mathlib::{DStarLite, Graph, astar, dijkstra, dstar_lite};

// --- Dijkstra ---

#[test]
fn dijkstra_three_nodes() {
    let mut g = Graph::new(3);
    g.add_edge(0, 1, 1.0);
    g.add_edge(1, 2, 2.0);
    g.add_edge(0, 2, 5.0);
    let res = dijkstra(&g, 0);
    assert!((res.dist[0] - 0.0).abs() < 1e-10);
    assert!((res.dist[1] - 1.0).abs() < 1e-10);
    assert!((res.dist[2] - 3.0).abs() < 1e-10);
    assert_eq!(res.prev[1], Some(0));
    assert_eq!(res.prev[2], Some(1));
}

#[test]
fn dijkstra_five_nodes_known_path() {
    let mut g = Graph::new(5);
    g.add_edge(0, 1, 1.0);
    g.add_edge(0, 2, 4.0);
    g.add_edge(1, 2, 2.0);
    g.add_edge(1, 3, 6.0);
    g.add_edge(2, 3, 1.0);
    g.add_edge(2, 4, 3.0);
    g.add_edge(3, 4, 1.0);
    let res = dijkstra(&g, 0);
    assert!((res.dist[4] - 5.0).abs() < 1e-10);
    assert_eq!(res.prev[4], Some(3));
    assert_eq!(res.prev[3], Some(2));
    assert_eq!(res.prev[2], Some(1));
    assert_eq!(res.prev[1], Some(0));
}

// --- A* ---

#[test]
fn astar_grid_optimal_path() {
    let mut g = Graph::new(9);
    for i in 0..3 {
        for j in 0..3 {
            let u = i * 3 + j;
            if j + 1 < 3 {
                g.add_edge(u, i * 3 + (j + 1), 1.0);
            }
            if j > 0 {
                g.add_edge(u, i * 3 + (j - 1), 1.0);
            }
            if i + 1 < 3 {
                g.add_edge(u, (i + 1) * 3 + j, 1.0);
            }
            if i > 0 {
                g.add_edge(u, (i - 1) * 3 + j, 1.0);
            }
        }
    }
    let start = 0;
    let goal = 8;
    let cols = 3;
    let h = |u: usize, g_goal: usize| {
        let ux = (u % cols) as f64;
        let uy = (u / cols) as f64;
        let gx = (g_goal % cols) as f64;
        let gy = (g_goal / cols) as f64;
        (ux - gx).abs() + (uy - gy).abs()
    };
    let res = astar(&g, start, goal, h);
    assert!(!res.path.is_empty());
    assert_eq!(res.path[0], start);
    assert_eq!(res.path[res.path.len() - 1], goal);
    assert!((res.dist - 4.0).abs() < 1e-10);
}

#[test]
fn astar_admissible_heuristic_matches_dijkstra() {
    let mut g = Graph::new(4);
    g.add_edge(0, 1, 1.0);
    g.add_edge(0, 2, 2.0);
    g.add_edge(1, 3, 1.0);
    g.add_edge(2, 3, 1.0);
    let dijk = dijkstra(&g, 0);
    let zero_h = |_u: usize, _goal: usize| 0.0;
    let ares = astar(&g, 0, 3, zero_h);
    assert!((ares.dist - dijk.dist[3]).abs() < 1e-10);
}

// --- D* Lite ---

#[test]
fn dstar_lite_small_graph_change_edge() {
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

#[test]
fn dstar_lite_one_shot_then_update() {
    let mut g = Graph::new(3);
    g.add_edge(0, 1, 1.0);
    g.add_edge(1, 2, 1.0);
    let res = dstar_lite(&mut g, 0, 2);
    assert_eq!(res.path, [0, 1, 2]);
    assert!((res.dist - 2.0).abs() < 1e-10);
    g.add_edge(0, 2, 1.5);
    let res2 = dstar_lite(&mut g, 0, 2);
    assert_eq!(res2.path, [0, 2]);
    assert!((res2.dist - 1.5).abs() < 1e-10);
}
