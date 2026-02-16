//! Physics scene: XPBD rigid bodies, soft bodies, collision, sync to render World.

use collision::Aabb;
use mathlib::Quat4f;
use physics::spatial::broad_phase_pairs;
use physics::Particle;
use physics::{
    CollisionShape, Integrator, PhysicsState, PlaneConstraint, RigidBody, RigidContactConstraint,
    SoftBody, TetMesh, XpbdIntegrator,
};
use render::scene::{EntityId, Primitive, Primitive3D, Transform, World};
use render::{Button, ControlId, Label, Rect, Window};

const GROUND_SCALE: f32 = 6.0;
const GROUND_Y: f32 = 0.0;
const MAX_BODIES: usize = 64;
const MAX_SOFT_BODIES: usize = 4;

/// Index of the ground body in `state.rigid_bodies` (always 0).
const GROUND_IDX: usize = 0;

/// Jelly vertex render scale and ground constraint radius.
const JELLY_PARTICLE_RADIUS: f32 = 0.025;

/// ControlId for the spawn panel window.
pub const SPAWN_WINDOW_ID: ControlId = ControlId(0xFF00_0100);
/// ControlId for the "Drop Sphere" button.
pub const SPAWN_SPHERE_ID: ControlId = ControlId(0xFF00_0101);
/// ControlId for the "Drop Box" button.
pub const SPAWN_BOX_ID: ControlId = ControlId(0xFF00_0102);
/// ControlId for the "Reset" button.
pub const RESET_BUTTON_ID: ControlId = ControlId(0xFF00_0103);
/// ControlId for the "Drop Jelly" button.
pub const SPAWN_JELLY_ID: ControlId = ControlId(0xFF00_0104);
/// ControlId for the bodies tree panel.
pub const BODIES_TREE_WINDOW_ID: ControlId = ControlId(0xFF00_0110);

/// Coral color for soft body vertices.
fn jelly_color() -> [f32; 4] {
    [1.0, 0.5, 0.4, 1.0]
}

/// Scene state: physics state, integrator, and entity IDs for each body.
pub struct PhysicsScene {
    /// Physics state (rigid bodies + soft bodies).
    pub state: PhysicsState,
    /// XPBD integrator (holds constraints).
    pub integrator: XpbdIntegrator,
    /// Entity IDs for rigid bodies (same length as state.rigid_bodies).
    pub rb_entities: Vec<EntityId>,
    /// One Vec<EntityId> per soft body (one entity per vertex).
    pub sb_particle_entities: Vec<Vec<EntityId>>,
    /// Entity ID for the ground plane visual.
    pub ground_entity: EntityId,
    /// Spawn counter: alternates sphere/box.
    pub spawn_counter: u32,
}

/// Convert a physics body quaternion `[w, x, y, z]` to `Quat4f`.
fn body_quat_to_quat4f(q: &[f32; 4]) -> Quat4f {
    Quat4f {
        w: q[0],
        x: q[1],
        y: q[2],
        z: q[3],
    }
}

/// Map collision shape to render scale.
fn shape_to_scale(shape: &CollisionShape) -> [f32; 3] {
    match shape {
        CollisionShape::Sphere { radius } => [*radius, *radius, *radius],
        CollisionShape::Box { half_extents } => *half_extents,
        CollisionShape::Capsule {
            radius,
            half_height,
        } => [*radius, half_height + radius, *radius],
    }
}

/// Map collision shape to render primitive.
fn shape_to_primitive(shape: &CollisionShape) -> Primitive {
    match shape {
        CollisionShape::Sphere { .. } => Primitive::ThreeD(Primitive3D::Sphere),
        CollisionShape::Box { .. } => Primitive::ThreeD(Primitive3D::Cube),
        CollisionShape::Capsule { .. } => Primitive::ThreeD(Primitive3D::Capsule),
    }
}

/// Color palette for bodies.
fn body_color(index: usize) -> [f32; 4] {
    const COLORS: [[f32; 4]; 5] = [
        [0.4, 0.65, 1.0, 1.0],  // blue
        [0.4, 0.85, 0.5, 1.0],  // green
        [1.0, 0.65, 0.3, 1.0],  // orange
        [0.85, 0.45, 0.9, 1.0], // purple
        [1.0, 0.85, 0.3, 1.0],  // yellow
    ];
    COLORS[index % COLORS.len()]
}

