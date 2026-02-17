//! WASM platform: canvas, requestAnimationFrame, kinematics scene, orbit + IK target, UI and FPS overlay.
//!
//! Expects the page to define `window.set_stats_overlay(fps, cpu_ms, gpu_ms, elements)` and
//! optionally `window.show_render_error(msg)` for init errors (same pattern as render-demo).

#[cfg(feature = "bvh")]
use crate::scene::build_chain_from_armature;
use crate::scene::{
    apply_kinematics_action, build_armature_controls_panel, build_armature_tree_panel, build_chain,
    build_kinematics_scene, build_scene_entity_panel, camera_view_forward, format_armature_tree,
    randomize_ik_target_for_chain, screen_to_plane_at_point, set_target_to_end_effector_for_chain,
    step_ik, ArmPreset, ChainIndex, IkSolverType, KinematicsScene, ARMATURE_CONTROLS_WINDOW_ID,
    ARMATURE_TREE_WINDOW_ID, SCENE_ENTITY_WINDOW_ID,
};
use js_sys::Reflect;
#[cfg(feature = "bvh")]
use kinematics::Armature;
use render::input_constants::PAN_SENSITIVITY;
use render::{
    build_stats_panel, pick_entity, pick_gizmo_handle, Engine, FrameStats, GizmoMode,
    GIZMO_DEFAULT_SIZE, STATS_WINDOW_ID,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

thread_local! {
    static PENDING_SOLVER: RefCell<Option<IkSolverType>> = RefCell::new(None);
    static PENDING_EE_IDX: RefCell<Option<usize>> = RefCell::new(None);
    static PENDING_RANDOM_TARGET: RefCell<bool> = RefCell::new(false);
    static PENDING_SET_TARGET_TO_EE: RefCell<bool> = RefCell::new(false);
    static PENDING_RESET_SCENE: RefCell<bool> = RefCell::new(false);
    #[cfg(feature = "bvh")]
    static PENDING_BVH_ARMATURE: RefCell<Option<Armature>> = RefCell::new(None);
    static PENDING_ACTIVE_CHAIN: RefCell<Option<ChainIndex>> = RefCell::new(None);
    static PENDING_ARM_PRESET: RefCell<Option<ArmPreset>> = RefCell::new(None);
    static ACTIVE_CHAIN_DUMP: RefCell<String> = RefCell::new("a".to_string());
    static ARMATURE_TREE_DUMP: RefCell<String> = RefCell::new(String::new());
    static EE_POSITION_DUMP: RefCell<String> = RefCell::new(String::new());
    static TARGET_POSITION_DUMP: RefCell<String> = RefCell::new(String::new());
    static SOLVER_DUMP: RefCell<String> = RefCell::new("fabrik".to_string());
    static CURRENT_EE_IDX: RefCell<u32> = RefCell::new(3);
    static NODE_COUNT_DUMP: RefCell<u32> = RefCell::new(4);
    /// Last Hessian snapshot (data row-major, size n, error); updated each frame when solver is Hessian.
    static HESSIAN_SNAPSHOT: RefCell<Option<(Vec<f64>, usize, f32)>> = RefCell::new(None);
}
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlCanvasElement;

const ORBIT_SENSITIVITY: f32 = 0.005;
const ZOOM_SENSITIVITY: f32 = 0.001;
const FPS_EMA_ALPHA: f32 = 0.9;
const PICK_CLICK_THRESHOLD_PX: f32 = 6.0;

type EngineAndScene = Option<(Engine, KinematicsScene)>;

/// Frame timing: (last_time_ms, fps_ema).
type FrameTiming = (f64, f32);

/// Canvas-relative pixel coordinates from a pointer event.
fn canvas_coords(e: &web_sys::PointerEvent, canvas: &HtmlCanvasElement) -> (f32, f32) {
    let rect = canvas.get_bounding_client_rect();
    let w = rect.width().max(1.0);
    let h = rect.height().max(1.0);
    let x = (e.client_x() as f64 - rect.left()) / w as f64 * f64::from(canvas.width());
    let y = (e.client_y() as f64 - rect.top()) / h as f64 * f64::from(canvas.height());
    (x as f32, y as f32)
}

/// Orbit state: (pressing, last_cursor, down_pos for click detection).
type OrbitState = (bool, Option<(f64, f64)>, Option<(f32, f32)>);
/// IK target state: (pressing, last_cursor).
type IkTargetState = (bool, Option<(f64, f64)>);

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
    // If layout hasn't given the canvas real size yet (e.g. flex before paint), use fallback so WebGPU gets a usable viewport.
    const MIN_SIZE: u32 = 100;
    let (w, h) = if buffer_w >= MIN_SIZE && buffer_h >= MIN_SIZE {
        (buffer_w, buffer_h)
    } else {
        (800u32.max(buffer_w), 600u32.max(buffer_h))
    };
    canvas.set_width(w);
    canvas.set_height(h);
}

