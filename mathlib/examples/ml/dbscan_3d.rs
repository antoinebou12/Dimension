//! Example: DBSCAN clustering on 3D points (rows = samples, cols = x,y,z).

use mathlib::{Matrix, NOISE, Storage, dbscan};

fn main() {
    let mut data = Matrix::with_storage(8, 3, Storage::Column);
    // Blob 1 near (0, 0, 0)
    data.set(0, 0, 0.0);
    data.set(0, 1, 0.0);
    data.set(0, 2, 0.0);
    data.set(1, 0, 0.1);
    data.set(1, 1, 0.0);
    data.set(1, 2, 0.0);
    data.set(2, 0, 0.0);
    data.set(2, 1, 0.1);
    data.set(2, 2, 0.0);
    // Blob 2 near (5, 5, 5)
    data.set(3, 0, 5.0);
    data.set(3, 1, 5.0);
    data.set(3, 2, 5.0);
    data.set(4, 0, 5.1);
    data.set(4, 1, 5.0);
    data.set(4, 2, 5.0);
    data.set(5, 0, 5.0);
    data.set(5, 1, 5.1);
    data.set(5, 2, 5.0);
    // Noise points
    data.set(6, 0, 100.0);
    data.set(6, 1, 100.0);
    data.set(6, 2, 100.0);
    data.set(7, 0, 200.0);
    data.set(7, 1, 200.0);
    data.set(7, 2, 200.0);

    let eps = 1.0;
    let min_pts = 2;
    let result = dbscan(&data, eps, min_pts);

    let noise_count = result.labels().iter().filter(|&&l| l == NOISE).count();
    let mut cluster_sizes: Vec<usize> = (0..result.n_clusters()).map(|_| 0).collect();
    for &l in result.labels() {
        if l != NOISE {
            cluster_sizes[l] += 1;
        }
    }

    println!(
        "DBSCAN 3D on {} points, eps = {}, min_pts = {}:",
        data.rows(),
        eps,
        min_pts
    );
    println!("  n_clusters: {}", result.n_clusters());
    println!("  noise count: {}", noise_count);
    println!("  labels: {:?}", result.labels());
    for (c, &size) in cluster_sizes.iter().enumerate() {
        println!("  cluster {} size: {}", c, size);
    }
}
