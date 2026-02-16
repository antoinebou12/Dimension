//! Kinematics scene: dual kinematic chains, sync to render World, IK target from screen.
//!
//! Renders two chains side-by-side (joint spheres, link segments, IK target cubes). Each chain
//! has its own armature, solver (FABRIK / Jacobian / CCD / Halley), and arm preset.

use std::collections::HashMap;

use kinematics::ik::{CcdIk, FabrikIk, FabrikSqpIk, HalleyIk, HessianIk, JacobianIk};
use kinematics::joints::SphericalJoint;
use kinematics::{Armature, FixedJoint, JointData, JointVariant, RevoluteJoint};
use mathlib::cg::vector3;
use mathlib::cg::{
    from_homogeneous, matrix4_extract_rotation_quat, matrix4f_translation, new_translation,
    vector4_from_point,
};
use mathlib::math3d::matrix4f_inverse;
use mathlib::math3d::{transform_vector, vector3_cross, Matrix4f, Vector3f};
use mathlib::Quat4f;
#[cfg(feature = "neural")]
use neural::{denormalize_joints, normalize_position, ChainConfig, OnnxIkSession};
use render::scene::{CurvePoint, EntityId, Primitive, Primitive3D, Transform, World};
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

/// Which of the two chains (A = left, B = right).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainIndex {
    A,
    B,
}

/// IK solver used for position-only inverse kinematics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IkSolverType {
    /// FABRIK (Forward And Backward Reaching IK).
    Fabrik,
    /// Jacobian-based IK (SVD pseudoinverse + line search).
    Jacobian,
    /// CCD (Cyclic Coordinate Descent).
    Ccd,
    /// Halley-based IK (6D pose, QuIK-style).
    Halley,
    /// FABRIK + SQP hybrid (Xu et al., arXiv:2209.02532).
    FabrikSqp,
    /// Hessian IK (exact Hessian Newton, Erleben & Andrews MIG 2017).
    Hessian,
    /// Neural IK (ONNX model, single forward pass). Valid only for Revolute 3-DOF.
    #[cfg(feature = "neural")]
    Neural,
}

/// Arm configuration preset: joint types for the demo armature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArmPreset {
    /// All spherical joints (full 3D orientation per joint).
    Spherical,
    /// Root fixed, then revolute joints (rotation about Y, robot-arm style).
    Revolute,
    /// 6 nodes: spherical root, revolute + revolute(limited), spherical wrist, revolute hand, fixed EE.
    MixedArm,
    /// 7 nodes: all spherical, shorter links (0.2 each), highly redundant.
    SphericalSnake,
    /// Loaded from BVH file (feature-gated).
    #[cfg(feature = "bvh")]
    Bvh,
}

/// Action triggered by controls or scene panel.
#[derive(Clone, Copy, Debug)]
pub enum KinematicsAction {
    /// Set the IK end-effector node index for a chain.
    SetEndEffector(ChainIndex, usize),
    /// Select an entity (show gizmo).
    SelectEntity(EntityId),
    /// Randomize the IK target position within reach for a chain.
    RandomTarget(ChainIndex),
    /// Set the IK solver type for a chain.
    SetSolverType(ChainIndex, IkSolverType),
    /// Set the arm preset for a chain and rebuild that chain.
    SetArmPreset(ChainIndex, ArmPreset),
    /// Set which chain is active (for right-drag target move / UI focus).
    SetActiveChain(ChainIndex),
    /// Set the IK target to the current end-effector position for a chain.
    SetTargetToEndEffector(ChainIndex),
    /// Reset the scene to initial state (despawn all chain entities, rebuild both chains).
    ResetScene,
    /// Toggle visibility of joint rotation-limit cones.
    ToggleShowJointCones,
    /// Replace the given chain with an armature loaded from BVH (native: opens file dialog). Requires `bvh` feature.
    #[cfg(feature = "bvh")]
    LoadBvh(ChainIndex),
}

/// State for one kinematic chain: armature, render entities, IK target, solver, colors, root offset.
pub struct ChainState {
    pub armature: Armature,
    pub joint_entities: Vec<EntityId>,
    pub link_entities: Vec<EntityId>,
    /// One cone entity per joint (rotation-limit visualization); hidden for fixed joints or when show_joint_cones is false.
    pub cone_entities: Vec<EntityId>,
    pub target_entity: EntityId,
    pub ik_target: [f32; 3],
    pub ik_solver_type: IkSolverType,
    pub end_effector_idx: usize,
    pub arm_preset: ArmPreset,
    pub color: [f32; 4],
    pub target_color: [f32; 4],
    pub root_offset: [f32; 3],
    /// Lazy-loaded ONNX session for Neural solver (3-DOF only).
    #[cfg(feature = "neural")]
    pub neural_session: Option<OnnxIkSession>,
}

/// Scene state: two chains (A left, B right), active chain, shared plane for unproject.
pub struct KinematicsScene {
    pub chain_a: ChainState,
    pub chain_b: ChainState,
    pub active_chain: ChainIndex,
    /// Horizontal plane height (y) used when unprojecting screen to IK target.
    pub ik_target_plane_y: f32,
    /// When true, show semi-transparent cones for each joint's possible rotation range.
    pub show_joint_cones: bool,
}