/// Builds a physics scene with XPBD rigid bodies.
#[must_use]
pub fn build_physics_scene(world: &mut World) -> PhysicsScene {
    let mut state = PhysicsState::new();
    state.config.substeps = 4;
    state.config.solver_iterations = 8;
    state.config.gravity = [0.0, -9.81, 0.0];

    // Body 0: kinematic ground plane
    let ground = RigidBody::kinematic(
        [0.0, GROUND_Y, 0.0],
        CollisionShape::Box {
            half_extents: [5.0, 0.05, 5.0],
        },
    );
    state.add_rigid_body(ground);

    // Body 1: blue sphere
    let mut sphere1 = RigidBody::new(
        [0.0, 2.0, 0.0],
        1.0,
        CollisionShape::Sphere { radius: 0.15 },
    );
    sphere1.phase = 1;
    state.add_rigid_body(sphere1);

    // Body 2: green box with initial angular velocity
    let mut box1 = RigidBody::new(
        [-0.3, 3.0, 0.1],
        1.5,
        CollisionShape::Box {
            half_extents: [0.12, 0.12, 0.12],
        },
    );
    box1.omega = [0.0, 2.0, 0.0];
    box1.phase = 1;
    state.add_rigid_body(box1);

    // Body 3: orange sphere
    let mut sphere2 = RigidBody::new(
        [0.3, 4.0, -0.1],
        0.8,
        CollisionShape::Sphere { radius: 0.1 },
    );
    sphere2.phase = 1;
    state.add_rigid_body(sphere2);

    let mut integrator = XpbdIntegrator::new();

    // Jelly cube (soft body): TetMesh::cube(2,2,2,0.12), offset above ground
    let (jelly_verts, jelly_mesh) = TetMesh::cube(2, 2, 2, 0.12);
    let jelly_offset = state.particles.len();
    let jelly_pos_offset: [f32; 3] = [0.12, 1.5, -0.5];
    for v in &jelly_verts {
        let mut p = Particle::new(
            [
                v[0] + jelly_pos_offset[0],
                v[1] + jelly_pos_offset[1],
                v[2] + jelly_pos_offset[2],
            ],
            1.0,
        );
        p.radius = JELLY_PARTICLE_RADIUS;
        state.particles.push(p);
    }
    let jelly = SoftBody::new(jelly_offset, jelly_verts.len(), jelly_mesh, 200.0, 1000.0);
    state.add_soft_body(jelly);

    // Render entities
    let root = world.root_entity();

    // Ground visual (quad)
    let ground_entity = world.spawn(root);
    world.set_primitive(ground_entity, Primitive::ThreeD(Primitive3D::Quad));
    world.set_transform(
        ground_entity,
        Transform {
            position: [0.0, GROUND_Y, 0.0],
            rotation: [-std::f32::consts::FRAC_PI_2, 0.0, 0.0],
            rotation_quat: None,
            scale: [GROUND_SCALE, GROUND_SCALE, GROUND_SCALE],
        },
    );
    world.set_color(ground_entity, [0.35, 0.38, 0.4, 1.0]);

    // Entities for each rigid body (including ground at index 0)
    let mut rb_entities = Vec::with_capacity(state.rigid_bodies.len());
    for (i, rb) in state.rigid_bodies.iter().enumerate() {
        if i == GROUND_IDX {
            // Ground body doesn't get a separate shape entity; use the quad
            rb_entities.push(ground_entity);
            continue;
        }
        let e = world.spawn(root);
        world.set_primitive(e, shape_to_primitive(&rb.shape));
        let q = body_quat_to_quat4f(&rb.q);
        world.set_transform(
            e,
            Transform {
                position: rb.x,
                rotation: [0.0, 0.0, 0.0],
                rotation_quat: Some(q),
                scale: shape_to_scale(&rb.shape),
            },
        );
        world.set_color(e, body_color(i - 1));
        rb_entities.push(e);
    }

    // Soft body particle entities (one small sphere per vertex per soft body)
    let mut sb_particle_entities = Vec::with_capacity(state.soft_bodies.len());
    for sb in &state.soft_bodies {
        let mut entities = Vec::with_capacity(sb.num_vertices);
        for i in 0..sb.num_vertices {
            let idx = sb.particle_offset + i;
            let pos = state.particles[idx].x;
            let e = world.spawn(root);
            world.set_primitive(e, Primitive::ThreeD(Primitive3D::Sphere));
            world.set_transform(
                e,
                Transform {
                    position: pos,
                    rotation: [0.0, 0.0, 0.0],
                    rotation_quat: None,
                    scale: [
                        JELLY_PARTICLE_RADIUS * 2.0,
                        JELLY_PARTICLE_RADIUS * 2.0,
                        JELLY_PARTICLE_RADIUS * 2.0,
                    ],
                },
            );
            world.set_color(e, jelly_color());
            entities.push(e);
        }
        sb_particle_entities.push(entities);
    }

    add_ground_plane_constraints(&mut integrator, &state, GROUND_Y);

    PhysicsScene {
        state,
        integrator,
        rb_entities,
        sb_particle_entities,
        ground_entity,
        spawn_counter: 0,
    }
}

