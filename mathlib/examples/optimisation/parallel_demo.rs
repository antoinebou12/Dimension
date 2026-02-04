//! Parallel feature demonstration: compare sequential vs parallel performance.
//!
//! Run without parallel: `cargo run --example parallel_demo`
//! Run with parallel: `cargo run --example parallel_demo --features parallel`
//! Run with full features: `cargo run --example parallel_demo --features full`

use mathlib::prelude::*;
use mathlib::{clustering, graph::*, stats};
use std::time::Instant;

fn main() {
    println!("=== Mathlib Parallel Feature Demo ===\n");

    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    println!("✓ Parallel feature ENABLED (using par-iter with chili backend)");
    #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
    println!("✗ Parallel feature DISABLED (using sequential backend)");

    #[cfg(feature = "simd")]
    println!("✓ SIMD feature ENABLED");
    #[cfg(not(feature = "simd"))]
    println!("✗ SIMD feature DISABLED");

    println!();

    // K-means clustering
    println!("--- K-means Clustering (1000 points, 5 clusters) ---");
    let kmeans_data: Vec<f64> = (0..2000)
        .map(|i| {
            let cluster = i % 5;
            let base = cluster as f64 * 10.0;
            base + (i as f64 * 0.1).sin() * 2.0
        })
        .collect();
    let data = Matrix::from_vec(&kmeans_data, 1000, 2, Storage::Column);

    let start = Instant::now();
    let result = clustering::kmeans(&data, 5, Some(50));
    let elapsed = start.elapsed();
    println!("K-means completed in {:?}", elapsed);
    println!("Found {} clusters", result.n_clusters());
    println!();

    // DBSCAN clustering
    println!("--- DBSCAN Clustering (500 points) ---");
    let dbscan_data: Vec<f64> = (0..1000).map(|i| (i as f64 * 0.1).sin() * 10.0).collect();
    let data = Matrix::from_vec(&dbscan_data, 500, 2, Storage::Column);

    let start = Instant::now();
    let result = clustering::dbscan(&data, 2.0, 3);
    let elapsed = start.elapsed();
    println!("DBSCAN completed in {:?}", elapsed);
    println!("Found {} clusters", result.n_clusters());
    println!();

    // Dijkstra pathfinding
    println!("--- Dijkstra Pathfinding (1000 nodes) ---");
    let mut g = Graph::new(1000);
    for i in 0..999 {
        g.add_edge(i, i + 1, ((i * 7) % 10 + 1) as f64);
        if i % 10 == 0 && i + 10 < 1000 {
            g.add_edge(i, i + 10, ((i * 3) % 5 + 1) as f64);
        }
    }

    let start = Instant::now();
    let result = dijkstra(&g, 0);
    let elapsed = start.elapsed();
    println!("Dijkstra completed in {:?}", elapsed);
    println!("Shortest path to node 999: {:.2}", result.dist[999]);
    println!();

    // A* pathfinding
    println!("--- A* Pathfinding (1000 nodes) ---");
    let start = Instant::now();
    let result = astar(&g, 0, 999, |_, _| 0.0);
    let elapsed = start.elapsed();
    println!("A* completed in {:?}", elapsed);
    println!(
        "Path found: {}, distance: {:.2}",
        !result.path.is_empty(),
        result.dist
    );
    println!();

    // BFS traversal
    println!("--- BFS Traversal (1000 nodes) ---");
    let mut g_undirected = Graph::new(1000);
    for i in 0..999 {
        g_undirected.add_edge_undirected(i, i + 1, 1.0);
        if i % 10 == 0 && i + 10 < 1000 {
            g_undirected.add_edge_undirected(i, i + 10, 1.0);
        }
    }

    let start = Instant::now();
    let result = bfs(&g_undirected, 0);
    let elapsed = start.elapsed();
    println!("BFS completed in {:?}", elapsed);
    println!("Visited {} nodes", result.order.len());
    println!();

    // Covariance computation
    println!("--- Covariance Matrix (100x100) ---");
    let cov_data: Vec<f64> = (0..10000).map(|i| (i as f64 * 0.1).sin()).collect();
    let data = Matrix::from_vec(&cov_data, 100, 100, Storage::Column);

    let start = Instant::now();
    let cov = stats::covariance(&data);
    let elapsed = start.elapsed();
    println!("Covariance computed in {:?}", elapsed);
    println!("Covariance matrix: {}x{}", cov.rows(), cov.cols());
    println!();

    // Particle Swarm Optimization
    println!("--- PSO Optimization (10D Sphere Function) ---");
    use mathlib::argmin::{PsoOptions, pso};
    let cost = |x: &[f64]| x.iter().map(|&v| v * v).sum::<f64>();
    let low = vec![-10.0; 10];
    let high = vec![10.0; 10];

    let start = Instant::now();
    let result = pso((low, high), 50, cost, 100, Some(PsoOptions::default()));
    let elapsed = start.elapsed();
    println!("PSO completed in {:?}", elapsed);
    println!("Best cost: {:.6}", result.best_cost);
    println!("Iterations: {}", result.iterations);
    println!();

    // 1D Convolution
    println!("--- 1D Convolution (10000 samples) ---");
    use mathlib::transforms::conv_1d;
    let signal: Vec<f64> = (0..10000).map(|i| (i as f64 * 0.1).sin()).collect();
    let kernel = vec![0.25, 0.5, 0.25];

    let start = Instant::now();
    let result = conv_1d(&signal, &kernel);
    let elapsed = start.elapsed();
    println!("Convolution completed in {:?}", elapsed);
    println!("Output length: {}", result.len());
    println!();

    // Vector operations
    println!("--- Vector Dot Product (100000 elements) ---");
    let a_data: Vec<f64> = (0..100000).map(|i| i as f64).collect();
    let b_data: Vec<f64> = (0..100000).map(|i| (i * 2) as f64).collect();
    let a = Vector::from_slice(&a_data);
    let b = Vector::from_slice(&b_data);

    let start = Instant::now();
    let dot = a.dot(&b);
    let elapsed = start.elapsed();
    println!("Dot product completed in {:?}", elapsed);
    println!("Result: {:.2e}", dot);
    println!();

    println!("=== Demo Complete ===");
    println!("\nTip: Run with --features parallel to enable parallel execution");
    println!("     Run with --features full to enable both parallel and SIMD");
}
