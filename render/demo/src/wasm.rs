//! WASM platform: canvas, request_animation_frame, orbit/zoom, resize.
//!
//! Picking on the canvas requires holding **Shift** and clicking (Shift+click to pick an object).

use crate::aabb2d::update_aabb2d_demo;
use js_sys::Reflect;
use render::input_constants::{
    FPS_EMA_ALPHA, ORBIT_SENSITIVITY, PAN_SENSITIVITY, ZOOM_SENSITIVITY,
};
use render::ControlId;
use render::{
    apply_scene_action, auto_select_first_entity, build_demo_scene, build_stats_panel,
    format_primitive_tree, parse_primitive_name, pick_entity, pick_gizmo_handle, Aabb2dIds, Engine,
    EntityId, FrameStats, GizmoMode, RenderError, RunDemo, SceneAction, GIZMO_DEFAULT_SIZE,
    STATS_WINDOW_ID,
};
#[cfg(not(target_arch = "wasm32"))]
use render::{build_material_panel, build_scene_panel};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlCanvasElement;

/// Sentinel for "no entity selected" in JS-facing API (u32::MAX).
pub const NO_ENTITY: u32 = u32::MAX;

// Pending demo switch (set_demo) and tree dump (get_primitive_tree). thread_local so we don't require Sync (WASM is single-threaded).
thread_local! {
    static PENDING_DEMO: RefCell<Option<RunDemo>> = RefCell::new(None);
    static TREE_DUMP: RefCell<String> = RefCell::new(String::new());
    /// Current selected entity id for get_selected_entity(); NO_ENTITY when none. Updated each frame.
    static SELECTED_ENTITY_ID: RefCell<u32> = RefCell::new(NO_ENTITY);
    /// Material names (newline-separated) for get_material_names(). Updated each frame.
    static MATERIAL_NAMES_DUMP: RefCell<String> = RefCell::new(String::new());
    static PENDING_SELECTION: RefCell<Option<Option<u32>>> = RefCell::new(None);
    static PENDING_SET_MATERIAL: RefCell<Option<(u32, String)>> = RefCell::new(None);
    static PENDING_SET_PRIMITIVE: RefCell<Option<(u32, String)>> = RefCell::new(None);
    static PENDING_REMOVE_ENTITY: RefCell<Option<u32>> = RefCell::new(None);
    static PENDING_GIZMO_MODE: RefCell<Option<GizmoMode>> = RefCell::new(None);
    /// Current gizmo mode for get_gizmo_mode(); updated each frame.
    static GIZMO_MODE_DUMP: RefCell<String> = RefCell::new("translate".to_string());
    /// Primitive name to add as new entity next frame (e.g. "cube").
    static PENDING_ADD_ENTITY: RefCell<Option<String>> = RefCell::new(None);
    /// (entity_id, [x, y, z]) to set local position next frame.
    static PENDING_SET_LOCAL_POSITION: RefCell<Option<(u32, [f32; 3])>> = RefCell::new(None);
    /// (entity_id, [x, y, z]) to set world position next frame.
    static PENDING_SET_WORLD_POSITION: RefCell<Option<(u32, [f32; 3])>> = RefCell::new(None);
    /// (entity_id, [roll, pitch, yaw]) in radians to set local rotation next frame.
    static PENDING_SET_LOCAL_ROTATION: RefCell<Option<(u32, [f32; 3])>> = RefCell::new(None);
    /// (entity_id, [x, y, z]) to set local scale next frame.
    static PENDING_SET_LOCAL_SCALE: RefCell<Option<(u32, [f32; 3])>> = RefCell::new(None);
    /// Selected entity local rotation "r,p,y" (radians) for editor panel. Updated each frame.
    static LOCAL_ROTATION_DUMP: RefCell<String> = RefCell::new(String::new());
    /// Selected entity local scale "x,y,z" for editor panel. Updated each frame.
    static LOCAL_SCALE_DUMP: RefCell<String> = RefCell::new(String::new());
    /// When demo is Aabb2d, entity IDs for per-frame update.
    static AABB2D_IDS: RefCell<Option<Aabb2dIds>> = RefCell::new(None);
    /// When demo is Aabb2d, start time in ms (performance.now()) for elapsed.
    static AABB2D_START_MS: RefCell<Option<f64>> = RefCell::new(None);
    /// Selected entity local position "x,y,z" for editor panel. Updated each frame.
    static LOCAL_POSITION_DUMP: RefCell<String> = RefCell::new(String::new());
    /// Selected entity world position "x,y,z" for editor panel. Updated each frame.
    static WORLD_POSITION_DUMP: RefCell<String> = RefCell::new(String::new());
}

fn wasm_log(s: &str) {
    web_sys::console::log_1(&JsValue::from_str(s));
}

/// (last frame end time in ms, smoothed FPS)
type FrameTimingState = (Option<f64>, f32);

/// Orbit state: (pressing, last_cursor, pointer_down_pos for click-vs-drag).
type OrbitState = (bool, Option<(f64, f64)>, Option<(f32, f32)>);