/// Add plane constraints for all non-kinematic particles (ground collision).
fn add_ground_plane_constraints(
    integrator: &mut XpbdIntegrator,
    state: &PhysicsState,
    ground_y: f32,
) {
    add_ground_plane_constraints_range(integrator, state, ground_y, 0, state.particles.len());
}

/// Add plane constraints for particles in the range [start, end).
fn add_ground_plane_constraints_range(
    integrator: &mut XpbdIntegrator,
    state: &PhysicsState,
    ground_y: f32,
    start: usize,
    end: usize,
) {
    let end = end.min(state.particles.len());
    for (i, p) in state.particles[start..end].iter().enumerate() {
        if p.inv_mass > 0.0 {
            let idx = start + i;
            integrator.add_particle_constraint(Box::new(PlaneConstraint::new(
                idx,
                [0.0, ground_y, 0.0],
                [0.0, 1.0, 0.0],
            )));
        }
    }
}

/// Spawn a sphere at the given position. Returns `false` if at max body cap.
pub fn spawn_sphere(
    scene: &mut PhysicsScene,
    world: &mut World,
    root: EntityId,
    position: [f32; 3],
) -> bool {
    spawn_rigid_body(
        scene,
        world,
        root,
        position,
        CollisionShape::Sphere { radius: 0.12 },
        1.0,
    )
}

/// Spawn a box at the given position. Returns `false` if at max body cap.
pub fn spawn_box_body(
    scene: &mut PhysicsScene,
    world: &mut World,
    root: EntityId,
    position: [f32; 3],
) -> bool {
    spawn_rigid_body(
        scene,
        world,
        root,
        position,
        CollisionShape::Box {
            half_extents: [0.1, 0.1, 0.1],
        },
        1.2,
    )
}

/// Spawn a jelly (soft body) cube at the given position. Returns `false` if at max soft body cap.
pub fn spawn_jelly(
    scene: &mut PhysicsScene,
    world: &mut World,
    root: EntityId,
    position: [f32; 3],
) -> bool {
    if scene.state.soft_bodies.len() >= MAX_SOFT_BODIES {
        return false;
    }
    let (jelly_verts, mesh) = TetMesh::cube(2, 2, 2, 0.12);
    let offset = scene.state.particles.len();
    for v in &jelly_verts {
        let mut p = Particle::new(
            [v[0] + position[0], v[1] + position[1], v[2] + position[2]],
            1.0,
        );
        p.radius = JELLY_PARTICLE_RADIUS;
        scene.state.particles.push(p);
    }
    // Remap tet indices to global particle indices
    let tets: Vec<[usize; 4]> = mesh
        .tets
        .iter()
        .map(|t| [t[0] + offset, t[1] + offset, t[2] + offset, t[3] + offset])
        .collect();
    let remapped_mesh = TetMesh {
        tets,
        dm_inv: mesh.dm_inv.clone(),
        rest_volumes: mesh.rest_volumes.clone(),
    };
    let sb = SoftBody::new(offset, jelly_verts.len(), remapped_mesh, 200.0, 1000.0);
    scene.state.add_soft_body(sb);

    let sb_ref = scene.state.soft_bodies.last().unwrap();
    let mut entities = Vec::with_capacity(sb_ref.num_vertices);
    for i in 0..sb_ref.num_vertices {
        let idx = sb_ref.particle_offset + i;
        let pos = scene.state.particles[idx].x;
        let e = world.spawn(root);
        world.set_primitive(e, Primitive::ThreeD(Primitive3D::Sphere));
        world.set_transform(
            e,
            Transform {
                position: pos,
                rotation: [0.0, 0.0, 0.0],
                rotation_quat: None,
                scale: [
                    JELLY_PARTICLE_RADIUS * 2.0,
                    JELLY_PARTICLE_RADIUS * 2.0,
                    JELLY_PARTICLE_RADIUS * 2.0,
                ],
            },
        );
        world.set_color(e, jelly_color());
        entities.push(e);
    }
    scene.sb_particle_entities.push(entities);

    add_ground_plane_constraints_range(
        &mut scene.integrator,
        &scene.state,
        GROUND_Y,
        offset,
        offset + jelly_verts.len(),
    );
    true
}