/// Show init error: set #error div and call optional JS callback (same pattern as render-demo).
fn show_error_on_page(message: &str) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let document = match window.document() {
        Some(d) => d,
        None => return,
    };
    if let Some(el) = document.get_element_by_id("error") {
        if let Some(msg_el) = document.get_element_by_id("error-message") {
            msg_el.set_text_content(Some(message));
        }
        el.class_list().add_1("visible").ok();
    }
    let key = JsValue::from_str("show_render_error");
    if let Ok(cb) = Reflect::get(&window, &key) {
        if let Some(func) = cb.dyn_ref::<js_sys::Function>() {
            let _ = func.call1(&window, &JsValue::from_str(message));
        }
    }
}

fn get_canvas() -> Option<HtmlCanvasElement> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let el = document.get_element_by_id("canvas")?;
    el.dyn_into::<HtmlCanvasElement>().ok()
}

/// Call JS `set_stats_overlay(fps, cpu_ms, gpu_ms, elements)` if defined (matches render-demo).
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
    let gpu_ms = stats.gpu_time_ms.map_or(-1.0_f64, |g| f64::from(g));
    let _ = func.call4(
        &window,
        &JsValue::from(f64::from(stats.fps)),
        &JsValue::from(f64::from(stats.cpu_time_ms)),
        &JsValue::from(gpu_ms),
        &JsValue::from(stats.element_count as f64),
    );
}

/// Run on WASM (canvas, requestAnimationFrame).
pub fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    let engine_scene: Rc<RefCell<EngineAndScene>> = Rc::new(RefCell::new(None));
    let orbit_state: Rc<RefCell<OrbitState>> = Rc::new(RefCell::new((false, None, None)));
    let ik_target_state: Rc<RefCell<IkTargetState>> = Rc::new(RefCell::new((false, None)));
    let ctrl_held: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
    let frame_timing: Rc<RefCell<FrameTiming>> = Rc::new(RefCell::new((0.0, 60.0)));
    let frame_count: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));

    let engine_clone = Rc::clone(&engine_scene);
    let orbit_clone = Rc::clone(&orbit_state);
    let ik_clone = Rc::clone(&ik_target_state);
    let ctrl_clone = Rc::clone(&ctrl_held);
    let timing_clone = Rc::clone(&frame_timing);
    let frame_count_clone = Rc::clone(&frame_count);
    wasm_bindgen_futures::spawn_local(async move {
        match Engine::new_wasm(canvas.clone()).await {
            Ok(mut eng) => {
                let scene = build_kinematics_scene(eng.world_mut());
                let _ = eng.enable_ui();
                eng.set_gizmo_mode(GizmoMode::Translate);
                if let Some(ui) = eng.ui.as_mut() {
                    ui.set_theme(render::Theme::dark());
                }
                *engine_clone.borrow_mut() = Some((eng, scene));
                if canvas.width() > 0 && canvas.height() > 0 {
                    if let Some((eng, _)) = engine_clone.borrow_mut().as_mut() {
                        eng.resize(canvas.width(), canvas.height());
                    }
                }
                setup_listeners(&engine_clone, &orbit_clone, &ik_clone, &ctrl_clone, &canvas);
                schedule_frame(&engine_clone, &timing_clone, &frame_count_clone);
            }
            Err(e) => {
                let msg = format!("kinematics-demo init failed: {e}");
                web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&msg));
                show_error_on_page(&msg);
            }
        }
    });

    Ok(())
}

