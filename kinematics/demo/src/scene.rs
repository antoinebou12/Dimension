//! Kinematics scene: 3-joint arm, sync to render World, IK target from screen.

use std::collections::HashMap;

use std::f32::consts::PI;

use kinematics::ik::{FabrikIk, JacobianIk};
use kinematics::joints::RevoluteJoint;
use kinematics::{Armature, JointData, JointVariant};
use mathlib::cg::vector3;
use mathlib::cg::{from_homogeneous, matrix4_extract_rotation_quat, vector4_from_point};
use mathlib::math3d::matrix4f_inverse;
use mathlib::math3d::Matrix4f;
use render::scene::{EntityId, Primitive, Primitive3D, Transform, World};
use render::ui::{Button, ControlId, Label, Rect, Window};

const LINK_LENGTH: f32 = 0.5;
const JOINT_SCALE: f32 = 0.12;
const TARGET_SCALE: f32 = 0.10;

/// Window ID for the armature tree panel.
pub const ARMATURE_TREE_WINDOW_ID: ControlId = ControlId(0xFF00_1000);
/// Window ID for the armature controls (end-effector selector) panel.
pub const ARMATURE_CONTROLS_WINDOW_ID: ControlId = ControlId(0xFF00_1001);
/// Window ID for the scene entity tree (joints + target, select for gizmo).
pub const SCENE_ENTITY_WINDOW_ID: ControlId = ControlId(0xFF00_1002);

/// IK solver used for position-only inverse kinematics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IkSolverType {
    /// FABRIK (Forward And Backward Reaching IK).
    Fabrik,
    /// Jacobian-based IK (SVD pseudoinverse + line search).
    Jacobian,
}

/// Action triggered by controls or scene panel.
#[derive(Clone, Copy, Debug)]
pub enum KinematicsAction {
    /// Set the IK end-effector node index.
    SetEndEffector(usize),
    /// Select an entity (show gizmo).
    SelectEntity(EntityId),
    /// Randomize the IK target position within reach.
    RandomTarget,
    /// Set the IK solver type (FABRIK or Jacobian).
    SetSolverType(IkSolverType),
}

/// Scene state: armature, entity IDs, IK target.
pub struct KinematicsScene {
    /// 3-joint arm (4 nodes: root + 3 links).
    pub armature: Armature,
    /// Entity IDs for joint positions (4 cubes at joints).
    pub joint_entities: Vec<EntityId>,
    /// Entity ID for IK target indicator.
    pub target_entity: EntityId,
    /// IK target in world space.
    pub ik_target: [f32; 3],
    /// Horizontal plane height (y) used when unprojecting screen to IK target. Default 0.5 for 3D placement.
    pub ik_target_plane_y: f32,
    /// IK solver type (FABRIK or Jacobian).
    pub ik_solver_type: IkSolverType,
    /// End-effector node index for IK.
    pub end_effector_idx: usize,
}

/// Builds a 3-joint arm and render entities. All link entities are children of root.
#[must_use]
pub fn build_kinematics_scene(world: &mut World) -> KinematicsScene {
    // Root: Y axis. Child joints: X then Z so the chain can move in 3D.
    let root = JointData::new(JointVariant::Revolute(RevoluteJoint::new(
        vector3(0.0, 0.0, 0.0),
        (0.0, 1.0, 0.0),
        0.0,
    )));
    let mut armature = Armature::new(root);
    // Joint axes: 0->1 X, 1->2 Z, 2->3 Y for full 3D motion.
    let axes = [(1.0_f32, 0.0, 0.0), (0.0, 0.0, 1.0), (0.0, 1.0, 0.0)];
    for i in 1..4 {
        let axis = axes[i - 1];
        let joint = RevoluteJoint::new(vector3(LINK_LENGTH, 0.0, 0.0), axis, 0.0)
            .with_angle_limits(-PI, PI);
        armature.add_child(i - 1, i, JointData::new(JointVariant::Revolute(joint)));
    }
    armature.update_kinematics();

    let root_ent = world.root_entity();
    let mut joint_entities = Vec::with_capacity(4);
    for _ in 0..4 {
        let e = world.spawn(root_ent);
        world.set_primitive(e, Primitive::ThreeD(Primitive3D::Sphere));
        world.set_transform(
            e,
            Transform {
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                rotation_quat: None,
                scale: [JOINT_SCALE, JOINT_SCALE, JOINT_SCALE],
            },
        );
        world.set_color(e, [0.4, 0.6, 1.0, 1.0]);
        joint_entities.push(e);
    }

    let target_entity = world.spawn(root_ent);
    world.set_primitive(target_entity, Primitive::ThreeD(Primitive3D::Cube));
    world.set_color(target_entity, [1.0, 0.4, 0.2, 1.0]);

    let ee_pos = armature.end_effector_position(3);
    let ik_target = [ee_pos.get(0), ee_pos.get(1), ee_pos.get(2)];

    #[cfg(target_arch = "wasm32")]
    {
        let _ = web_sys::console::log_1(
            &format!(
                "[kinematics] build_kinematics_scene: EE pos=({:.3},{:.3},{:.3}) ik_target=({:.3},{:.3},{:.3})",
                ee_pos.get(0),
                ee_pos.get(1),
                ee_pos.get(2),
                ik_target[0],
                ik_target[1],
                ik_target[2]
            )
            .into(),
        );
        for (i, node) in armature.tree().nodes.iter().take(4).enumerate() {
            let m = &node.data.world_transform;
            let _ = web_sys::console::log_1(
                &format!(
                    "[kinematics] node {} world pos=({:.3},{:.3},{:.3})",
                    i,
                    m.get(0, 3),
                    m.get(1, 3),
                    m.get(2, 3)
                )
                .into(),
            );
        }
    }

    KinematicsScene {
        armature,
        joint_entities,
        target_entity,
        ik_target,
        ik_target_plane_y: 0.5,
        ik_solver_type: IkSolverType::Fabrik,
        end_effector_idx: 3,
    }
}

