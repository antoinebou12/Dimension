//! Native platform: winit + forte (thread pool for async init).

use crate::demo::{
    apply_scene_action, build_demo_scene, build_material_panel, build_scene_panel,
    build_stats_panel, SceneAction, MATERIAL_WINDOW_ID, SCENE_WINDOW_ID, STATS_WINDOW_ID,
};
use crate::engine::{Engine, FrameStats};
use crate::error::RenderError;
use crate::pick_entity;
use crate::platform::input_constants::{
    FPS_EMA_ALPHA, ORBIT_SENSITIVITY, PAN_SENSITIVITY, ZOOM_SENSITIVITY,
};
use crate::platform::RunDemo;
use crate::ui::ControlId;
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

#[cfg(not(feature = "sdl3"))]
static FORTE_POOL: forte::ThreadPool = forte::ThreadPool::new();

fn control_held(mods: &Modifiers) -> bool {
    mods.lcontrol_state() == ModifiersKeyState::Pressed
        || mods.rcontrol_state() == ModifiersKeyState::Pressed
}

fn shift_held(mods: &Modifiers) -> bool {
    mods.lshift_state() == ModifiersKeyState::Pressed
        || mods.rshift_state() == ModifiersKeyState::Pressed
}

/// Movement threshold in pixels below which press->release is treated as a click (pick without Shift).
const PICK_CLICK_THRESHOLD_PX: f64 = 4.0;

struct RenderApp {
    window: Option<Arc<Window>>,
    engine: Option<Engine>,
    /// Mapping from scene UI control id to action (from [`build_scene_panel`]).
    scene_mapping: Option<HashMap<ControlId, crate::SceneAction>>,
    orbit_pressing: bool,
    /// Previous cursor position for orbit delta.
    last_cursor: Option<(f64, f64)>,
    /// Current cursor position (for picking on click).
    cursor_position: Option<(f64, f64)>,
    /// Cursor position at mouse down (for click-vs-drag: pick on click without Shift).
    mouse_down_pos: Option<(f64, f64)>,
    /// Current keyboard modifiers (for Ctrl = camera-only mode).
    modifiers: Modifiers,
    close_requested: bool,
    /// Last frame end time for FPS calculation.
    last_frame_time: Option<Instant>,
    /// Smoothed FPS (exponential moving average).
    fps_ema: f32,
    /// Initial scene preset (used in resumed).
    demo: RunDemo,
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
        }
    }
}

impl ApplicationHandler for RenderApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("render")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600));
        let window = Arc::new(event_loop.create_window(attrs).expect("create window"));
        #[cfg(not(feature = "sdl3"))]
        {
            FORTE_POOL.populate();
        }
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
        let _ = build_demo_scene(&mut engine, self.demo);
        let mut scene_mapping = build_scene_panel(&mut engine);
        scene_mapping.extend(build_material_panel(&mut engine));
        engine.set_last_frame_stats(FrameStats {
            fps: 60.0,
            cpu_time_ms: 0.0,
            gpu_time_ms: engine.last_gpu_time_ms(),
            element_count: engine.active_world_element_count(),
        });
        let stats = engine.last_frame_stats().cloned().unwrap_or_default();
        let viewport_width = window.inner_size().width as f32;
        if let Some(ui) = engine.ui.as_mut() {
            ui.add_window(build_stats_panel(&stats, viewport_width));
        }
        self.window = Some(window);
        self.engine = Some(engine);
        self.scene_mapping = Some(scene_mapping);
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
                engine.set_gizmo_hidden(control_held(&self.modifiers));
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
                    element_count: engine.active_world_element_count(),
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
                } else {
                    engine.ui_mouse_up();
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
                                    .filter(|(k, _)| k.0 >= MATERIAL_WINDOW_ID.0)
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
                    let should_pick = !engine.is_cursor_over_ui()
                        && !control_held(&self.modifiers)
                        && (shift_held(&self.modifiers) || is_click);
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
                self.orbit_pressing = state == ElementState::Pressed && !engine.is_cursor_over_ui();
                if state == ElementState::Pressed {
                    self.last_cursor = None;
                }
                window.request_redraw();
            }
            WindowEvent::CursorLeft { .. } => {
                self.orbit_pressing = false;
                self.last_cursor = None;
                window.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x, position.y);
                self.cursor_position = Some((x, y));
                engine.ui_mouse_move(x as f32, y as f32);
                if self.orbit_pressing && !shift_held(&self.modifiers) {
                    if let Some((lx, ly)) = self.last_cursor {
                        if control_held(&self.modifiers) {
                            let dx = (x - lx) as f32 * PAN_SENSITIVITY;
                            let dy = (y - ly) as f32 * PAN_SENSITIVITY;
                            engine.pan(dx, dy);
                        } else {
                            let dyaw = (x - lx) as f32 * ORBIT_SENSITIVITY;
                            let dpitch = (ly - y) as f32 * ORBIT_SENSITIVITY;
                            engine.orbit(dyaw, dpitch);
                        }
                    }
                    self.last_cursor = Some((x, y));
                } else if self.orbit_pressing {
                    self.last_cursor = Some((x, y));
                }
                window.request_redraw();
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
                window.request_redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(_, dy) => dy * 0.5,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 * ZOOM_SENSITIVITY,
                };
                if engine.is_cursor_over_ui() {
                    engine.ui_scroll(scroll);
                } else {
                    engine.zoom(-scroll);
                }
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

/// Run on native (winit, forte for async init). Called via [`crate::run_demo`] when using default scene.
#[allow(dead_code)]
pub fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    run_with_demo(RunDemo::Default)
}

/// Run with initial scene preset (curves vs default).
pub fn run_with_demo(demo: RunDemo) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let event_loop = EventLoop::new()?;
    let mut app = RenderApp {
        demo,
        ..Default::default()
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