fn setup_listeners(
    engine_scene: &Rc<RefCell<EngineAndScene>>,
    orbit_state: &Rc<RefCell<OrbitState>>,
    ik_target_state: &Rc<RefCell<IkTargetState>>,
    ctrl_held: &Rc<RefCell<bool>>,
    canvas: &HtmlCanvasElement,
) {
    let engine_scene = Rc::clone(engine_scene);
    let canvas = canvas.clone();
    let ctrl_held = Rc::clone(ctrl_held);

    // Track Ctrl for pan. Ignore key repeat.
    let ctrl_down = Rc::clone(&ctrl_held);
    let on_key_down: Closure<dyn FnMut(web_sys::KeyboardEvent)> =
        Closure::new(move |e: web_sys::KeyboardEvent| {
            if e.repeat() {
                return;
            }
            if e.key() == "Control" {
                *ctrl_down.borrow_mut() = true;
                return;
            }
            match e.key().as_str() {
                "1" => {
                    let _ = e.prevent_default();
                    set_active_chain("a");
                }
                "2" => {
                    let _ = e.prevent_default();
                    set_active_chain("b");
                }
                "Tab" => {
                    let _ = e.prevent_default();
                    let cur = ACTIVE_CHAIN_DUMP.with(|c| c.borrow().clone());
                    set_active_chain(if cur == "a" { "b" } else { "a" });
                }
                "r" | "R" => {
                    let _ = e.prevent_default();
                    randomize_target();
                }
                _ => {}
            }
        });
    let ctrl_up = Rc::clone(&ctrl_held);
    let on_key_up: Closure<dyn FnMut(web_sys::KeyboardEvent)> =
        Closure::new(move |e: web_sys::KeyboardEvent| {
            if e.repeat() {
                return;
            }
            if e.key() == "Control" {
                *ctrl_up.borrow_mut() = false;
            }
        });
    if let Some(w) = web_sys::window() {
        let _ = w.add_event_listener_with_callback("keydown", on_key_down.as_ref().unchecked_ref());
        let _ = w.add_event_listener_with_callback("keyup", on_key_up.as_ref().unchecked_ref());
        on_key_down.forget();
        on_key_up.forget();
    }

    // Pointer down: left = orbit or UI, right = IK target
    let engine_down = Rc::clone(&engine_scene);
    let orbit_down = Rc::clone(orbit_state);
    let ik_down = Rc::clone(ik_target_state);
    let canvas_down = canvas.clone();
    let on_pointer_down: Closure<dyn FnMut(web_sys::PointerEvent)> =
        Closure::new(move |e: web_sys::PointerEvent| {
            let (x, y) = canvas_coords(&e, &canvas_down);
            if let Some((eng, _)) = engine_down.borrow_mut().as_mut() {
                eng.ui_mouse_move(x, y);
                eng.ui_mouse_down();
                if e.button() == 0 && !eng.is_cursor_over_ui() {
                    let mut started_gizmo = false;
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
                                    started_gizmo = true;
                                }
                            }
                        }
                    }
                    if !started_gizmo {
                        *orbit_down.borrow_mut() = (true, None, Some((x, y)));
                    } else {
                        *orbit_down.borrow_mut() = (false, None, None);
                    }
                } else if e.button() == 2 {
                    *ik_down.borrow_mut() = (true, None);
                }
            }
        });
    canvas
        .add_event_listener_with_callback("pointerdown", on_pointer_down.as_ref().unchecked_ref())
        .expect("add pointerdown listener");
    on_pointer_down.forget();

    // Pointer up
    let engine_up = Rc::clone(&engine_scene);
    let orbit_up = Rc::clone(orbit_state);
    let ik_up = Rc::clone(ik_target_state);
    let canvas_up = canvas.clone();
    let on_pointer_up: Closure<dyn FnMut(web_sys::PointerEvent)> =
        Closure::new(move |e: web_sys::PointerEvent| {
            let (x, y) = canvas_coords(&e, &canvas_up);
            let (_, _, down_pos) = *orbit_up.borrow();
            let is_click = down_pos.map_or(false, |(dx, dy)| {
                let dist_sq = (x - dx).powi(2) + (y - dy).powi(2);
                dist_sq <= PICK_CLICK_THRESHOLD_PX * PICK_CLICK_THRESHOLD_PX
            });
            if let Some((eng, _)) = engine_up.borrow_mut().as_mut() {
                eng.ui_mouse_up();
                eng.gizmo_drag_end();
                if is_click && !eng.is_cursor_over_ui() {
                    if let Some(id) = pick_entity(eng.world(), eng.camera(), x, y) {
                        eng.set_selected_entity(Some(id));
                    }
                }
            }
            *orbit_up.borrow_mut() = (false, None, None);
            *ik_up.borrow_mut() = (false, None);
        });
    canvas
        .add_event_listener_with_callback("pointerup", on_pointer_up.as_ref().unchecked_ref())
        .expect("add pointerup listener");
    on_pointer_up.forget();

    // Pointer leave
    let orbit_leave = Rc::clone(orbit_state);
    let ik_leave = Rc::clone(ik_target_state);
    let on_pointer_leave: Closure<dyn FnMut(web_sys::PointerEvent)> =
        Closure::new(move |_e: web_sys::PointerEvent| {
            *orbit_leave.borrow_mut() = (false, None, None);
            *ik_leave.borrow_mut() = (false, None);
        });
    canvas
        .add_event_listener_with_callback("pointerleave", on_pointer_leave.as_ref().unchecked_ref())
        .expect("add pointerleave listener");
    on_pointer_leave.forget();

    // Pointer move: UI hit-test, then orbit/pan or IK target when not over UI
    let engine_move = Rc::clone(&engine_scene);
    let orbit_move = Rc::clone(orbit_state);
    let ik_move = Rc::clone(ik_target_state);
    let ctrl_move = Rc::clone(&ctrl_held);
    let canvas_move = canvas.clone();
    let on_pointer_move: Closure<dyn FnMut(web_sys::PointerEvent)> =
        Closure::new(move |e: web_sys::PointerEvent| {
            let (x, y) = canvas_coords(&e, &canvas_move);
            let x64 = e.client_x() as f64;
            let y64 = e.client_y() as f64;
            let mut g = engine_move.borrow_mut();
            if let Some((eng, scene)) = g.as_mut() {
                eng.ui_mouse_move(x, y);
                if eng.is_gizmo_dragging() {
                    eng.gizmo_drag_move(x, y);
                    return;
                }
                let (orbit_pressing, orbit_last, orbit_down) = *orbit_move.borrow();
                let (ik_pressing, _ik_last) = *ik_move.borrow();
                if orbit_pressing && !eng.is_cursor_over_ui() {
                    if let Some((lx, ly)) = orbit_last {
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
                    *orbit_move.borrow_mut() = (true, Some((x64, y64)), orbit_down);
                } else if ik_pressing && !eng.is_cursor_over_ui() {
                    let target = match scene.active_chain {
                        ChainIndex::A => scene.chain_a.ik_target,
                        ChainIndex::B => scene.chain_b.ik_target,
                    };
                    if let Some(pos) = screen_to_plane_at_point(eng.camera(), x, y, target) {
                        match scene.active_chain {
                            ChainIndex::A => scene.chain_a.ik_target = pos,
                            ChainIndex::B => scene.chain_b.ik_target = pos,
                        }
                        step_ik(scene, eng.world_mut());
                    }
                    *ik_move.borrow_mut() = (true, Some((x64, y64)));
                }
            }
        });
    canvas
        .add_event_listener_with_callback("pointermove", on_pointer_move.as_ref().unchecked_ref())
        .expect("add pointermove listener");
    on_pointer_move.forget();

    // Prevent context menu on right-click
    let on_context: Closure<dyn FnMut(web_sys::Event)> =
        Closure::new(|e: web_sys::Event| e.prevent_default());
    canvas
        .add_event_listener_with_callback("contextmenu", on_context.as_ref().unchecked_ref())
        .expect("add contextmenu listener");
    on_context.forget();

    // Wheel: zoom, or move IK target along view direction when right-dragging
    const DEPTH_SCROLL_SENSITIVITY: f32 = 0.05;
    let engine_wheel = Rc::clone(&engine_scene);
    let ik_wheel = Rc::clone(&ik_target_state);
    let on_wheel: Closure<dyn FnMut(web_sys::WheelEvent)> =
        Closure::new(move |e: web_sys::WheelEvent| {
            e.prevent_default();
            let delta = e.delta_y() as f32;
            let ik_pressing = ik_wheel.borrow().0;
            if let Some((eng, scene)) = engine_wheel.borrow_mut().as_mut() {
                if ik_pressing && !eng.is_cursor_over_ui() {
                    let forward = camera_view_forward(eng.camera());
                    let step = delta * DEPTH_SCROLL_SENSITIVITY;
                    let chain = match scene.active_chain {
                        ChainIndex::A => &mut scene.chain_a,
                        ChainIndex::B => &mut scene.chain_b,
                    };
                    chain.ik_target[0] += forward[0] * step;
                    chain.ik_target[1] += forward[1] * step;
                    chain.ik_target[2] += forward[2] * step;
                    step_ik(scene, eng.world_mut());
                } else {
                    eng.zoom(-delta * ZOOM_SENSITIVITY);
                }
            }
        });
    canvas
        .add_event_listener_with_callback("wheel", on_wheel.as_ref().unchecked_ref())
        .expect("add wheel listener");
    on_wheel.forget();

    // Resize
    let engine_resize = Rc::clone(&engine_scene);
    let canvas_resize = canvas.clone();
    let on_resize: Closure<dyn FnMut()> = Closure::new(move || {
        set_canvas_size_from_display(&canvas_resize);
        let w = canvas_resize.width();
        let h = canvas_resize.height();
        if w > 0 && h > 0 {
            if let Some((eng, _scene)) = engine_resize.borrow_mut().as_mut() {
                eng.resize(w, h);
            }
        }
    });
    web_sys::window()
        .expect("no window")
        .add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref())
        .expect("add resize listener");
    on_resize.forget();
}

