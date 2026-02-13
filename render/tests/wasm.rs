//! WASM build / integration tests: scene types compile and work on wasm32.
//!
//! Run with: cargo test --target wasm32-unknown-unknown --test wasm

#[cfg(target_arch = "wasm32")]
use render::{EntityId, FrameStats, Primitive, Primitive2D, Transform, World};

#[cfg(target_arch = "wasm32")]
#[test]
fn world_spawn_wasm() {
    let mut world = World::new();
    let root = EntityId(0);
    let a = world.spawn(root);

    world.set_transform(a, Transform::default());
    world.set_primitive(a, Primitive::TwoD(Primitive2D::Quad));

    let node = world.get(a).unwrap();
    assert!(node.primitive == Some(Primitive::TwoD(Primitive2D::Quad)));
}

#[cfg(target_arch = "wasm32")]
#[test]
fn frame_stats_wasm() {
    let stats = FrameStats {
        fps: 60.0,
        cpu_time_ms: 1.2,
        gpu_time_ms: None,
        element_count: 3,
    };
    assert_eq!(stats.element_count, 3);
}

// When building for non-wasm, provide a dummy test so the test binary compiles.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn wasm_test_skipped_on_native() {
    // This test binary is intended for wasm32; on native, run `cargo test --test native` instead.
}