/// Spawn a rigid body with a given shape at the given position.
fn spawn_rigid_body(
    scene: &mut PhysicsScene,
    world: &mut World,
    root: EntityId,
    position: [f32; 3],
    shape: CollisionShape,
    mass: f32,
) -> bool {
    if scene.rb_entities.len() >= MAX_BODIES {
        return false;
    }
    let color_idx = scene.state.rigid_bodies.len() - 1; // -1 to skip ground
    let mut rb = RigidBody::new(position, mass, shape);
    // Give a small random angular velocity based on spawn counter
    let c = scene.spawn_counter as f32;
    rb.omega = [
        (c * 0.7).sin() * 1.5,
        (c * 1.3).cos() * 2.0,
        (c * 0.3).sin() * 1.0,
    ];
    rb.phase = 1;
    scene.state.add_rigid_body(rb);
    scene.spawn_counter += 1;

    let rb_ref = scene.state.rigid_bodies.last().unwrap();
    let e = world.spawn(root);
    world.set_primitive(e, shape_to_primitive(&rb_ref.shape));
    let q = body_quat_to_quat4f(&rb_ref.q);
    world.set_transform(
        e,
        Transform {
            position: rb_ref.x,
            rotation: [0.0, 0.0, 0.0],
            rotation_quat: Some(q),
            scale: shape_to_scale(&rb_ref.shape),
        },
    );
    world.set_color(e, body_color(color_idx));
    scene.rb_entities.push(e);
    true
}

/// Generate contact constraints for the current frame.
fn generate_rigid_contacts(scene: &mut PhysicsScene) {
    scene.integrator.clear_contacts();

    let bodies = &scene.state.rigid_bodies;

    // Sphere/box vs ground plane (y = GROUND_Y)
    for (i, rb) in bodies.iter().enumerate().skip(1) {
        if rb.is_kinematic() {
            continue;
        }
        let r = rb.shape.bounding_radius();
        let depth = r - (rb.x[1] - GROUND_Y);
        if depth > 0.0 {
            scene
                .integrator
                .rigid_contacts
                .push(RigidContactConstraint::new(
                    i,
                    GROUND_IDX,
                    [0.0, -r, 0.0],
                    [rb.x[0], 0.0, rb.x[2]],
                    [0.0, 1.0, 0.0],
                    depth,
                ));
        }
    }

    // Body-body: AABB broad phase then narrow phase (bounding radius overlap)
    let aabb_items: Vec<(usize, Aabb)> = bodies
        .iter()
        .enumerate()
        .map(|(i, rb)| {
            let (min, max) = rb.aabb_min_max();
            (i, Aabb::new(min, max))
        })
        .collect();
    let pairs = broad_phase_pairs(&aabb_items, Some(0..1)); // skip ground index 0
    for (i, j) in pairs {
        if bodies[i].is_kinematic() || bodies[j].is_kinematic() {
            continue;
        }
        let ri = bodies[i].shape.bounding_radius();
        let rj = bodies[j].shape.bounding_radius();
        let dx = bodies[i].x[0] - bodies[j].x[0];
        let dy = bodies[i].x[1] - bodies[j].x[1];
        let dz = bodies[i].x[2] - bodies[j].x[2];
        let dist_sq = dx * dx + dy * dy + dz * dz;
        let min_dist = ri + rj;
        if dist_sq < min_dist * min_dist && dist_sq > 1e-12 {
            let dist = dist_sq.sqrt();
            let inv_dist = 1.0 / dist;
            let normal = [dx * inv_dist, dy * inv_dist, dz * inv_dist];
            let depth = min_dist - dist;
            scene
                .integrator
                .rigid_contacts
                .push(RigidContactConstraint::new(
                    i,
                    j,
                    [-normal[0] * ri, -normal[1] * ri, -normal[2] * ri],
                    [normal[0] * rj, normal[1] * rj, normal[2] * rj],
                    normal,
                    depth,
                ));
        }
    }
}