fn schedule_frame(
    engine_scene: &Rc<RefCell<EngineAndScene>>,
    frame_timing: &Rc<RefCell<FrameTiming>>,
    frame_count: &Rc<RefCell<u32>>,
) {
    let engine_scene = Rc::clone(engine_scene);
    let frame_timing = Rc::clone(frame_timing);
    let frame_count = Rc::clone(frame_count);
    let closure = Closure::once(move || {
        let now_ms = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);
        let stats_to_show = {
            let mut g = engine_scene.borrow_mut();
            let (last_time, fps_ema) = *frame_timing.borrow();
            let dt = if last_time > 0.0 {
                (((now_ms - last_time) / 1000.0_f64) as f32).min(0.1)
            } else {
                0.0_f32
            };
            let (cpu_time_ms, fps) = if last_time > 0.0 {
                let elapsed_ms = now_ms - last_time;
                let fps = if elapsed_ms > 0.0 {
                    1000.0 / elapsed_ms
                } else {
                    60.0
                };
                let ema = fps_ema * FPS_EMA_ALPHA + (fps as f32) * (1.0 - FPS_EMA_ALPHA);
                (elapsed_ms as f32, ema)
            } else {
                (0.0, 60.0)
            };
            *frame_timing.borrow_mut() = (now_ms, fps);

            let mut stats_to_show = None;
            if let Some((eng, scene)) = g.as_mut() {
                if PENDING_RESET_SCENE.with(|c| {
                    let mut r = c.borrow_mut();
                    let prev = *r;
                    *r = false;
                    prev
                }) {
                    let world = eng.world_mut();
                    for &e in scene
                        .chain_a
                        .joint_entities
                        .iter()
                        .chain(scene.chain_a.link_entities.iter())
                        .chain(scene.chain_a.cone_entities.iter())
                        .chain(std::iter::once(&scene.chain_a.target_entity))
                    {
                        world.despawn(e);
                    }
                    for &e in scene
                        .chain_b
                        .joint_entities
                        .iter()
                        .chain(scene.chain_b.link_entities.iter())
                        .chain(scene.chain_b.cone_entities.iter())
                        .chain(std::iter::once(&scene.chain_b.target_entity))
                    {
                        world.despawn(e);
                    }
                    *scene = build_kinematics_scene(world);
                }
                #[cfg(feature = "bvh")]
                if let Some(armature) = PENDING_BVH_ARMATURE.with(|c| c.borrow_mut().take()) {
                    let active = scene.active_chain;
                    let chain = match active {
                        ChainIndex::A => &mut scene.chain_a,
                        ChainIndex::B => &mut scene.chain_b,
                    };
                    let to_despawn: Vec<_> = chain
                        .joint_entities
                        .iter()
                        .chain(chain.link_entities.iter())
                        .chain(chain.cone_entities.iter())
                        .copied()
                        .chain(std::iter::once(chain.target_entity))
                        .collect();
                    let world = eng.world_mut();
                    for &e in &to_despawn {
                        world.despawn(e);
                    }
                    let new_chain = build_chain_from_armature(
                        world,
                        armature,
                        chain.root_offset,
                        chain.color,
                        chain.target_color,
                        chain.ik_solver_type,
                    );
                    match active {
                        ChainIndex::A => scene.chain_a = new_chain,
                        ChainIndex::B => scene.chain_b = new_chain,
                    }
                }
                eng.set_last_frame_stats(FrameStats {
                    fps,
                    cpu_time_ms,
                    gpu_time_ms: eng.last_gpu_time_ms(),
                    element_count: eng.active_world_element_count(),
                });
                stats_to_show = eng.last_frame_stats().cloned();
                let viewport_w = eng.viewport_width() as f32;

                eng.update_ui_springs(dt);

                // Sync canvas size on first frames so layout (e.g. flex) is picked up (matches render-demo).
                {
                    let mut n = frame_count.borrow_mut();
                    let frame_num = *n;
                    *n = n.saturating_add(1);
                    let do_sync = frame_num <= 5 || (frame_num >= 6 && frame_num <= 8);
                    if do_sync {
                        if let Some(canvas) = get_canvas() {
                            set_canvas_size_from_display(&canvas);
                            let w = canvas.width();
                            let h = canvas.height();
                            if w > 0 && h > 0 {
                                eng.resize(w, h);
                            }
                        }
                    }
                    // One-time viewport/panel debug log after first successful frame
                    if frame_num == 1 {
                        crate::wasm_debug::maybe_enable_debug_from_url();
                        let vw = eng.viewport_width();
                        let vh = eng.camera().height;
                        let ctrl_x = (390.0_f32).min(vw as f32 - 190.0);
                        crate::wasm_debug::log_always(&format!(
                            "[kinematics] viewport {}x{} panels: scene(10,10,160,_) tree(180,10,200,_) controls({:.0},10,180,_)",
                            vw, vh, ctrl_x
                        ));
                    }
                }

                // Sync when canvas display size changes without a window resize (e.g. container or panel resized).
                if let Some(canvas) = get_canvas() {
                    let expected = web_sys::window().and_then(|w| {
                        let dpr = w.device_pixel_ratio().max(1.0);
                        let lw = canvas.client_width().max(1);
                        let lh = canvas.client_height().max(1);
                        let bw = (f64::from(lw) * dpr).round().max(1.0) as u32;
                        let bh = (f64::from(lh) * dpr).round().max(1.0) as u32;
                        const MIN: u32 = 100;
                        Some(if bw >= MIN && bh >= MIN {
                            (bw, bh)
                        } else {
                            (800u32.max(bw), 600u32.max(bh))
                        })
                    });
                    if let Some((ew, eh)) = expected {
                        if (ew, eh) != (canvas.width(), canvas.height()) {
                            set_canvas_size_from_display(&canvas);
                            let w = canvas.width();
                            let h = canvas.height();
                            if w > 0 && h > 0 {
                                eng.resize(w, h);
                            }
                        }
                    }
                }

                if let Some(selected) = eng.selected_entity() {
                    if selected == scene.chain_a.target_entity {
                        if let Some(data) = eng.world().get(selected) {
                            scene.chain_a.ik_target = data.transform.position;
                        }
                    } else if selected == scene.chain_b.target_entity {
                        if let Some(data) = eng.world().get(selected) {
                            scene.chain_b.ik_target = data.transform.position;
                        }
                    }
                }
                // Apply pending HTML UI actions. Apply active-chain switch first so solver/EE/random use the correct chain.
                if let Some(c) = PENDING_ACTIVE_CHAIN.with(|cell| cell.borrow_mut().take()) {
                    scene.active_chain = c;
                }
                let active = scene.active_chain;
                if let Some(preset) = PENDING_ARM_PRESET.with(|c| c.borrow_mut().take()) {
                    let chain = match active {
                        ChainIndex::A => &mut scene.chain_a,
                        ChainIndex::B => &mut scene.chain_b,
                    };
                    let to_despawn: Vec<_> = chain
                        .joint_entities
                        .iter()
                        .chain(chain.link_entities.iter())
                        .chain(chain.cone_entities.iter())
                        .copied()
                        .chain(std::iter::once(chain.target_entity))
                        .collect();
                    let world = eng.world_mut();
                    for &e in &to_despawn {
                        world.despawn(e);
                    }
                    let new_chain = build_chain(
                        world,
                        preset,
                        chain.root_offset,
                        chain.color,
                        chain.target_color,
                        chain.ik_solver_type,
                    );
                    match active {
                        ChainIndex::A => scene.chain_a = new_chain,
                        ChainIndex::B => scene.chain_b = new_chain,
                    }
                }
                if let Some(solver) = PENDING_SOLVER.with(|c| c.borrow_mut().take()) {
                    let chain = match active {
                        ChainIndex::A => &mut scene.chain_a,
                        ChainIndex::B => &mut scene.chain_b,
                    };
                    chain.ik_solver_type = solver;
                    let name = match solver {
                        IkSolverType::Fabrik => "FABRIK",
                        IkSolverType::FabrikSqp => "FABRIK+SQP",
                        IkSolverType::Jacobian => "Jacobian",
                        IkSolverType::Ccd => "CCD",
                        IkSolverType::Halley => "Halley",
                        IkSolverType::Hessian => "Hessian",
                    };
                    crate::wasm_debug::log(&format!(
                        "[kinematics] solver={} EE={}",
                        name, chain.end_effector_idx
                    ));
                }
                if let Some(idx) = PENDING_EE_IDX.with(|c| c.borrow_mut().take()) {
                    let chain = match active {
                        ChainIndex::A => &mut scene.chain_a,
                        ChainIndex::B => &mut scene.chain_b,
                    };
                    let n = chain.armature.tree().num_nodes();
                    if idx < n {
                        chain.end_effector_idx = idx;
                        crate::wasm_debug::log(&format!("[kinematics] EE_idx={}", idx));
                    }
                }
                if PENDING_RANDOM_TARGET.with(|c| {
                    let mut g = c.borrow_mut();
                    let prev = *g;
                    *g = false;
                    prev
                }) {
                    let chain = match active {
                        ChainIndex::A => &mut scene.chain_a,
                        ChainIndex::B => &mut scene.chain_b,
                    };
                    randomize_ik_target_for_chain(chain);
                }
                if PENDING_SET_TARGET_TO_EE.with(|c| {
                    let mut g = c.borrow_mut();
                    let prev = *g;
                    *g = false;
                    prev
                }) {
                    let chain = match active {
                        ChainIndex::A => &mut scene.chain_a,
                        ChainIndex::B => &mut scene.chain_b,
                    };
                    set_target_to_end_effector_for_chain(chain);
                }
                step_ik(scene, eng.world_mut());
                // Update dumps for HTML UI getters (active chain).
                ARMATURE_TREE_DUMP.with(|c| *c.borrow_mut() = format_armature_tree(scene));
                let chain = match scene.active_chain {
                    ChainIndex::A => &scene.chain_a,
                    ChainIndex::B => &scene.chain_b,
                };
                let ee = chain.armature.end_effector_position(chain.end_effector_idx);
                let ee_wx = chain.root_offset[0] + ee.get(0);
                let ee_wy = chain.root_offset[1] + ee.get(1);
                let ee_wz = chain.root_offset[2] + ee.get(2);
                EE_POSITION_DUMP
                    .with(|c| *c.borrow_mut() = format!("{:.3},{:.3},{:.3}", ee_wx, ee_wy, ee_wz));
                TARGET_POSITION_DUMP.with(|c| {
                    *c.borrow_mut() = format!(
                        "{:.3},{:.3},{:.3}",
                        chain.ik_target[0], chain.ik_target[1], chain.ik_target[2]
                    )
                });
                SOLVER_DUMP.with(|c| {
                    *c.borrow_mut() = match chain.ik_solver_type {
                        IkSolverType::Fabrik => "fabrik".to_string(),
                        IkSolverType::FabrikSqp => "fabrik_sqp".to_string(),
                        IkSolverType::Jacobian => "jacobian".to_string(),
                        IkSolverType::Ccd => "ccd".to_string(),
                        IkSolverType::Halley => "halley".to_string(),
                        IkSolverType::Hessian => "hessian".to_string(),
                    }
                });
                CURRENT_EE_IDX.with(|c| *c.borrow_mut() = chain.end_effector_idx as u32);
                NODE_COUNT_DUMP
                    .with(|c| *c.borrow_mut() = chain.armature.tree().num_nodes() as u32);
                // Update Hessian snapshot for viz when active chain solver is Hessian.
                {
                    let chain_mut = match scene.active_chain {
                        ChainIndex::A => &mut scene.chain_a,
                        ChainIndex::B => &mut scene.chain_b,
                    };
                    if chain_mut.ik_solver_type == IkSolverType::Hessian {
                        let target = mathlib::cg::vector3(
                            chain_mut.ik_target[0],
                            chain_mut.ik_target[1],
                            chain_mut.ik_target[2],
                        );
                        let snap = kinematics::hessian_snapshot(
                            &mut chain_mut.armature,
                            chain_mut.end_effector_idx,
                            target,
                        );
                        HESSIAN_SNAPSHOT.with(|c| *c.borrow_mut() = snap);
                    } else {
                        HESSIAN_SNAPSHOT.with(|c| *c.borrow_mut() = None);
                    }
                }
                ACTIVE_CHAIN_DUMP.with(|c| {
                    *c.borrow_mut() = match scene.active_chain {
                        ChainIndex::A => "a".to_string(),
                        ChainIndex::B => "b".to_string(),
                    }
                });
                let (scene_entity_window, scene_entity_mapping) =
                    build_scene_entity_panel(scene, eng.world());
                let mut merged_opt: Option<HashMap<_, _>> = None;
                if let Some(ui) = eng.ui.as_mut() {
                    ui.windows_mut().retain(|w| {
                        w.id != STATS_WINDOW_ID
                            && w.id != ARMATURE_TREE_WINDOW_ID
                            && w.id != ARMATURE_CONTROLS_WINDOW_ID
                            && w.id != SCENE_ENTITY_WINDOW_ID
                    });
                    if let Some(ref stats) = stats_to_show {
                        ui.add_window(build_stats_panel(stats, viewport_w));
                    }
                    ui.add_window(scene_entity_window);
                    ui.add_window(build_armature_tree_panel(scene, viewport_w));
                    let (controls_window, controls_mapping) =
                        build_armature_controls_panel(scene, viewport_w);
                    ui.add_window(controls_window);
                    let mut merged: HashMap<_, _> = scene_entity_mapping;
                    merged.extend(controls_mapping);
                    merged_opt = Some(merged);
                }
                if let (Some(merged), Some(cid)) = (merged_opt, eng.take_clicked_control()) {
                    apply_kinematics_action(scene, eng, &merged, cid);
                }

                let _ = eng.render_frame();
            }
            stats_to_show
        };
        if let Some(ref stats) = stats_to_show {
            call_stats_overlay(stats);
        }
        schedule_frame(&engine_scene, &frame_timing, &frame_count);
    });
    web_sys::window()
        .and_then(|w| {
            w.request_animation_frame(closure.as_ref().unchecked_ref())
                .ok()
        })
        .expect("request_animation_frame failed");
    closure.forget();
}

