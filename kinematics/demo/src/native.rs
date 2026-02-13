//! Native platform: winit, kinematics scene, orbit + IK target from right-drag, UI and FPS overlay.

use crate::scene::{
    apply_kinematics_action, build_armature_controls_panel, build_armature_tree_panel,
    build_kinematics_scene, build_scene_entity_panel, camera_view_forward,
    screen_to_plane_at_point, step_ik, sync_armature_to_world, KinematicsAction, KinematicsScene,
    ARMATURE_CONTROLS_WINDOW_ID, ARMATURE_TREE_WINDOW_ID, SCENE_ENTITY_WINDOW_ID,
};
use render::backend::Projection;
use render::input_constants::PAN_SENSITIVITY;
use render::ui::ControlId;
use render::{build_stats_panel, Engine, FrameStats, STATS_WINDOW_ID};
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

static FORTE_POOL: forte::ThreadPool = forte::ThreadPool::new();

const ORBIT_SENSITIVITY: f32 = 0.005;
const ZOOM_SENSITIVITY: f32 = 0.001;
const DEPTH_SCROLL_SENSITIVITY: f32 = 0.05;
const FPS_EMA_ALPHA: f32 = 0.9;

fn control_held(mods: &Modifiers) -> bool {
    mods.lcontrol_state() == ModifiersKeyState::Pressed
        || mods.rcontrol_state() == ModifiersKeyState::Pressed
}

struct KinematicsApp {
    window: Option<Arc<Window>>,
    engine: Option<Engine>,
    scene: Option<KinematicsScene>,
    orbit_pressing: bool,
    ik_target_pressing: bool,
    last_cursor: Option<(f64, f64)>,
    modifiers: Modifiers,
    close_requested: bool,
    last_frame_time: Option<Instant>,
    fps_ema: f32,
    /// Action mapping for UI button clicks.
    action_mapping: HashMap<ControlId, KinematicsAction>,
}

impl Default for KinematicsApp {
    fn default() -> Self {
        Self {
            window: None,
            engine: None,
            scene: None,
            orbit_pressing: false,
            ik_target_pressing: false,
            last_cursor: None,
            modifiers: Modifiers::default(),
            close_requested: false,
            last_frame_time: None,
            fps_ema: 60.0,
            action_mapping: HashMap::new(),
        }
    }
}