/// Syncs rigid body and soft body positions to render World.
pub fn sync_bodies_to_world(scene: &PhysicsScene, world: &mut World) {
    for (i, &ent_id) in scene.rb_entities.iter().enumerate() {
        if i == GROUND_IDX {
            continue; // ground is static
        }
        if let Some(rb) = scene.state.rigid_bodies.get(i) {
            let q = body_quat_to_quat4f(&rb.q);
            if let Some(data) = world.get_mut(ent_id) {
                data.transform = Transform {
                    position: rb.x,
                    rotation: [0.0, 0.0, 0.0],
                    rotation_quat: Some(q),
                    scale: shape_to_scale(&rb.shape),
                };
            }
        }
    }
    for (sb, entities) in scene
        .state
        .soft_bodies
        .iter()
        .zip(scene.sb_particle_entities.iter())
    {
        for (i, &ent_id) in entities.iter().enumerate() {
            let idx = sb.particle_offset + i;
            if let Some(p) = scene.state.particles.get(idx) {
                if let Some(data) = world.get_mut(ent_id) {
                    data.transform.position = p.x;
                }
            }
        }
    }
}

/// Runs one physics step and syncs to World.
pub fn step_physics(scene: &mut PhysicsScene, world: &mut World, dt: f32) {
    #[cfg(target_arch = "wasm32")]
    {
        if crate::wasm_debug::get_debug_physics() {
            let n = crate::wasm_debug::current_frame();
            if n > 0 && n % 60 == 0 {
                for (i, rb) in scene.state.rigid_bodies.iter().skip(1).take(4).enumerate() {
                    let msg = format!(
                        "[physics] frame {} body {} pos=({:.2},{:.2},{:.2}) v=({:.2},{:.2},{:.2}) ω=({:.2},{:.2},{:.2})",
                        n, i + 1, rb.x[0], rb.x[1], rb.x[2], rb.v[0], rb.v[1], rb.v[2],
                        rb.omega[0], rb.omega[1], rb.omega[2]
                    );
                    crate::wasm_debug::log(&msg);
                }
            }
        }
    }
    generate_rigid_contacts(scene);
    scene.integrator.step(&mut scene.state, dt);
    sync_bodies_to_world(scene, world);
}

/// Reset scene: remove all dynamic bodies and soft bodies, rebuild initial scene.
pub fn reset_scene(scene: &mut PhysicsScene, world: &mut World) {
    // Remove render entities for dynamic rigid bodies
    for (i, &ent_id) in scene.rb_entities.iter().enumerate() {
        if i == GROUND_IDX {
            continue;
        }
        world.despawn(ent_id);
    }
    // Remove soft body particle entities
    for entities in &scene.sb_particle_entities {
        for &ent_id in entities {
            world.despawn(ent_id);
        }
    }
    // Rebuild (new integrator with plane constraints re-added in build_physics_scene)
    *scene = build_physics_scene(world);
}

