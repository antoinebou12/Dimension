//! Example: WASM clustering API (WasmKmeans, WasmDbscan).
//! Run with: cargo run --example wasm_clustering --features wasm
//!
//! Demonstrates K-means and DBSCAN using the same API that JavaScript would use
//! after `wasm-pack build --target web --features wasm`.

#![cfg_attr(not(feature = "wasm"), allow(dead_code))]

#[cfg(not(feature = "wasm"))]
fn main() {
    eprintln!("Build with: cargo run --example wasm_clustering --features wasm");
}

#[cfg(feature = "wasm")]
fn main() {
    use mathlib::wasm::{WasmDbscan, WasmKmeans, WasmMatrix};

    // ---- K-means: 6 points in 2D, k = 2 ----
    // Column-major 6×2: [x0..x5, y0..y5]
    let data_k = vec![
        0.0, 1.0, 2.0, 10.0, 11.0, 12.0, 0.0, 0.0, 0.0, 10.0, 10.0, 10.0,
    ];
    let m_k = WasmMatrix::from_array(6, 2, &data_k).expect("6×2 matrix");
    let km = WasmKmeans::new(&m_k, 2, 100).expect("K-means");
    println!("K-means (6 points, k=2):");
    println!("  labels: {:?}", km.get_labels());
    println!("  n_clusters: {}", km.n_clusters());
    let cents = km.get_centroids();
    println!("  centroids: {}×{}", cents.rows(), cents.cols());

    // ---- DBSCAN: 4 points with one outlier ----
    // (0,0), (1,0), (0,1), (10,10); eps=2, min_pts=2
    let data_d = vec![0.0, 1.0, 0.0, 10.0, 0.0, 0.0, 1.0, 10.0];
    let m_d = WasmMatrix::from_array(4, 2, &data_d).expect("4×2 matrix");
    let db = WasmDbscan::new(&m_d, 2.0, 2).expect("DBSCAN");
    println!("DBSCAN (4 points, eps=2, min_pts=2):");
    println!("  labels: {:?}", db.get_labels());
    println!("  n_clusters: {}", db.n_clusters());
}