/// Movement threshold in pixels below which pointerdown->pointerup is treated as a click (pick without Shift).
const PICK_CLICK_THRESHOLD_PX: f32 = 4.0;

fn canvas_coords(e: &web_sys::PointerEvent, canvas: &HtmlCanvasElement) -> (f32, f32) {
    let rect = canvas.get_bounding_client_rect();
    let w = rect.width().max(1.0);
    let h = rect.height().max(1.0);
    let x = (e.client_x() as f64 - rect.left()) / w as f64 * f64::from(canvas.width());
    let y = (e.client_y() as f64 - rect.top()) / h as f64 * f64::from(canvas.height());
    (x as f32, y as f32)
}

fn set_canvas_size_from_display(canvas: &HtmlCanvasElement) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let dpr = window.device_pixel_ratio().max(1.0);
    let logical_w = canvas.client_width().max(1);
    let logical_h = canvas.client_height().max(1);
    let buffer_w = (f64::from(logical_w) * dpr).round().max(1.0) as u32;
    let buffer_h = (f64::from(logical_h) * dpr).round().max(1.0) as u32;
    canvas.set_width(buffer_w);
    canvas.set_height(buffer_h);
}

pub fn run(demo: RunDemo) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let canvas = document
        .get_element_by_id("canvas")
        .ok_or("no canvas element with id 'canvas'")?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| "element 'canvas' is not an HtmlCanvasElement")?;

    set_canvas_size_from_display(&canvas);
    if canvas.width() == 0 || canvas.height() == 0 {
        canvas.set_width(800);
        canvas.set_height(600);
    }
    let dpr = window.device_pixel_ratio().max(1.0);
    let logical_w = canvas.client_width().max(1);
    let logical_h = canvas.client_height().max(1);
    wasm_log(&format!(
        "render-demo: canvas buffer {}x{}, logical {}x{}, dpr {}",
        canvas.width(),
        canvas.height(),
        logical_w,
        logical_h,
        dpr
    ));
    wasm_log(&format!("render-demo: starting with scene {:?}", demo));

    type EngineAndMapping = (Option<Engine>, Option<HashMap<ControlId, SceneAction>>);
    let engine: Rc<RefCell<EngineAndMapping>> = Rc::new(RefCell::new((None, None)));
    let orbit_state: Rc<RefCell<OrbitState>> = Rc::new(RefCell::new((false, None, None)));
    let ctrl_held: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
    let shift_held: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
    let frame_timing: Rc<RefCell<FrameTimingState>> = Rc::new(RefCell::new((None, 60.0)));

    let engine_clone = Rc::clone(&engine);
    let orbit_clone = Rc::clone(&orbit_state);
    let ctrl_clone = Rc::clone(&ctrl_held);
    let shift_clone = Rc::clone(&shift_held);
    let frame_timing_clone = Rc::clone(&frame_timing);
    let frame_count: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
    let frame_count_clone = Rc::clone(&frame_count);
    let demo_clone = demo;
    wasm_bindgen_futures::spawn_local(async move {
        match Engine::new_wasm(canvas).await {
            Ok(mut eng) => {
                wasm_log(&format!(
                    "render-demo: engine ready, viewport width {}",
                    eng.viewport_width()
                ));
                let _ = eng.enable_ui();
                eng.scene_lighting.ambient_intensity = 0.6;
                let aabb2d_ids = build_demo_scene(&mut eng, demo_clone);
                if let Some(ids) = aabb2d_ids {
                    AABB2D_IDS.set(Some(ids));
                    AABB2D_START_MS.set(None);
                }
                let entity_count = eng.world().entities_dfs().len();
                wasm_log(&format!(
                    "render-demo: scene built, {} entities, materials: {:?}",
                    entity_count,
                    eng.materials.keys().collect::<Vec<_>>()
                ));
                #[cfg(not(target_arch = "wasm32"))]
                let mapping = {
                    let mut m = build_scene_panel(&mut eng);
                    m.extend(build_material_panel(&mut eng));
                    Some(m)
                };
                #[cfg(target_arch = "wasm32")]
                let mapping = {
                    auto_select_first_entity(&mut eng);
                    None::<HashMap<ControlId, SceneAction>>
                };
                *engine_clone.borrow_mut() = (Some(eng), mapping);
                if let Some(canvas) = get_canvas() {
                    setup_orbit_listeners(
                        &engine_clone,
                        &orbit_clone,
                        &ctrl_clone,
                        &shift_clone,
                        &canvas,
                    );
                    setup_resize_observer(&engine_clone, &canvas);
                }
                schedule_frame(
                    &engine_clone,
                    &ctrl_clone,
                    &frame_timing_clone,
                    &frame_count_clone,
                );
            }
            Err(e) => {
                let msg = format!("Engine init failed: {e}");
                wasm_log(&msg);
                show_render_error(&msg);
            }
        }
    });

    Ok(())
}

