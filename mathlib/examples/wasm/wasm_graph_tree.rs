//! Example: WASM graph tree traversals (BFS, DFS).
//! Run with: cargo run --example wasm_graph_tree --features wasm

#![cfg_attr(not(feature = "wasm"), allow(dead_code))]

#[cfg(not(feature = "wasm"))]
fn main() {
    eprintln!("Build with: cargo run --example wasm_graph_tree --features wasm");
}

#[cfg(feature = "wasm")]
fn main() {
    use mathlib::wasm::{WasmBfsResult, WasmGraph};

    let mut g = WasmGraph::new(6);
    g.add_edge_undirected(0, 1, 1.0).unwrap();
    g.add_edge_undirected(0, 2, 1.0).unwrap();
    g.add_edge_undirected(1, 3, 1.0).unwrap();
    g.add_edge_undirected(2, 4, 1.0).unwrap();
    g.add_edge_undirected(3, 5, 1.0).unwrap();

    let bfs_res: WasmBfsResult = g.run_bfs(0).expect("bfs");
    println!("BFS from 0: order {:?}", bfs_res.get_order());
    println!("           depth {:?}", bfs_res.get_depth());

    let pre = g.dfs_preorder(0).expect("dfs_preorder");
    let post = g.dfs_postorder(0).expect("dfs_postorder");
    println!("DFS preorder:  {:?}", pre);
    println!("DFS postorder: {:?}", post);
}
