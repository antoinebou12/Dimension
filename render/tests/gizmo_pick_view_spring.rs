//! Tests for gizmo mesh, picking, view mode, spring UI, and curve primitives (no GPU).

use render::backend::primitive_mesh;
use render::gizmo::{gizmo_mesh, GizmoMode, GIZMO_X_COLOR, GIZMO_Y_COLOR, GIZMO_Z_COLOR};
use render::pick_entity;
use render::ui::{Slider, SLIDER_SPRING_DAMPING, SLIDER_SPRING_STIFFNESS};
use render::view_mode::ViewMode;
use render::{Camera, ControlId, CurvePoint, Primitive, Primitive3D, Rect, World};

#[test]
fn gizmo_mesh_translate_vertex_count_and_colors() {
    let mesh = gizmo_mesh(GizmoMode::Translate, 0.5);
    let (vertices, indices) = match &mesh {
        render::gizmo::GizmoMesh::Arrows(v, i) => (v, i),
        render::gizmo::GizmoMesh::Rings(..) => panic!("Translate returns Arrows"),
    };
    assert_eq!(vertices.len(), 24, "three boxes × 8 vertices");
    assert_eq!(
        indices.len(),
        108,
        "three boxes × 6 quads × 6 indices per quad"
    );
    assert_eq!(vertices[0].color, GIZMO_X_COLOR);
    assert_eq!(vertices[8].color, GIZMO_Y_COLOR);
    assert_eq!(vertices[16].color, GIZMO_Z_COLOR);
}

#[test]
fn gizmo_mesh_rotate_rings_and_scale_arrows() {
    let mesh_t = gizmo_mesh(GizmoMode::Translate, 0.5);
    let mesh_r = gizmo_mesh(GizmoMode::Rotate, 0.5);
    let mesh_s = gizmo_mesh(GizmoMode::Scale, 0.5);
    let (v_t, i_t) = match &mesh_t {
        render::gizmo::GizmoMesh::Arrows(v, i) => (v.len(), i.len()),
        _ => (0, 0),
    };
    let (v_r, i_r) = match &mesh_r {
        render::gizmo::GizmoMesh::Rings(v, i) => (v.len(), i.len()),
        _ => (0, 0),
    };
    let (v_s, i_s) = match &mesh_s {
        render::gizmo::GizmoMesh::Arrows(v, i) => (v.len(), i.len()),
        _ => (0, 0),
    };
    assert_eq!(v_t, 24);
    assert_eq!(i_t, 108);
    assert!(v_r > 0 && i_r > 0, "Rotate returns ring geometry");
    assert_eq!(v_s, v_t);
    assert_eq!(i_s, i_t);
}

#[test]
fn pick_entity_perspective_hit_cube() {
    let mut world = World::new();
    let root = world.root_entity();
    let e = world.spawn(root);
    world.set_primitive(e, Primitive::ThreeD(Primitive3D::Cube));
    let mut camera = Camera::new_perspective(800.0, 600.0, std::f32::consts::FRAC_PI_4);
    camera.orbit_yaw = 0.0;
    camera.orbit_pitch = 0.0;
    camera.orbit_distance = 5.0;

    let hit = pick_entity(&world, &camera, 400.0, 300.0);
    assert!(
        hit.is_some(),
        "ray through center should hit the cube at origin"
    );
    assert_eq!(hit.unwrap(), e);
}

#[test]
fn pick_entity_orthographic_returns_none() {
    let world = World::new();
    let camera = Camera::new(800.0, 600.0);
    let hit = pick_entity(&world, &camera, 400.0, 300.0);
    assert!(hit.is_none(), "orthographic picking not implemented");
}

