//! Example: Linear SVM for binary classification (rows = samples, cols = features).

use mathlib::{Matrix, Storage, SvmOptions, svm};

fn main() {
    // Simple 2D separable data: class +1 above line, -1 below.
    // 4 samples, 2 features.
    let mut x = Matrix::with_storage(6, 2, Storage::Column);
    // Class +1 (upper)
    x.set(0, 0, 1.0);
    x.set(0, 1, 2.0);
    x.set(1, 0, 2.0);
    x.set(1, 1, 3.0);
    x.set(2, 0, 2.0);
    x.set(2, 1, 2.0);
    // Class -1 (lower)
    x.set(3, 0, 0.0);
    x.set(3, 1, 0.0);
    x.set(4, 0, 1.0);
    x.set(4, 1, 0.0);
    x.set(5, 0, 0.0);
    x.set(5, 1, 1.0);

    let y = [1.0, 1.0, 1.0, -1.0, -1.0, -1.0];

    let opts = SvmOptions {
        c: 10.0,
        max_iters: 10_000,
        tol: 1e-3,
    };
    let result = svm(&x, &y, Some(opts)).expect("svm fit");
    let w = result.weights();
    let b = result.bias();
    println!("Linear SVM (6 samples, 2 features):");
    println!("  w = ({}, {})", w.get(0), w.get(1));
    println!("  b = {}", b);
    let pred = result.predict(&x);
    println!("  Predictions on training data: {:?}", pred);
}
