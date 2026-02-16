//! Demo scene UI: entity tree, primitive selector, world switch, stats panel.
//! Builds a "Scene" window and maps control clicks to scene actions.
//!
//! Also provides shared scene presets ([`build_demo_scene`]) and string-to-primitive parsing
//! ([`parse_primitive_name`]) used by native, SDL3, and WASM demos.

use crate::engine::{Engine, FrameStats};
use crate::platform::RunDemo;
use crate::scene::{CurvePoint, EntityId, Primitive, Primitive2D, Primitive3D, Transform};
use crate::ui::{Button, ControlId, Label, Rect, Window};
use crate::view_mode::ViewMode;
use std::collections::HashMap;
use std::fmt::Write;

/// ControlId reserved for the stats overlay window. Used to replace the stats window each frame.
pub const STATS_WINDOW_ID: ControlId = ControlId(0xFF00_0000);

/// ControlId reserved for the material window.
pub const MATERIAL_WINDOW_ID: ControlId = ControlId(0xFF00_0001);

/// ControlId reserved for the Scene window (entity tree, Remove, primitives). Used to rebuild the panel after RemoveSelected.
pub const SCENE_WINDOW_ID: ControlId = ControlId(0);

/// Entity IDs for the Aabb2d demo (two circles + four AABBs, each AABB = 4 line segments).
/// Used by the platform to update the scene each frame.
#[derive(Clone, Debug)]
pub struct Aabb2dIds {
    /// Moving circle (Lissajous).
    pub ball1: EntityId,
    /// Fixed circle at origin.
    pub ball2: EntityId,
    /// AABB of ball1 (4 edges).
    pub aabb1: [EntityId; 4],
    /// AABB of ball2 (4 edges).
    pub aabb2: [EntityId; 4],
    /// Union of both AABBs (4 edges).
    pub bounding: [EntityId; 4],
    /// Expanded AABB of ball2 (4 edges).
    pub loose: [EntityId; 4],
}

/// Placeholder line segment for AABB edge (updated each frame).
fn aabb2d_segment_placeholder() -> Primitive {
    Primitive::ThreeD(Primitive3D::LineSegment {
        start: CurvePoint([0.0, 0.0, 0.0]),
        end: CurvePoint([1.0, 0.0, 0.0]),
    })
}