#[test]
fn pick_entity_perspective_hit_line_segment() {
    let mut world = World::new();
    let root = world.root_entity();
    let e = world.spawn(root);
    world.set_primitive(
        e,
        Primitive::ThreeD(Primitive3D::LineSegment {
            start: CurvePoint([-1.0, 0.0, 0.0]),
            end: CurvePoint([1.0, 0.0, 0.0]),
        }),
    );
    let mut camera = Camera::new_perspective(800.0, 600.0, std::f32::consts::FRAC_PI_4);
    camera.orbit_yaw = 0.0;
    camera.orbit_pitch = 0.0;
    camera.orbit_distance = 5.0;

    let hit = pick_entity(&world, &camera, 400.0, 300.0);
    assert!(
        hit.is_some(),
        "ray through center should hit the line segment at origin"
    );
    assert_eq!(hit.unwrap(), e);
}

#[test]
fn view_mode_default_is_solid() {
    assert_eq!(ViewMode::default(), ViewMode::Solid);
}

#[test]
fn slider_spring_moves_value_toward_target() {
    let mut slider = Slider::new(ControlId(0), Rect::new(0.0, 0.0, 100.0, 20.0), 0.0);
    slider.set_target_value(1.0);
    let initial = slider.value;
    for _ in 0..100 {
        slider.update_spring(1.0 / 60.0);
    }
    assert!(slider.value > initial);
    assert!(slider.value <= 1.0 + 0.01);
}

#[test]
fn slider_spring_constants_are_positive() {
    assert!(SLIDER_SPRING_STIFFNESS > 0.0);
    assert!(SLIDER_SPRING_DAMPING > 0.0 && SLIDER_SPRING_DAMPING <= 1.0);
}

#[test]
fn curve_line_segment_mesh_two_vertices() {
    let prim = Primitive::ThreeD(Primitive3D::LineSegment {
        start: CurvePoint([0.0, 0.0, 0.0]),
        end: CurvePoint([1.0, 0.0, 0.0]),
    });
    assert!(prim.is_line_list());
    let (vertices, indices) = primitive_mesh(&prim);
    assert_eq!(vertices.len(), 2);
    assert_eq!(indices.len(), 2);
    assert_eq!(vertices[0].position, [0.0, 0.0, 0.0]);
    assert_eq!(vertices[1].position, [1.0, 0.0, 0.0]);
}

#[test]
fn curve_bezier_mesh_segments() {
    let prim = Primitive::ThreeD(Primitive3D::Bezier {
        control_points: [
            CurvePoint([0.0, 0.0, 0.0]),
            CurvePoint([0.33, 0.0, 0.0]),
            CurvePoint([0.66, 0.0, 0.0]),
            CurvePoint([1.0, 0.0, 0.0]),
        ],
    });
    assert!(prim.is_line_list());
    let (vertices, indices) = primitive_mesh(&prim);
    assert_eq!(vertices.len(), 33, "32 segments + 1");
    assert_eq!(indices.len(), 64, "32 segments * 2");
    assert_eq!(vertices[0].position[0], 0.0);
    assert!((vertices[32].position[0] - 1.0).abs() < 1e-5);
}

#[test]
fn curve_hermite_and_bspline_mesh_build() {
    let herm = Primitive::ThreeD(Primitive3D::Hermite {
        p0: CurvePoint([0.0, 0.0, 0.0]),
        p1: CurvePoint([1.0, 0.0, 0.0]),
        m0: CurvePoint([0.0, 0.0, 0.0]),
        m1: CurvePoint([0.0, 0.0, 0.0]),
    });
    let (v, i) = primitive_mesh(&herm);
    assert!(v.len() > 2 && i.len() >= 2);
    let bs = Primitive::ThreeD(Primitive3D::BSpline {
        control_points: [
            CurvePoint([0.0, 0.0, 0.0]),
            CurvePoint([0.33, 0.0, 0.0]),
            CurvePoint([0.66, 0.0, 0.0]),
            CurvePoint([1.0, 0.0, 0.0]),
        ],
    });
    let (v2, i2) = primitive_mesh(&bs);
    assert!(v2.len() > 2 && i2.len() >= 2);
}
