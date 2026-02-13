//! WasmDualQuat — Dual quaternion rigid transforms for JavaScript.

use wasm_bindgen::prelude::*;

use crate::Quat4f;
use crate::cg::vector3;
use crate::dual_quaternion::DualQuat4f;

use super::matrix::WasmMatrix32;

/// Dual quaternion for rigid transforms (rotation + translation), usable from JavaScript.
#[wasm_bindgen(js_name = WasmDualQuat)]
pub struct WasmDualQuat {
    inner: DualQuat4f,
}

#[wasm_bindgen]
impl WasmDualQuat {
    /// Create from 8 components: [real_w, real_x, real_y, real_z, dual_w, dual_x, dual_y, dual_z].
    #[wasm_bindgen(js_name = fromArray)]
    pub fn from_array(data: &[f32]) -> Result<WasmDualQuat, JsError> {
        if data.len() != 8 {
            return Err(JsError::new(&format!(
                "DualQuat fromArray expects 8 elements, got {}",
                data.len()
            )));
        }
        Ok(WasmDualQuat {
            inner: DualQuat4f {
                real_w: data[0],
                real_x: data[1],
                real_y: data[2],
                real_z: data[3],
                dual_w: data[4],
                dual_x: data[5],
                dual_y: data[6],
                dual_z: data[7],
            },
        })
    }

    /// Return components as array [real_w, real_x, real_y, real_z, dual_w, dual_x, dual_y, dual_z].
    #[wasm_bindgen(js_name = toArray)]
    pub fn to_array(&self) -> Vec<f32> {
        vec![
            self.inner.real_w,
            self.inner.real_x,
            self.inner.real_y,
            self.inner.real_z,
            self.inner.dual_w,
            self.inner.dual_x,
            self.inner.dual_y,
            self.inner.dual_z,
        ]
    }

    /// Transform a 3D point (x, y, z). Returns [x', y', z'].
    #[wasm_bindgen(js_name = transformPoint)]
    pub fn transform_point(&self, x: f32, y: f32, z: f32) -> Vec<f32> {
        let p = vector3(x, y, z);
        let out = self.inner.transform_point(&p);
        vec![out.get(0), out.get(1), out.get(2)]
    }

    /// Compose with another dual quaternion: this * other.
    pub fn mul(&self, other: &WasmDualQuat) -> WasmDualQuat {
        WasmDualQuat {
            inner: self.inner * other.inner,
        }
    }

    /// Extract 4×4 rigid transform matrix.
    #[wasm_bindgen(js_name = toMatrix4)]
    pub fn to_matrix4(&self) -> WasmMatrix32 {
        WasmMatrix32::from_inner(self.inner.to_matrix4())
    }

    /// Build from rotation quaternion (4 components w,x,y,z) and translation (3 components x,y,z).
    #[wasm_bindgen(js_name = fromRotationAndTranslation)]
    pub fn from_rotation_and_translation(quat: &[f32], t: &[f32]) -> Result<WasmDualQuat, JsError> {
        if quat.len() != 4 {
            return Err(JsError::new(&format!(
                "Quaternion must have 4 elements (w,x,y,z), got {}",
                quat.len()
            )));
        }
        if t.len() != 3 {
            return Err(JsError::new(&format!(
                "Translation must have 3 elements (x,y,z), got {}",
                t.len()
            )));
        }
        let q = Quat4f {
            w: quat[0],
            x: quat[1],
            y: quat[2],
            z: quat[3],
        };
        let tv = vector3(t[0], t[1], t[2]);
        let dq = DualQuat4f::from_rotation_and_translation(&q, &tv);
        Ok(WasmDualQuat { inner: dq })
    }

    /// Identity rigid transform.
    #[wasm_bindgen(js_name = identity)]
    pub fn identity() -> WasmDualQuat {
        WasmDualQuat {
            inner: DualQuat4f::identity(),
        }
    }
}
