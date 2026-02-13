//! Example: convex hull of 2D points and point-in-polygon test.
//!
//! Run: `cargo run -p collision --example convex_hull2d`

use collision::{convex_hull_2d, point_in_polygon_2d};

fn main() {
    let points = [
        [-1.0, 1.0],
        [-0.5, -0.5],
        [0.0, 0.5],
        [0.5, -0.5],
        [1.0, 1.0],
    ];
    let hull = convex_hull_2d(&points);
    println!("Convex hull ({} vertices):", hull.len());
    for (i, p) in hull.iter().enumerate() {
        println!("  {}: [{}, {}]", i, p[0], p[1]);
    }
    let test_inside = [0.0, 0.0];
    let test_outside = [2.0, 2.0];
    println!(
        "Point {:?} inside hull: {}",
        test_inside,
        point_in_polygon_2d(&test_inside, &hull)
    );
    println!(
        "Point {:?} inside hull: {}",
        test_outside,
        point_in_polygon_2d(&test_outside, &hull)
    );
}