/// Build the demo scene for the given preset. Used by native, SDL3, and WASM platforms.
/// Returns [`Some(Aabb2dIds)`] when `demo` is [`RunDemo::Aabb2d`] for per-frame updates; otherwise `None`.
#[must_use]
pub fn build_demo_scene(engine: &mut Engine, demo: RunDemo) -> Option<Aabb2dIds> {
    let root = engine.world().root_entity();
    match demo {
        RunDemo::Default => {
            let a = engine.world_mut().spawn(root);
            engine
                .world_mut()
                .set_primitive(a, Primitive::ThreeD(Primitive3D::Cube));
            engine
                .world_mut()
                .set_transform(a, Transform::with_position(0.0, 0.0, 0.0));
            let b = engine.world_mut().spawn(root);
            engine
                .world_mut()
                .set_primitive(b, Primitive::ThreeD(Primitive3D::Tetrahedron));
            engine
                .world_mut()
                .set_transform(b, Transform::with_position(1.5, -0.3, 0.0));
            let c = engine.world_mut().spawn(root);
            engine
                .world_mut()
                .set_primitive(c, Primitive::ThreeD(Primitive3D::Cylinder));
            engine
                .world_mut()
                .set_transform(c, Transform::with_position(-1.5, -0.3, 0.0));
            engine.world_mut().set_material(a, Some("procedural"));
            engine.world_mut().set_material(b, Some("flat"));
            engine.world_mut().set_material(c, Some("flat"));
            return None;
        }
        RunDemo::Curves => {
            const LINE_Y: f32 = 0.0;
            const BEZIER_Y: f32 = 0.6;
            const HERMITE_Y: f32 = 1.2;
            const BSPLINE_Y: f32 = 1.8;
            let line_ent = engine.world_mut().spawn(root);
            engine.world_mut().set_primitive(
                line_ent,
                Primitive::ThreeD(Primitive3D::LineSegment {
                    start: CurvePoint([0.0, LINE_Y, 0.0]),
                    end: CurvePoint([1.0, LINE_Y, 0.0]),
                }),
            );
            let bez_ent = engine.world_mut().spawn(root);
            engine.world_mut().set_primitive(
                bez_ent,
                Primitive::ThreeD(Primitive3D::Bezier {
                    control_points: [
                        CurvePoint([0.0, BEZIER_Y, 0.0]),
                        CurvePoint([0.33, 0.5 + BEZIER_Y, 0.0]),
                        CurvePoint([0.66, 0.5 + BEZIER_Y, 0.0]),
                        CurvePoint([1.0, BEZIER_Y, 0.0]),
                    ],
                }),
            );
            let herm_ent = engine.world_mut().spawn(root);
            engine.world_mut().set_primitive(
                herm_ent,
                Primitive::ThreeD(Primitive3D::Hermite {
                    p0: CurvePoint([0.0, HERMITE_Y, 0.0]),
                    p1: CurvePoint([1.0, HERMITE_Y, 0.0]),
                    m0: CurvePoint([0.0, 1.0, 0.0]),
                    m1: CurvePoint([0.0, -1.0, 0.0]),
                }),
            );
            let bspline_ent = engine.world_mut().spawn(root);
            engine.world_mut().set_primitive(
                bspline_ent,
                Primitive::ThreeD(Primitive3D::BSpline {
                    control_points: [
                        CurvePoint([0.0, BSPLINE_Y, 0.0]),
                        CurvePoint([0.33, BSPLINE_Y, 0.0]),
                        CurvePoint([0.66, BSPLINE_Y, 0.0]),
                        CurvePoint([1.0, BSPLINE_Y, 0.0]),
                    ],
                }),
            );
            engine.set_selected_entity(Some(line_ent));
            return None;
        }
        RunDemo::AllShapes => {
            let xs = [-2.5_f32, -1.4, -0.3, 0.8, 1.9, 3.0];
            let solids = [
                Primitive::ThreeD(Primitive3D::Cube),
                Primitive::ThreeD(Primitive3D::Tetrahedron),
                Primitive::ThreeD(Primitive3D::Cylinder),
                Primitive::ThreeD(Primitive3D::Sphere),
                Primitive::ThreeD(Primitive3D::Cone),
                Primitive::ThreeD(Primitive3D::Capsule),
            ];
            for (i, &prim) in solids.iter().enumerate() {
                let e = engine.world_mut().spawn(root);
                engine.world_mut().set_primitive(e, prim);
                engine
                    .world_mut()
                    .set_transform(e, Transform::with_position(xs[i], 0.4, 0.0));
                engine
                    .world_mut()
                    .set_material(e, Some(if i == 0 { "procedural" } else { "flat" }));
            }
            let cy = -1.0;
            let cx = [-1.2, -0.4, 0.4, 1.2];
            let line_ent = engine.world_mut().spawn(root);
            engine.world_mut().set_primitive(
                line_ent,
                Primitive::ThreeD(Primitive3D::LineSegment {
                    start: CurvePoint([0.0, 0.0, 0.0]),
                    end: CurvePoint([0.5, 0.0, 0.0]),
                }),
            );
            engine
                .world_mut()
                .set_transform(line_ent, Transform::with_position(cx[0], cy, 0.0));
            let bez_ent = engine.world_mut().spawn(root);
            engine.world_mut().set_primitive(
                bez_ent,
                Primitive::ThreeD(Primitive3D::Bezier {
                    control_points: [
                        CurvePoint([0.0, 0.0, 0.0]),
                        CurvePoint([0.15, 0.25, 0.0]),
                        CurvePoint([0.35, 0.25, 0.0]),
                        CurvePoint([0.5, 0.0, 0.0]),
                    ],
                }),
            );
            engine
                .world_mut()
                .set_transform(bez_ent, Transform::with_position(cx[1], cy, 0.0));
            let herm_ent = engine.world_mut().spawn(root);
            engine.world_mut().set_primitive(
                herm_ent,
                Primitive::ThreeD(Primitive3D::Hermite {
                    p0: CurvePoint([0.0, 0.0, 0.0]),
                    p1: CurvePoint([0.5, 0.0, 0.0]),
                    m0: CurvePoint([0.0, 0.3, 0.0]),
                    m1: CurvePoint([0.0, -0.3, 0.0]),
                }),
            );
            engine
                .world_mut()
                .set_transform(herm_ent, Transform::with_position(cx[2], cy, 0.0));
            let bspline_ent = engine.world_mut().spawn(root);
            engine.world_mut().set_primitive(
                bspline_ent,
                Primitive::ThreeD(Primitive3D::BSpline {
                    control_points: [
                        CurvePoint([0.0, 0.0, 0.0]),
                        CurvePoint([0.15, 0.0, 0.0]),
                        CurvePoint([0.35, 0.0, 0.0]),
                        CurvePoint([0.5, 0.0, 0.0]),
                    ],
                }),
            );
            engine
                .world_mut()
                .set_transform(bspline_ent, Transform::with_position(cx[3], cy, 0.0));
            engine.world_mut().set_material(line_ent, None::<String>);
            engine.world_mut().set_material(bez_ent, None::<String>);
            engine.world_mut().set_material(herm_ent, None::<String>);
            engine.world_mut().set_material(bspline_ent, None::<String>);
            engine.set_selected_entity(Some(line_ent));
            return None;
        }
        RunDemo::Aabb2d => {
            use crate::backend::Projection;

            let cam = engine.camera_mut();
            cam.projection = Projection::Orthographic;
            cam.orbit_yaw = 0.0;
            cam.orbit_pitch = 0.0;
            cam.orbit_distance = 8.0;
            cam.orbit_target = [0.0, 0.0, 0.0];

            let ball1 = engine.world_mut().spawn(root);
            engine
                .world_mut()
                .set_primitive(ball1, Primitive::TwoD(Primitive2D::Circle));
            engine
                .world_mut()
                .set_transform(ball1, Transform::with_position(0.0, 0.0, 0.0));
            {
                let t = engine.world_mut().get_mut(ball1).unwrap();
                t.transform.scale = [0.5, 0.5, 1.0];
            }

            let ball2 = engine.world_mut().spawn(root);
            engine
                .world_mut()
                .set_primitive(ball2, Primitive::TwoD(Primitive2D::Circle));
            engine
                .world_mut()
                .set_transform(ball2, Transform::with_position(0.0, 0.0, 0.0));
            {
                let t = engine.world_mut().get_mut(ball2).unwrap();
                t.transform.scale = [1.0, 1.0, 1.0];
            }

            let mut aabb1 = [root; 4];
            let mut aabb2 = [root; 4];
            let mut bounding = [root; 4];
            let mut loose = [root; 4];

            for id in &mut aabb1 {
                *id = engine.world_mut().spawn(root);
                engine
                    .world_mut()
                    .set_primitive(*id, aabb2d_segment_placeholder());
                engine
                    .world_mut()
                    .set_transform(*id, Transform::with_position(0.0, 0.0, 0.0));
            }
            for id in &mut aabb2 {
                *id = engine.world_mut().spawn(root);
                engine
                    .world_mut()
                    .set_primitive(*id, aabb2d_segment_placeholder());
                engine
                    .world_mut()
                    .set_transform(*id, Transform::with_position(0.0, 0.0, 0.0));
            }
            for id in &mut bounding {
                *id = engine.world_mut().spawn(root);
                engine
                    .world_mut()
                    .set_primitive(*id, aabb2d_segment_placeholder());
                engine
                    .world_mut()
                    .set_transform(*id, Transform::with_position(0.0, 0.0, 0.0));
            }
            for id in &mut loose {
                *id = engine.world_mut().spawn(root);
                engine
                    .world_mut()
                    .set_primitive(*id, aabb2d_segment_placeholder());
                engine
                    .world_mut()
                    .set_transform(*id, Transform::with_position(0.0, 0.0, 0.0));
            }

            return Some(Aabb2dIds {
                ball1,
                ball2,
                aabb1,
                aabb2,
                bounding,
                loose,
            });
        }
    }
}

