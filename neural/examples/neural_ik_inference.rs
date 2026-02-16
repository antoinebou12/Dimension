//! Example: use chain config and normalization for neural IK inference.
//!
//! Run: cargo run -p neural --example neural_ik_inference
//! With ONNX: cargo run -p neural --example neural_ik_inference --features onnx

use neural::{denormalize_joints, normalize_position, ChainConfig};

fn main() {
    let chain = ChainConfig::new(3, false).with_workspace([-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]);
    let target = [0.5, 0.0, 0.0];
    let normalized = normalize_position(target, &chain);
    println!("Target {:?} -> normalized {:?}", target, normalized);
    // In a real app you'd run model inference on `normalized`, then:
    let fake_output = vec![-0.5f32, 0.0, 0.5]; // normalized joint angles
    let mut joints = vec![0.0; 3];
    denormalize_joints(&fake_output, &chain, &mut joints);
    println!("Denormalized joints (rad): {:?}", joints);
}
