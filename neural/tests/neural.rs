//! Basic tests for neural crate (no train/onnx required).

use neural::{denormalize_position, normalize_position, ChainConfig};

#[test]
fn chain_config_input_output_size() {
    let chain = ChainConfig::new(5, false);
    assert_eq!(chain.input_size(), 3);
    assert_eq!(chain.output_size(), 5);

    let chain_with_state = ChainConfig::new(4, true);
    assert_eq!(chain_with_state.input_size(), 3 + 4);
    assert_eq!(chain_with_state.output_size(), 4);
}

#[test]
fn normalize_denormalize_position() {
    let chain = ChainConfig::new(1, false).with_workspace([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
    let pos = [1.0, 1.0, 1.0];
    let norm = normalize_position(pos, &chain);
    assert!((norm[0] - 0.0).abs() < 1e-5);
    let back = denormalize_position(norm, &chain);
    assert!((back[0] - pos[0]).abs() < 1e-5);
}
