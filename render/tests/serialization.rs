//! Round-trip serialization tests for World (tree structure and node data). Requires `serde` feature.

#![cfg(feature = "serde")]

use render::{
    world_from_bytes, world_to_bytes, EntityId, Primitive, Primitive2D, Transform, World,
};

#[test]
fn world_roundtrip_tree_and_data() {
    let mut world = World::new();
    let root = world.root_entity();
    let a = world.spawn(root);
    let b = world.spawn(a);
    let c = world.spawn(root);

    world.set_transform(a, Transform::default());
    world.set_primitive(a, Primitive::TwoD(Primitive2D::Quad));
    world.set_color(a, [1.0, 0.0, 0.0, 1.0]);

    world.set_transform(b, Transform::default());
    world.set_primitive(b, Primitive::TwoD(Primitive2D::Triangle));
    world.set_color(b, [0.0, 1.0, 0.0, 1.0]);

    world.set_primitive(c, Primitive::TwoD(Primitive2D::Circle));
    world.set_color(c, [0.0, 0.0, 1.0, 1.0]);
    world.set_material(a, Some("flat"));

    let bytes = world_to_bytes(&world).expect("serialize");
    let restored: World = world_from_bytes(&bytes).expect("deserialize");

    assert_eq!(restored.root_entity(), root);
    assert_eq!(restored.children(root), vec![a, c]);
    assert_eq!(restored.parent(a), Some(root));
    assert_eq!(restored.parent(b), Some(a));
    assert_eq!(restored.parent(c), Some(root));
    assert_eq!(restored.children(a), vec![b]);
    assert_eq!(restored.children(b), Vec::<EntityId>::new());
    assert_eq!(restored.children(c), Vec::<EntityId>::new());

    let node_a = restored.get(a).expect("entity a");
    assert_eq!(node_a.primitive, Some(Primitive::TwoD(Primitive2D::Quad)));
    assert_eq!(node_a.color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(node_a.material.as_deref(), Some("flat"));

    let node_b = restored.get(b).expect("entity b");
    assert_eq!(
        node_b.primitive,
        Some(Primitive::TwoD(Primitive2D::Triangle))
    );
    assert_eq!(node_b.color, [0.0, 1.0, 0.0, 1.0]);

    let node_c = restored.get(c).expect("entity c");
    assert_eq!(node_c.primitive, Some(Primitive::TwoD(Primitive2D::Circle)));
    assert_eq!(node_c.color, [0.0, 0.0, 1.0, 1.0]);
}