#[cfg(target_arch = "wasm32")]
static SYNC_DEBUG_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Syncs armature world transforms to render World, and target entity.
pub fn sync_armature_to_world(scene: &mut KinematicsScene, world: &mut World) {
    scene.armature.update_kinematics();

    #[cfg(target_arch = "wasm32")]
    {
        let n = SYNC_DEBUG_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if n < 2 {
            for (node_idx, &_ent_id) in scene.joint_entities.iter().enumerate().take(4) {
                if node_idx < scene.armature.tree().num_nodes() {
                    let m = &scene.armature.tree().nodes[node_idx].data.world_transform;
                    let _ = web_sys::console::log_1(
                        &format!(
                            "[kinematics] sync frame {} joint {} pos=({:.3},{:.3},{:.3})",
                            n,
                            node_idx,
                            m.get(0, 3),
                            m.get(1, 3),
                            m.get(2, 3)
                        )
                        .into(),
                    );
                }
            }
        }
    }

    for (node_idx, &ent_id) in scene.joint_entities.iter().enumerate() {
        if node_idx < scene.armature.tree().num_nodes() {
            let m = &scene.armature.tree().nodes[node_idx].data.world_transform;
            let pos = [m.get(0, 3), m.get(1, 3), m.get(2, 3)];
            let q = matrix4_extract_rotation_quat(m);
            if let Some(data) = world.get_mut(ent_id) {
                data.transform = Transform::from_position_quat(pos, q);
                data.transform.scale = [JOINT_SCALE, JOINT_SCALE, JOINT_SCALE];
            }
        }
    }

    if let Some(data) = world.get_mut(scene.target_entity) {
        data.transform = Transform {
            position: scene.ik_target,
            rotation: [0.0, 0.0, 0.0],
            rotation_quat: None,
            scale: [TARGET_SCALE, TARGET_SCALE, TARGET_SCALE],
        };
    }
}

