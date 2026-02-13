//! Per-frame update for the Aabb2d demo: Lissajous motion, collision Aabb2/Circle, world updates.

use collision::{Aabb2, Circle};
use render::scene::CurvePoint;
use render::{Aabb2dIds, Engine, Primitive, Primitive3D, Transform};

const LISSAJOUS_SCALE: f32 = 0.4;
const BALL1_RADIUS: f32 = 0.5;
const BALL2_RADIUS: f32 = 1.0;
const LOOSE_EXPAND: f32 = 2.0;

/// Colors (RGBA) for intersection and inclusion.
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const MAGENTA: [f32; 4] = [1.0, 0.0, 1.0, 1.0];

fn lissajous_2d(t: f32) -> [f32; 2] {
    use std::f32::consts::PI;
    [
        LISSAJOUS_SCALE * (2.0 * PI * t).sin(),
        LISSAJOUS_SCALE * (3.0 * PI * t).cos(),
    ]
}

fn set_aabb_edges(
    world: &mut render::World,
    ids: &[render::EntityId; 4],
    aabb: &Aabb2,
    color: [f32; 4],
) {
    let c = aabb.corners();
    let edges = [(0, 1), (1, 2), (2, 3), (3, 0)];
    for (i, &(s, e)) in edges.iter().enumerate() {
        world.set_primitive(
            ids[i],
            Primitive::ThreeD(Primitive3D::LineSegment {
                start: CurvePoint([c[s][0], c[s][1], 0.0]),
                end: CurvePoint([c[e][0], c[e][1], 0.0]),
            }),
        );
        world.set_color(ids[i], color);
    }
}

/// Update the Aabb2d demo scene for this frame. Call before `render_frame` when demo is `RunDemo::Aabb2d`.
pub fn update_aabb2d_demo(engine: &mut Engine, ids: &Aabb2dIds, elapsed_secs: f32) {
    let t = elapsed_secs * 0.7;
    let ball1_xy = lissajous_2d(t);
    let circle1 = Circle::new(ball1_xy, BALL1_RADIUS);
    let circle2 = Circle::new([0.0, 0.0], BALL2_RADIUS);

    let aabb_ball1 = circle1.aabb();
    let aabb_ball2 = circle2.aabb();
    let bounding = aabb_ball1.union(&aabb_ball2);
    let loose_ball2 = aabb_ball2.expand(LOOSE_EXPAND);

    let intersect_color = if aabb_ball1.intersects(&aabb_ball2) {
        RED
    } else {
        GREEN
    };
    let include_color = if loose_ball2.contains(&aabb_ball1) {
        BLUE
    } else {
        MAGENTA
    };

    let world = engine.world_mut();

    world.set_transform(
        ids.ball1,
        Transform::with_position(ball1_xy[0], ball1_xy[1], 0.0),
    );
    if let Some(node) = world.get_mut(ids.ball1) {
        node.transform.scale = [BALL1_RADIUS, BALL1_RADIUS, 1.0];
    }
    world.set_color(ids.ball1, intersect_color);

    world.set_transform(ids.ball2, Transform::with_position(0.0, 0.0, 0.0));
    if let Some(node) = world.get_mut(ids.ball2) {
        node.transform.scale = [BALL2_RADIUS, BALL2_RADIUS, 1.0];
    }
    world.set_color(ids.ball2, intersect_color);

    set_aabb_edges(world, &ids.aabb1, &aabb_ball1, intersect_color);
    set_aabb_edges(world, &ids.aabb2, &aabb_ball2, intersect_color);
    set_aabb_edges(world, &ids.bounding, &bounding, YELLOW);
    set_aabb_edges(world, &ids.loose, &loose_ball2, include_color);
}