fn setup_orbit_listeners(
    engine: &Rc<RefCell<(Option<Engine>, Option<HashMap<ControlId, SceneAction>>)>>,
    orbit_state: &Rc<RefCell<OrbitState>>,
    ctrl_held: &Rc<RefCell<bool>>,
    shift_held: &Rc<RefCell<bool>>,
    canvas: &HtmlCanvasElement,
) {
    let engine = Rc::clone(engine);
    let canvas = canvas.clone();
    let ctrl_held = Rc::clone(ctrl_held);
    let shift_held = Rc::clone(shift_held);

    // Track Ctrl and Shift. Ignore key repeat to avoid redundant updates (helps frame rate when holding keys).
    let ctrl_down = Rc::clone(&ctrl_held);
    let shift_down = Rc::clone(&shift_held);
    let on_key_down: Closure<dyn FnMut(web_sys::KeyboardEvent)> =
        Closure::new(move |e: web_sys::KeyboardEvent| {
            if e.repeat() {
                return;
            }
            match e.key().as_str() {
                "Control" => *ctrl_down.borrow_mut() = true,
                "Shift" => *shift_down.borrow_mut() = true,
                _ => {}
            }
        });
    let ctrl_up = Rc::clone(&ctrl_held);
    let shift_up = Rc::clone(&shift_held);
    let on_key_up: Closure<dyn FnMut(web_sys::KeyboardEvent)> =
        Closure::new(move |e: web_sys::KeyboardEvent| {
            if e.repeat() {
                return;
            }
            match e.key().as_str() {
                "Control" => *ctrl_up.borrow_mut() = false,
                "Shift" => *shift_up.borrow_mut() = false,
                _ => {}
            }
        });
    if let Some(window) = web_sys::window() {
        let _ = window
            .add_event_listener_with_callback("keydown", on_key_down.as_ref().unchecked_ref());
        let _ =
            window.add_event_listener_with_callback("keyup", on_key_up.as_ref().unchecked_ref());
        on_key_down.forget();
        on_key_up.forget();
    }

    let orbit_down = Rc::clone(orbit_state);
    let engine_down = Rc::clone(&engine);
    let canvas_down = canvas.clone();
    let on_pointer_down: Closure<dyn FnMut(web_sys::PointerEvent)> =
        Closure::new(move |e: web_sys::PointerEvent| {
            if e.button() == 0 {
                let (x, y) = canvas_coords(&e, &canvas_down);
                if let Ok(mut g) = engine_down.try_borrow_mut() {
                    if let Some(ref mut eng) = g.0 {
                        eng.ui_mouse_move(x, y);
                        eng.ui_mouse_down();
                        let over_ui = eng.is_cursor_over_ui();
                        let mut pressing = !over_ui;
                        if pressing && !eng.gizmo_hidden() {
                            if let Some(selected) = eng.selected_entity() {
                                if let Some(world_mat) = eng.world().entity_world_matrix(selected) {
                                    if let Some(axis) = pick_gizmo_handle(
                                        eng.camera(),
                                        &world_mat,
                                        eng.gizmo_mode(),
                                        GIZMO_DEFAULT_SIZE,
                                        x,
                                        y,
                                    ) {
                                        if eng.gizmo_drag_start(selected, axis, x, y) {
                                            pressing = false;
                                        }
                                    }
                                }
                            }
                        }
                        *orbit_down.borrow_mut() = (pressing, None, Some((x, y)));
                    } else {
                        *orbit_down.borrow_mut() = (true, None, Some((x, y)));
                    }
                }
            }
        });
    canvas
        .add_event_listener_with_callback("pointerdown", on_pointer_down.as_ref().unchecked_ref())
        .expect("add pointerdown listener");
    on_pointer_down.forget();

    let orbit_up = Rc::clone(orbit_state);
    let ctrl_up_for_release = Rc::clone(&ctrl_held);
    let shift_up_for_release = Rc::clone(&shift_held);
    let engine_up = Rc::clone(&engine);
    let canvas_up = canvas.clone();
    let on_pointer_up: Closure<dyn FnMut(web_sys::PointerEvent)> =
        Closure::new(move |e: web_sys::PointerEvent| {
            let (x, y) = canvas_coords(&e, &canvas_up);
            let (_, _, down_pos) = *orbit_up.borrow();
            let is_click = down_pos.map_or(false, |(dx, dy)| {
                let dist_sq = (x - dx).powi(2) + (y - dy).powi(2);
                dist_sq <= PICK_CLICK_THRESHOLD_PX * PICK_CLICK_THRESHOLD_PX
            });
            if let Ok(g) = engine_up.try_borrow_mut() {
                let (mut eng_ref, mapping_ref) = RefMut::map_split(g, |g| (&mut g.0, &mut g.1));
                if let Some(ref mut eng) = *eng_ref {
                    eng.ui_mouse_move(x, y);
                    eng.ui_mouse_up();
                    eng.gizmo_drag_end();
                    if let Some(ref mapping) = *mapping_ref {
                        if let Some(cid) = eng.take_clicked_control() {
                            apply_scene_action(eng, mapping, cid);
                        }
                    }
                    // Pick when not over UI and not camera-only: on click (no drag) or when Shift held.
                    let should_pick = !eng.is_cursor_over_ui()
                        && !*ctrl_up_for_release.borrow()
                        && (*shift_up_for_release.borrow() || is_click);
                    if should_pick {
                        if let Some(id) = pick_entity(eng.world(), eng.camera(), x, y) {
                            eng.set_selected_entity(Some(id));
                        }
                    }
                }
            }
            *orbit_up.borrow_mut() = (false, None, None);
        });
    canvas
        .add_event_listener_with_callback("pointerup", on_pointer_up.as_ref().unchecked_ref())
        .expect("add pointerup listener");
    on_pointer_up.forget();

    let orbit_leave = Rc::clone(orbit_state);
    let engine_leave = Rc::clone(&engine);
    let on_pointer_leave: Closure<dyn FnMut(web_sys::PointerEvent)> =
        Closure::new(move |_e: web_sys::PointerEvent| {
            *orbit_leave.borrow_mut() = (false, None, None);
            if let Ok(mut g) = engine_leave.try_borrow_mut() {
                if let Some(ref mut eng) = g.0 {
                    eng.gizmo_drag_end();
                }
            }
        });
    canvas
        .add_event_listener_with_callback("pointerleave", on_pointer_leave.as_ref().unchecked_ref())
        .expect("add pointerleave listener");
    on_pointer_leave.forget();

    let engine_move = Rc::clone(&engine);
    let orbit_move = Rc::clone(orbit_state);
    let ctrl_move = Rc::clone(&ctrl_held);
    let shift_move = Rc::clone(&shift_held);
    let canvas_move = canvas.clone();
    let on_pointer_move: Closure<dyn FnMut(web_sys::PointerEvent)> =
        Closure::new(move |e: web_sys::PointerEvent| {
            let (x, y) = canvas_coords(&e, &canvas_move);
            // Use try_borrow_mut to avoid re-entrancy panic if rAF or another handler holds the borrow.
            let mut g = match engine_move.try_borrow_mut() {
                Ok(g) => g,
                Err(_) => return,
            };
            let cancel_orbit = if let Some(ref mut eng) = g.0 {
                eng.ui_mouse_move(x, y);
                // Cancel orbit if cursor moved over UI during drag (orbit must not run over UI).
                orbit_move.borrow().0 && eng.is_cursor_over_ui()
            } else {
                false
            };
            drop(g);
            if cancel_orbit {
                let (_, _, down) = *orbit_move.borrow();
                *orbit_move.borrow_mut() = (false, None, down);
            }
            let (pressing, last, down) = *orbit_move.borrow();
            if let Ok(mut g) = engine_move.try_borrow_mut() {
                if let Some(ref mut eng) = g.0 {
                    if eng.is_gizmo_dragging() {
                        eng.gizmo_drag_move(x, y);
                        return;
                    }
                }
            }
            if pressing {
                let x64 = e.client_x() as f64;
                let y64 = e.client_y() as f64;
                // When Shift is held, disable camera (no orbit/pan).
                if !*shift_move.borrow() {
                    if let Ok(mut g) = engine_move.try_borrow_mut() {
                        if let Some(ref mut eng) = g.0 {
                            if let Some((lx, ly)) = last {
                                if *ctrl_move.borrow() {
                                    let dx = (x64 - lx) as f32 * PAN_SENSITIVITY;
                                    let dy = (y64 - ly) as f32 * PAN_SENSITIVITY;
                                    eng.pan(dx, dy);
                                } else {
                                    let dyaw = (x64 - lx) as f32 * ORBIT_SENSITIVITY;
                                    let dpitch = (ly - y64) as f32 * ORBIT_SENSITIVITY;
                                    eng.orbit(dyaw, dpitch);
                                }
                            }
                        }
                    }
                }
                *orbit_move.borrow_mut() = (true, Some((x64, y64)), down);
            }
        });
    canvas
        .add_event_listener_with_callback("pointermove", on_pointer_move.as_ref().unchecked_ref())
        .expect("add pointermove listener");
    on_pointer_move.forget();

    // Wheel listener is intentionally non-passive so we can prevent_default() for camera zoom only (no page scroll).
    let engine_wheel = Rc::clone(&engine);
    let on_wheel: Closure<dyn FnMut(web_sys::WheelEvent)> =
        Closure::new(move |e: web_sys::WheelEvent| {
            e.prevent_default();
            let delta = e.delta_y() as f32 * ZOOM_SENSITIVITY;
            // Use try_borrow_mut to avoid re-entrancy panic if rAF or another handler holds the borrow.
            if let Ok(mut g) = engine_wheel.try_borrow_mut() {
                if let Some(ref mut eng) = g.0 {
                    eng.zoom(-delta);
                }
            }
        });
    canvas
        .add_event_listener_with_callback("wheel", on_wheel.as_ref().unchecked_ref())
        .expect("add wheel listener");
    on_wheel.forget();
}

