//! Example: dual quaternions for rigid transforms (compose, transform point, roundtrip to matrix).
//!
//! Run with: `cargo run -p mathlib --example dual_quaternion`

use mathlib::cg::{transform_point, vector3};
use mathlib::{DualQuat4f, Quat4f};

fn main() {
    let rot_a = Quat4f::from_axis_angle(&vector3(0.0, 1.0, 0.0), 0.3);
    let t_a = vector3(1.0, 0.0, 0.0);
    let dq_a = DualQuat4f::from_rotation_and_translation(&rot_a, &t_a);

    let rot_b = Quat4f::from_axis_angle(&vector3(1.0, 0.0, 0.0), 0.1);
    let t_b = vector3(0.0, 0.5, 0.0);
    let dq_b = DualQuat4f::from_rotation_and_translation(&rot_b, &t_b);

    let composed = dq_a * dq_b;
    let p = vector3(0.0, 0.0, 0.0);
    let out_dq = composed.transform_point(&p);
    let m = composed.to_matrix4();
    let out_m = transform_point(&m, &p);

    println!("Dual quaternion rigid transform example");
    println!("  dq_a = rotation(Y, 0.3) * translation(1, 0, 0)");
    println!("  dq_b = rotation(X, 0.1) * translation(0, 0.5, 0)");
    println!("  composed = dq_a * dq_b");
    println!(
        "  transform (0,0,0) via DQ: ({}, {}, {})",
        out_dq.get(0),
        out_dq.get(1),
        out_dq.get(2)
    );
    println!(
        "  transform (0,0,0) via 4x4: ({}, {}, {})",
        out_m.get(0),
        out_m.get(1),
        out_m.get(2)
    );
    println!(
        "  Roundtrip: from_matrix4(to_matrix4(composed)) matches: {}",
        {
            let dq_round = DualQuat4f::from_matrix4(&m);
            let out_round = dq_round.transform_point(&p);
            (out_round.get(0) - out_dq.get(0)).abs() < 1e-5
                && (out_round.get(1) - out_dq.get(1)).abs() < 1e-5
                && (out_round.get(2) - out_dq.get(2)).abs() < 1e-5
        }
    );
}
