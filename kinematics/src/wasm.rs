//! WebAssembly bindings for kinematics.
//!
//! Enable with `--features wasm`. Build with:
//! ```bash
//! wasm-pack build --target web --features wasm
//! ```
//!
//! Exposes [`WasmArmature`] for building kinematic chains, forward kinematics,
//! and Jacobian IK from JavaScript.

use wasm_bindgen::prelude::*;

use crate::armature::{Armature, JointData, JointVariant};
use crate::ik::JacobianIk;
use crate::joints::{Prismatic2dJoint, Revolute2dJoint, RevoluteJoint};
use mathlib::Vector3f;
use mathlib::cg::vector3;

/// Armature accessible from JavaScript: revolute chains, forward kinematics, Jacobian IK.
#[wasm_bindgen]
pub struct WasmArmature {
    inner: Armature,
}

#[wasm_bindgen]
impl WasmArmature {
    /// Creates a chain of `n` revolute joints along X, axis Y, link length `link_length`.
    #[wasm_bindgen(constructor)]
    pub fn new(n: usize, link_length: f32) -> Result<WasmArmature, JsError> {
        if n == 0 {
            return Err(JsError::new("n must be >= 1"));
        }
        let root = JointData::new(JointVariant::Revolute(RevoluteJoint::new(
            vector3(0.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            0.0,
        )));
        let mut arm = Armature::new(root);
        for i in 1..n {
            arm.add_child(
                i - 1,
                i,
                JointData::new(JointVariant::Revolute(RevoluteJoint::new(
                    vector3(link_length, 0.0, 0.0),
                    (0.0, 1.0, 0.0),
                    0.0,
                ))),
            );
        }
        arm.update_kinematics();
        Ok(WasmArmature { inner: arm })
    }

    /// Adds a revolute joint: parent idx, child idx, translation (x,y,z), axis (x,y,z), initial angle (rad).
    #[wasm_bindgen(js_name = addRevoluteJoint)]
    pub fn add_revolute_joint(
        &mut self,
        parent_idx: usize,
        child_idx: usize,
        tx: f32,
        ty: f32,
        tz: f32,
        ax: f32,
        ay: f32,
        az: f32,
        angle: f32,
    ) -> Result<(), JsError> {
        let t = vector3(tx, ty, tz);
        let data = JointData::new(JointVariant::Revolute(RevoluteJoint::new(
            t,
            (ax, ay, az),
            angle,
        )));
        self.inner.add_child(parent_idx, child_idx, data);
        Ok(())
    }

    /// Adds a 2D revolute joint (XY plane, rotation about Z): parent idx, child idx, position (x, y), initial angle (rad).
    #[wasm_bindgen(js_name = addRevolute2dJoint)]
    pub fn add_revolute2d_joint(
        &mut self,
        parent_idx: usize,
        child_idx: usize,
        x: f32,
        y: f32,
        angle: f32,
    ) -> Result<(), JsError> {
        let data = JointData::new(JointVariant::Revolute2d(Revolute2dJoint::at(x, y, angle)));
        self.inner.add_child(parent_idx, child_idx, data);
        Ok(())
    }

    /// Adds a 2D prismatic joint (XY plane): parent idx, child idx, position (x, y), axis (ax, ay) unit vector, offset.
    #[wasm_bindgen(js_name = addPrismatic2dJoint)]
    pub fn add_prismatic2d_joint(
        &mut self,
        parent_idx: usize,
        child_idx: usize,
        x: f32,
        y: f32,
        axis_x: f32,
        axis_y: f32,
        offset: f32,
    ) -> Result<(), JsError> {
        let data = JointData::new(JointVariant::Prismatic2d(Prismatic2dJoint::at(
            x,
            y,
            (axis_x, axis_y),
            offset,
        )));
        self.inner.add_child(parent_idx, child_idx, data);
        Ok(())
    }

    /// Updates all world transforms (forward kinematics).
    #[wasm_bindgen(js_name = updateKinematics)]
    pub fn update_kinematics(&mut self) {
        self.inner.update_kinematics();
    }

    /// Returns end-effector position [x, y, z] for the given node index.
    #[wasm_bindgen(js_name = getEndEffectorPosition)]
    pub fn get_end_effector_position(&self, node_idx: usize) -> Result<Vec<f32>, JsError> {
        if node_idx >= self.inner.tree().num_nodes() {
            return Err(JsError::new(&format!(
                "Node index {} out of range [0, {})",
                node_idx,
                self.inner.tree().num_nodes()
            )));
        }
        let v = self.inner.end_effector_position(node_idx);
        Ok(vec![v.get(0), v.get(1), v.get(2)])
    }

    /// Packs joint DOFs into a flat array (Float32Array in JS).
    #[wasm_bindgen(js_name = pack)]
    pub fn pack(&self) -> Vec<f32> {
        self.inner.pack()
    }

    /// Unpacks joint state from a flat array.
    #[wasm_bindgen(js_name = unpack)]
    pub fn unpack(&mut self, data: &[f32]) {
        self.inner.unpack(data);
    }

    /// Solves Jacobian IK to reach target (x,y,z). Returns final error.
    #[wasm_bindgen(js_name = solveJacobianIk)]
    pub fn solve_jacobian_ik(
        &mut self,
        end_effector_idx: usize,
        target_x: f32,
        target_y: f32,
        target_z: f32,
        max_iters: usize,
    ) -> Result<f32, JsError> {
        let n = self.inner.tree().num_nodes();
        if end_effector_idx >= n {
            return Err(JsError::new(&format!(
                "End-effector index {} out of range [0, {})",
                end_effector_idx, n
            )));
        }
        let mut target = Vector3f::with_capacity(3);
        target.set(0, target_x);
        target.set(1, target_y);
        target.set(2, target_z);
        let err = JacobianIk::new(&mut self.inner, end_effector_idx, target)
            .with_max_iters(max_iters)
            .solve();
        Ok(err)
    }

    /// Returns the number of nodes.
    #[wasm_bindgen(getter, js_name = numNodes)]
    pub fn num_nodes(&self) -> usize {
        self.inner.tree().num_nodes()
    }
}