/// Format the bodies list as a newline-separated string for the HTML tree panel.
#[must_use]
pub fn format_bodies_tree(scene: &PhysicsScene) -> String {
    let mut lines =
        Vec::with_capacity(scene.state.rigid_bodies.len() + scene.state.soft_bodies.len());
    for (i, rb) in scene.state.rigid_bodies.iter().enumerate() {
        if i == GROUND_IDX {
            continue;
        }
        let shape_str = match &rb.shape {
            CollisionShape::Sphere { radius } => format!("Sphere r={radius:.2}"),
            CollisionShape::Box { half_extents: h } => {
                format!("Box {:.2}x{:.2}x{:.2}", h[0], h[1], h[2])
            }
            CollisionShape::Capsule {
                radius,
                half_height,
            } => format!("Cap r={radius:.2} h={half_height:.2}"),
        };
        let line = format!(
            "{}: {} pos({:.2},{:.2},{:.2}) v({:.2},{:.2},{:.2}) \u{03c9}({:.2},{:.2},{:.2})",
            i,
            shape_str,
            rb.x[0],
            rb.x[1],
            rb.x[2],
            rb.v[0],
            rb.v[1],
            rb.v[2],
            rb.omega[0],
            rb.omega[1],
            rb.omega[2]
        );
        lines.push(line);
    }
    for (idx, sb) in scene.state.soft_bodies.iter().enumerate() {
        let mut com = [0.0f32; 3];
        let mut total_mass = 0.0f32;
        for i in 0..sb.num_vertices {
            let p = &scene.state.particles[sb.particle_offset + i];
            if p.inv_mass > 0.0 {
                let m = 1.0 / p.inv_mass;
                com[0] += p.x[0] * m;
                com[1] += p.x[1] * m;
                com[2] += p.x[2] * m;
                total_mass += m;
            }
        }
        if total_mass > 0.0 {
            com[0] /= total_mass;
            com[1] /= total_mass;
            com[2] /= total_mass;
        }
        let line = format!(
            "SB{}: Jelly {}v \u{03bc}={:.0} \u{03bb}={:.0} com({:.2},{:.2},{:.2})",
            idx, sb.num_vertices, sb.mu, sb.lambda, com[0], com[1], com[2]
        );
        lines.push(line);
    }
    lines.join("\n")
}

/// Compute total linear momentum of all dynamic bodies.
#[must_use]
pub fn total_momentum(scene: &PhysicsScene) -> [f32; 3] {
    let mut p = [0.0f32; 3];
    for rb in &scene.state.rigid_bodies {
        if rb.is_kinematic() {
            continue;
        }
        let mass = if rb.inv_mass > 0.0 {
            1.0 / rb.inv_mass
        } else {
            0.0
        };
        p[0] += mass * rb.v[0];
        p[1] += mass * rb.v[1];
        p[2] += mass * rb.v[2];
    }
    p
}

/// Build the bodies tree panel (rigid and soft body info). Placed top-left.
#[must_use]
pub fn build_bodies_tree_panel(scene: &PhysicsScene, viewport_width: f32) -> Window {
    const ROW_H: f32 = 18.0;
    const W: f32 = 320.0;
    let n_rb = scene.state.rigid_bodies.len().saturating_sub(1);
    let n_sb = scene.state.soft_bodies.len();
    let n = n_rb + n_sb;
    let h = (n as f32 * (ROW_H + 2.0) + 28.0).max(80.0);
    let x: f32 = 10.0;
    let y: f32 = 10.0;
    let max_x = (viewport_width - W - 10.0).max(10.0);
    let x_final = x.min(max_x);
    let rect = Rect::new(x_final, y, W, h);
    let mut window = Window::new(BODIES_TREE_WINDOW_ID, rect);
    window.title_bar_height = 20.0;
    let body = window.body_rect();
    let mut y_pos = body.y + 4.0;
    let mut next_id = BODIES_TREE_WINDOW_ID.0 + 1;
    for (i, rb) in scene.state.rigid_bodies.iter().enumerate() {
        if i == GROUND_IDX {
            continue;
        }
        let shape_str = match &rb.shape {
            CollisionShape::Sphere { radius } => format!("S r={radius:.2}"),
            CollisionShape::Box { half_extents: h } => {
                format!("B {:.2}\u{00b3}", h[0])
            }
            CollisionShape::Capsule { radius, .. } => format!("C r={radius:.2}"),
        };
        let text = format!(
            "{}: {} ({:.2},{:.2},{:.2}) v({:.2},{:.2},{:.2})",
            i, shape_str, rb.x[0], rb.x[1], rb.x[2], rb.v[0], rb.v[1], rb.v[2]
        );
        let r = Rect::new(body.x + 4.0, y_pos, body.w - 8.0, ROW_H);
        y_pos += ROW_H + 2.0;
        window.add_label(Label::new(ControlId(next_id), r, text));
        next_id += 1;
    }
    for (idx, sb) in scene.state.soft_bodies.iter().enumerate() {
        let mut com = [0.0f32; 3];
        let mut total_mass = 0.0f32;
        for i in 0..sb.num_vertices {
            let p = &scene.state.particles[sb.particle_offset + i];
            if p.inv_mass > 0.0 {
                let m = 1.0 / p.inv_mass;
                com[0] += p.x[0] * m;
                com[1] += p.x[1] * m;
                com[2] += p.x[2] * m;
                total_mass += m;
            }
        }
        if total_mass > 0.0 {
            com[0] /= total_mass;
            com[1] /= total_mass;
            com[2] /= total_mass;
        }
        let text = format!(
            "SB{}: Jelly {}v \u{03bc}={:.0} \u{03bb}={:.0} com({:.2},{:.2},{:.2})",
            idx, sb.num_vertices, sb.mu, sb.lambda, com[0], com[1], com[2]
        );
        let r = Rect::new(body.x + 4.0, y_pos, body.w - 8.0, ROW_H);
        y_pos += ROW_H + 2.0;
        window.add_label(Label::new(ControlId(next_id), r, text));
        next_id += 1;
    }
    window
}

