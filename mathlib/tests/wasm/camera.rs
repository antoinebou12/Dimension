//! Integration tests for wasm camera bindings (demo: Camera matrices).

#![cfg(feature = "wasm")]

use mathlib::wasm::WasmCg;

#[test]
fn wasm_demo_camera() {
    // Demo: perspective (aspect=16/9, fov=π/4, near=0.1, far=100), look-at RH (eye 0,0,5 → target 0,0,0, up 0,1,0)
    let aspect = 16.0_f32 / 9.0;
    let fov_y = std::f32::consts::FRAC_PI_4;
    let persp = WasmCg::new_perspective(aspect, fov_y, 0.1, 100.0);
    assert_eq!(persp.rows(), 4);
    assert_eq!(persp.cols(), 4);
    let p = persp.to_array();
    assert!(p[0] > 1.0 && p[5] > 2.0);
    assert!(p[10] < 0.0);
    let look_at = WasmCg::look_at_rh(0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
    assert_eq!(look_at.rows(), 4);
    assert_eq!(look_at.cols(), 4);
    let la = look_at.to_array();
    assert!((la[14] + 5.0).abs() < 0.01);
}
