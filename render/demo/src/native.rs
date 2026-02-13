//! Native platform: winit + forte (thread pool for async init).

use crate::aabb2d::update_aabb2d_demo;
use render::input_constants::{FPS_EMA_ALPHA, ORBIT_SENSITIVITY, ZOOM_SENSITIVITY};
use render::{
    apply_scene_action, build_demo_scene, build_material_panel, build_scene_panel,
    build_stats_panel, pick_entity, pick_gizmo_handle, Aabb2dIds, ControlId, Engine, FrameStats,
    RenderError, RunDemo, SceneAction, GIZMO_DEFAULT_SIZE, SCENE_WINDOW_ID, STATS_WINDOW_ID,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::ElementState;
use winit::event::Modifiers;
use winit::event::MouseButton;
use winit::event::MouseScrollDelta;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::EventLoop;
use winit::keyboard::ModifiersKeyState;
use winit::window::Window;

fn shift_held(mods: &Modifiers) -> bool {
    mods.lshift_state() == ModifiersKeyState::Pressed
        || mods.rshift_state() == ModifiersKeyState::Pressed
}

#[cfg(not(feature = "sdl3"))]
static FORTE_POOL: forte::ThreadPool = forte::ThreadPool::new();

/// Movement threshold in pixels below which press->release is treated as a click (pick without Shift).
const PICK_CLICK_THRESHOLD_PX: f64 = 4.0;

struct RenderApp {
    window: Option<Arc<Window>>,
    engine: Option<Engine>,
    scene_mapping: Option<HashMap<ControlId, SceneAction>>,
    orbit_pressing: bool,
    last_cursor: Option<(f64, f64)>,
    cursor_position: Option<(f64, f64)>,
    /// Cursor position at mouse down (for click-vs-drag: pick on click without Shift).
    mouse_down_pos: Option<(f64, f64)>,
    modifiers: Modifiers,
    close_requested: bool,
    last_frame_time: Option<Instant>,
    fps_ema: f32,
    demo: RunDemo,
    /// When demo is Aabb2d, entity IDs and start time for per-frame update.
    aabb2d_ids: Option<Aabb2dIds>,
    aabb2d_start: Option<Instant>,
}

impl Default for RenderApp {
    fn default() -> Self {
        Self {
            window: None,
            engine: None,
            scene_mapping: None,
            orbit_pressing: false,
            last_cursor: None,
            cursor_position: None,
            mouse_down_pos: None,
            modifiers: Modifiers::default(),
            close_requested: false,
            last_frame_time: None,
            fps_ema: 60.0,
            demo: RunDemo::Default,
            aabb2d_ids: None,
            aabb2d_start: None,
        }
    }
}

impl ApplicationHandler for RenderApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("render-demo")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600));
        let window = Arc::new(event_loop.create_window(attrs).expect("create window"));
        #[cfg(not(feature = "sdl3"))]
        FORTE_POOL.populate();
        let engine = {
            #[cfg(not(feature = "sdl3"))]
            {
                FORTE_POOL.block_on(Engine::new(Arc::clone(&window)))
            }
            #[cfg(feature = "sdl3")]
            {
                pollster::block_on(Engine::new(Arc::clone(&window)))
            }
        }
        .expect("init engine");
        let mut engine = engine;
        let _ = engine.enable_ui();
        let aabb2d_ids = build_demo_scene(&mut engine, self.demo);
        let aabb2d_start = if aabb2d_ids.is_some() {
            Some(Instant::now())
        } else {
            None
        };
        let mut scene_mapping = build_scene_panel(&mut engine);
        scene_mapping.extend(build_material_panel(&mut engine));
        engine.set_last_frame_stats(FrameStats {
            fps: 60.0,
            cpu_time_ms: 0.0,
            gpu_time_ms: engine.last_gpu_time_ms(),
            element_count: 0,
        });
        let stats = engine.last_frame_stats().cloned().unwrap_or_default();
        let viewport_width = window.inner_size().width as f32;
        if let Some(ui) = engine.ui.as_mut() {
            ui.add_window(build_stats_panel(&stats, viewport_width));
        }
        self.window = Some(window);
        self.engine = Some(engine);
        self.scene_mapping = Some(scene_mapping);
        self.aabb2d_ids = aabb2d_ids;
        self.aabb2d_start = aabb2d_start;
        self.window.as_ref().unwrap().request_redraw();
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let (window, engine) = match (self.window.as_ref(), self.engine.as_mut()) {
            (Some(w), Some(e)) => (w, e),
            _ => return,
        };

        match event {
            WindowEvent::CloseRequested => self.close_requested = true,
            WindowEvent::RedrawRequested => {
                let stats = engine.last_frame_stats().cloned().unwrap_or_default();
                let viewport_width = window.inner_size().width as f32;
                if let Some(ui) = engine.ui.as_mut() {
                    ui.windows_mut().retain(|w| w.id != STATS_WINDOW_ID);
                    ui.add_window(build_stats_panel(&stats, viewport_width));
                }
                if let Some(last) = self.last_frame_time {
                    let dt = last.elapsed().as_secs_f32().min(0.1);
                    engine.update_ui_springs(dt);
                }
                if let (Some(ref ids), Some(start)) = (&self.aabb2d_ids, self.aabb2d_start) {
                    let elapsed = start.elapsed().as_secs_f32();
                    update_aabb2d_demo(engine, ids, elapsed);
                }
                let t0 = Instant::now();
                if let Err(e) = engine.render_frame() {
                    if matches!(e, RenderError::SurfaceLost) {
                        let size = window.inner_size();
                        engine.resize(size.width, size.height);
                    }
                }
                engine.poll_device();
                let cpu_time_ms = t0.elapsed().as_secs_f32() * 1000.0;
                let now = Instant::now();
                let fps = if let Some(last) = self.last_frame_time {
                    let delta_secs = now.duration_since(last).as_secs_f32();
                    if delta_secs > 0.0 {
                        1.0 / delta_secs
                    } else {
                        self.fps_ema
                    }
                } else {
                    60.0
                };
                self.last_frame_time = Some(now);
                self.fps_ema = FPS_EMA_ALPHA * self.fps_ema + (1.0 - FPS_EMA_ALPHA) * fps;
                engine.set_last_frame_stats(FrameStats {
                    fps: self.fps_ema,
                    cpu_time_ms,
                    gpu_time_ms: engine.last_gpu_time_ms(),
                    element_count: 0,
                });
                window.request_redraw();
            }
            WindowEvent::Resized(size) => {
                engine.resize(size.width, size.height);
                window.request_redraw();
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                if state == ElementState::Pressed {
                    engine.ui_mouse_down();
                    self.mouse_down_pos = self.cursor_position;
                    let mut orbit = !engine.is_cursor_over_ui();
                    if orbit && !engine.gizmo_hidden() {
                        if let Some((x, y)) = self.cursor_position {
                            if let Some(selected) = engine.selected_entity() {
                                if let Some(world_mat) =
                                    engine.world().entity_world_matrix(selected)
                                {
                                    if let Some(axis) = pick_gizmo_handle(
                                        engine.camera(),
                                        &world_mat,
                                        engine.gizmo_mode(),
                                        GIZMO_DEFAULT_SIZE,
                                        x as f32,
                                        y as f32,
                                    ) {
                                        if engine
                                            .gizmo_drag_start(selected, axis, x as f32, y as f32)
                                        {
                                            orbit = false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    self.orbit_pressing = orbit;
                } else {
                    engine.ui_mouse_up();
                    engine.gizmo_drag_end();
                    if let Some(ref mapping) = self.scene_mapping {
                        if let Some(cid) = engine.take_clicked_control() {
                            let was_remove =
                                mapping.get(&cid) == Some(&SceneAction::RemoveSelected);
                            apply_scene_action(engine, mapping, cid);
                            if was_remove {
                                if let Some(ui) = engine.ui.as_mut() {
                                    ui.windows_mut().retain(|w| w.id != SCENE_WINDOW_ID);
                                }
                                let mut new_scene = build_scene_panel(engine);
                                let material: HashMap<_, _> = self
                                    .scene_mapping
                                    .as_ref()
                                    .unwrap()
                                    .iter()
                                    .filter(|(k, _)| k.0 >= render::MATERIAL_WINDOW_ID.0)
                                    .map(|(k, v)| (*k, v.clone()))
                                    .collect();
                                new_scene.extend(material);
                                self.scene_mapping = Some(new_scene);
                            }
                        }
                    }
                    let is_click = self.mouse_down_pos.zip(self.cursor_position).map_or(
                        false,
                        |((dx, dy), (x, y))| {
                            let dist_sq = (x - dx).powi(2) + (y - dy).powi(2);
                            dist_sq <= PICK_CLICK_THRESHOLD_PX * PICK_CLICK_THRESHOLD_PX
                        },
                    );
                    let should_pick =
                        !engine.is_cursor_over_ui() && (is_click || shift_held(&self.modifiers));
                    if should_pick {
                        if let Some((x, y)) = self.cursor_position {
                            if let Some(id) =
                                pick_entity(engine.world(), engine.camera(), x as f32, y as f32)
                            {
                                engine.set_selected_entity(Some(id));
                            }
                        }
                    }
                    self.mouse_down_pos = None;
                }
                if state != ElementState::Pressed {
                    self.orbit_pressing = false;
                }
                if state == ElementState::Pressed {
                    self.last_cursor = None;
                }
                window.request_redraw();
            }
            WindowEvent::CursorLeft { .. } => {
                self.orbit_pressing = false;
                self.last_cursor = None;
                engine.gizmo_drag_end();
                window.request_redraw();
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
                window.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x, position.y);
                self.cursor_position = Some((x, y));
                engine.ui_mouse_move(x as f32, y as f32);
                if engine.is_gizmo_dragging() {
                    engine.gizmo_drag_move(x as f32, y as f32);
                } else if self.orbit_pressing {
                    if let Some((lx, ly)) = self.last_cursor {
                        let dyaw = (x - lx) as f32 * ORBIT_SENSITIVITY;
                        let dpitch = (ly - y) as f32 * ORBIT_SENSITIVITY;
                        engine.orbit(dyaw, dpitch);
                    }
                    self.last_cursor = Some((x, y));
                }
                window.request_redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(_, dy) => dy * 0.5,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 * ZOOM_SENSITIVITY,
                };
                engine.zoom(-scroll);
                window.request_redraw();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
        if self.close_requested {
            event_loop.exit();
        }
    }
}

pub fn run(demo: RunDemo) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let event_loop = EventLoop::new()?;
    let mut app = RenderApp {
        demo,
        ..Default::default()
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
