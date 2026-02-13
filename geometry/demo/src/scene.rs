//! Geometry demo scene: primitives and mesh-derived shapes.

use geometry::TetMesh;
use render::scene::{Primitive, Primitive3D, Transform, World};

/// Builds the geometry demo scene: cube, tetrahedron, sphere.
#[must_use]
pub fn build_geometry_scene(world: &mut World) {
    let (_, _tet_mesh) = TetMesh::tetrahedralize_grid(1, 1, 1, 0.5, false);
    let root = world.root_entity();

    let e1 = world.spawn(root);
    world.set_primitive(e1, Primitive::ThreeD(Primitive3D::Cube));
    world.set_transform(
        e1,
        Transform {
            position: [-0.6, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            rotation_quat: None,
            scale: [0.3, 0.3, 0.3],
        },
    );
    world.set_color(e1, [0.4, 0.6, 1.0, 1.0]);

    let e2 = world.spawn(root);
    world.set_primitive(e2, Primitive::ThreeD(Primitive3D::Tetrahedron));
    world.set_transform(
        e2,
        Transform {
            position: [0.6, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            rotation_quat: None,
            scale: [0.4, 0.4, 0.4],
        },
    );
    world.set_color(e2, [1.0, 0.6, 0.4, 1.0]);

    let e3 = world.spawn(root);
    world.set_primitive(e3, Primitive::ThreeD(Primitive3D::Sphere));
    world.set_transform(
        e3,
        Transform {
            position: [0.0, 0.0, -0.8],
            rotation: [0.0, 0.0, 0.0],
            rotation_quat: None,
            scale: [0.25, 0.25, 0.25],
        },
    );
    world.set_color(e3, [0.5, 1.0, 0.5, 1.0]);
}
