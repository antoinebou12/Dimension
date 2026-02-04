//! Example: economical SVD of a small matrix.

use mathlib::{Matrix, Storage, svd_econ};

fn main() {
    let mut a = Matrix::with_storage(3, 2, Storage::Column);
    a.set(0, 0, 1.0);
    a.set(1, 0, 0.0);
    a.set(2, 0, 0.0);
    a.set(0, 1, 0.0);
    a.set(1, 1, 2.0);
    a.set(2, 1, 0.0);

    let econ = svd_econ(&a);
    let u = econ.u();
    let sigma = econ.sigma();
    let v = econ.v();

    println!("SVD of 3x2 matrix:");
    println!("  U: {}x{}", u.rows(), u.cols());
    println!("  sigma: {} values", sigma.rows());
    for j in 0..sigma.rows() {
        println!("    sigma[{}] = {}", j, sigma.get(j));
    }
    println!("  V: {}x{}", v.rows(), v.cols());
    println!("  First column of V: ({}, {})", v.get(0, 0), v.get(1, 0));
}