/// Builds one kinematic chain: armature from preset, render entities, colors and root offset.
#[must_use]
pub fn build_chain(
    world: &mut World,
    preset: ArmPreset,
    root_offset: [f32; 3],
    color: [f32; 4],
    target_color: [f32; 4],
    solver: IkSolverType,
) -> ChainState {
    let (root, child_joints) = match preset {
        ArmPreset::Spherical => {
            let root = JointData::new(JointVariant::Spherical(SphericalJoint::new(
                vector3(0.0, 0.0, 0.0),
                Quat4f::identity(),
            )));
            let child_joints: Vec<JointData> = (0..3)
                .map(|_| {
                    JointData::new(JointVariant::Spherical(SphericalJoint::new(
                        vector3(LINK_LENGTH, 0.0, 0.0),
                        Quat4f::identity(),
                    )))
                })
                .collect();
            (root, child_joints)
        }
        ArmPreset::Revolute => {
            let root = JointData::new(JointVariant::Fixed(FixedJoint::default()));
            let axis_y = (0.0_f32, 1.0, 0.0);
            let child_joints: Vec<JointData> = (0..3)
                .map(|_| {
                    JointData::new(JointVariant::Revolute(RevoluteJoint::new(
                        vector3(LINK_LENGTH, 0.0, 0.0),
                        axis_y,
                        0.0,
                    )))
                })
                .collect();
            (root, child_joints)
        }
        ArmPreset::MixedArm => {
            // 6 nodes: spherical root, revolute, revolute(limited), spherical wrist, revolute hand, fixed EE
            let root = JointData::new(JointVariant::Spherical(SphericalJoint::new(
                vector3(0.0, 0.0, 0.0),
                Quat4f::identity(),
            )));
            let l1 = 0.35_f32;
            let l2 = 0.30;
            let l3 = 0.25;
            let l4 = 0.20;
            let l5 = 0.10;
            let axis_y = (0.0_f32, 1.0, 0.0);
            let child_joints = vec![
                JointData::new(JointVariant::Revolute(RevoluteJoint::new(
                    vector3(l1, 0.0, 0.0),
                    axis_y,
                    0.0,
                ))),
                JointData::new(JointVariant::Revolute(
                    RevoluteJoint::new(vector3(l2, 0.0, 0.0), axis_y, 0.0)
                        .with_angle_limits(-1.2, 1.2),
                )),
                JointData::new(JointVariant::Spherical(SphericalJoint::new(
                    vector3(l3, 0.0, 0.0),
                    Quat4f::identity(),
                ))),
                JointData::new(JointVariant::Revolute(RevoluteJoint::new(
                    vector3(l4, 0.0, 0.0),
                    axis_y,
                    0.0,
                ))),
                JointData::new(JointVariant::Fixed(FixedJoint::new(
                    vector3(l5, 0.0, 0.0),
                    (0.0, 0.0, 0.0),
                ))),
            ];
            (root, child_joints)
        }
        ArmPreset::SphericalSnake => {
            let root = JointData::new(JointVariant::Spherical(SphericalJoint::new(
                vector3(0.0, 0.0, 0.0),
                Quat4f::identity(),
            )));
            const L: f32 = 0.2;
            let child_joints: Vec<JointData> = (0..6)
                .map(|_| {
                    JointData::new(JointVariant::Spherical(SphericalJoint::new(
                        vector3(L, 0.0, 0.0),
                        Quat4f::identity(),
                    )))
                })
                .collect();
            (root, child_joints)
        }
        #[cfg(feature = "bvh")]
        ArmPreset::Bvh => unreachable!("use build_chain_from_armature for BVH-loaded chains"),
    };

    let mut armature = Armature::new(root);
    for (i, data) in child_joints.into_iter().enumerate() {
        armature.add_child(i, i + 1, data);
    }
    armature.update_kinematics();

    let n_nodes = armature.tree().num_nodes();
    let n_links = n_nodes.saturating_sub(1);
    let root_ent = world.root_entity();
    let mut joint_entities = Vec::with_capacity(n_nodes);
    for _ in 0..n_nodes {
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
        world.set_color(e, color);
        joint_entities.push(e);
    }

    let mut link_entities = Vec::with_capacity(n_links);
    let link_primitive = Primitive::ThreeD(Primitive3D::LineSegment {
        start: CurvePoint([0.0, 0.0, 0.0]),
        end: CurvePoint([1.0, 0.0, 0.0]),
    });
    for _ in 0..n_links {
        let e = world.spawn(root_ent);
        world.set_primitive(e, link_primitive);
        world.set_transform(
            e,
            Transform {
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                rotation_quat: None,
                scale: [LINK_LENGTH, 1.0, 1.0],
            },
        );
        world.set_color(e, color);
        link_entities.push(e);
    }

    let target_entity = world.spawn(root_ent);
    world.set_primitive(target_entity, Primitive::ThreeD(Primitive3D::Cube));
    world.set_color(target_entity, target_color);

    // One cone per joint for rotation-limit visualization (sync_chain_to_world positions and hides as needed).
    let cone_color = [color[0], color[1], color[2], 1.0];
    let mut cone_entities = Vec::with_capacity(n_nodes);
    for _ in 0..n_nodes {
        let e = world.spawn(root_ent);
        world.set_primitive(e, Primitive::ThreeD(Primitive3D::Cone));
        world.set_transform(
            e,
            Transform {
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                rotation_quat: None,
                scale: [0.1, 0.1, 0.1],
            },
        );
        world.set_color(e, cone_color);
        cone_entities.push(e);
    }

    let ee_idx = n_nodes - 1;
    let ee_pos = armature.end_effector_position(ee_idx);
    let ik_target = [
        root_offset[0] + ee_pos.get(0),
        root_offset[1] + ee_pos.get(1),
        root_offset[2] + ee_pos.get(2),
    ];

    ChainState {
        armature,
        joint_entities,
        link_entities,
        cone_entities,
        target_entity,
        ik_target,
        ik_solver_type: solver,
        end_effector_idx: ee_idx,
        arm_preset: preset,
        color,
        target_color,
        root_offset,
        #[cfg(feature = "neural")]
        neural_session: None,
    }
}

/// Builds the dual-chain kinematics scene. Chain A: left (blue), Chain B: right (green).
#[must_use]
pub fn build_kinematics_scene(world: &mut World) -> KinematicsScene {
    let chain_a = build_chain(
        world,
        ArmPreset::MixedArm,
        [-0.8, 0.0, 0.0],
        [0.3, 0.5, 1.0, 1.0],
        [1.0, 0.5, 0.2, 1.0],
        IkSolverType::Fabrik,
    );
    let chain_b = build_chain(
        world,
        ArmPreset::SphericalSnake,
        [0.8, 0.0, 0.0],
        [0.2, 0.8, 0.4, 1.0],
        [0.9, 0.25, 0.2, 1.0],
        IkSolverType::Ccd,
    );
    crate::wasm_debug::log_always(
        "[kinematics] dual-chain init: A=MixedArm+FABRIK @ -0.8, B=SphericalSnake+CCD @ 0.8",
    );
    KinematicsScene {
        chain_a,
        chain_b,
        active_chain: ChainIndex::A,
        ik_target_plane_y: 0.5,
        show_joint_cones: false,
    }
}

/// Builds a `ChainState` from an existing armature (e.g. from BVH). Used when replacing a chain.
#[cfg(feature = "bvh")]
#[must_use]
pub fn build_chain_from_armature(
    world: &mut World,
    mut armature: Armature,
    root_offset: [f32; 3],
    color: [f32; 4],
    target_color: [f32; 4],
    solver: IkSolverType,
) -> ChainState {
    armature.update_kinematics();
    let n_nodes = armature.tree().num_nodes();
    let n_links = n_nodes.saturating_sub(1);
    let root_ent = world.root_entity();
    let mut joint_entities = Vec::with_capacity(n_nodes);
    for _ in 0..n_nodes {
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
        world.set_color(e, color);
        joint_entities.push(e);
    }
    let link_primitive = Primitive::ThreeD(Primitive3D::LineSegment {
        start: CurvePoint([0.0, 0.0, 0.0]),
        end: CurvePoint([1.0, 0.0, 0.0]),
    });
    let mut link_entities = Vec::with_capacity(n_links);
    for _ in 0..n_links {
        let e = world.spawn(root_ent);
        world.set_primitive(e, link_primitive);
        world.set_transform(
            e,
            Transform {
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                rotation_quat: None,
                scale: [LINK_LENGTH, 1.0, 1.0],
            },
        );
        world.set_color(e, color);
        link_entities.push(e);
    }
    let target_entity = world.spawn(root_ent);
    world.set_primitive(target_entity, Primitive::ThreeD(Primitive3D::Cube));
    world.set_color(target_entity, target_color);

    let cone_color = [color[0], color[1], color[2], 1.0];
    let mut cone_entities = Vec::with_capacity(n_nodes);
    for _ in 0..n_nodes {
        let e = world.spawn(root_ent);
        world.set_primitive(e, Primitive::ThreeD(Primitive3D::Cone));
        world.set_transform(
            e,
            Transform {
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                rotation_quat: None,
                scale: [0.1, 0.1, 0.1],
            },
        );
        world.set_color(e, cone_color);
        cone_entities.push(e);
    }

    let ee_idx = n_nodes.saturating_sub(1);
    let ee_pos = armature.end_effector_position(ee_idx);
    let ik_target = [
        root_offset[0] + ee_pos.get(0),
        root_offset[1] + ee_pos.get(1),
        root_offset[2] + ee_pos.get(2),
    ];
    ChainState {
        armature,
        joint_entities,
        link_entities,
        cone_entities,
        target_entity,
        ik_target,
        ik_solver_type: solver,
        end_effector_idx: ee_idx,
        arm_preset: ArmPreset::Bvh,
        color,
        target_color,
        root_offset,
        #[cfg(feature = "neural")]
        neural_session: None,
    }
}