/// Parse a primitive name (e.g. "cube", "sphere") to a default [`Primitive`].
/// Used by WASM JS API for `set_entity_primitive` and `add_entity`.
#[must_use]
pub fn parse_primitive_name(name: &str) -> Option<Primitive> {
    let s = name.trim().to_lowercase();
    let name = s.as_str();
    Some(match name {
        "cube" => Primitive::ThreeD(Primitive3D::Cube),
        "tetrahedron" => Primitive::ThreeD(Primitive3D::Tetrahedron),
        "cylinder" => Primitive::ThreeD(Primitive3D::Cylinder),
        "sphere" => Primitive::ThreeD(Primitive3D::Sphere),
        "cone" => Primitive::ThreeD(Primitive3D::Cone),
        "capsule" => Primitive::ThreeD(Primitive3D::Capsule),
        "line" | "linesegment" => Primitive::ThreeD(Primitive3D::LineSegment {
            start: CurvePoint([0.0, 0.0, 0.0]),
            end: CurvePoint([1.0, 0.0, 0.0]),
        }),
        "bezier" => Primitive::ThreeD(Primitive3D::Bezier {
            control_points: [
                CurvePoint([0.0, 0.0, 0.0]),
                CurvePoint([0.33, 0.5, 0.0]),
                CurvePoint([0.66, 0.5, 0.0]),
                CurvePoint([1.0, 0.0, 0.0]),
            ],
        }),
        "hermite" => Primitive::ThreeD(Primitive3D::Hermite {
            p0: CurvePoint([0.0, 0.0, 0.0]),
            p1: CurvePoint([1.0, 0.0, 0.0]),
            m0: CurvePoint([0.0, 1.0, 0.0]),
            m1: CurvePoint([0.0, -1.0, 0.0]),
        }),
        "bspline" | "b-spline" => Primitive::ThreeD(Primitive3D::BSpline {
            control_points: [
                CurvePoint([0.0, 0.0, 0.0]),
                CurvePoint([0.33, 0.0, 0.0]),
                CurvePoint([0.66, 0.0, 0.0]),
                CurvePoint([1.0, 0.0, 0.0]),
            ],
        }),
        "quad" => Primitive::ThreeD(Primitive3D::Quad),
        "triangle" => Primitive::ThreeD(Primitive3D::Triangle),
        _ => return None,
    })
}