/// Set the IK solver for the active chain. Call with `"fabrik"`, `"jacobian"`, `"ccd"`, or `"halley"`; applied on next frame.
pub fn set_ik_solver(name: &str) {
    let solver = if name.eq_ignore_ascii_case("jacobian") {
        IkSolverType::Jacobian
    } else if name.eq_ignore_ascii_case("ccd") {
        IkSolverType::Ccd
    } else if name.eq_ignore_ascii_case("halley") {
        IkSolverType::Halley
    } else if name.eq_ignore_ascii_case("hessian") {
        IkSolverType::Hessian
    } else if name.eq_ignore_ascii_case("fabrik_sqp") {
        IkSolverType::FabrikSqp
    } else {
        IkSolverType::Fabrik
    };
    PENDING_SOLVER.with(|c| c.borrow_mut().replace(solver));
}

/// Return the active chain as `"a"` or `"b"`. Updated each frame.
#[must_use]
#[allow(dead_code)] // Called from JS via WASM; no in-crate references
pub fn get_active_chain() -> String {
    ACTIVE_CHAIN_DUMP.with(|c| c.borrow().clone())
}

/// Set the active chain. Call with `"a"` or `"b"`; applied on next frame.
pub fn set_active_chain(name: &str) {
    let c = if name.eq_ignore_ascii_case("b") {
        ChainIndex::B
    } else {
        ChainIndex::A
    };
    PENDING_ACTIVE_CHAIN.with(|cell| cell.borrow_mut().replace(c));
}