/// Returns a unit quaternion that rotates the positive X-axis (1, 0, 0) to the given unit direction.
/// Handles degenerate cases: dir ≈ +X (identity), dir ≈ -X (π about Y).
#[must_use]
fn quat_from_x_to_direction(dir: &Vector3f) -> Quat4f {
    const EPS: f32 = 1e-6;
    let x_axis = vector3(1.0, 0.0, 0.0);
    let dot = x_axis.dot(dir);
    if dot >= 1.0 - EPS {
        return Quat4f::identity();
    }
    if dot <= -1.0 + EPS {
        return Quat4f::from_axis_angle(&vector3(0.0, 1.0, 0.0), std::f32::consts::PI);
    }
    let axis = vector3_cross(&x_axis, dir);
    let axis_norm = axis.norm();
    if axis_norm < EPS {
        return Quat4f::identity();
    }
    let scale = 1.0 / axis_norm;
    let axis_scaled = vector3(
        axis.get(0) * scale,
        axis.get(1) * scale,
        axis.get(2) * scale,
    );
    let angle = dot.clamp(-1.0, 1.0).acos();
    Quat4f::from_axis_angle(&axis_scaled, angle)
}

/// Returns a unit quaternion that rotates the positive Z-axis (0, 0, 1) to the given unit direction.
/// Used to orient the cone primitive (tip at +Z) along the joint cone axis.
#[must_use]
fn quat_from_z_to_direction(dir: &Vector3f) -> Quat4f {
    const EPS: f32 = 1e-6;
    let z_axis = vector3(0.0, 0.0, 1.0);
    let dot = z_axis.dot(dir);
    if dot >= 1.0 - EPS {
        return Quat4f::identity();
    }
    if dot <= -1.0 + EPS {
        return Quat4f::from_axis_angle(&vector3(1.0, 0.0, 0.0), std::f32::consts::PI);
    }
    let axis = vector3_cross(&z_axis, dir);
    let axis_norm = axis.norm();
    if axis_norm < EPS {
        return Quat4f::identity();
    }
    let scale = 1.0 / axis_norm;
    let axis_scaled = vector3(
        axis.get(0) * scale,
        axis.get(1) * scale,
        axis.get(2) * scale,
    );
    let angle = dot.clamp(-1.0, 1.0).acos();
    Quat4f::from_axis_angle(&axis_scaled, angle)
}

/// Cone mesh: tip at +Z, base at -Z; total length 1.0, base radius 0.5 (see render mesh).
const CONE_MESH_HALF_LENGTH: f32 = 0.5;
const CONE_MESH_BASE_RADIUS: f32 = 0.5;
/// Scale factor for cone length vs next link to avoid overlapping the link.
const CONE_LENGTH_FACTOR: f32 = 0.88;
/// Default half-angle (rad) for spherical or unlimited revolute cones.
const DEFAULT_CONE_HALF_ANGLE_RAD: f32 = std::f32::consts::FRAC_PI_2;
/// Minimum base radius in world units so narrow cones remain visible.
const CONE_MIN_BASE_RADIUS: f32 = 0.02;

/// Syncs one chain's armature to render World (joints, links, target, cones) with root_offset applied.
fn sync_chain_to_world(chain: &mut ChainState, world: &mut World, show_cones: bool) {
    chain.armature.update_kinematics();
    let off = &chain.root_offset;
    let n_nodes = chain.armature.tree().num_nodes();

    for (node_idx, &ent_id) in chain.joint_entities.iter().enumerate() {
        if node_idx < n_nodes {
            let m = &chain.armature.tree().nodes[node_idx].data.world_transform;
            let pos = [
                off[0] + m.get(0, 3),
                off[1] + m.get(1, 3),
                off[2] + m.get(2, 3),
            ];
            let q = matrix4_extract_rotation_quat(m);
            if let Some(data) = world.get_mut(ent_id) {
                data.transform = Transform::from_position_quat(pos, q);
                data.transform.scale = [JOINT_SCALE, JOINT_SCALE, JOINT_SCALE];
            }
        }
    }

    for (i, &ent_id) in chain.link_entities.iter().enumerate() {
        if i + 1 < n_nodes {
            let pos_a = matrix4f_translation(&chain.armature.tree().nodes[i].data.world_transform);
            let pos_b =
                matrix4f_translation(&chain.armature.tree().nodes[i + 1].data.world_transform);
            let mut diff = vector3(
                pos_b.get(0) - pos_a.get(0),
                pos_b.get(1) - pos_a.get(1),
                pos_b.get(2) - pos_a.get(2),
            );
            let len = diff.norm();
            if len >= 1e-9 {
                diff.set(0, diff.get(0) / len);
                diff.set(1, diff.get(1) / len);
                diff.set(2, diff.get(2) / len);
                let q = quat_from_x_to_direction(&diff);
                let pos = [
                    off[0] + pos_a.get(0),
                    off[1] + pos_a.get(1),
                    off[2] + pos_a.get(2),
                ];
                if let Some(data) = world.get_mut(ent_id) {
                    data.transform = Transform::from_position_quat(pos, q);
                    data.transform.scale = [len, 1.0, 1.0];
                }
            }
        }
    }

    if let Some(data) = world.get_mut(chain.target_entity) {
        data.transform = Transform {
            position: chain.ik_target,
            rotation: [0.0, 0.0, 0.0],
            rotation_quat: None,
            scale: [TARGET_SCALE, TARGET_SCALE, TARGET_SCALE],
        };
    }

    // Sync cone entities (possible rotation range visualization).
    for (node_idx, &cone_ent) in chain.cone_entities.iter().enumerate() {
        let data = match world.get_mut(cone_ent) {
            Some(d) => d,
            None => continue,
        };
        if !show_cones || node_idx >= n_nodes {
            data.active = false;
            continue;
        }
        let tree = chain.armature.tree();
        let joint = &tree.nodes[node_idx].data.joint;
        let (cone_axis_world, half_angle_rad, link_length) = match joint {
            JointVariant::Revolute(r) => {
                let world_m = &tree.nodes[node_idx].data.world_transform;
                let axis_local = vector3(r.axis.0, r.axis.1, r.axis.2);
                let axis_w = transform_vector(world_m, &axis_local);
                let norm = axis_w.norm();
                if norm < 1e-6 {
                    data.active = false;
                    continue;
                }
                let axis_unit = vector3(
                    axis_w.get(0) / norm,
                    axis_w.get(1) / norm,
                    axis_w.get(2) / norm,
                );
                let half_angle = joint
                    .angle_limits()
                    .map(|(lo, hi)| (hi - lo) * 0.5)
                    .unwrap_or(DEFAULT_CONE_HALF_ANGLE_RAD);
                let len = if node_idx + 1 < n_nodes {
                    let pa = matrix4f_translation(&tree.nodes[node_idx].data.world_transform);
                    let pb = matrix4f_translation(&tree.nodes[node_idx + 1].data.world_transform);
                    vector3(
                        pb.get(0) - pa.get(0),
                        pb.get(1) - pa.get(1),
                        pb.get(2) - pa.get(2),
                    )
                    .norm()
                } else {
                    data.active = false;
                    continue;
                };
                (axis_unit, half_angle, len)
            }
            JointVariant::Spherical(_) => {
                if node_idx + 1 >= n_nodes {
                    data.active = false;
                    continue;
                }
                let pos_a = matrix4f_translation(&tree.nodes[node_idx].data.world_transform);
                let pos_b = matrix4f_translation(&tree.nodes[node_idx + 1].data.world_transform);
                let mut diff = vector3(
                    pos_b.get(0) - pos_a.get(0),
                    pos_b.get(1) - pos_a.get(1),
                    pos_b.get(2) - pos_a.get(2),
                );
                let len = diff.norm();
                if len < 1e-9 {
                    data.active = false;
                    continue;
                }
                diff.set(0, diff.get(0) / len);
                diff.set(1, diff.get(1) / len);
                diff.set(2, diff.get(2) / len);
                (diff, DEFAULT_CONE_HALF_ANGLE_RAD, len)
            }
            JointVariant::Revolute2d(_) => {
                if node_idx + 1 >= n_nodes {
                    data.active = false;
                    continue;
                }
                let pos_a = matrix4f_translation(&tree.nodes[node_idx].data.world_transform);
                let pos_b = matrix4f_translation(&tree.nodes[node_idx + 1].data.world_transform);
                let mut diff = vector3(
                    pos_b.get(0) - pos_a.get(0),
                    pos_b.get(1) - pos_a.get(1),
                    pos_b.get(2) - pos_a.get(2),
                );
                let len = diff.norm();
                if len < 1e-9 {
                    data.active = false;
                    continue;
                }
                diff.set(0, diff.get(0) / len);
                diff.set(1, diff.get(1) / len);
                diff.set(2, diff.get(2) / len);
                let half_angle = joint
                    .angle_limits()
                    .map(|(lo, hi)| (hi - lo) * 0.5)
                    .unwrap_or(DEFAULT_CONE_HALF_ANGLE_RAD);
                (diff, half_angle, len)
            }
            JointVariant::Fixed(_)
            | JointVariant::Fixed2d(_)
            | JointVariant::Prismatic(_)
            | JointVariant::Prismatic2d(_) => {
                data.active = false;
                continue;
            }
        };

        let cone_length = link_length * CONE_LENGTH_FACTOR;
        let base_radius = (cone_length * half_angle_rad.tan()).max(CONE_MIN_BASE_RADIUS);
        let pos = matrix4f_translation(&tree.nodes[node_idx].data.world_transform);
        let position = [
            off[0] + pos.get(0),
            off[1] + pos.get(1),
            off[2] + pos.get(2),
        ];
        let q = quat_from_z_to_direction(&cone_axis_world);
        let scale_z = cone_length / (2.0 * CONE_MESH_HALF_LENGTH);
        let scale_xy = base_radius / CONE_MESH_BASE_RADIUS;
        data.active = true;
        data.transform = Transform::from_position_quat(position, q);
        data.transform.scale = [scale_xy, scale_xy, scale_z];
    }
}