/// Action triggered by a scene UI button click.
#[derive(Clone, Debug, PartialEq)]
pub enum SceneAction {
    /// Select this entity (for primitive selector).
    SelectEntity(EntityId),
    /// Set the selected entity's primitive.
    SetPrimitive(Primitive),
    /// Switch active world by index.
    SetActiveWorld(usize),
    /// Set view mode (solid, wireframe, vertex points, color map).
    SetViewMode(ViewMode),
    /// Remove the selected entity from the scene (one at a time; uses mathlib tree).
    RemoveSelected,
    /// Set the selected entity's material. `None` = vertex color mode.
    SetMaterial(Option<String>),
    /// Reset camera orbit to default (yaw 0, pitch 0, distance 2.5).
    ResetCamera,
}

/// Short display name for a view mode.
fn view_mode_label(mode: ViewMode) -> &'static str {
    match mode {
        ViewMode::Solid => "Solid",
        ViewMode::Wireframe => "Wireframe",
        ViewMode::VertexPoints => "Vertex Points",
        ViewMode::ColorMap => "Color Map",
        ViewMode::Normals => "Normals",
    }
}

/// Short display name for a primitive.
fn primitive_label(prim: &Primitive) -> &'static str {
    match prim {
        Primitive::TwoD(Primitive2D::Quad) => "Quad",
        Primitive::TwoD(Primitive2D::Triangle) => "Triangle",
        Primitive::TwoD(Primitive2D::Square) => "Square",
        Primitive::TwoD(Primitive2D::Circle) => "Circle",
        Primitive::TwoD(Primitive2D::Ellipse) => "Ellipse",
        Primitive::ThreeD(Primitive3D::Quad) => "Quad 3D",
        Primitive::ThreeD(Primitive3D::Triangle) => "Triangle 3D",
        Primitive::ThreeD(Primitive3D::Cube) => "Cube",
        Primitive::ThreeD(Primitive3D::Tetrahedron) => "Tetrahedron",
        Primitive::ThreeD(Primitive3D::Cylinder) => "Cylinder",
        Primitive::ThreeD(Primitive3D::Sphere) => "Sphere",
        Primitive::ThreeD(Primitive3D::Cone) => "Cone",
        Primitive::ThreeD(Primitive3D::Capsule) => "Capsule",
        Primitive::ThreeD(Primitive3D::LineSegment { .. }) => "Line",
        Primitive::ThreeD(Primitive3D::Bezier { .. }) => "Bezier",
        Primitive::ThreeD(Primitive3D::Hermite { .. }) => "Hermite",
        Primitive::ThreeD(Primitive3D::BSpline { .. }) => "BSpline",
    }
}

