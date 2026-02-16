//! Native platform: winit, physics scene, orbit + pan + zoom, UI and FPS overlay.

use crate::scene::{
    build_bodies_tree_panel, build_physics_scene, build_spawn_panel, reset_scene, spawn_box_body,
    spawn_jelly, spawn_sphere, step_physics, PhysicsScene, BODIES_TREE_WINDOW_ID, RESET_BUTTON_ID,
    SPAWN_BOX_ID, SPAWN_JELLY_ID, SPAWN_SPHERE_ID, SPAWN_WINDOW_ID,
};
use render::backend::Projection;
use render::input_constants::{ORBIT_SENSITIVITY, PAN_SENSITIVITY, ZOOM_SENSITIVITY};
use render::{build_stats_panel, Engine, FrameStats, STATS_WINDOW_ID};
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
use winit::keyboard::{Key, ModifiersKeyState, NamedKey};
use winit::window::Window;

static FORTE_POOL: forte::ThreadPool = forte::ThreadPool::new();

const FPS_EMA_ALPHA: f32 = 0.9;
const DT: f32 = 1.0 / 60.0;

fn control_held(mods: &Modifiers) -> bool {
    mods.lcontrol_state() == ModifiersKeyState::Pressed
        || mods.rcontrol_state() == ModifiersKeyState::Pressed
}

struct PhysicsApp {
    window: Option<Arc<Window>>,
    engine: Option<Engine>,
    scene: Option<PhysicsScene>,
    orbit_pressing: bool,
    last_cursor: Option<(f64, f64)>,
    modifiers: Modifiers,
    close_requested: bool,
    last_frame_time: Option<Instant>,
    fps_ema: f32,
}

impl Default for PhysicsApp {
    fn default() -> Self {
        Self {
            window: None,
            engine: None,
            scene: None,
            orbit_pressing: false,
            last_cursor: None,
            modifiers: Modifiers::default(),
            close_requested: false,
            last_frame_time: None,
            fps_ema: 60.0,
        }
    }
}

impl ApplicationHandler for PhysicsApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("physics-demo — XPBD Rigid Bodies")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600));
        let window = Arc::new(event_loop.create_window(attrs).expect("create window"));
        FORTE_POOL.populate();
        let mut engine = FORTE_POOL
            .block_on(Engine::new(Arc::clone(&window)))
            .expect("init engine");
        engine.camera_mut().projection = Projection::Perspective {
            fov_y_rad: std::f32::consts::FRAC_PI_4,
        };

        let scene = build_physics_scene(engine.world_mut());

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
                    element_count: 0,
                };
                let viewport_width = window.inner_size().width as f32;
                let stats_window = build_stats_panel(&stats, viewport_width);
                engine.set_last_frame_stats(stats);
                if let Some(ui) = engine.ui.as_mut() {
                    ui.update_springs(DT);
                    ui.windows_mut().retain(|w| {
                        w.id != STATS_WINDOW_ID
                            && w.id != SPAWN_WINDOW_ID
                            && w.id != BODIES_TREE_WINDOW_ID
                    });
                    ui.add_window(stats_window);
                    ui.add_window(build_bodies_tree_panel(scene, viewport_width));
                    ui.add_window(build_spawn_panel(scene, viewport_width));
                }

                // Handle button clicks
                if let Some(clicked) = engine.take_clicked_control() {
                    let root = engine.world().root_entity();
                    if clicked == SPAWN_SPHERE_ID {
                        let _ = spawn_sphere(scene, engine.world_mut(), root, [0.0, 2.5, 0.0]);
                    } else if clicked == SPAWN_BOX_ID {
                        let _ = spawn_box_body(scene, engine.world_mut(), root, [0.0, 2.5, 0.0]);
                    } else if clicked == SPAWN_JELLY_ID {
                        let _ = spawn_jelly(scene, engine.world_mut(), root, [0.0, 2.5, 0.0]);
                    } else if clicked == RESET_BUTTON_ID {
                        reset_scene(scene, engine.world_mut());
                    }
                }

                step_physics(scene, engine.world_mut(), DT);
                let _ = engine.render_frame();
                window.request_redraw();
            }
            WindowEvent::Resized(size) => {
                engine.resize(size.width, size.height);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    match event.logical_key {
                        Key::Character(ref c) if c.as_str() == "s" || c.as_str() == "S" => {
                            let root = engine.world().root_entity();
                            let _ = spawn_sphere(scene, engine.world_mut(), root, [0.0, 2.5, 0.0]);
                        }
                        Key::Character(ref c) if c.as_str() == "b" || c.as_str() == "B" => {
                            let root = engine.world().root_entity();
                            let _ =
                                spawn_box_body(scene, engine.world_mut(), root, [0.0, 2.5, 0.0]);
                        }
                        Key::Character(ref c) if c.as_str() == "j" || c.as_str() == "J" => {
                            let root = engine.world().root_entity();
                            let _ = spawn_jelly(scene, engine.world_mut(), root, [0.0, 2.5, 0.0]);
                        }
                        Key::Character(ref c) if c.as_str() == "r" || c.as_str() == "R" => {
                            reset_scene(scene, engine.world_mut());
                        }
                        Key::Named(NamedKey::Escape) => {
                            self.close_requested = true;
                        }
                        _ => {}
                    }
                }
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
            WindowEvent::CursorLeft { .. } => {
                self.orbit_pressing = false;
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
                }
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
                engine.zoom(-scroll);
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
    let mut app = PhysicsApp::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}
