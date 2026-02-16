//! Train a neural IK model for a serial chain.
//!
//! Run with: cargo run --bin train --features train
//! Optional: --features "train wgpu" for GPU training.

#![cfg(feature = "train")]

use burn::data::dataset::Dataset;
use neural::dataset::IkDataset;
use neural::model::NeuralIkConfig;
use neural::training::train_neural_ik;
use neural::ChainConfig;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use kinematics::joints::Revolute2dJoint;
    use kinematics::{Armature, JointData, JointVariant};

    // Build a simple 3-DOF arm: root -> link1 -> link2 -> end-effector
    let root = JointData::new(JointVariant::Revolute2d(Revolute2dJoint::at(0.0, 0.0, 0.0)));
    let mut arm = Armature::new(root);
    arm.add_child(
        0,
        1,
        JointData::new(JointVariant::Revolute2d(Revolute2dJoint::at(1.0, 0.0, 0.0))),
    );
    arm.add_child(
        1,
        2,
        JointData::new(JointVariant::Revolute2d(Revolute2dJoint::at(1.0, 0.0, 0.0))),
    );
    arm.add_child(
        2,
        3,
        JointData::new(JointVariant::Revolute2d(Revolute2dJoint::at(1.0, 0.0, 0.0))),
    );
    let end_effector_idx = 3;
    let dof = arm.pack().len();
    assert_eq!(dof, 3);

    let chain = ChainConfig::new(dof, false).with_workspace([-2.0, -2.0, -0.1], [2.0, 2.0, 0.1]);
    let num_samples = 2000_usize;
    let dataset =
        IkDataset::generate_with_halley(&mut arm, end_effector_idx, chain.clone(), num_samples, 42);
    println!("Generated {} IK samples", dataset.len());

    type Backend = burn::backend::Autodiff<burn::backend::NdArray>;
    let device = burn::backend::ndarray::NdArrayDevice::default();
    let model_config = NeuralIkConfig::new(chain)
        .with_hidden_size(128)
        .with_num_hidden_layers(2);

    let model = train_neural_ik::<Backend>(dataset, model_config, 50, 64, 1e-3, device);
    println!(
        "Training done. Save the model with Burn's record API for inference or export to ONNX."
    );
    let _ = model;
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eprintln!("Train binary is not supported on wasm32; train on host and load ONNX in WASM.");
}