/// Format the current world's entity tree as a multi-line string (DFS, indent by depth).
/// Each line: indent + "entity_id: PrimitiveName [material]". Used by WASM demo for the tree panel.
#[must_use]
pub fn format_primitive_tree(engine: &Engine) -> String {
    let world = engine.world();
    let root = world.root_entity();
    let mut out = String::new();
    fn visit(
        world: &crate::scene::World,
        id: crate::scene::EntityId,
        depth: usize,
        out: &mut String,
    ) {
        let node = match world.get(id) {
            Some(n) if n.active => n,
            _ => return, // Skip despawned or non-existent nodes
        };
        let indent = "  ".repeat(depth);
        let prim_str = node
            .primitive
            .as_ref()
            .map(primitive_label)
            .unwrap_or("Entity")
            .to_string();
        let mat_str = node
            .material
            .as_deref()
            .map(|s| format!(" [{s}]"))
            .unwrap_or_default();
        let _ = writeln!(out, "{indent}{}: {prim_str}{mat_str}", id.0);
        for &child in &world.children(id) {
            visit(world, child, depth + 1, out);
        }
    }
    visit(world, root, 0, &mut out);
    out
}

/// Auto-select the first entity with a primitive if none is selected. Ensures primitive buttons work immediately.
pub fn auto_select_first_entity(engine: &mut Engine) {
    if engine.selected_entity().is_some() {
        return;
    }
    let first = engine
        .world()
        .entities_dfs()
        .into_iter()
        .find(|id| engine.world().get(*id).and_then(|n| n.primitive).is_some());
    if let Some(id) = first {
        engine.set_selected_entity(Some(id));
    }
}