fn setup_resize_observer(
    engine: &Rc<RefCell<(Option<Engine>, Option<HashMap<ControlId, SceneAction>>)>>,
    canvas: &HtmlCanvasElement,
) {
    let engine = Rc::clone(engine);
    let canvas = canvas.clone();

    let closure: Closure<dyn FnMut()> = Closure::new(move || {
        set_canvas_size_from_display(&canvas);
        let w = canvas.width();
        let h = canvas.height();
        wasm_log(&format!("render-demo: resize event -> buffer {}x{}", w, h));
        if w > 0 && h > 0 {
            // Use try_borrow_mut to avoid re-entrancy panic if rAF or another handler holds the borrow.
            if let Ok(mut g) = engine.try_borrow_mut() {
                if let Some(ref mut eng) = g.0 {
                    eng.resize(w, h);
                }
            }
        }
    });

    let window = web_sys::window().expect("no window");
    window
        .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
        .expect("add resize listener");
    closure.forget();
}

fn schedule_frame(
    engine: &Rc<RefCell<(Option<Engine>, Option<HashMap<ControlId, SceneAction>>)>>,
    ctrl_held: &Rc<RefCell<bool>>,
    frame_timing: &Rc<RefCell<FrameTimingState>>,
    frame_count: &Rc<RefCell<u32>>,
) {
    let engine = Rc::clone(engine);
    let ctrl_held = Rc::clone(ctrl_held);
    let frame_timing = Rc::clone(frame_timing);
    let frame_count = Rc::clone(frame_count);
    let closure = Closure::once(move || {
        let stats_to_show = {
            let perf = web_sys::window()
                .and_then(|w| w.performance())
                .expect("performance");
            let t_now = perf.now();
            let dt = {
                let timing = frame_timing.borrow();
                timing
                    .0
                    .map(|last| ((t_now - last) / 1000.0_f64) as f32)
                    .unwrap_or(0.0)
                    .min(0.1)
            };
            let mut g = engine.borrow_mut();
            // Handle world switch (set_demo). Do in a block so we can assign g.1 without holding eng.
            let new_mapping = if let Some(ref mut eng) = g.0 {
                // Apply pending HTML-driven actions (set selection, material, primitive).
                if let Some(sel) = PENDING_SELECTION.with(|c| c.borrow_mut().take()) {
                    eng.set_selected_entity(sel.map(|id| EntityId(id as usize)));
                }
                if let Some((id, name)) = PENDING_SET_MATERIAL.with(|c| c.borrow_mut().take()) {
                    let eid = EntityId(id as usize);
                    if eng.world().get(eid).is_some() {
                        eng.world_mut()
                            .set_material(eid, if name.is_empty() { None } else { Some(name) });
                    }
                }
                if let Some((id, prim_name)) = PENDING_SET_PRIMITIVE.with(|c| c.borrow_mut().take())
                {
                    if let Some(prim) = parse_primitive_name(&prim_name) {
                        let eid = EntityId(id as usize);
                        if eng.world().get(eid).is_some() {
                            eng.world_mut().set_primitive(eid, prim);
                        }
                    }
                }
                if let Some(id) = PENDING_REMOVE_ENTITY.with(|c| c.borrow_mut().take()) {
                    let eid = EntityId(id as usize);
                    if eng.world().get(eid).is_some() && eid != eng.world().root_entity() {
                        eng.world_mut().despawn(eid);
                        eng.set_selected_entity(None);
                    }
                }
                if let Some(mode) = PENDING_GIZMO_MODE.with(|c| c.borrow_mut().take()) {
                    eng.set_gizmo_mode(mode);
                }
                if let Some(prim_name) = PENDING_ADD_ENTITY.with(|c| c.borrow_mut().take()) {
                    let root = eng.world().root_entity();
                    let new_id = eng.world_mut().spawn(root);
                    if let Some(prim) = parse_primitive_name(&prim_name) {
                        eng.world_mut().set_primitive(new_id, prim);
                    }
                    eng.set_selected_entity(Some(new_id));
                }
                if let Some((id, pos)) = PENDING_SET_LOCAL_POSITION.with(|c| c.borrow_mut().take())
                {
                    let eid = EntityId(id as usize);
                    if let Some(data) = eng.world_mut().get_mut(eid) {
                        data.transform.position = pos;
                    }
                }
                if let Some((id, pos)) = PENDING_SET_WORLD_POSITION.with(|c| c.borrow_mut().take())
                {
                    let eid = EntityId(id as usize);
                    if eng.world().get(eid).is_some() {
                        eng.world_mut()
                            .set_entity_world_position(eid, pos[0], pos[1], pos[2]);
                    }
                }
                if let Some((id, rot)) = PENDING_SET_LOCAL_ROTATION.with(|c| c.borrow_mut().take())
                {
                    let eid = EntityId(id as usize);
                    if let Some(data) = eng.world_mut().get_mut(eid) {
                        data.transform.rotation = rot;
                        data.transform.rotation_quat = None;
                    }
                }
                if let Some((id, scl)) = PENDING_SET_LOCAL_SCALE.with(|c| c.borrow_mut().take()) {
                    let eid = EntityId(id as usize);
                    if let Some(data) = eng.world_mut().get_mut(eid) {
                        data.transform.scale = scl;
                    }
                }
                PENDING_DEMO
                    .with(|c| c.borrow_mut().take())
                    .and_then(|pending| {
                        let root = eng.world().root_entity();
                        let to_despawn: Vec<_> = eng
                            .world()
                            .entities_dfs()
                            .into_iter()
                            .filter(|id| *id != root)
                            .collect();
                        for id in &to_despawn {
                            eng.world_mut().despawn(*id);
                        }
                        eng.worlds_mut()[0] = render::World::new();
                        let aabb2d_ids = build_demo_scene(eng, pending);
                        if let Some(ids) = aabb2d_ids {
                            AABB2D_IDS.set(Some(ids));
                            AABB2D_START_MS.set(None);
                        } else {
                            AABB2D_IDS.set(None);
                            AABB2D_START_MS.set(None);
                        }
                        wasm_log(&format!("render-demo: switched scene to {:?}", pending));
                        #[cfg(not(target_arch = "wasm32"))]
                        let mapping = {
                            let mut m = build_scene_panel(eng);
                            m.extend(build_material_panel(eng));
                            Some(m)
                        };
                        #[cfg(target_arch = "wasm32")]
                        let mapping: Option<
                            HashMap<ControlId, SceneAction>,
                        > = {
                            auto_select_first_entity(eng);
                            None
                        };
                        mapping
                    })
            } else {
                None
            };
            if let Some(m) = new_mapping {
                g.1 = Some(m);
            }
            if let Some(ref mut eng) = g.0 {
                eng.update_ui_springs(dt);
                // Sync canvas size for the first few frames so we pick up layout (fixes broken UI when canvas is in flex container).
                // Also run a delayed sync at frames 6–8 to catch late layout (e.g. flex container settling).
                {
                    let mut n = frame_count.borrow_mut();
                    let frame_num = *n;
                    *n = n.saturating_add(1);
                    if frame_num == 0 {
                        wasm_log("render-demo: first frame starting");
                    }
                    let do_sync = frame_num <= 5 || (frame_num >= 6 && frame_num <= 8);
                    if do_sync {
                        if let Some(canvas) = get_canvas() {
                            set_canvas_size_from_display(&canvas);
                            let w = canvas.width();
                            let h = canvas.height();
                            if w > 0 && h > 0 {
                                eng.resize(w, h);
                                if frame_num > 0 {
                                    wasm_log(&format!(
                                        "render-demo: resize sync frame {} -> {}x{}",
                                        frame_num, w, h
                                    ));
                                }
                            }
                        }
                    }
                }
                eng.set_gizmo_hidden(*ctrl_held.borrow());
                if let Some(ref ids) = AABB2D_IDS.with(|c| c.borrow().clone()) {
                    let start = {
                        let opt = AABB2D_START_MS.with(|c| c.borrow().clone());
                        match opt {
                            Some(s) => s,
                            None => {
                                let t = perf.now();
                                AABB2D_START_MS.set(Some(t));
                                t
                            }
                        }
                    };
                    let elapsed_secs = ((perf.now() - start) / 1000.0) as f32;
                    update_aabb2d_demo(eng, ids, elapsed_secs);
                }
                let t0 = perf.now();
                let render_err = eng.render_frame();
                let t1 = perf.now();
                let cpu_time_ms = (t1 - t0) as f32;
                if let Err(e) = render_err {
                    if matches!(e, RenderError::SurfaceLost) {
                        wasm_log(&format!("render error: {e}"));
                        show_render_error("WebGPU surface lost; try resizing the window.");
                        if let Some(canvas) = get_canvas() {
                            eng.resize(canvas.width(), canvas.height());
                        }
                    } else {
                        wasm_log(&format!("render error: {e}"));
                        show_render_error(&format!("Render error: {e}"));
                    }
                }
                let mut timing = frame_timing.borrow_mut();
                let fps = if let Some(last) = timing.0 {
                    let delta_ms = t1 - last;
                    if delta_ms > 0.0 {
                        1000.0 / (delta_ms as f32)
                    } else {
                        timing.1
                    }
                } else {
                    60.0
                };
                timing.0 = Some(t1);
                timing.1 = FPS_EMA_ALPHA * timing.1 + (1.0 - FPS_EMA_ALPHA) * fps;
                eng.poll_device();
                eng.set_last_frame_stats(FrameStats {
                    fps: timing.1,
                    cpu_time_ms,
                    gpu_time_ms: eng.last_gpu_time_ms(),
                    element_count: eng.active_world_element_count(),
                });
                // Update tree dump for get_primitive_tree() (JS menu "Show tree").
                TREE_DUMP.with(|c| *c.borrow_mut() = format_primitive_tree(eng));
                SELECTED_ENTITY_ID.with(|c| {
                    *c.borrow_mut() = eng
                        .selected_entity()
                        .map(|e| e.0 as u32)
                        .unwrap_or(NO_ENTITY);
                });
                if let Some(id) = eng.selected_entity() {
                    let node = eng.world().get(id);
                    let local = node
                        .map(|n| n.transform.position)
                        .unwrap_or([0.0, 0.0, 0.0]);
                    let rot = node
                        .map(|n| n.transform.rotation)
                        .unwrap_or([0.0, 0.0, 0.0]);
                    let scale = node.map(|n| n.transform.scale).unwrap_or([1.0, 1.0, 1.0]);
                    let world = eng
                        .world()
                        .entity_world_position(id)
                        .unwrap_or([0.0, 0.0, 0.0]);
                    LOCAL_POSITION_DUMP.with(|c| {
                        *c.borrow_mut() =
                            format!("{:.4},{:.4},{:.4}", local[0], local[1], local[2]);
                    });
                    LOCAL_ROTATION_DUMP.with(|c| {
                        *c.borrow_mut() = format!("{:.6},{:.6},{:.6}", rot[0], rot[1], rot[2]);
                    });
                    LOCAL_SCALE_DUMP.with(|c| {
                        *c.borrow_mut() =
                            format!("{:.4},{:.4},{:.4}", scale[0], scale[1], scale[2]);
                    });
                    WORLD_POSITION_DUMP.with(|c| {
                        *c.borrow_mut() =
                            format!("{:.4},{:.4},{:.4}", world[0], world[1], world[2]);
                    });
                } else {
                    LOCAL_POSITION_DUMP.with(|c| *c.borrow_mut() = String::new());
                    LOCAL_ROTATION_DUMP.with(|c| *c.borrow_mut() = String::new());
                    LOCAL_SCALE_DUMP.with(|c| *c.borrow_mut() = String::new());
                    WORLD_POSITION_DUMP.with(|c| *c.borrow_mut() = String::new());
                }
                MATERIAL_NAMES_DUMP.with(|c| {
                    *c.borrow_mut() = eng.materials.keys().cloned().collect::<Vec<_>>().join("\n");
                });
                GIZMO_MODE_DUMP.with(|c| {
                    *c.borrow_mut() = match eng.gizmo_mode() {
                        GizmoMode::Translate => "translate".to_string(),
                        GizmoMode::Rotate => "rotate".to_string(),
                        GizmoMode::Scale => "scale".to_string(),
                    };
                });

                let stats_for_ui = eng.last_frame_stats().cloned();
                let viewport_width = eng.viewport_width() as f32;
                if let (Some(ref stats), Some(ui)) = (stats_for_ui.as_ref(), eng.ui.as_mut()) {
                    ui.windows_mut().retain(|w| w.id != STATS_WINDOW_ID);
                    ui.add_window(build_stats_panel(stats, viewport_width));
                    if *frame_count.borrow() == 1 {
                        wasm_log(&format!("render-demo: UI windows: {}", ui.windows.len()));
                    }
                }
                stats_for_ui
            } else {
                None
            }
        };
        if let Some(ref stats) = stats_to_show {
            call_stats_overlay(stats);
        }
        schedule_frame(&engine, &ctrl_held, &frame_timing, &frame_count);
    });

    web_sys::window()
        .and_then(|w| {
            w.request_animation_frame(closure.as_ref().unchecked_ref())
                .ok()
        })
        .expect("request_animation_frame failed");

    closure.forget();
}

