//! Example: K-means clustering on a small 2D dataset (rows = samples, cols = features).

use mathlib::{Matrix, Storage, kmeans};

fn main() {
    let mut data = Matrix::with_storage(6, 2, Storage::Column);
    data.set(0, 0, 0.0);
    data.set(0, 1, 0.0);
    data.set(1, 0, 1.0);
    data.set(1, 1, 0.0);
    data.set(2, 0, 2.0);
    data.set(2, 1, 0.0);
    data.set(3, 0, 10.0);
    data.set(3, 1, 10.0);
    data.set(4, 0, 11.0);
    data.set(4, 1, 10.0);
    data.set(5, 0, 12.0);
    data.set(5, 1, 10.0);

    let k = 2;
    let result = kmeans(&data, k, Some(50));

    println!("K-means on {} samples, k = {}:", data.rows(), k);
    println!("  n_clusters: {}", result.n_clusters());
    println!("  labels: {:?}", result.labels());
    println!("  centroids:");
    for c in 0..result.n_clusters() {
        println!(
            "    cluster {}: ({}, {})",
            c,
            result.centroids().get(c, 0),
            result.centroids().get(c, 1)
        );
    }
}