/// Builds the spawn panel window. Position below stats panel.
#[must_use]
pub fn build_spawn_panel(scene: &PhysicsScene, viewport_width: f32) -> Window {
    const ROW_H: f32 = 24.0;
    const W: f32 = 180.0;
    const H: f32 = 148.0;
    let x = (viewport_width - W - 10.0).max(10.0);
    let y = 10.0 + 110.0 + 10.0;
    let rect = Rect::new(x, y, W, H);
    let mut window = Window::new(SPAWN_WINDOW_ID, rect);
    window.title_bar_height = 24.0;
    let body = window.body_rect();

    let btn_y = body.y + 4.0;
    let btn_w = (body.w - 16.0) / 2.0;

    // Drop Sphere button
    let sphere_rect = Rect::new(body.x + 4.0, btn_y, btn_w, ROW_H);
    window.add_button(Button::new(SPAWN_SPHERE_ID, sphere_rect));
    window.add_label(Label::new_overlay(
        ControlId(SPAWN_SPHERE_ID.0 | 0x8000),
        sphere_rect,
        "Drop Sphere".to_string(),
    ));

    // Drop Box button
    let box_rect = Rect::new(body.x + 8.0 + btn_w, btn_y, btn_w, ROW_H);
    window.add_button(Button::new(SPAWN_BOX_ID, box_rect));
    window.add_label(Label::new_overlay(
        ControlId(SPAWN_BOX_ID.0 | 0x8000),
        box_rect,
        "Drop Box".to_string(),
    ));

    // Drop Jelly button
    let jelly_rect = Rect::new(body.x + 4.0, btn_y + ROW_H + 6.0, body.w - 8.0, ROW_H);
    window.add_button(Button::new(SPAWN_JELLY_ID, jelly_rect));
    window.add_label(Label::new_overlay(
        ControlId(SPAWN_JELLY_ID.0 | 0x8000),
        jelly_rect,
        "Drop Jelly".to_string(),
    ));

    // Reset button
    let reset_rect = Rect::new(
        body.x + 4.0,
        btn_y + 2.0 * (ROW_H + 6.0),
        body.w - 8.0,
        ROW_H,
    );
    window.add_button(Button::new(RESET_BUTTON_ID, reset_rect));
    window.add_label(Label::new_overlay(
        ControlId(RESET_BUTTON_ID.0 | 0x8000),
        reset_rect,
        "Reset Scene".to_string(),
    ));

    // Momentum label
    let p = total_momentum(scene);
    let p_mag = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
    let momentum_rect = Rect::new(
        body.x + 4.0,
        btn_y + 3.0 * (ROW_H + 6.0),
        body.w - 8.0,
        ROW_H,
    );
    window.add_label(Label::new(
        ControlId(RESET_BUTTON_ID.0 + 0x100),
        momentum_rect,
        format!("|p|={p_mag:.3}"),
    ));

    window
}