fn get_canvas() -> Option<HtmlCanvasElement> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let el = document.get_element_by_id("canvas")?;
    el.dyn_into::<HtmlCanvasElement>().ok()
}

fn show_render_error(msg: &str) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let key = JsValue::from_str("show_render_error");
    let cb = match Reflect::get(&window, &key) {
        Ok(v) => v,
        Err(_) => return,
    };
    if let Some(func) = cb.dyn_ref::<js_sys::Function>() {
        let _ = func.call1(&window, &JsValue::from_str(msg));
    }
}

fn call_stats_overlay(stats: &FrameStats) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let key = JsValue::from_str("set_stats_overlay");
    let cb = match Reflect::get(&window, &key) {
        Ok(v) => v,
        Err(_) => return,
    };
    let func = match cb.dyn_ref::<js_sys::Function>() {
        Some(f) => f,
        None => return,
    };
    let gpu_ms = stats.gpu_time_ms.map_or(-1.0_f64, |g| g as f64);
    let _ = func.call4(
        &window,
        &JsValue::from(stats.fps as f64),
        &JsValue::from(stats.cpu_time_ms as f64),
        &JsValue::from(gpu_ms),
        &JsValue::from(stats.element_count as f64),
    );
}

/// Set the scene to load on the next frame. Call from JS: `set_demo("default")`, `set_demo("curves")`, `set_demo("allshapes")`, or `set_demo("aabb2d")`.
pub fn set_demo(name: &str) {
    let demo = if name.eq_ignore_ascii_case("curves") {
        RunDemo::Curves
    } else if name.eq_ignore_ascii_case("allshapes") {
        RunDemo::AllShapes
    } else if name.eq_ignore_ascii_case("aabb2d") {
        RunDemo::Aabb2d
    } else {
        RunDemo::Default
    };
    PENDING_DEMO.with(|c| c.borrow_mut().replace(demo));
}

