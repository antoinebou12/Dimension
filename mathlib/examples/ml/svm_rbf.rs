//! Example: RBF-kernel SVM for binary classification on non-linearly separable 2D data.
//!
//! Uses an XOR-like layout: class +1 in corners (0,0) and (1,1), class -1 in (1,0) and (0,1).

use mathlib::{Matrix, Storage, SvmOptions, svm_rbf};

fn main() {
    // XOR-like: +1 at (0,0), (1,1); -1 at (1,0), (0,1). Add a few more points for stability.
    let mut x = Matrix::with_storage(8, 2, Storage::Column);
    // Class +1 (corners)
    x.set(0, 0, 0.0);
    x.set(0, 1, 0.0);
    x.set(1, 0, 1.0);
    x.set(1, 1, 1.0);
    x.set(2, 0, 0.1);
    x.set(2, 1, 0.1);
    x.set(3, 0, 0.9);
    x.set(3, 1, 0.9);
    // Class -1 (other corners)
    x.set(4, 0, 1.0);
    x.set(4, 1, 0.0);
    x.set(5, 0, 0.0);
    x.set(5, 1, 1.0);
    x.set(6, 0, 0.9);
    x.set(6, 1, 0.1);
    x.set(7, 0, 0.1);
    x.set(7, 1, 0.9);

    let y = [1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0];

    let gamma = 2.0;
    let opts = SvmOptions {
        c: 10.0,
        max_iters: 10_000,
        tol: 1e-3,
    };
    let result = svm_rbf(&x, &y, gamma, Some(opts)).expect("svm_rbf fit");
    println!("RBF SVM (8 samples, 2 features), gamma = {}", gamma);
    println!("  n_support_vectors: {}", result.n_support_vectors());
    println!("  bias: {}", result.bias());
    let pred = result.predict(&x);
    let correct: usize = pred
        .iter()
        .zip(y.iter())
        .filter(|(a, b)| (*a - *b).abs() < 0.5)
        .count();
    println!("  Predictions on training data: {:?}", pred);
    println!("  Accuracy on training set: {}/{}", correct, y.len());
}
