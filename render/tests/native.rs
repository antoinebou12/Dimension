//! Native integration tests: World, Transform, Tree, Camera, FrameStats.

#![cfg(not(target_arch = "wasm32"))]

use render::{EntityId, FrameStats, Primitive, Primitive2D, Primitive3D, Transform, World};

#[test]
fn world_spawn_and_hierarchy() {
    let mut world = World::new();
    let root = EntityId(0);
    let a = world.spawn(root);
    let b = world.spawn(a);

    world.set_transform(a, Transform::default());
    world.set_primitive(a, Primitive::TwoD(Primitive2D::Quad));
    world.set_color(a, [1.0, 0.0, 0.0, 1.0]);

    world.set_transform(b, Transform::default());
    world.set_primitive(b, Primitive::TwoD(Primitive2D::Triangle));

    let node_a = world.get(a).unwrap();
    assert!(node_a.primitive == Some(Primitive::TwoD(Primitive2D::Quad)));
    assert_eq!(node_a.color, [1.0, 0.0, 0.0, 1.0]);

    let node_b = world.get(b).unwrap();
    assert!(node_b.primitive == Some(Primitive::TwoD(Primitive2D::Triangle)));

    world.set_material(a, Some("flat"));
    assert_eq!(world.get(a).unwrap().material.as_deref(), Some("flat"));
    world.set_material(a, None::<String>);
    assert!(world.get(a).unwrap().material.is_none());
}

#[test]
fn frame_stats_default_and_fields() {
    let stats = FrameStats::default();
    assert_eq!(stats.fps, 0.0);
    assert_eq!(stats.cpu_time_ms, 0.0);
    assert!(stats.gpu_time_ms.is_none());
    assert_eq!(stats.element_count, 0);

    let stats = FrameStats {
        fps: 60.0,
        cpu_time_ms: 1.5,
        gpu_time_ms: Some(0.8),
        element_count: 5,
    };
    assert_eq!(stats.fps, 60.0);
    assert_eq!(stats.cpu_time_ms, 1.5);
    assert_eq!(stats.gpu_time_ms, Some(0.8));
    assert_eq!(stats.element_count, 5);
}

#[test]
fn element_count_entities_with_primitive() {
    let mut world = World::new();
    let root = world.root_entity();
    let a = world.spawn(root);
    let b = world.spawn(a);
    let _c = world.spawn(root);

    world.set_primitive(a, Primitive::ThreeD(Primitive3D::Cube));
    world.set_primitive(b, Primitive::ThreeD(Primitive3D::Tetrahedron));
    // c has no primitive

    let count = world
        .entities_dfs()
        .iter()
        .filter(|id| world.get(**id).and_then(|n| n.primitive).is_some())
        .count();
    assert_eq!(
        count, 2,
        "root has no primitive, a and b have primitives, c has none"
    );
}