/// Build the "Scene" window (entity tree, primitive buttons, world switch) and add it to the engine's UI.
/// Returns a mapping from control id to action so the platform can call [`apply_scene_action`] on click.
///
/// Call after [`Engine::enable_ui`]. Uses the current world's entity list and number of worlds.
#[must_use]
pub fn build_scene_panel(engine: &mut Engine) -> HashMap<ControlId, SceneAction> {
    let mut mapping = HashMap::new();
    let mut next_id: u32 = 1; // 0 reserved for SCENE_WINDOW_ID

    let win_rect = Rect::new(10.0, 10.0, 240.0, 520.0);
    let window_id = SCENE_WINDOW_ID;
    let mut window = Window::new(window_id, win_rect);
    let body = window.body_rect();
    let mut y = body.y + 8.0;
    const ROW_H: f32 = 24.0;
    const BTN_W: f32 = 150.0;
    const INDENT: f32 = 16.0;

    // Reset camera button at top
    let rect = Rect::new(body.x + 8.0, y, BTN_W, ROW_H);
    let cid = ControlId(next_id);
    next_id += 1;
    mapping.insert(cid, SceneAction::ResetCamera);
    window.add_button(Button::new(cid, rect));
    window.add_label(Label::new_overlay(
        ControlId(next_id),
        rect,
        "Reset".to_string(),
    ));
    next_id += 1;
    y += ROW_H + 2.0;
    y += 4.0;

    let world = engine.world();
    for (depth, id) in world.entities_dfs().into_iter().enumerate() {
        let indent = body.x + 8.0 + (depth as f32) * INDENT;
        let rect = Rect::new(indent, y, body.w - (indent - body.x) - 8.0, ROW_H);
        let cid = ControlId(next_id);
        next_id += 1;
        mapping.insert(cid, SceneAction::SelectEntity(id));
        window.add_button(Button::new(cid, rect));
        let label = world
            .get(id)
            .and_then(|n| n.primitive.as_ref())
            .map(primitive_label)
            .unwrap_or("Entity");
        window.add_label(Label::new_overlay(
            ControlId(next_id),
            rect,
            label.to_string(),
        ));
        next_id += 1;
        y += ROW_H + 2.0;
    }
    if world.entities_dfs().len() > 0 {
        y += 8.0;
    }

    // Remove selected entity (one at a time; root cannot be removed).
    let rect = Rect::new(body.x + 8.0, y, BTN_W, ROW_H);
    let cid = ControlId(next_id);
    next_id += 1;
    mapping.insert(cid, SceneAction::RemoveSelected);
    window.add_button(Button::new(cid, rect));
    window.add_label(Label::new_overlay(
        ControlId(next_id),
        rect,
        "Remove".to_string(),
    ));
    next_id += 1;
    y += ROW_H + 2.0;

    let view_modes: [ViewMode; 4] = [
        ViewMode::Solid,
        ViewMode::Wireframe,
        ViewMode::VertexPoints,
        ViewMode::ColorMap,
    ];
    y += 4.0;
    for mode in view_modes.iter() {
        let rect = Rect::new(body.x + 8.0, y, BTN_W, ROW_H);
        let cid = ControlId(next_id);
        next_id += 1;
        mapping.insert(cid, SceneAction::SetViewMode(*mode));
        window.add_button(Button::new(cid, rect));
        window.add_label(Label::new_overlay(
            ControlId(next_id),
            rect,
            view_mode_label(*mode).to_string(),
        ));
        next_id += 1;
        y += ROW_H + 2.0;
    }
    y += 8.0;

    let prims: [Primitive; 13] = [
        Primitive::TwoD(Primitive2D::Quad),
        Primitive::TwoD(Primitive2D::Triangle),
        Primitive::TwoD(Primitive2D::Square),
        Primitive::TwoD(Primitive2D::Circle),
        Primitive::TwoD(Primitive2D::Ellipse),
        Primitive::ThreeD(Primitive3D::Quad),
        Primitive::ThreeD(Primitive3D::Triangle),
        Primitive::ThreeD(Primitive3D::Cube),
        Primitive::ThreeD(Primitive3D::Tetrahedron),
        Primitive::ThreeD(Primitive3D::Cylinder),
        Primitive::ThreeD(Primitive3D::Sphere),
        Primitive::ThreeD(Primitive3D::Cone),
        Primitive::ThreeD(Primitive3D::Capsule),
    ];
    y += 4.0;
    for prim in prims.iter() {
        let rect = Rect::new(body.x + 8.0, y, BTN_W, ROW_H);
        let cid = ControlId(next_id);
        next_id += 1;
        mapping.insert(cid, SceneAction::SetPrimitive(*prim));
        window.add_button(Button::new(cid, rect));
        window.add_label(Label::new_overlay(
            ControlId(next_id),
            rect,
            primitive_label(prim).to_string(),
        ));
        next_id += 1;
        y += ROW_H + 2.0;
    }

    let num_worlds = engine.worlds().len();
    if num_worlds > 1 {
        y += 8.0;
        for i in 0..num_worlds {
            let rect = Rect::new(body.x + 8.0, y, BTN_W, ROW_H);
            let cid = ControlId(next_id);
            next_id += 1;
            mapping.insert(cid, SceneAction::SetActiveWorld(i));
            window.add_button(Button::new(cid, rect));
            window.add_label(Label::new_overlay(
                ControlId(next_id),
                rect,
                format!("World {}", i + 1),
            ));
            next_id += 1;
            y += ROW_H + 2.0;
        }
    }

    if let Some(ui) = engine.ui.as_mut() {
        ui.add_window(window);
    }
    auto_select_first_entity(engine);
    mapping
}

/// Build the "Material" window (material list for selected entity).
/// Returns a mapping from control id to action.
///
/// Call after [`Engine::enable_ui`]. Adds the Material window to the engine's UI.
#[must_use]
pub fn build_material_panel(engine: &mut Engine) -> HashMap<ControlId, SceneAction> {
    let mut mapping = HashMap::new();
    let mut next_id: u32 = MATERIAL_WINDOW_ID.0 + 1000;
    const ROW_H: f32 = 24.0;
    const BTN_W: f32 = 150.0;
    let win_rect = Rect::new(260.0, 10.0, 200.0, 320.0);
    let mut window = Window::new(MATERIAL_WINDOW_ID, win_rect);
    window.title_bar_height = 20.0;
    let body = window.body_rect();
    let mut y = body.y + 8.0;

    let rect = Rect::new(body.x + 8.0, y, BTN_W, ROW_H);
    let cid = ControlId(next_id);
    next_id += 1;
    mapping.insert(cid, SceneAction::SetMaterial(None));
    window.add_button(Button::new(cid, rect));
    window.add_label(Label::new_overlay(
        ControlId(next_id),
        rect,
        "None (vertex color)".to_string(),
    ));
    next_id += 1;
    y += ROW_H + 2.0;

    for name in engine.materials.keys() {
        let rect = Rect::new(body.x + 8.0, y, BTN_W, ROW_H);
        let cid = ControlId(next_id);
        next_id += 1;
        let name_clone = name.clone();
        mapping.insert(cid, SceneAction::SetMaterial(Some(name_clone)));
        window.add_button(Button::new(cid, rect));
        window.add_label(Label::new_overlay(ControlId(next_id), rect, name.clone()));
        next_id += 1;
        y += ROW_H + 2.0;
    }

    if let Some(ui) = engine.ui.as_mut() {
        ui.add_window(window);
    }
    mapping
}

