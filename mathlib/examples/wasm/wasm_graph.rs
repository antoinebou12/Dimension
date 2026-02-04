//! Example: WASM graph API (WasmGraph, Dijkstra, A*, D* Lite, coloring, BFS, DFS).
//! Run with: cargo run --example wasm_graph --features wasm
//!
//! Demonstrates building a graph from edges and running Dijkstra, A*, D* Lite,
//! vertex coloring (greedy, DSatur, bipartite), and tree traversals (BFS, DFS)
//! using the same API that JavaScript would use after
//! `wasm-pack build --target web --features wasm`.

#![cfg_attr(not(feature = "wasm"), allow(dead_code))]

#[cfg(not(feature = "wasm"))]
fn main() {
    eprintln!("Build with: cargo run --example wasm_graph --features wasm");
}

#[cfg(feature = "wasm")]
fn main() {
    use mathlib::wasm::{WasmAstarResult, WasmBfsResult, WasmDijkstraResult, WasmGraph};

    // Graph: 4 nodes, edges 0→1 (1), 0→2 (4), 1→2 (2), 1→3 (6), 2→3 (1)
    let edges = [
        0.0, 1.0, 1.0, 0.0, 2.0, 4.0, 1.0, 2.0, 2.0, 1.0, 3.0, 6.0, 2.0, 3.0, 1.0,
    ];
    let g = WasmGraph::from_edges(4, &edges).expect("graph from edges");
    println!("Graph: {} nodes, {} edges", g.num_nodes(), g.num_edges());

    let res: WasmDijkstraResult = g.run_dijkstra(0).expect("dijkstra");
    println!("Dijkstra from 0:");
    println!("  distances: {:?}", res.get_distances());
    println!("  path to 3: {:?}", res.path_to(3));

    let astar_res: WasmAstarResult = g.run_astar(0, 3).expect("astar");
    println!(
        "A* from 0 to 3: path {:?}, dist {}",
        astar_res.get_path(),
        astar_res.get_dist()
    );

    let mut g2 = WasmGraph::from_edges(4, &edges).expect("graph");
    let dstar_res = g2.dstar_lite(0, 3).expect("dstar_lite");
    println!(
        "D* Lite: path {:?}, dist {}",
        dstar_res.get_path(),
        dstar_res.get_dist()
    );

    // Undirected graph for coloring and tree
    let mut g_undir = WasmGraph::new(4);
    g_undir.add_edge_undirected(0, 1, 1.0).unwrap();
    g_undir.add_edge_undirected(0, 2, 1.0).unwrap();
    g_undir.add_edge_undirected(1, 2, 1.0).unwrap();
    g_undir.add_edge_undirected(1, 3, 1.0).unwrap();

    let greedy = g_undir.greedy_vertex_coloring();
    let dsatur = g_undir.dsatur_coloring();
    println!("Greedy coloring: {:?}", greedy);
    println!("DSatur coloring: {:?}", dsatur);
    if let Some(bip) = g_undir.is_bipartite() {
        println!("Bipartite: yes, 2-coloring {:?}", bip);
    } else {
        println!("Bipartite: no (odd cycle)");
    }

    let bfs_res: WasmBfsResult = g_undir.run_bfs(0).expect("bfs");
    println!(
        "BFS from 0: order {:?}, depth {:?}",
        bfs_res.get_order(),
        bfs_res.get_depth()
    );

    let pre = g_undir.dfs_preorder(0).expect("dfs_preorder");
    let post = g_undir.dfs_postorder(0).expect("dfs_postorder");
    println!("DFS preorder: {:?}", pre);
    println!("DFS postorder: {:?}", post);
}