/// Set the arm preset for the active chain. Call with `"spherical"`, `"revolute"`, `"mixed"`, or `"snake"`; applied on next frame.
pub fn set_arm_preset(name: &str) {
    let preset = if name.eq_ignore_ascii_case("revolute") {
        ArmPreset::Revolute
    } else if name.eq_ignore_ascii_case("mixed") {
        ArmPreset::MixedArm
    } else if name.eq_ignore_ascii_case("snake") {
        ArmPreset::SphericalSnake
    } else {
        ArmPreset::Spherical
    };
    PENDING_ARM_PRESET.with(|c| c.borrow_mut().replace(preset));
}

/// Return the current IK solver: `"fabrik"`, `"jacobian"`, `"ccd"`, or `"halley"`. Updated each frame.
#[must_use]
pub fn get_ik_solver() -> String {
    SOLVER_DUMP.with(|c| c.borrow().clone())
}

/// Return comma-separated list of solver names available on this build.
#[must_use]
pub fn get_available_solvers() -> String {
    "fabrik,fabrik_sqp,jacobian,ccd,halley,hessian".to_string()
}

/// Return the armature tree as a newline-separated string. Updated each frame.
#[must_use]
pub fn get_armature_tree() -> String {
    ARMATURE_TREE_DUMP.with(|c| c.borrow().clone())
}

