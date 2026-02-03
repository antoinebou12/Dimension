//! Example: easing and interpolation (scalar easings, hermite, bspline, slerp).
//!
//! Run with: `cargo run -p mathlib --example easing`

use mathlib::Quat4f;
use mathlib::cg::vector3;
use mathlib::easing::{bspline, ease_in_out_cubic, ease_out_bounce, hermite, lerp, linear};

fn main() {
    let ts = [0.0_f64, 0.25, 0.5, 0.75, 1.0];

    println!("Scalar easing (t in [0, 1]):");
    println!("  t      | linear | ease_in_out_cubic | ease_out_bounce");
    println!("  ------ + ------ + ------------------ + ---------------");
    for &t in &ts {
        println!(
            "  {:.2}   | {:.4} | {:18.4} | {:14.4}",
            t,
            linear(t),
            ease_in_out_cubic(t),
            ease_out_bounce(t)
        );
    }

    println!("\nlerp(0, 10, t):");
    for &t in &ts {
        println!("  t = {} -> {}", t, lerp(0.0, 10.0, t));
    }

    println!("\nhermite(0, 1, 0, 0, t) (linear segment):");
    for &t in &ts {
        println!("  t = {} -> {}", t, hermite(0.0, 1.0, 0.0, 0.0, t));
    }

    let pts = [0.0_f64, 1.0, 2.0, 3.0];
    println!("\nbspline(&[0, 1, 2, 3], t):");
    for &t in &ts {
        println!("  t = {} -> {}", t, bspline(&pts, t));
    }

    println!("\nQuat4f::slerp (Y-axis 0 -> π/2 at t = 0.5):");
    let axis = vector3(0.0_f32, 1.0, 0.0);
    let q0 = Quat4f::from_axis_angle(&axis, 0.0);
    let q1 = Quat4f::from_axis_angle(&axis, std::f32::consts::FRAC_PI_2);
    let q_mid = q0.slerp(&q1, 0.5);
    println!(
        "  slerp(q0, q1, 0.5) = (w={}, x={}, y={}, z={})",
        q_mid.w, q_mid.x, q_mid.y, q_mid.z
    );
}
