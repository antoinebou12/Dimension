//! Example: create vectors, set elements, dot product, norm.

use mathlib::Vector;

fn main() {
    let mut u = Vector::with_capacity(3);
    u.set(0, 1.0);
    u.set(1, 2.0);
    u.set(2, 3.0);

    let mut v = Vector::with_capacity(3);
    v.set(0, 4.0);
    v.set(1, 5.0);
    v.set(2, 6.0);

    println!("Vector u: ({}, {}, {})", u.get(0), u.get(1), u.get(2));
    println!("Vector v: ({}, {}, {})", v.get(0), v.get(1), v.get(2));

    let dot = u.dot(&v);
    println!("\nu · v = {}", dot);

    let norm_u = u.norm();
    let norm_v = v.norm();
    println!("||u|| = {}", norm_u);
    println!("||v|| = {}", norm_v);
}