/// Return the end-effector position as `"x,y,z"`. Updated each frame.
#[must_use]
pub fn get_ee_position() -> String {
    EE_POSITION_DUMP.with(|c| c.borrow().clone())
}

/// Return the current end-effector node index. Updated each frame.
#[must_use]
pub fn get_end_effector_index() -> u32 {
    CURRENT_EE_IDX.with(|c| *c.borrow())
}

/// Return the number of nodes in the active chain's armature. Updated each frame.
#[must_use]
pub fn get_armature_node_count() -> u32 {
    NODE_COUNT_DUMP.with(|c| *c.borrow())
}

/// Return the IK target position as `"x,y,z"`. Updated each frame.
#[must_use]
pub fn get_target_position() -> String {
    TARGET_POSITION_DUMP.with(|c| c.borrow().clone())
}

/// Return the last Hessian snapshot for the active chain when solver is Hessian: `{ hessian: Float64Array (row-major n×n), size: number, error: number }` or null.
#[must_use]
pub fn get_hessian_snapshot() -> JsValue {
    let snap = HESSIAN_SNAPSHOT.with(|c| c.borrow().clone());
    let Some((data, n, err)) = snap else {
        return JsValue::NULL;
    };
    let arr = js_sys::Float64Array::new_with_length(data.len() as u32);
    for (i, &v) in data.iter().enumerate() {
        arr.set_index(i as u32, v);
    }
    let obj = js_sys::Object::new();
    let _ = Reflect::set(&obj, &"hessian".into(), &arr.into());
    let _ = Reflect::set(&obj, &"size".into(), &(n as u32).into());
    let _ = Reflect::set(&obj, &"error".into(), &err.into());
    obj.into()
}