/// Return the current world's primitive tree as a string (updated each frame). Used by the "Show tree" menu.
#[must_use]
pub fn get_primitive_tree() -> String {
    TREE_DUMP.with(|c| c.borrow().clone())
}

/// Return the currently selected entity id, or [`NO_ENTITY`] if none. Updated each frame.
#[must_use]
pub fn get_selected_entity() -> u32 {
    SELECTED_ENTITY_ID.with(|c| *c.borrow())
}

/// Set the selected entity. Pass [`NO_ENTITY`] to clear selection. Applied on next frame.
pub fn set_selected_entity(id: u32) {
    PENDING_SELECTION.with(|c| {
        c.borrow_mut()
            .replace(if id == NO_ENTITY { None } else { Some(id) });
    });
}

/// Return material names (newline-separated) for the HTML material dropdown. Updated each frame.
#[must_use]
pub fn get_material_names() -> String {
    MATERIAL_NAMES_DUMP.with(|c| c.borrow().clone())
}

/// Set the material for an entity. No-op if id is invalid. Applied on next frame.
pub fn set_entity_material(entity_id: u32, material_name: &str) {
    PENDING_SET_MATERIAL.with(|c| {
        c.borrow_mut()
            .replace((entity_id, material_name.to_string()))
    });
}

/// Set the primitive for an entity. Pass a name like "cube", "sphere", "line", "bezier". No-op if id or name invalid. Applied on next frame.
pub fn set_entity_primitive(entity_id: u32, primitive_name: &str) {
    PENDING_SET_PRIMITIVE.with(|c| {
        c.borrow_mut()
            .replace((entity_id, primitive_name.to_string()))
    });
}