/// Unproject screen (x,y) to world position on horizontal plane y = plane_y.
/// Returns None if the view ray does not hit the plane (e.g. ray parallel to plane).
#[must_use]
pub fn screen_to_plane_y(
    view_matrix: &Matrix4f,
    proj_matrix: &Matrix4f,
    screen_x: f32,
    screen_y: f32,
    width: f32,
    height: f32,
    plane_y: f32,
) -> Option<[f32; 3]> {
    let ndc_x = 2.0 * screen_x / width - 1.0;
    let ndc_y = 1.0 - 2.0 * screen_y / height;
    let ndc_near = vector4_from_point(ndc_x, ndc_y, -1.0);
    let ndc_far = vector4_from_point(ndc_x, ndc_y, 1.0);
    let proj_inv = matrix4f_inverse(proj_matrix);
    let view_inv = matrix4f_inverse(view_matrix);
    let view_near = &proj_inv * &ndc_near;
    let view_far = &proj_inv * &ndc_far;
    let near_pt = from_homogeneous(&view_near).unwrap_or_else(|| {
        mathlib::cg::vector3(view_near.get(0), view_near.get(1), view_near.get(2))
    });
    let far_pt = from_homogeneous(&view_far)
        .unwrap_or_else(|| mathlib::cg::vector3(view_far.get(0), view_far.get(1), view_far.get(2)));
    let world_near =
        &view_inv * &vector4_from_point(near_pt.get(0), near_pt.get(1), near_pt.get(2));
    let world_far = &view_inv * &vector4_from_point(far_pt.get(0), far_pt.get(1), far_pt.get(2));
    let ow = world_near.get(3);
    let fw = world_far.get(3);
    let ox = world_near.get(0) / ow;
    let oy = world_near.get(1) / ow;
    let oz = world_near.get(2) / ow;
    let dx = world_far.get(0) / fw - ox;
    let dy = world_far.get(1) / fw - oy;
    let dz = world_far.get(2) / fw - oz;
    if dy.abs() < 1e-8 {
        return None;
    }
    let t = (plane_y - oy) / dy;
    if t < 0.0 {
        return None;
    }
    Some([ox + t * dx, plane_y, oz + t * dz])
}

/// Unproject screen (x,y) to world position on plane y = 0.
#[must_use]
pub fn screen_to_plane_y0(
    view_matrix: &Matrix4f,
    proj_matrix: &Matrix4f,
    screen_x: f32,
    screen_y: f32,
    width: f32,
    height: f32,
) -> Option<[f32; 3]> {
    screen_to_plane_y(
        view_matrix,
        proj_matrix,
        screen_x,
        screen_y,
        width,
        height,
        0.0,
    )
}

/// Unproject screen (x,y) to world position on a plane through `plane_point` with normal =
/// camera view direction (into scene). Use for dragging the IK target in the view plane (full 3D movement).
/// Returns `None` if the ray does not hit the plane (parallel) or hits behind the camera.
#[must_use]
pub fn screen_to_plane_at_point(
    camera: &impl render::backend::Camera3d,
    screen_x: f32,
    screen_y: f32,
    plane_point: [f32; 3],
) -> Option<[f32; 3]> {
    let (origin, dir) = render::pick::screen_ray_to_world(camera, screen_x, screen_y)?;
    let view = camera.view_matrix();
    let view_inv = matrix4f_inverse(&view);
    let nx = -view_inv.get(0, 2);
    let ny = -view_inv.get(1, 2);
    let nz = -view_inv.get(2, 2);
    let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-9);
    let normal = [nx / len, ny / len, nz / len];
    let denom = normal[0] * dir[0] + normal[1] * dir[1] + normal[2] * dir[2];
    if denom.abs() < 1e-9 {
        return None;
    }
    let dx = plane_point[0] - origin[0];
    let dy = plane_point[1] - origin[1];
    let dz = plane_point[2] - origin[2];
    let t = (dx * normal[0] + dy * normal[1] + dz * normal[2]) / denom;
    if t < 0.0 {
        return None;
    }
    Some([
        origin[0] + t * dir[0],
        origin[1] + t * dir[1],
        origin[2] + t * dir[2],
    ])
}

/// View-forward direction (camera into scene) from the camera. Use for moving the IK target along depth.
#[must_use]
pub fn camera_view_forward(camera: &impl render::backend::Camera3d) -> [f32; 3] {
    let view = camera.view_matrix();
    let view_inv = matrix4f_inverse(&view);
    let nx = -view_inv.get(0, 2);
    let ny = -view_inv.get(1, 2);
    let nz = -view_inv.get(2, 2);
    let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-9);
    [nx / len, ny / len, nz / len]
}

