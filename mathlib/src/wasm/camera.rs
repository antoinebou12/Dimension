//! WasmCg — Camera and projection helpers for JavaScript.

use wasm_bindgen::prelude::*;

use crate::cg::{look_at_lh, look_at_rh, new_orthographic, new_perspective, new_translation};

use super::matrix::WasmMatrix32;

/// Camera and projection matrix builders (4×4, column-major).
#[wasm_bindgen(js_name = WasmCg)]
pub struct WasmCg;

#[wasm_bindgen]
impl WasmCg {
    /// Right-handed look-at view matrix: eye (x,y,z), target (x,y,z), up (x,y,z).
    #[wasm_bindgen(js_name = lookAtRh)]
    pub fn look_at_rh(
        eye_x: f32,
        eye_y: f32,
        eye_z: f32,
        target_x: f32,
        target_y: f32,
        target_z: f32,
        up_x: f32,
        up_y: f32,
        up_z: f32,
    ) -> WasmMatrix32 {
        let eye = crate::vector3(eye_x, eye_y, eye_z);
        let target = crate::vector3(target_x, target_y, target_z);
        let up = crate::vector3(up_x, up_y, up_z);
        WasmMatrix32::from_inner(look_at_rh(&eye, &target, &up))
    }

    /// Left-handed look-at view matrix.
    #[wasm_bindgen(js_name = lookAtLh)]
    pub fn look_at_lh(
        eye_x: f32,
        eye_y: f32,
        eye_z: f32,
        target_x: f32,
        target_y: f32,
        target_z: f32,
        up_x: f32,
        up_y: f32,
        up_z: f32,
    ) -> WasmMatrix32 {
        let eye = crate::vector3(eye_x, eye_y, eye_z);
        let target = crate::vector3(target_x, target_y, target_z);
        let up = crate::vector3(up_x, up_y, up_z);
        WasmMatrix32::from_inner(look_at_lh(&eye, &target, &up))
    }

    /// Perspective projection: aspect = width/height, fov_y_rad vertical FOV (radians), near/far positive.
    #[wasm_bindgen(js_name = newPerspective)]
    pub fn new_perspective(aspect: f32, fov_y_rad: f32, near: f32, far: f32) -> WasmMatrix32 {
        WasmMatrix32::from_inner(new_perspective(aspect, fov_y_rad, near, far))
    }

    /// Orthographic projection: left, right, bottom, top, near, far.
    #[wasm_bindgen(js_name = newOrthographic)]
    pub fn new_orthographic(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> WasmMatrix32 {
        WasmMatrix32::from_inner(new_orthographic(left, right, bottom, top, near, far))
    }

    /// Translation matrix with (x, y, z).
    #[wasm_bindgen(js_name = newTranslation)]
    pub fn new_translation(x: f32, y: f32, z: f32) -> WasmMatrix32 {
        let t = crate::vector3(x, y, z);
        WasmMatrix32::from_inner(new_translation(&t))
    }
}