/// Build the "Stats" window (FPS, CPU time, GPU time, element count). Replace the stats window
/// each frame by removing windows with [`STATS_WINDOW_ID`] and adding the result of this function.
/// Positioned at top-right so it does not overlap the scene panel.
///
/// # Examples
///
/// ```ignore
/// if let Some(stats) = engine.last_frame_stats() {
///     ui.windows_mut().retain(|w| w.id != STATS_WINDOW_ID);
///     ui.add_window(build_stats_panel(stats, viewport_width));
/// }
/// ```
#[must_use]
pub fn build_stats_panel(stats: &FrameStats, viewport_width: f32) -> Window {
    const ROW_H: f32 = 18.0;
    const W: f32 = 160.0;
    const H: f32 = 110.0;
    let x = (viewport_width - W - 10.0).max(10.0);
    let rect = Rect::new(x, 10.0, W, H);
    let mut window = Window::new(STATS_WINDOW_ID, rect);
    window.title_bar_height = 20.0;
    let body = window.body_rect();
    let mut y = body.y + 4.0;
    let gpu_str = stats.gpu_time_ms.map_or_else(
        || {
            #[cfg(target_arch = "wasm32")]
            {
                "N/A (timing disabled on Web)".to_string()
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                "N/A (no adapter support)".to_string()
            }
        },
        |g| format!("{g:.2} ms"),
    );
    let ids = [
        STATS_WINDOW_ID.0 + 1,
        STATS_WINDOW_ID.0 + 2,
        STATS_WINDOW_ID.0 + 3,
        STATS_WINDOW_ID.0 + 4,
    ];
    let lines = [
        format!("FPS: {:.1}", stats.fps),
        format!("CPU: {:.2} ms", stats.cpu_time_ms),
        format!("GPU: {gpu_str}"),
        format!("Elements: {}", stats.element_count),
    ];
    for (id, text) in ids.iter().zip(lines.iter()) {
        let r = Rect::new(body.x + 4.0, y, body.w - 8.0, ROW_H);
        y += ROW_H + 2.0;
        window.add_label(Label::new(ControlId(*id), r, text.clone()));
    }
    window
}

/// Apply a scene action from a clicked control. Call with the mapping returned by [`build_scene_panel`]
/// and the control id from [`Engine::take_clicked_control`].
pub fn apply_scene_action(
    engine: &mut Engine,
    mapping: &HashMap<ControlId, SceneAction>,
    cid: ControlId,
) {
    let Some(action) = mapping.get(&cid) else {
        return;
    };
    match action {
        SceneAction::SelectEntity(id) => {
            if engine.world().get(*id).map_or(false, |n| n.active) {
                engine.set_selected_entity(Some(*id));
            }
        }
        SceneAction::SetPrimitive(prim) => {
            if let Some(id) = engine.selected_entity() {
                engine.world_mut().set_primitive(id, *prim);
            }
        }
        SceneAction::SetActiveWorld(i) => engine.set_active_world(*i),
        SceneAction::SetViewMode(mode) => engine.set_view_mode(*mode),
        SceneAction::RemoveSelected => {
            if let Some(id) = engine.selected_entity() {
                engine.world_mut().despawn(id);
                engine.set_selected_entity(None);
            }
        }
        SceneAction::SetMaterial(mat) => {
            if let Some(id) = engine.selected_entity() {
                engine.world_mut().set_material(id, mat.as_ref().cloned());
            }
        }
        SceneAction::ResetCamera => {
            engine.reset_camera();
        }
    }
}