#[cfg(target_arch = "wasm32")]
static STEP_IK_DEBUG_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Run multiple IK steps per frame and sync. Keeps solver running even when no exact solution.
pub fn step_ik(scene: &mut KinematicsScene, world: &mut World) {
    let ee_before = scene.armature.end_effector_position(scene.end_effector_idx);
    let mut target = mathlib::Vector3f::with_capacity(3);
    target.set(0, scene.ik_target[0]);
    target.set(1, scene.ik_target[1]);
    target.set(2, scene.ik_target[2]);
    let _err_before = ((ee_before.get(0) - scene.ik_target[0]).powi(2)
        + (ee_before.get(1) - scene.ik_target[1]).powi(2)
        + (ee_before.get(2) - scene.ik_target[2]).powi(2))
    .sqrt();
    let _err_after = match scene.ik_solver_type {
        IkSolverType::Fabrik => FabrikIk::new(&mut scene.armature, scene.end_effector_idx, target)
            .with_max_iters(10)
            .solve(),
        IkSolverType::Jacobian => {
            JacobianIk::new(&mut scene.armature, scene.end_effector_idx, target)
                .with_max_iters(20)
                .solve()
        }
    };
    sync_armature_to_world(scene, world);

    #[cfg(target_arch = "wasm32")]
    {
        let n = STEP_IK_DEBUG_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if n < 3 {
            let ee = scene.armature.end_effector_position(scene.end_effector_idx);
            let _ = web_sys::console::log_1(
                &format!(
                    "[kinematics] step_ik frame {} EE=({:.3},{:.3},{:.3}) target=({:.3},{:.3},{:.3}) err_before={:.6} err_after={:.6}",
                    n,
                    ee.get(0),
                    ee.get(1),
                    ee.get(2),
                    scene.ik_target[0],
                    scene.ik_target[1],
                    scene.ik_target[2],
                    _err_before,
                    _err_after
                )
                .into(),
            );
        }
    }
}

/// Format joint variant as type name, optional angle (rad), and optional limits for display.
fn joint_label(joint: &JointVariant) -> (String, Option<f32>, Option<(f32, f32)>) {
    let limits = joint.angle_limits();
    match joint {
        JointVariant::Revolute(r) => ("Revolute".to_string(), Some(r.angle), limits),
        JointVariant::Revolute2d(r) => ("Revolute2d".to_string(), Some(r.angle), limits),
        JointVariant::Fixed(_) => ("Fixed".to_string(), None, None),
        JointVariant::Prismatic(_) => ("Prismatic".to_string(), None, None),
        JointVariant::Spherical(_) => ("Spherical".to_string(), None, None),
        JointVariant::Fixed2d(_) => ("Fixed2d".to_string(), None, None),
        JointVariant::Prismatic2d(_) => ("Prismatic2d".to_string(), None, None),
    }
}

/// Build the armature tree panel (node index, joint type, angle). Placed right of scene entity panel.
#[must_use]
pub fn build_armature_tree_panel(scene: &KinematicsScene, _viewport_width: f32) -> Window {
    const ROW_H: f32 = 18.0;
    const W: f32 = 200.0;
    const SCENE_PANEL_W: f32 = 160.0;
    let tree = scene.armature.tree();
    let n = tree.num_nodes();
    let h = (n as f32 * (ROW_H + 2.0)) + 28.0;
    let x = 10.0 + SCENE_PANEL_W + 10.0;
    let rect = Rect::new(x, 10.0, W, h);
    let mut window = Window::new(ARMATURE_TREE_WINDOW_ID, rect);
    window.title_bar_height = 20.0;
    let body = window.body_rect();
    let mut y = body.y + 4.0;
    let mut next_id = ARMATURE_TREE_WINDOW_ID.0 + 1;
    for (idx, node) in tree.nodes.iter().take(n).enumerate() {
        let (type_name, angle_opt, limits_opt) = joint_label(&node.data.joint);
        let text = match (angle_opt, limits_opt) {
            (Some(a), Some((lo, hi))) => {
                format!("{}: {} {:.3} rad [{:.2}, {:.2}]", idx, type_name, a, lo, hi)
            }
            (Some(a), None) => format!("{}: {} {:.3} rad", idx, type_name, a),
            (None, _) => format!("{}: {}", idx, type_name),
        };
        let r = Rect::new(body.x + 4.0, y, body.w - 8.0, ROW_H);
        y += ROW_H + 2.0;
        window.add_label(Label::new(ControlId(next_id), r, text));
        next_id += 1;
    }
    window
}

