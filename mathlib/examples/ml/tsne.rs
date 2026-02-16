//! Example: t-SNE dimensionality reduction on synthetic data.

use mathlib::{Matrix, Storage, TsneOptions, tsne};

fn main() {
    let mut data = Matrix::with_storage(60, 5, Storage::Column);
    for i in 0..60 {
        for j in 0..5 {
            data.set(i, j, (i as f64) * 0.2 + (j as f64) * 0.3);
        }
    }

    let opts = TsneOptions {
        n_components: 2,
        perplexity: 15.0,
        max_iters: 200,
        seed: Some(42),
        ..Default::default()
    };
    let embedding = tsne(&data, &opts);

    println!(
        "t-SNE of {}×{} data -> {}×{} embedding:",
        data.rows(),
        data.cols(),
        embedding.rows(),
        embedding.cols()
    );
    let mut min0 = f64::INFINITY;
    let mut max0 = f64::NEG_INFINITY;
    let mut min1 = f64::INFINITY;
    let mut max1 = f64::NEG_INFINITY;
    for i in 0..embedding.rows() {
        let a = embedding.get(i, 0);
        let b = embedding.get(i, 1);
        min0 = min0.min(a);
        max0 = max0.max(a);
        min1 = min1.min(b);
        max1 = max1.max(b);
    }
    println!("  X range: [{:.3}, {:.3}]", min0, max0);
    println!("  Y range: [{:.3}, {:.3}]", min1, max1);
    println!("  Sample points (first 5):");
    for i in 0..5 {
        println!(
            "    ({:.3}, {:.3})",
            embedding.get(i, 0),
            embedding.get(i, 1)
        );
    }
}
