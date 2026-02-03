//! Example: solve Ax = b for a 2x2 system.

use mathlib::{Matrix, Storage, Vector, solve};

fn main() {
    let mut a = Matrix::with_storage(2, 2, Storage::Column);
    a.set(0, 0, 2.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 1.0);
    a.set(1, 1, 2.0);

    let mut b = Vector::with_capacity(2);
    b.set(0, 3.0);
    b.set(1, 3.0);

    let x = solve(&a, &b).unwrap();
    println!("Solve Ax = b:");
    println!("  A = [[2, 1], [1, 2]], b = [3, 3]");
    println!("  x = [{}, {}]", x.get(0), x.get(1));

    let ax0 = a.get(0, 0) * x.get(0) + a.get(0, 1) * x.get(1);
    let ax1 = a.get(1, 0) * x.get(0) + a.get(1, 1) * x.get(1);
    println!("  A*x = [{}, {}] (should match b)", ax0, ax1);
}