/// Set the end-effector node index. Applied on next frame.
pub fn set_end_effector(idx: u32) {
    PENDING_EE_IDX.with(|c| c.borrow_mut().replace(idx as usize));
}

/// Trigger randomize IK target. Applied on next frame.
pub fn randomize_target() {
    PENDING_RANDOM_TARGET.with(|c| *c.borrow_mut() = true);
}

/// Set the IK target to the current end-effector position for the active chain. Applied on next frame.
pub fn set_target_to_ee() {
    PENDING_SET_TARGET_TO_EE.with(|c| *c.borrow_mut() = true);
}

/// Reset the scene to initial state. Applied on next frame.
pub fn reset_scene() {
    PENDING_RESET_SCENE.with(|c| *c.borrow_mut() = true);
}

/// Load BVH from bytes and replace the active chain on next frame. No-op if parse or conversion fails. Requires `bvh` feature.
#[cfg(feature = "bvh")]
pub fn load_bvh_from_bytes(bytes: &[u8]) -> bool {
    let Ok(bvh) = parse::bvh::parse(bytes) else {
        return false;
    };
    let Some(armature) = crate::bvh_import::armature_from_bvh(&bvh) else {
        return false;
    };
    PENDING_BVH_ARMATURE.with(|c| c.borrow_mut().replace(armature));
    true
}