/// Build the armature controls panel (end-effector selector) and return the action mapping.
#[must_use]
pub fn build_armature_controls_panel(
    scene: &KinematicsScene,
    viewport_width: f32,
) -> (Window, HashMap<ControlId, KinematicsAction>) {
    let mut mapping = HashMap::new();
    const ROW_H: f32 = 24.0;
    const W: f32 = 180.0;
    const BTN_W: f32 = 80.0;
    let n = scene.armature.tree().num_nodes();
    let num_ee = n.saturating_sub(1);
    const READOUT_ROWS: f32 = 2.0;
    const RANDOM_ROW: f32 = 1.0;
    const SOLVER_ROWS: f32 = 2.0;
    let h = (SOLVER_ROWS * (ROW_H + 2.0))
        + (num_ee as f32 * (ROW_H + 2.0))
        + (RANDOM_ROW * (ROW_H + 2.0))
        + (READOUT_ROWS * (ROW_H + 2.0))
        + 60.0;
    const SCENE_PANEL_W: f32 = 160.0;
    const ARMATURE_TREE_W: f32 = 200.0;
    let x = (10.0 + SCENE_PANEL_W + 10.0 + ARMATURE_TREE_W + 10.0).min(viewport_width - W - 10.0);
    let rect = Rect::new(x, 10.0, W, h);
    let mut window = Window::new(ARMATURE_CONTROLS_WINDOW_ID, rect);
    window.title_bar_height = 20.0;
    let body = window.body_rect();
    let mut y = body.y + 4.0;
    let mut next_id = ARMATURE_CONTROLS_WINDOW_ID.0 + 1;

    let solver_label_rect = Rect::new(body.x + 8.0, y, body.w - 16.0, ROW_H);
    window.add_label(Label::new(
        ControlId(next_id),
        solver_label_rect,
        "Solver:".to_string(),
    ));
    next_id += 1;
    y += ROW_H + 2.0;

    const SOLVER_LABEL_W: f32 = 50.0;
    const SOLVER_BTN_W: f32 = 30.0;
    const SOLVER_GAP: f32 = 4.0;
    let fabrik_label = if scene.ik_solver_type == IkSolverType::Fabrik {
        "FABRIK (active)"
    } else {
        "FABRIK"
    };
    let jacobian_label = if scene.ik_solver_type == IkSolverType::Jacobian {
        "Jacobian (active)"
    } else {
        "Jacobian"
    };
    let fabrik_label_rect = Rect::new(body.x + 8.0, y, SOLVER_LABEL_W, ROW_H);
    let fabrik_btn_rect = Rect::new(
        body.x + 8.0 + SOLVER_LABEL_W + 2.0,
        y,
        SOLVER_BTN_W,
        ROW_H - 2.0,
    );
    window.add_label(Label::new(
        ControlId(next_id),
        fabrik_label_rect,
        fabrik_label.to_string(),
    ));
    next_id += 1;
    let fabrik_cid = ControlId(next_id);
    next_id += 1;
    window.add_button(Button::new(fabrik_cid, fabrik_btn_rect));
    mapping.insert(
        fabrik_cid,
        KinematicsAction::SetSolverType(IkSolverType::Fabrik),
    );

    let jacobian_x = body.x + 8.0 + SOLVER_LABEL_W + 2.0 + SOLVER_BTN_W + SOLVER_GAP;
    let jacobian_label_rect = Rect::new(jacobian_x, y, SOLVER_LABEL_W + 2.0, ROW_H);
    let jacobian_btn_rect = Rect::new(
        jacobian_x + SOLVER_LABEL_W + 2.0,
        y,
        SOLVER_BTN_W,
        ROW_H - 2.0,
    );
    window.add_label(Label::new(
        ControlId(next_id),
        jacobian_label_rect,
        jacobian_label.to_string(),
    ));
    next_id += 1;
    let jacobian_cid = ControlId(next_id);
    next_id += 1;
    window.add_button(Button::new(jacobian_cid, jacobian_btn_rect));
    mapping.insert(
        jacobian_cid,
        KinematicsAction::SetSolverType(IkSolverType::Jacobian),
    );
    y += ROW_H + 2.0;

    for idx in 1..n {
        let label_rect = Rect::new(body.x + 8.0, y, body.w - 16.0, ROW_H);
        let btn_rect = Rect::new(body.x + body.w - BTN_W - 12.0, y, BTN_W, ROW_H - 2.0);
        let btn_cid = ControlId(next_id);
        next_id += 1;
        let label_text = format!(
            "EE: {} {}",
            idx,
            if idx == scene.end_effector_idx {
                "(active)"
            } else {
                ""
            }
        );
        window.add_label(Label::new(ControlId(next_id), label_rect, label_text));
        next_id += 1;
        window.add_button(Button::new(btn_cid, btn_rect));
        mapping.insert(btn_cid, KinematicsAction::SetEndEffector(idx));
        y += ROW_H + 2.0;
    }
    let random_btn_rect = Rect::new(body.x + 8.0, y, BTN_W, ROW_H - 2.0);
    let random_cid = ControlId(next_id);
    next_id += 1;
    window.add_button(Button::new(random_cid, random_btn_rect));
    mapping.insert(random_cid, KinematicsAction::RandomTarget);
    let random_label_rect = Rect::new(
        body.x + 8.0 + BTN_W + 4.0,
        y,
        body.w - 16.0 - BTN_W - 4.0,
        ROW_H,
    );
    window.add_label(Label::new(
        ControlId(next_id),
        random_label_rect,
        "Random target".to_string(),
    ));
    next_id += 1;
    y += ROW_H + 2.0;

    let ee = scene.armature.end_effector_position(scene.end_effector_idx);
    let ee_text = format!("EE: ({:.3}, {:.3}, {:.3})", ee.get(0), ee.get(1), ee.get(2));
    let r_ee = Rect::new(body.x + 4.0, y, body.w - 8.0, ROW_H);
    y += ROW_H + 2.0;
    window.add_label(Label::new(ControlId(next_id), r_ee, ee_text));
    next_id += 1;
    let target_text = format!(
        "Target: ({:.3}, {:.3}, {:.3})",
        scene.ik_target[0], scene.ik_target[1], scene.ik_target[2]
    );
    let r_tgt = Rect::new(body.x + 4.0, y, body.w - 8.0, ROW_H);
    window.add_label(Label::new(ControlId(next_id), r_tgt, target_text));
    (window, mapping)
}