impl ApplicationHandler for KinematicsApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("kinematics-demo")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600));
        let window = Arc::new(event_loop.create_window(attrs).expect("create window"));
        FORTE_POOL.populate();
        let mut engine = FORTE_POOL
            .block_on(Engine::new(Arc::clone(&window)))
            .expect("init engine");

        // Use perspective for 3D view and unproject
        engine.camera_mut().projection = Projection::Perspective {
            fov_y_rad: std::f32::consts::FRAC_PI_4,
        };

        let scene = build_kinematics_scene(engine.world_mut());

        let _ = engine.enable_ui();
        if let Some(ui) = engine.ui.as_mut() {
            ui.set_theme(render::Theme::dark());
        }

        self.window = Some(window);
        self.engine = Some(engine);
        self.scene = Some(scene);
        self.window.as_ref().unwrap().request_redraw();
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let (window, engine, scene) = match (
            self.window.as_ref(),
            self.engine.as_mut(),
            self.scene.as_mut(),
        ) {
            (Some(w), Some(e), Some(s)) => (w, e, s),
            _ => return,
        };

        match event {
            WindowEvent::CloseRequested => self.close_requested = true,
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let (cpu_time_ms, _fps) = if let Some(prev) = self.last_frame_time {
                    let elapsed = now.duration_since(prev).as_secs_f32();
                    let cpu_ms = elapsed * 1000.0;
                    let fps = if elapsed > 0.0 { 1.0 / elapsed } else { 60.0 };
                    let ema = self.fps_ema * FPS_EMA_ALPHA + fps * (1.0 - FPS_EMA_ALPHA);
                    self.fps_ema = ema;
                    (cpu_ms, ema)
                } else {
                    (0.0, 60.0)
                };
                self.last_frame_time = Some(now);

                let stats = FrameStats {
                    fps: self.fps_ema,
                    cpu_time_ms,
                    gpu_time_ms: None,
                    element_count: engine.active_world_element_count(),
                };
                let viewport_width = window.inner_size().width as f32;
                let stats_window = build_stats_panel(&stats, viewport_width);
                engine.set_last_frame_stats(stats);

                // Build UI panels
                let (scene_entity_window, scene_entity_mapping) =
                    build_scene_entity_panel(scene, engine.world());
                let armature_tree_window = build_armature_tree_panel(scene, viewport_width);
                let (controls_window, controls_mapping) =
                    build_armature_controls_panel(scene, viewport_width);

                // Merge action mappings
                self.action_mapping.clear();
                self.action_mapping.extend(scene_entity_mapping);
                self.action_mapping.extend(controls_mapping);

                if let Some(ui) = engine.ui.as_mut() {
                    ui.update_springs(0.016);
                    ui.windows_mut().retain(|w| {
                        w.id != STATS_WINDOW_ID
                            && w.id != SCENE_ENTITY_WINDOW_ID
                            && w.id != ARMATURE_TREE_WINDOW_ID
                            && w.id != ARMATURE_CONTROLS_WINDOW_ID
                    });
                    ui.add_window(stats_window);
                    ui.add_window(scene_entity_window);
                    ui.add_window(armature_tree_window);
                    ui.add_window(controls_window);
                }

                // Handle UI button clicks
                if let Some(cid) = engine.take_clicked_control() {
                    apply_kinematics_action(scene, engine, &self.action_mapping, cid);
                }

                if engine.selected_entity() == Some(scene.target_entity) {
                    if let Some(data) = engine.world().get(scene.target_entity) {
                        scene.ik_target = data.transform.position;
                    }
                }
                sync_armature_to_world(scene, engine.world_mut());
                step_ik(scene, engine.world_mut());
                let _ = engine.render_frame();
                window.request_redraw();
            }
            WindowEvent::Resized(size) => {
                engine.resize(size.width, size.height);
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                if state == ElementState::Pressed {
                    engine.ui_mouse_down();
                } else {
                    engine.ui_mouse_up();
                }
                if !engine.is_cursor_over_ui() {
                    self.orbit_pressing = state == ElementState::Pressed;
                }
                self.last_cursor = None;
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Right,
                ..
            } => {
                self.ik_target_pressing = state == ElementState::Pressed;
                self.last_cursor = None;
            }
            WindowEvent::CursorLeft { .. } => {
                self.orbit_pressing = false;
                self.ik_target_pressing = false;
                self.last_cursor = None;
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x, position.y);
                engine.ui_mouse_move(x as f32, y as f32);

                if self.orbit_pressing && !engine.is_cursor_over_ui() {
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
                } else if self.ik_target_pressing && !engine.is_cursor_over_ui() {
                    if let Some(pos) = screen_to_plane_at_point(
                        engine.camera(),
                        x as f32,
                        y as f32,
                        scene.ik_target,
                    ) {
                        scene.ik_target = pos;
                        step_ik(scene, engine.world_mut());
                    }
                    self.last_cursor = Some((x, y));
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(_, dy) => dy * 0.5,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 * ZOOM_SENSITIVITY,
                };
                if self.ik_target_pressing && !engine.is_cursor_over_ui() {
                    let forward = camera_view_forward(engine.camera());
                    let step = scroll * DEPTH_SCROLL_SENSITIVITY;
                    scene.ik_target[0] += forward[0] * step;
                    scene.ik_target[1] += forward[1] * step;
                    scene.ik_target[2] += forward[2] * step;
                    step_ik(scene, engine.world_mut());
                } else {
                    engine.zoom(-scroll);
                }
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

/// Run on native (winit).
pub fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let event_loop = EventLoop::new()?;
    let mut app = KinematicsApp::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}
