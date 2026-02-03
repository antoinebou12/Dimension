//! Integration tests for A*.

use mathlib::{Graph, astar};

#[test]
fn astar_same_node() {
    let g = Graph::new(1);
    let res = astar(&g, 0, 0, |_, _| 0.0);
    assert_eq!(res.path, [0]);
    assert!((res.dist - 0.0).abs() < 1e-10);
}

#[test]
fn astar_two_nodes() {
    let mut g = Graph::new(2);
    g.add_edge(0, 1, 3.0);
    let res = astar(&g, 0, 1, |_, _| 0.0);
    assert_eq!(res.path, [0, 1]);
    assert!((res.dist - 3.0).abs() < 1e-10);
}

#[test]
fn astar_small_graph() {
    let mut g = Graph::new(4);
    g.add_edge(0, 1, 1.0);
    g.add_edge(0, 2, 4.0);
    g.add_edge(1, 2, 2.0);
    g.add_edge(1, 3, 6.0);
    g.add_edge(2, 3, 1.0);
    let res = astar(&g, 0, 3, |_, _| 0.0);
    assert_eq!(res.path, [0, 1, 2, 3]);
    assert!((res.dist - 4.0).abs() < 1e-10);
}

#[test]
fn astar_grid_manhattan_heuristic() {
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
    let h = |u: usize, goal: usize| {
        let ux = (u % 3) as f64;
        let uy = (u / 3) as f64;
        let gx = (goal % 3) as f64;
        let gy = (goal / 3) as f64;
        (ux - gx).abs() + (uy - gy).abs()
    };
    let res = astar(&g, 0, 8, h);
    assert!(!res.path.is_empty());
    assert_eq!(res.path[0], 0);
    assert_eq!(res.path[res.path.len() - 1], 8);
    assert!((res.dist - 4.0).abs() < 1e-10);
}

#[test]
fn astar_no_path() {
    let mut g = Graph::new(3);
    g.add_edge(0, 1, 1.0);
    let res = astar(&g, 0, 2, |_, _| 0.0);
    assert!(res.path.is_empty());
    assert!(res.dist.is_infinite());
}
