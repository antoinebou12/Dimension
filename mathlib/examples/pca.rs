//! Example: PCA on a small data matrix (rows = samples, cols = features).

use mathlib::{Matrix, Storage, pca};

fn main() {
    let mut data = Matrix::with_storage(10, 4, Storage::Column);
    for i in 0..10 {
        for j in 0..4 {
            data.set(i, j, (i as f64) * 0.5 + (j as f64));
        }
    }

    let result = pca(&data, Some(2));
    let mean = result.mean();
    let ev = result.explained_variance();

    println!("PCA (first 2 components) of 10x4 data:");
    println!(
        "  Mean (4 features): ({}, {}, {}, {})",
        mean.get(0),
        mean.get(1),
        mean.get(2),
        mean.get(3)
    );
    println!("  n_components: {}", result.n_components());
    println!("  Explained variance[0]: {}", ev.get(0));
    println!("  Explained variance[1]: {}", ev.get(1));
}