/// Syncs both chains to the render World.
pub fn sync_armature_to_world(scene: &mut KinematicsScene, world: &mut World) {
    let show = scene.show_joint_cones;
    sync_chain_to_world(&mut scene.chain_a, world, show);
    sync_chain_to_world(&mut scene.chain_b, world, show);
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

/// Maximum fraction of chain length to allow for IK target distance from root (clamp beyond this).
const TARGET_REACH_FRAC: f32 = 0.9;

/// Fraction of reach used for random target radius (slightly inside reach to avoid boundary).
const RANDOM_TARGET_REACH_FRAC: f32 = 0.85;

/// Minimum radius for random target sphere when chain reach is very small.
const RANDOM_TARGET_MIN_RADIUS: f32 = 0.05;

/// Iteration counts per solver per frame (real-time smoothing).
fn solver_iters(solver: IkSolverType) -> usize {
    match solver {
        IkSolverType::Fabrik => 10,
        IkSolverType::Jacobian => 8,
        IkSolverType::Ccd => 12,
        IkSolverType::Halley => 4,
        IkSolverType::FabrikSqp => 8,
        IkSolverType::Hessian => 6,
        #[cfg(feature = "neural")]
        IkSolverType::Neural => 1,
    }
}

/// Run one chain's IK step: clamp target to reach, run solver, reject if error increases.
fn step_chain_ik(chain: &mut ChainState) {
    chain.armature.update_kinematics();
    let root = chain.armature.end_effector_position(0);
    let reach = path_length_to(&chain.armature, chain.end_effector_idx) * TARGET_REACH_FRAC;
    // Target in armature-local space (world target minus root offset)
    let tlx = chain.ik_target[0] - chain.root_offset[0];
    let tly = chain.ik_target[1] - chain.root_offset[1];
    let tlz = chain.ik_target[2] - chain.root_offset[2];
    let dx = tlx - root.get(0);
    let dy = tly - root.get(1);
    let dz = tlz - root.get(2);
    let dist_sq = dx * dx + dy * dy + dz * dz;
    let dist = dist_sq.sqrt();
    if dist > reach && dist > 1e-6 {
        let scale = reach / dist;
        chain.ik_target[0] = chain.root_offset[0] + root.get(0) + dx * scale;
        chain.ik_target[1] = chain.root_offset[1] + root.get(1) + dy * scale;
        chain.ik_target[2] = chain.root_offset[2] + root.get(2) + dz * scale;
    }
    let local_tx = chain.ik_target[0] - chain.root_offset[0];
    let local_ty = chain.ik_target[1] - chain.root_offset[1];
    let local_tz = chain.ik_target[2] - chain.root_offset[2];
    let mut target = mathlib::Vector3f::with_capacity(3);
    target.set(0, local_tx);
    target.set(1, local_ty);
    target.set(2, local_tz);
    let ee_before = chain.armature.end_effector_position(chain.end_effector_idx);
    let err_before = ((ee_before.get(0) - local_tx).powi(2)
        + (ee_before.get(1) - local_ty).powi(2)
        + (ee_before.get(2) - local_tz).powi(2))
    .sqrt();
    let saved_state = chain.armature.pack();
    let iters = solver_iters(chain.ik_solver_type);
    let err_after = match chain.ik_solver_type {
        IkSolverType::Fabrik => FabrikIk::new(&mut chain.armature, chain.end_effector_idx, target)
            .with_max_iters(iters)
            .solve(),
        IkSolverType::Jacobian => {
            JacobianIk::new(&mut chain.armature, chain.end_effector_idx, target)
                .with_max_iters(iters)
                .with_damping(1e-3)
                .with_max_delta_rad(Some(0.8))
                .solve()
        }
        IkSolverType::Ccd => CcdIk::new(&mut chain.armature, chain.end_effector_idx, target)
            .with_max_iters(iters)
            .with_max_rotation_rad(0.8)
            .solve(),
        IkSolverType::Halley => {
            let target_pose = new_translation(&target);
            HalleyIk::new(&mut chain.armature, chain.end_effector_idx, target_pose)
                .with_max_iters(iters)
                .solve()
        }
        IkSolverType::FabrikSqp => {
            FabrikSqpIk::new(&mut chain.armature, chain.end_effector_idx, target)
                .with_nl(iters)
                .with_max_iters_fallback(50)
                .solve()
        }
        IkSolverType::Hessian => {
            HessianIk::new(&mut chain.armature, chain.end_effector_idx, target)
                .with_max_iters(iters)
                .solve()
        }
        #[cfg(feature = "neural")]
        IkSolverType::Neural => {
            let dof = chain.armature.pack().len();
            if dof != 3 {
                chain.armature.unpack(&saved_state);
                err_before
            } else {
                if chain.neural_session.is_none() {
                    let path = std::env::var("NEURAL_IK_ONNX").ok().unwrap_or_else(|| {
                        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                        p.push("../../neural/iknet.onnx");
                        p.to_string_lossy().into_owned()
                    });
                    chain.neural_session = OnnxIkSession::load_path(&path);
                }
                match &chain.neural_session {
                    None => {
                        chain.armature.unpack(&saved_state);
                        err_before
                    }
                    Some(session) => {
                        let chain_config = ChainConfig::new(3, false)
                            .with_workspace([-1.5, -1.5, -1.5], [1.5, 1.5, 1.5]);
                        let pos = [local_tx, local_ty, local_tz];
                        let normalized = normalize_position(pos, &chain_config);
                        let output = session.predict(&normalized);
                        if output.len() != 3 {
                            chain.armature.unpack(&saved_state);
                            err_before
                        } else {
                            let mut joints = [0.0_f32; 3];
                            denormalize_joints(&output, &chain_config, &mut joints);
                            chain.armature.unpack(&joints);
                            let ee_after =
                                chain.armature.end_effector_position(chain.end_effector_idx);
                            let err = ((ee_after.get(0) - local_tx).powi(2)
                                + (ee_after.get(1) - local_ty).powi(2)
                                + (ee_after.get(2) - local_tz).powi(2))
                            .sqrt();
                            err
                        }
                    }
                }
            }
        }
    };
    if err_after > err_before {
        chain.armature.unpack(&saved_state);
    }
}

/// Run IK for both chains and sync to world.
pub fn step_ik(scene: &mut KinematicsScene, world: &mut World) {
    let _n = crate::wasm_debug::next_frame();
    step_chain_ik(&mut scene.chain_a);
    step_chain_ik(&mut scene.chain_b);
    sync_armature_to_world(scene, world);
}

/// Format one chain's armature tree as a newline-separated string.
#[must_use]
pub fn format_armature_tree_for_chain(chain: &ChainState) -> String {
    let tree = chain.armature.tree();
    let n = tree.num_nodes();
    let mut lines = Vec::with_capacity(n);
    for (idx, node) in tree.nodes.iter().take(n).enumerate() {
        let (type_name, angle_opt, limits_opt) = joint_label(&node.data.joint);
        let text = match (angle_opt, limits_opt) {
            (Some(a), Some((lo, hi))) => {
                format!("{}: {} {:.3} rad [{:.2}, {:.2}]", idx, type_name, a, lo, hi)
            }
            (Some(a), None) => format!("{}: {} {:.3} rad", idx, type_name, a),
            (None, _) => format!("{}: {}", idx, type_name),
        };
        lines.push(text);
    }
    lines.join("\n")
}

/// Format the active chain's armature tree for the HTML tree panel.
#[must_use]
pub fn format_armature_tree(scene: &KinematicsScene) -> String {
    let chain = match scene.active_chain {
        ChainIndex::A => &scene.chain_a,
        ChainIndex::B => &scene.chain_b,
    };
    let prefix = match scene.active_chain {
        ChainIndex::A => "A: ",
        ChainIndex::B => "B: ",
    };
    format!("{}{}", prefix, format_armature_tree_for_chain(chain))
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

/// Build the armature tree panel (active chain: node index, joint type, angle).
#[must_use]
pub fn build_armature_tree_panel(scene: &KinematicsScene, _viewport_width: f32) -> Window {
    const ROW_H: f32 = 18.0;
    const W: f32 = 200.0;
    const SCENE_PANEL_W: f32 = 160.0;
    let chain = match scene.active_chain {
        ChainIndex::A => &scene.chain_a,
        ChainIndex::B => &scene.chain_b,
    };
    let tree = chain.armature.tree();
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

/// Build the armature controls panel (active chain: solver, arm, EE, random target, readouts).
#[must_use]
pub fn build_armature_controls_panel(
    scene: &KinematicsScene,
    viewport_width: f32,
) -> (Window, HashMap<ControlId, KinematicsAction>) {
    let mut mapping = HashMap::new();
    let active = scene.active_chain;
    let chain = match active {
        ChainIndex::A => &scene.chain_a,
        ChainIndex::B => &scene.chain_b,
    };
    const ROW_H: f32 = 24.0;
    const W: f32 = 220.0;
    const BTN_W: f32 = 36.0;
    const SOLVER_BTN_W: f32 = 36.0;
    let n = chain.armature.tree().num_nodes();
    let num_ee = n.saturating_sub(1);
    let chain_row = 1.0;
    let solver_rows = 2.0;
    let arm_rows = 2.0;
    let h = (chain_row * (ROW_H + 2.0))
        + (solver_rows * (ROW_H + 2.0))
        + (arm_rows * (ROW_H + 2.0))
        + (num_ee as f32 * (ROW_H + 2.0))
        + (2.0 * (ROW_H + 2.0))
        + (2.0 * (ROW_H + 2.0))
        + (1.0 * (ROW_H + 2.0))
        + (1.0 * (ROW_H + 2.0))
        + (if cfg!(all(feature = "bvh", not(target_arch = "wasm32"))) {
            1.0 * (ROW_H + 2.0)
        } else {
            0.0
        })
        + 60.0;
    const SCENE_PANEL_W: f32 = 160.0;
    const ARMATURE_TREE_W: f32 = 200.0;
    let preferred_x = 10.0 + SCENE_PANEL_W + 10.0 + ARMATURE_TREE_W + 10.0;
    let max_x = (viewport_width - W - 10.0).max(10.0);
    let x = preferred_x.min(max_x).max(10.0);
    let rect = Rect::new(x, 10.0, W, h);
    let mut window = Window::new(ARMATURE_CONTROLS_WINDOW_ID, rect);
    window.title_bar_height = 20.0;
    let body = window.body_rect();
    let mut y = body.y + 4.0;
    let mut next_id = ARMATURE_CONTROLS_WINDOW_ID.0 + 1;

    // Chain A / B selector (labels so user can tell which is which and which is active)
    window.add_label(Label::new(
        ControlId(next_id),
        Rect::new(body.x + 8.0, y, 50.0, ROW_H),
        "Chain:".to_string(),
    ));
    next_id += 1;
    let a_rect = Rect::new(body.x + 58.0, y, SOLVER_BTN_W, ROW_H - 2.0);
    let b_rect = Rect::new(
        body.x + 58.0 + SOLVER_BTN_W + 4.0,
        y,
        SOLVER_BTN_W,
        ROW_H - 2.0,
    );
    window.add_button(Button::new(ControlId(next_id), a_rect));
    mapping.insert(
        ControlId(next_id),
        KinematicsAction::SetActiveChain(ChainIndex::A),
    );
    next_id += 1;
    window.add_button(Button::new(ControlId(next_id), b_rect));
    mapping.insert(
        ControlId(next_id),
        KinematicsAction::SetActiveChain(ChainIndex::B),
    );
    next_id += 1;
    let a_label = if active == ChainIndex::A { "A*" } else { "A" };
    let b_label = if active == ChainIndex::B { "B*" } else { "B" };
    window.add_label(Label::new(ControlId(next_id), a_rect, a_label.to_string()));
    next_id += 1;
    window.add_label(Label::new(ControlId(next_id), b_rect, b_label.to_string()));
    next_id += 1;
    y += ROW_H + 2.0;

    // Solver: FABRIK | Jacobian | CCD | Halley (6D pose)
    window.add_label(Label::new(
        ControlId(next_id),
        Rect::new(body.x + 8.0, y, body.w - 16.0, ROW_H),
        "Solver:".to_string(),
    ));
    next_id += 1;
    y += ROW_H + 2.0;
    let solvers = [
        (IkSolverType::Fabrik, "FABRIK"),
        (IkSolverType::FabrikSqp, "FABRIK+SQP"),
        (IkSolverType::Jacobian, "Jacob"),
        (IkSolverType::Ccd, "CCD"),
        (IkSolverType::Hessian, "Hessian"),
    ];
    let mut x_off = body.x + 8.0;
    for (solver_type, label) in solvers {
        let is_active = chain.ik_solver_type == solver_type;
        let text = if is_active {
            format!("{}*", label)
        } else {
            label.to_string()
        };
        window.add_label(Label::new(
            ControlId(next_id),
            Rect::new(x_off, y, 40.0, ROW_H),
            text,
        ));
        next_id += 1;
        let btn_rect = Rect::new(x_off + 42.0, y, SOLVER_BTN_W, ROW_H - 2.0);
        let cid = ControlId(next_id);
        next_id += 1;
        window.add_button(Button::new(cid, btn_rect));
        mapping.insert(cid, KinematicsAction::SetSolverType(active, solver_type));
        x_off += 42.0 + SOLVER_BTN_W + 4.0;
    }
    let solver_type = IkSolverType::Halley;
    let is_active = chain.ik_solver_type == solver_type;
    let text = if is_active { "Halley*" } else { "Halley" };
    window.add_label(Label::new(
        ControlId(next_id),
        Rect::new(x_off, y, 44.0, ROW_H),
        text.to_string(),
    ));
    next_id += 1;
    let cid = ControlId(next_id);
    next_id += 1;
    window.add_button(Button::new(
        cid,
        Rect::new(x_off + 46.0, y, SOLVER_BTN_W, ROW_H - 2.0),
    ));
    mapping.insert(cid, KinematicsAction::SetSolverType(active, solver_type));
    #[cfg(feature = "neural")]
    {
        x_off += 46.0 + SOLVER_BTN_W + 4.0;
        let solver_type = IkSolverType::Neural;
        let is_active = chain.ik_solver_type == solver_type;
        let text = if is_active { "Neural*" } else { "Neural" };
        window.add_label(Label::new(
            ControlId(next_id),
            Rect::new(x_off, y, 44.0, ROW_H),
            text.to_string(),
        ));
        next_id += 1;
        let cid = ControlId(next_id);
        next_id += 1;
        window.add_button(Button::new(
            cid,
            Rect::new(x_off + 46.0, y, SOLVER_BTN_W, ROW_H - 2.0),
        ));
        mapping.insert(cid, KinematicsAction::SetSolverType(active, solver_type));
    }
    y += ROW_H + 2.0;

    // Arm preset: Spherical | Revolute | MixedArm | SphericalSnake
    window.add_label(Label::new(
        ControlId(next_id),
        Rect::new(body.x + 8.0, y, body.w - 16.0, ROW_H),
        "Arm:".to_string(),
    ));
    next_id += 1;
    y += ROW_H + 2.0;
    let presets = [
        (ArmPreset::Spherical, "Sph"),
        (ArmPreset::Revolute, "Rev"),
        (ArmPreset::MixedArm, "Mixed"),
        (ArmPreset::SphericalSnake, "Snake"),
        #[cfg(feature = "bvh")]
        (ArmPreset::Bvh, "BVH"),
    ];
    x_off = body.x + 8.0;
    for (preset, label) in presets {
        let is_active = chain.arm_preset == preset;
        let text = if is_active {
            format!("{}*", label)
        } else {
            label.to_string()
        };
        window.add_label(Label::new(
            ControlId(next_id),
            Rect::new(x_off, y, 38.0, ROW_H),
            text,
        ));
        next_id += 1;
        let cid = ControlId(next_id);
        next_id += 1;
        window.add_button(Button::new(
            cid,
            Rect::new(x_off + 40.0, y, SOLVER_BTN_W, ROW_H - 2.0),
        ));
        mapping.insert(cid, KinematicsAction::SetArmPreset(active, preset));
        x_off += 40.0 + SOLVER_BTN_W + 4.0;
    }
    y += ROW_H + 2.0;

    for idx in 1..n {
        let label_rect = Rect::new(body.x + 8.0, y, body.w - 16.0, ROW_H);
        let btn_rect = Rect::new(body.x + body.w - BTN_W - 12.0, y, BTN_W, ROW_H - 2.0);
        let btn_cid = ControlId(next_id);
        next_id += 1;
        let label_text = format!(
            "EE: {} {}",
            idx,
            if idx == chain.end_effector_idx {
                "(active)"
            } else {
                ""
            }
        );
        window.add_label(Label::new(ControlId(next_id), label_rect, label_text));
        next_id += 1;
        window.add_button(Button::new(btn_cid, btn_rect));
        mapping.insert(btn_cid, KinematicsAction::SetEndEffector(active, idx));
        y += ROW_H + 2.0;
    }
    let random_cid = ControlId(next_id);
    next_id += 1;
    window.add_button(Button::new(
        random_cid,
        Rect::new(body.x + 8.0, y, BTN_W, ROW_H - 2.0),
    ));
    mapping.insert(random_cid, KinematicsAction::RandomTarget(active));
    window.add_label(Label::new(
        ControlId(next_id),
        Rect::new(body.x + 8.0 + BTN_W + 4.0, y, body.w - 20.0 - BTN_W, ROW_H),
        "Random target".to_string(),
    ));
    next_id += 1;
    y += ROW_H + 2.0;
    let set_ee_cid = ControlId(next_id);
    next_id += 1;
    window.add_button(Button::new(
        set_ee_cid,
        Rect::new(body.x + 8.0, y, BTN_W, ROW_H - 2.0),
    ));
    mapping.insert(set_ee_cid, KinematicsAction::SetTargetToEndEffector(active));
    window.add_label(Label::new(
        ControlId(next_id),
        Rect::new(body.x + 8.0 + BTN_W + 4.0, y, body.w - 20.0 - BTN_W, ROW_H),
        "Set target to EE".to_string(),
    ));
    next_id += 1;
    y += ROW_H + 2.0;

    let cones_cid = ControlId(next_id);
    next_id += 1;
    window.add_button(Button::new(
        cones_cid,
        Rect::new(body.x + 8.0, y, BTN_W, ROW_H - 2.0),
    ));
    mapping.insert(cones_cid, KinematicsAction::ToggleShowJointCones);
    let cones_label = if scene.show_joint_cones {
        "Show cones (on)"
    } else {
        "Show cones (off)"
    };
    window.add_label(Label::new(
        ControlId(next_id),
        Rect::new(body.x + 8.0 + BTN_W + 4.0, y, body.w - 20.0 - BTN_W, ROW_H),
        cones_label.to_string(),
    ));
    next_id += 1;
    y += ROW_H + 2.0;

    let ee = chain.armature.end_effector_position(chain.end_effector_idx);
    let ee_world = [
        chain.root_offset[0] + ee.get(0),
        chain.root_offset[1] + ee.get(1),
        chain.root_offset[2] + ee.get(2),
    ];
    window.add_label(Label::new(
        ControlId(next_id),
        Rect::new(body.x + 4.0, y, body.w - 8.0, ROW_H),
        format!(
            "EE: ({:.3}, {:.3}, {:.3})",
            ee_world[0], ee_world[1], ee_world[2]
        ),
    ));
    next_id += 1;
    y += ROW_H + 2.0;
    window.add_label(Label::new(
        ControlId(next_id),
        Rect::new(body.x + 4.0, y, body.w - 8.0, ROW_H),
        format!(
            "Target: ({:.3}, {:.3}, {:.3})",
            chain.ik_target[0], chain.ik_target[1], chain.ik_target[2]
        ),
    ));
    next_id += 1;
    y += ROW_H + 2.0;
    let reset_cid = ControlId(next_id);
    next_id += 1;
    window.add_button(Button::new(
        reset_cid,
        Rect::new(body.x + 8.0, y, BTN_W, ROW_H - 2.0),
    ));
    mapping.insert(reset_cid, KinematicsAction::ResetScene);
    window.add_label(Label::new(
        ControlId(next_id),
        Rect::new(body.x + 8.0 + BTN_W + 4.0, y, body.w - 20.0 - BTN_W, ROW_H),
        "Reset scene".to_string(),
    ));
    #[cfg(all(feature = "bvh", not(target_arch = "wasm32")))]
    {
        next_id += 1;
        y += ROW_H + 2.0;
        let load_bvh_cid = ControlId(next_id);
        next_id += 1;
        window.add_button(Button::new(
            load_bvh_cid,
            Rect::new(body.x + 8.0, y, BTN_W, ROW_H - 2.0),
        ));
        mapping.insert(load_bvh_cid, KinematicsAction::LoadBvh(active));
        window.add_label(Label::new(
            ControlId(next_id),
            Rect::new(body.x + 8.0 + BTN_W + 4.0, y, body.w - 20.0 - BTN_W, ROW_H),
            "Load BVH…".to_string(),
        ));
    }
    (window, mapping)
}

/// Computes the total path length from root to the given node (sum of segment lengths).
#[must_use]
fn path_length_to(armature: &Armature, end_idx: usize) -> f32 {
    let path = armature.path_to(end_idx);
    if path.len() < 2 {
        return 0.0;
    }
    let positions: Vec<Vector3f> = path
        .iter()
        .map(|&i| armature.end_effector_position(i))
        .collect();
    (0..positions.len() - 1)
        .map(|i| {
            let a = &positions[i];
            let b = &positions[i + 1];
            let dx = b.get(0) - a.get(0);
            let dy = b.get(1) - a.get(1);
            let dz = b.get(2) - a.get(2);
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum()
}

/// Randomize the IK target for the given chain (world-space target uniformly inside reach sphere).
pub fn randomize_ik_target_for_chain(chain: &mut ChainState) {
    chain.armature.update_kinematics();
    let root = chain.armature.end_effector_position(0);
    let reach = path_length_to(&chain.armature, chain.end_effector_idx) * RANDOM_TARGET_REACH_FRAC;
    let radius = reach.max(RANDOM_TARGET_MIN_RADIUS);
    // Uniform sampling in sphere: r = radius * cbrt(u), then spherical coords for direction.
    let u: f32 = rand::random();
    let r = radius * u.cbrt();
    let theta = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
    let v = rand::random::<f32>() * 2.0 - 1.0;
    let phi = v.acos();
    let local_x = r * phi.sin() * theta.cos();
    let local_y = r * phi.sin() * theta.sin();
    let local_z = r * phi.cos();
    chain.ik_target = [
        chain.root_offset[0] + root.get(0) + local_x,
        chain.root_offset[1] + root.get(1) + local_y,
        chain.root_offset[2] + root.get(2) + local_z,
    ];
    crate::wasm_debug::log(&format!(
        "[kinematics] random_target ({:.3},{:.3},{:.3})",
        chain.ik_target[0], chain.ik_target[1], chain.ik_target[2]
    ));
}

/// Set the IK target to the current end-effector position for the given chain (world-space).
pub fn set_target_to_end_effector_for_chain(chain: &mut ChainState) {
    chain.armature.update_kinematics();
    let ee = chain.armature.end_effector_position(chain.end_effector_idx);
    chain.ik_target = [
        chain.root_offset[0] + ee.get(0),
        chain.root_offset[1] + ee.get(1),
        chain.root_offset[2] + ee.get(2),
    ];
    crate::wasm_debug::log(&format!(
        "[kinematics] set_target_to_ee ({:.3},{:.3},{:.3})",
        chain.ik_target[0], chain.ik_target[1], chain.ik_target[2]
    ));
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
    let action_desc = match *action {
        KinematicsAction::SetEndEffector(c, idx) => format!("SetEndEffector({:?}, {})", c, idx),
        KinematicsAction::SelectEntity(id) => format!("SelectEntity({:?})", id),
        KinematicsAction::RandomTarget(c) => format!("RandomTarget({:?})", c),
        KinematicsAction::SetSolverType(c, t) => {
            let name = match t {
                IkSolverType::Fabrik => "Fabrik",
                IkSolverType::FabrikSqp => "FabrikSqp",
                IkSolverType::Jacobian => "Jacobian",
                IkSolverType::Ccd => "Ccd",
                IkSolverType::Halley => "Halley",
                IkSolverType::Hessian => "Hessian",
                #[cfg(feature = "neural")]
                IkSolverType::Neural => "Neural",
            };
            format!("SetSolverType({:?}, {})", c, name)
        }
        KinematicsAction::SetArmPreset(c, p) => format!("SetArmPreset({:?}, {:?})", c, p),
        KinematicsAction::SetActiveChain(c) => format!("SetActiveChain({:?})", c),
        KinematicsAction::SetTargetToEndEffector(c) => format!("SetTargetToEndEffector({:?})", c),
        KinematicsAction::ResetScene => "ResetScene".to_string(),
        KinematicsAction::ToggleShowJointCones => "ToggleShowJointCones".to_string(),
        #[cfg(feature = "bvh")]
        KinematicsAction::LoadBvh(c) => format!("LoadBvh({:?})", c),
    };
    crate::wasm_debug::log(&format!("[kinematics] action: {}", action_desc));
    match *action {
        KinematicsAction::SetEndEffector(chain_idx, idx) => {
            let chain = match chain_idx {
                ChainIndex::A => &mut scene.chain_a,
                ChainIndex::B => &mut scene.chain_b,
            };
            if idx < chain.armature.tree().num_nodes() {
                chain.end_effector_idx = idx;
            }
        }
        KinematicsAction::SelectEntity(id) => {
            engine.set_selected_entity(Some(id));
        }
        KinematicsAction::RandomTarget(chain_idx) => {
            let chain = match chain_idx {
                ChainIndex::A => &mut scene.chain_a,
                ChainIndex::B => &mut scene.chain_b,
            };
            randomize_ik_target_for_chain(chain);
        }
        KinematicsAction::SetSolverType(chain_idx, t) => {
            let chain = match chain_idx {
                ChainIndex::A => &mut scene.chain_a,
                ChainIndex::B => &mut scene.chain_b,
            };
            chain.ik_solver_type = t;
        }
        KinematicsAction::SetArmPreset(chain_idx, preset) => {
            let chain = match chain_idx {
                ChainIndex::A => &mut scene.chain_a,
                ChainIndex::B => &mut scene.chain_b,
            };
            let to_despawn: Vec<EntityId> = chain
                .joint_entities
                .iter()
                .chain(chain.link_entities.iter())
                .chain(chain.cone_entities.iter())
                .copied()
                .chain(std::iter::once(chain.target_entity))
                .collect();
            for &e in &to_despawn {
                engine.world_mut().despawn(e);
            }
            let new_chain = build_chain(
                engine.world_mut(),
                preset,
                chain.root_offset,
                chain.color,
                chain.target_color,
                chain.ik_solver_type,
            );
            match chain_idx {
                ChainIndex::A => scene.chain_a = new_chain,
                ChainIndex::B => scene.chain_b = new_chain,
            }
        }
        KinematicsAction::SetActiveChain(c) => {
            scene.active_chain = c;
        }
        KinematicsAction::SetTargetToEndEffector(chain_idx) => {
            let chain = match chain_idx {
                ChainIndex::A => &mut scene.chain_a,
                ChainIndex::B => &mut scene.chain_b,
            };
            set_target_to_end_effector_for_chain(chain);
        }
        KinematicsAction::ResetScene => {
            let world = engine.world_mut();
            for &e in scene
                .chain_a
                .joint_entities
                .iter()
                .chain(scene.chain_a.link_entities.iter())
                .chain(scene.chain_a.cone_entities.iter())
                .chain(std::iter::once(&scene.chain_a.target_entity))
            {
                world.despawn(e);
            }
            for &e in scene
                .chain_b
                .joint_entities
                .iter()
                .chain(scene.chain_b.link_entities.iter())
                .chain(scene.chain_b.cone_entities.iter())
                .chain(std::iter::once(&scene.chain_b.target_entity))
            {
                world.despawn(e);
            }
            *scene = build_kinematics_scene(world);
        }
        KinematicsAction::ToggleShowJointCones => {
            scene.show_joint_cones = !scene.show_joint_cones;
        }
        #[cfg(feature = "bvh")]
        KinematicsAction::LoadBvh(chain_idx) => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("BVH", &["bvh"])
                    .pick_file()
                {
                    if let Ok(bytes) = std::fs::read(&path) {
                        if let Ok(bvh) = parse::bvh::parse(&bytes) {
                            if let Some(armature) = crate::bvh_import::armature_from_bvh(&bvh) {
                                let chain = match chain_idx {
                                    ChainIndex::A => &mut scene.chain_a,
                                    ChainIndex::B => &mut scene.chain_b,
                                };
                                let to_despawn: Vec<EntityId> = chain
                                    .joint_entities
                                    .iter()
                                    .chain(chain.link_entities.iter())
                                    .chain(chain.cone_entities.iter())
                                    .copied()
                                    .chain(std::iter::once(chain.target_entity))
                                    .collect();
                                for &e in &to_despawn {
                                    engine.world_mut().despawn(e);
                                }
                                let new_chain = build_chain_from_armature(
                                    engine.world_mut(),
                                    armature,
                                    chain.root_offset,
                                    chain.color,
                                    chain.target_color,
                                    chain.ik_solver_type,
                                );
                                match chain_idx {
                                    ChainIndex::A => scene.chain_a = new_chain,
                                    ChainIndex::B => scene.chain_b = new_chain,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Build the scene entity tree panel (both chains: A Joint/Link/Target, B Joint/Link/Target).
#[must_use]
pub fn build_scene_entity_panel(
    scene: &KinematicsScene,
    _world: &World,
) -> (Window, HashMap<ControlId, KinematicsAction>) {
    let mut mapping = HashMap::new();
    const ROW_H: f32 = 24.0;
    const W: f32 = 160.0;
    let x = 10.0;
    let n_rows = scene.chain_a.joint_entities.len()
        + scene.chain_a.link_entities.len()
        + 1
        + scene.chain_b.joint_entities.len()
        + scene.chain_b.link_entities.len()
        + 1;
    let h = (n_rows as f32 * (ROW_H + 2.0)) + 28.0;
    let rect = Rect::new(x, 10.0, W, h);
    let mut window = Window::new(SCENE_ENTITY_WINDOW_ID, rect);
    window.title_bar_height = 20.0;
    let body = window.body_rect();
    let mut y = body.y + 4.0;
    let mut next_id = SCENE_ENTITY_WINDOW_ID.0 + 1;
    for (i, &ent_id) in scene.chain_a.joint_entities.iter().enumerate() {
        let label_rect = Rect::new(body.x + 8.0, y, body.w - 16.0, ROW_H);
        let cid = ControlId(next_id);
        next_id += 1;
        window.add_label(Label::new(
            ControlId(next_id),
            label_rect,
            format!("A Joint {}", i),
        ));
        next_id += 1;
        window.add_button(Button::new(cid, label_rect));
        mapping.insert(cid, KinematicsAction::SelectEntity(ent_id));
        y += ROW_H + 2.0;
    }
    for (i, &ent_id) in scene.chain_a.link_entities.iter().enumerate() {
        let label_rect = Rect::new(body.x + 8.0, y, body.w - 16.0, ROW_H);
        let cid = ControlId(next_id);
        next_id += 1;
        window.add_label(Label::new(
            ControlId(next_id),
            label_rect,
            format!("A Link {}", i),
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
        "A Target".to_string(),
    ));
    next_id += 1;
    window.add_button(Button::new(cid, label_rect));
    mapping.insert(
        cid,
        KinematicsAction::SelectEntity(scene.chain_a.target_entity),
    );
    y += ROW_H + 2.0;
    for (i, &ent_id) in scene.chain_b.joint_entities.iter().enumerate() {
        let label_rect = Rect::new(body.x + 8.0, y, body.w - 16.0, ROW_H);
        let cid = ControlId(next_id);
        next_id += 1;
        window.add_label(Label::new(
            ControlId(next_id),
            label_rect,
            format!("B Joint {}", i),
        ));
        next_id += 1;
        window.add_button(Button::new(cid, label_rect));
        mapping.insert(cid, KinematicsAction::SelectEntity(ent_id));
        y += ROW_H + 2.0;
    }
    for (i, &ent_id) in scene.chain_b.link_entities.iter().enumerate() {
        let label_rect = Rect::new(body.x + 8.0, y, body.w - 16.0, ROW_H);
        let cid = ControlId(next_id);
        next_id += 1;
        window.add_label(Label::new(
            ControlId(next_id),
            label_rect,
            format!("B Link {}", i),
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
        "B Target".to_string(),
    ));
    window.add_button(Button::new(cid, label_rect));
    mapping.insert(
        cid,
        KinematicsAction::SelectEntity(scene.chain_b.target_entity),
    );
    (window, mapping)
}