/// Randomize the IK target to a point within ~80% of max arm reach (y = 0 plane).
pub fn randomize_ik_target(scene: &mut KinematicsScene) {
    let reach = LINK_LENGTH * 3.0 * 0.8;
    let angle = rand::random::<f32>() * std::f32::consts::TAU;
    let r = rand::random::<f32>().sqrt() * reach;
    scene.ik_target = [r * angle.cos(), 0.0, r * angle.sin()];
}

/// Apply a kinematics action from a clicked control. Requires engine for SelectEntity.
pub fn apply_kinematics_action(
    scene: &mut KinematicsScene,
    engine: &mut render::Engine,
    mapping: &HashMap<ControlId, KinematicsAction>,
    cid: ControlId,
) {
    let Some(action) = mapping.get(&cid) else {
        return;
    };
    match *action {
        KinematicsAction::SetEndEffector(idx) => {
            if idx < scene.armature.tree().num_nodes() {
                scene.end_effector_idx = idx;
            }
        }
        KinematicsAction::SelectEntity(id) => {
            engine.set_selected_entity(Some(id));
        }
        KinematicsAction::RandomTarget => {
            randomize_ik_target(scene);
        }
        KinematicsAction::SetSolverType(t) => {
            scene.ik_solver_type = t;
        }
    }
}

/// Build the scene entity tree panel (Joint 0..N, Target) for selection. Returns window and mapping.
#[must_use]
pub fn build_scene_entity_panel(
    scene: &KinematicsScene,
    _world: &World,
) -> (Window, HashMap<ControlId, KinematicsAction>) {
    let mut mapping = HashMap::new();
    const ROW_H: f32 = 24.0;
    const W: f32 = 160.0;
    let x = 10.0;
    let n_joints = scene.joint_entities.len();
    let h = ((n_joints + 1) as f32 * (ROW_H + 2.0)) + 28.0;
    let rect = Rect::new(x, 10.0, W, h);
    let mut window = Window::new(SCENE_ENTITY_WINDOW_ID, rect);
    window.title_bar_height = 20.0;
    let body = window.body_rect();
    let mut y = body.y + 4.0;
    let mut next_id = SCENE_ENTITY_WINDOW_ID.0 + 1;
    for (i, &ent_id) in scene.joint_entities.iter().enumerate() {
        let label_rect = Rect::new(body.x + 8.0, y, body.w - 16.0, ROW_H);
        let cid = ControlId(next_id);
        next_id += 1;
        window.add_label(Label::new(
            ControlId(next_id),
            label_rect,
            format!("Joint {}", i),
        ));
        next_id += 1;
        window.add_button(Button::new(cid, label_rect));
        mapping.insert(cid, KinematicsAction::SelectEntity(ent_id));
        y += ROW_H + 2.0;
    }
    let label_rect = Rect::new(body.x + 8.0, y, body.w - 16.0, ROW_H);
    let cid = ControlId(next_id);
    next_id += 1;
    window.add_label(Label::new(
        ControlId(next_id),
        label_rect,
        "Target".to_string(),
    ));
    window.add_button(Button::new(cid, label_rect));
    mapping.insert(cid, KinematicsAction::SelectEntity(scene.target_entity));
    (window, mapping)
}
