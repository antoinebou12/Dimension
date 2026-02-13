//! Physics scene: particles/bodies, sync to render World.

use physics::{
    ContactConstraint, Integrator, Particle, PbdIntegrator, PhysicsState, PlaneConstraint,
};
use render::scene::{EntityId, Primitive, Primitive3D, Transform, World};
use render::{Button, ControlId, Label, Rect, Window};

const BODY_SCALE: f32 = 0.08;
const GROUND_SCALE: f32 = 6.0;
const GROUND_Y: f32 = 0.0;
const MAX_BODIES: usize = 64;

/// ControlId for the spawn panel window (retain when rebuilding UI).
pub const SPAWN_WINDOW_ID: ControlId = ControlId(0xFF00_0100);
/// ControlId for the "Drop object" button.
pub const SPAWN_BUTTON_ID: ControlId = ControlId(0xFF00_0101);

/// Scene state: physics state, integrator, and entity IDs for each body.
pub struct PhysicsScene {
    /// Physics state (particles).
    pub state: PhysicsState,
    /// PBD integrator (holds constraints).
    pub integrator: PbdIntegrator,
    /// Entity ID for each particle (same length as state.particles).
    pub body_entities: Vec<EntityId>,
}

/// Builds a simple physics scene: particles with distance, plane (ground), and contact constraints.
#[must_use]
pub fn build_physics_scene(world: &mut World) -> PhysicsScene {
    let mut state = PhysicsState::with_particles(vec![
        Particle::new([0.0, 0.5, 0.0], 1.0).with_radius(0.05),
        Particle::new([0.2, 0.5, 0.0], 1.0).with_radius(0.05),
        Particle::new([0.4, 0.5, 0.0], 1.0).with_radius(0.05),
    ]);
    state.config.substeps = 4;
    state.config.solver_iterations = 8;
    state.particles[0].v = [0.5, 0.0, 0.0];

    let mut integrator = PbdIntegrator::new();
    integrator.add_constraint(Box::new(physics::DistanceConstraint::new(0, 1, 0.2)));
    integrator.add_constraint(Box::new(physics::DistanceConstraint::new(1, 2, 0.2)));

    let ground_point = [0.0, GROUND_Y, 0.0];
    let ground_normal = [0.0, 1.0, 0.0];
    for i in 0..state.particles.len() {
        integrator.add_constraint(Box::new(PlaneConstraint::new(
            i,
            ground_point,
            ground_normal,
        )));
    }
    for i in 0..state.particles.len() {
        for j in (i + 1)..state.particles.len() {
            integrator.add_constraint(Box::new(ContactConstraint::new(i, j)));
        }
    }

    let root = world.root_entity();
    let ground_ent = world.spawn(root);
    world.set_primitive(ground_ent, Primitive::ThreeD(Primitive3D::Quad));
    world.set_transform(
        ground_ent,
        Transform {
            position: [0.0, GROUND_Y, 0.0],
            rotation: [-std::f32::consts::FRAC_PI_2, 0.0, 0.0],
            rotation_quat: None,
            scale: [GROUND_SCALE, GROUND_SCALE, GROUND_SCALE],
        },
    );
    world.set_color(ground_ent, [0.35, 0.38, 0.4, 1.0]);

    let mut body_entities = Vec::with_capacity(state.particles.len());
    for _ in 0..state.particles.len() {
        let e = world.spawn(root);
        world.set_primitive(e, Primitive::ThreeD(Primitive3D::Cube));
        world.set_transform(
            e,
            Transform {
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                rotation_quat: None,
                scale: [BODY_SCALE, BODY_SCALE, BODY_SCALE],
            },
        );
        world.set_color(e, [0.4, 0.7, 1.0, 1.0]);
        body_entities.push(e);
    }

    PhysicsScene {
        state,
        integrator,
        body_entities,
    }
}

/// Spawns a new body at the given position (e.g. above ground). Returns `false` if at max body cap.
///
/// Adds a particle, plane and contact constraints, and a render entity. The new body will fall
/// and collide with the ground and other bodies.
pub fn spawn_body(
    scene: &mut PhysicsScene,
    world: &mut World,
    root: EntityId,
    position: [f32; 3],
) -> bool {
    if scene.body_entities.len() >= MAX_BODIES {
        return false;
    }
    let new_idx = scene.state.particles.len();
    scene
        .state
        .particles
        .push(Particle::new(position, 1.0).with_radius(0.05));

    let ground_point = [0.0, GROUND_Y, 0.0];
    let ground_normal = [0.0, 1.0, 0.0];
    scene
        .integrator
        .add_constraint(Box::new(PlaneConstraint::new(
            new_idx,
            ground_point,
            ground_normal,
        )));
    for j in 0..new_idx {
        scene
            .integrator
            .add_constraint(Box::new(ContactConstraint::new(new_idx, j)));
    }

    let e = world.spawn(root);
    world.set_primitive(e, Primitive::ThreeD(Primitive3D::Cube));
    world.set_transform(
        e,
        Transform {
            position,
            rotation: [0.0, 0.0, 0.0],
            rotation_quat: None,
            scale: [BODY_SCALE, BODY_SCALE, BODY_SCALE],
        },
    );
    world.set_color(e, [0.4, 0.7, 1.0, 1.0]);
    scene.body_entities.push(e);
    true
}

/// Builds the spawn panel window (one "Drop object" button). Position below stats panel.
#[must_use]
pub fn build_spawn_panel(viewport_width: f32) -> Window {
    const ROW_H: f32 = 24.0;
    const W: f32 = 160.0;
    const H: f32 = 56.0;
    let x = (viewport_width - W - 10.0).max(10.0);
    let y = 10.0 + 110.0 + 10.0; // below stats panel (stats H = 110)
    let rect = Rect::new(x, y, W, H);
    let mut window = Window::new(SPAWN_WINDOW_ID, rect);
    window.title_bar_height = 24.0;
    let body = window.body_rect();
    let btn_rect = Rect::new(body.x + 8.0, body.y + 8.0, 120.0, ROW_H);
    window.add_button(Button::new(SPAWN_BUTTON_ID, btn_rect));
    window.add_label(Label::new_overlay(
        ControlId(SPAWN_BUTTON_ID.0 + 1),
        btn_rect,
        "Drop object".to_string(),
    ));
    window
}

/// Syncs physics particle positions to render World.
pub fn sync_bodies_to_world(scene: &PhysicsScene, world: &mut World) {
    for (i, &ent_id) in scene.body_entities.iter().enumerate() {
        if let Some(p) = scene.state.particles.get(i) {
            if let Some(data) = world.get_mut(ent_id) {
                data.transform = Transform {
                    position: p.x,
                    rotation: [0.0, 0.0, 0.0],
                    rotation_quat: None,
                    scale: [BODY_SCALE, BODY_SCALE, BODY_SCALE],
                };
            }
        }
    }
}

/// Runs one physics step and syncs to World.
pub fn step_physics(scene: &mut PhysicsScene, world: &mut World, dt: f32) {
    scene.integrator.step(&mut scene.state, dt);
    sync_bodies_to_world(scene, world);
}
