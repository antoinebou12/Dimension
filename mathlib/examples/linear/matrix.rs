//! Example: create a matrix, set elements, transpose, multiply, and print.

use mathlib::{Matrix, Storage};

fn main() {
    let mut a = Matrix::with_storage(3, 3, Storage::Column);
    a.set(0, 0, 1.0);
    a.set(0, 1, 2.0);
    a.set(0, 2, 3.0);
    a.set(1, 0, 4.0);
    a.set(1, 1, 5.0);
    a.set(1, 2, 6.0);
    a.set(2, 0, 7.0);
    a.set(2, 1, 8.0);
    a.set(2, 2, 9.0);

    println!("Matrix A (3x3):");
    for i in 0..3 {
        for j in 0..3 {
            print!("  {}", a.get(i, j));
        }
        println!();
    }

    let at = a.transpose();
    println!("\nTranspose A^T:");
    for i in 0..3 {
        for j in 0..3 {
            print!("  {}", at.get(i, j));
        }
        println!();
    }

    let c = &a * &at;
    println!("\nA * A^T (3x3):");
    for i in 0..3 {
        for j in 0..3 {
            print!("  {}", c.get(i, j));
        }
        println!();
    }
}
