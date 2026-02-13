//! Example: 3D curve evaluation (linear, Bézier, Hermite, B-spline).
//!
//! Run with: `cargo run -p mathlib --example curve`

use mathlib::math::curve::{bezier_curve, bspline_curve, hermite_curve, linear_curve};

fn main() {
    let steps = 5;
    let ts: Vec<f32> = (0..=steps).map(|i| i as f32 / steps as f32).collect();

    println!("Linear segment [0,0,0] -> [1,2,0]:");
    for &t in &ts {
        let p = linear_curve([0.0, 0.0, 0.0], [1.0, 2.0, 0.0], t);
        println!("  t = {:.2}: [{:.4}, {:.4}, {:.4}]", t, p[0], p[1], p[2]);
    }

    println!("\nCubic Bézier (4 control points):");
    let p0 = [0.0, 0.0, 0.0];
    let p1 = [0.33, 0.5, 0.0];
    let p2 = [0.66, 0.5, 0.0];
    let p3 = [1.0, 0.0, 0.0];
    for &t in &ts {
        let p = bezier_curve(p0, p1, p2, p3, t);
        println!("  t = {:.2}: [{:.4}, {:.4}, {:.4}]", t, p[0], p[1], p[2]);
    }

    println!("\nCubic Hermite (p0, p1, m0, m1):");
    let p0 = [0.0, 0.0, 0.0];
    let p1 = [1.0, 0.0, 0.0];
    let m0 = [0.0, 1.0, 0.0];
    let m1 = [0.0, -1.0, 0.0];
    for &t in &ts {
        let p = hermite_curve(p0, p1, m0, m1, t);
        println!("  t = {:.2}: [{:.4}, {:.4}, {:.4}]", t, p[0], p[1], p[2]);
    }

    println!("\nCubic B-spline (4 control points):");
    let p0 = [0.0, 0.0, 0.0];
    let p1 = [0.33, 0.0, 0.0];
    let p2 = [0.66, 0.0, 0.0];
    let p3 = [1.0, 0.0, 0.0];
    for &t in &ts {
        let p = bspline_curve(p0, p1, p2, p3, t);
        println!("  t = {:.2}: [{:.4}, {:.4}, {:.4}]", t, p[0], p[1], p[2]);
    }
}
