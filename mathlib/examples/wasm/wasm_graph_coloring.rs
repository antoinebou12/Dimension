//! Example: WASM graph vertex coloring (greedy, DSatur, bipartite).
//! Run with: cargo run --example wasm_graph_coloring --features wasm

#![cfg_attr(not(feature = "wasm"), allow(dead_code))]

#[cfg(not(feature = "wasm"))]
fn main() {
    eprintln!("Build with: cargo run --example wasm_graph_coloring --features wasm");
}

#[cfg(feature = "wasm")]
fn main() {
    use mathlib::wasm::WasmGraph;

    let mut g = WasmGraph::new(5);
    g.add_edge_undirected(0, 1, 1.0).unwrap();
    g.add_edge_undirected(0, 2, 1.0).unwrap();
    g.add_edge_undirected(1, 2, 1.0).unwrap();
    g.add_edge_undirected(1, 3, 1.0).unwrap();
    g.add_edge_undirected(2, 4, 1.0).unwrap();

    let greedy = g.greedy_vertex_coloring();
    let dsatur = g.dsatur_coloring();
    println!(
        "Greedy: {:?} ({} colors)",
        greedy,
        greedy.iter().max().map(|c| c + 1).unwrap_or(0)
    );
    println!(
        "DSatur: {:?} ({} colors)",
        dsatur,
        dsatur.iter().max().map(|c| c + 1).unwrap_or(0)
    );
    match g.is_bipartite() {
        Some(bip) => println!("Bipartite: yes, 2-coloring {:?}", bip),
        None => println!("Bipartite: no (odd cycle)"),
    }
}