/// Remove the currently selected entity (one at a time). Root cannot be removed. Applied on next frame.
pub fn remove_selected_entity() {
    let id = get_selected_entity();
    if id != NO_ENTITY {
        PENDING_REMOVE_ENTITY.with(|c| c.borrow_mut().replace(id));
    }
}

/// Return the current gizmo mode: "translate", "rotate", or "scale". The gizmo is only shown when an entity is picked. Updated each frame.
#[must_use]
pub fn get_gizmo_mode() -> String {
    GIZMO_MODE_DUMP.with(|c| c.borrow().clone())
}

/// Set the gizmo mode for the next frame. Pass "translate", "rotate", or "scale" to change which transform handles are shown when an entity is selected.
pub fn set_gizmo_mode(mode: &str) {
    let m = match mode.trim().to_lowercase().as_str() {
        "rotate" => Some(GizmoMode::Rotate),
        "scale" => Some(GizmoMode::Scale),
        _ => Some(GizmoMode::Translate),
    };
    if let Some(g) = m {
        PENDING_GIZMO_MODE.with(|c| c.borrow_mut().replace(g));
    }
}

/// Add a new entity as child of the root with the given primitive (e.g. "cube", "sphere"). Applied next frame; the new entity becomes selected.
pub fn add_entity(primitive_name: &str) {
    PENDING_ADD_ENTITY.with(|c| c.borrow_mut().replace(primitive_name.to_string()));
}

