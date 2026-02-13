//! WASM bindings integration test. Run with: cargo test -p kinematics --features wasm wasm

#![cfg(feature = "wasm")]

use kinematics::wasm::WasmArmature;

#[test]
fn wasm_armature_chain() {
    let arm = WasmArmature::new(4, 1.0).unwrap();
    assert_eq!(arm.num_nodes(), 4);
    let pos = arm.get_end_effector_position(3).unwrap();
    assert_eq!(pos.len(), 3);
    assert!((pos[0] - 3.0).abs() < 0.1);
}

#[test]
fn wasm_armature_ik() {
    let mut arm = WasmArmature::new(3, 1.0).unwrap();
    let err = arm.solve_jacobian_ik(2, 2.0, 0.5, 0.0, 20).unwrap();
    assert!(err < 1.0);
}
