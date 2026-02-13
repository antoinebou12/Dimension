//! WASM platform: canvas, requestAnimationFrame, physics scene, orbit + pan + zoom, UI and FPS overlay.

use crate::scene::{
    build_physics_scene, build_spawn_panel, spawn_body, step_physics, PhysicsScene,
    SPAWN_BUTTON_ID, SPAWN_WINDOW_ID,
};
use render::input_constants::{ORBIT_SENSITIVITY, PAN_SENSITIVITY, ZOOM_SENSITIVITY};
use render::{build_stats_panel, Engine, FrameStats, STATS_WINDOW_ID};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

const FPS_EMA_ALPHA: f32 = 0.9;
const DT: f32 = 1.0 / 60.0;

type EngineAndScene = Option<(Engine, PhysicsScene)>;
type FrameTiming = (f64, f32);
type OrbitState = (bool, Option<(f64, f64)>);

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
    const MIN_SIZE: u32 = 100;
    let (w, h) = if buffer_w >= MIN_SIZE && buffer_h >= MIN_SIZE {
        (buffer_w, buffer_h)
    } else {
        (800u32.max(buffer_w), 600u32.max(buffer_h))
    };
    canvas.set_width(w);
    canvas.set_height(h);
}

fn show_error_on_page(message: &str) {
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        if let Some(el) = document.get_element_by_id("error") {
            if let Some(msg_el) = document.get_element_by_id("error-message") {
                msg_el.set_text_content(Some(message));
            }
            let _ = el.class_list().add_1("visible");
        }
    }
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
    let orbit_state: Rc<RefCell<OrbitState>> = Rc::new(RefCell::new((false, None)));
    let frame_timing: Rc<RefCell<FrameTiming>> = Rc::new(RefCell::new((0.0, 60.0)));

    let engine_clone = Rc::clone(&engine_scene);
    let orbit_clone = Rc::clone(&orbit_state);
    let timing_clone = Rc::clone(&frame_timing);
    wasm_bindgen_futures::spawn_local(async move {
        match Engine::new_wasm(canvas.clone()).await {
            Ok(mut eng) => {
                let scene = build_physics_scene(eng.world_mut());
                let _ = eng.enable_ui();
                if let Some(ui) = eng.ui.as_mut() {
                    ui.set_theme(render::Theme::dark());
                }
                *engine_clone.borrow_mut() = Some((eng, scene));
                setup_listeners(&engine_clone, &orbit_clone, &canvas);
                schedule_frame(&engine_clone, &timing_clone);
            }
            Err(e) => {
                let msg = format!("physics-demo init failed: {e}");
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
    canvas: &HtmlCanvasElement,
) {
    let engine_scene = Rc::clone(engine_scene);
    let canvas = canvas.clone();

    let engine_down = Rc::clone(&engine_scene);
    let orbit_down = Rc::clone(orbit_state);
    let canvas_down = canvas.clone();
    let on_pointer_down: Closure<dyn FnMut(web_sys::PointerEvent)> =
        Closure::new(move |e: web_sys::PointerEvent| {
            let (x, y) = canvas_coords(&e, &canvas_down);
            if let Some((eng, _)) = engine_down.borrow_mut().as_mut() {
                eng.ui_mouse_move(x, y);
                eng.ui_mouse_down();
                if e.button() == 0 && !eng.is_cursor_over_ui() {
                    *orbit_down.borrow_mut() = (true, None);
                }
            }
        });
    canvas
        .add_event_listener_with_callback("pointerdown", on_pointer_down.as_ref().unchecked_ref())
        .expect("add pointerdown listener");
    on_pointer_down.forget();

    let engine_up = Rc::clone(&engine_scene);
    let orbit_up = Rc::clone(orbit_state);
    let on_pointer_up: Closure<dyn FnMut(web_sys::PointerEvent)> =
        Closure::new(move |_e: web_sys::PointerEvent| {
            if let Some((eng, _)) = engine_up.borrow_mut().as_mut() {
                eng.ui_mouse_up();
            }
            *orbit_up.borrow_mut() = (false, None);
        });
    canvas
        .add_event_listener_with_callback("pointerup", on_pointer_up.as_ref().unchecked_ref())
        .expect("add pointerup listener");
    on_pointer_up.forget();

    let orbit_leave = Rc::clone(orbit_state);
    let on_pointer_leave: Closure<dyn FnMut(web_sys::PointerEvent)> =
        Closure::new(move |_e: web_sys::PointerEvent| {
            *orbit_leave.borrow_mut() = (false, None);
        });
    canvas
        .add_event_listener_with_callback("pointerleave", on_pointer_leave.as_ref().unchecked_ref())
        .expect("add pointerleave listener");
    on_pointer_leave.forget();

    let engine_move = Rc::clone(&engine_scene);
    let orbit_move = Rc::clone(orbit_state);
    let canvas_move = canvas.clone();
    let on_pointer_move: Closure<dyn FnMut(web_sys::PointerEvent)> =
        Closure::new(move |e: web_sys::PointerEvent| {
            let (x, y) = canvas_coords(&e, &canvas_move);
            let x64 = e.client_x() as f64;
            let y64 = e.client_y() as f64;
            let mut g = engine_move.borrow_mut();
            if let Some((eng, _scene)) = g.as_mut() {
                eng.ui_mouse_move(x, y);
                let (orbit_pressing, orbit_last) = *orbit_move.borrow();
                if orbit_pressing && !eng.is_cursor_over_ui() {
                    if let Some((lx, ly)) = orbit_last {
                        if e.ctrl_key() {
                            let dx = (x64 - lx) as f32 * PAN_SENSITIVITY;
                            let dy = (y64 - ly) as f32 * PAN_SENSITIVITY;
                            eng.pan(dx, dy);
                        } else {
                            let dyaw = (x64 - lx) as f32 * ORBIT_SENSITIVITY;
                            let dpitch = (ly - y64) as f32 * ORBIT_SENSITIVITY;
                            eng.orbit(dyaw, dpitch);
                        }
                    }
                    *orbit_move.borrow_mut() = (true, Some((x64, y64)));
                }
            }
        });
    canvas
        .add_event_listener_with_callback("pointermove", on_pointer_move.as_ref().unchecked_ref())
        .expect("add pointermove listener");
    on_pointer_move.forget();

    let on_context: Closure<dyn FnMut(web_sys::Event)> =
        Closure::new(|e: web_sys::Event| e.prevent_default());
    canvas
        .add_event_listener_with_callback("contextmenu", on_context.as_ref().unchecked_ref())
        .expect("add contextmenu listener");
    on_context.forget();

    let engine_wheel = Rc::clone(&engine_scene);
    let on_wheel: Closure<dyn FnMut(web_sys::WheelEvent)> =
        Closure::new(move |e: web_sys::WheelEvent| {
            e.prevent_default();
            let delta = e.delta_y() as f32 * ZOOM_SENSITIVITY;
            if let Some((eng, _scene)) = engine_wheel.borrow_mut().as_mut() {
                eng.zoom(-delta);
            }
        });
    canvas
        .add_event_listener_with_callback("wheel", on_wheel.as_ref().unchecked_ref())
        .expect("add wheel listener");
    on_wheel.forget();

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
) {
    let engine_scene = Rc::clone(engine_scene);
    let frame_timing = Rc::clone(frame_timing);
    let closure = Closure::once(move || {
        let now_ms = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);
        let mut g = engine_scene.borrow_mut();
        if let Some((eng, scene)) = g.as_mut() {
            let (last_time, fps_ema) = *frame_timing.borrow();
            let (cpu_time_ms, fps) = if last_time > 0.0 {
                let elapsed_ms = now_ms - last_time;
                let fps = if elapsed_ms > 0.0 {
                    (1000.0 / elapsed_ms) as f32
                } else {
                    60.0f32
                };
                let ema = fps_ema * FPS_EMA_ALPHA + fps * (1.0 - FPS_EMA_ALPHA);
                (elapsed_ms as f32, ema)
            } else {
                (0.0, 60.0f32)
            };
            *frame_timing.borrow_mut() = (now_ms, fps);

            eng.set_last_frame_stats(FrameStats {
                fps,
                cpu_time_ms,
                gpu_time_ms: None,
                element_count: 0,
            });
            let viewport_w = eng.viewport_width() as f32;
            let stats_clone = eng.last_frame_stats().cloned();
            if let Some(ui) = eng.ui.as_mut() {
                ui.update_springs(DT);
                if let Some(stats) = stats_clone {
                    ui.windows_mut()
                        .retain(|w| w.id != STATS_WINDOW_ID && w.id != SPAWN_WINDOW_ID);
                    ui.add_window(build_stats_panel(&stats, viewport_w));
                    ui.add_window(build_spawn_panel(viewport_w));
                }
            }

            if eng.take_clicked_control() == Some(SPAWN_BUTTON_ID) {
                let root = eng.world().root_entity();
                let _ = spawn_body(scene, eng.world_mut(), root, [0.0, 2.0, 0.0]);
            }

            step_physics(scene, eng.world_mut(), DT);
            let _ = eng.render_frame();
        }
        schedule_frame(&engine_scene, &frame_timing);
    });
    web_sys::window()
        .and_then(|w| {
            w.request_animation_frame(closure.as_ref().unchecked_ref())
                .ok()
        })
        .expect("request_animation_frame failed");
    closure.forget();
}