/// Local position of the selected entity as "x,y,z". Empty string if none selected. Updated each frame.
#[must_use]
pub fn get_selected_entity_local_position() -> String {
    LOCAL_POSITION_DUMP.with(|c| c.borrow().clone())
}

/// World position of the selected entity as "x,y,z". Empty string if none selected. Updated each frame.
#[must_use]
pub fn get_selected_entity_world_position() -> String {
    WORLD_POSITION_DUMP.with(|c| c.borrow().clone())
}

/// Set local position of an entity. Applied next frame.
pub fn set_entity_local_position(entity_id: u32, x: f32, y: f32, z: f32) {
    PENDING_SET_LOCAL_POSITION.with(|c| c.borrow_mut().replace((entity_id, [x, y, z])));
}

/// Set world position of an entity (adjusts local so world position becomes (x,y,z)). Applied next frame.
pub fn set_entity_world_position(entity_id: u32, x: f32, y: f32, z: f32) {
    PENDING_SET_WORLD_POSITION.with(|c| c.borrow_mut().replace((entity_id, [x, y, z])));
}

/// Local rotation (roll, pitch, yaw) of the selected entity in radians as "r,p,y". Empty string if none selected.
#[must_use]
pub fn get_selected_entity_local_rotation() -> String {
    LOCAL_ROTATION_DUMP.with(|c| c.borrow().clone())
}

/// Local scale of the selected entity as "x,y,z". Empty string if none selected.
#[must_use]
pub fn get_selected_entity_local_scale() -> String {
    LOCAL_SCALE_DUMP.with(|c| c.borrow().clone())
}

/// Set local rotation of an entity. Pass roll, pitch, yaw in radians. Applied next frame.
pub fn set_entity_local_rotation(entity_id: u32, roll: f32, pitch: f32, yaw: f32) {
    PENDING_SET_LOCAL_ROTATION.with(|c| {
        c.borrow_mut().replace((entity_id, [roll, pitch, yaw]));
    });
}

/// Set local scale of an entity. Applied next frame.
pub fn set_entity_local_scale(entity_id: u32, x: f32, y: f32, z: f32) {
    PENDING_SET_LOCAL_SCALE.with(|c| c.borrow_mut().replace((entity_id, [x, y, z])));
}
