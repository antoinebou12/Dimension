//! Native platform: winit, geometry scene, orbit + zoom.

use crate::scene::build_geometry_scene;
use render::backend::Projection;
use render::{build_stats_panel, Engine, FrameStats, STATS_WINDOW_ID};
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::ElementState;
use winit::event::MouseButton;
use winit::event::MouseScrollDelta;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::EventLoop;
use winit::window::Window;

static FORTE_POOL: forte::ThreadPool = forte::ThreadPool::new();

const ORBIT_SENSITIVITY: f32 = 0.005;
const ZOOM_SENSITIVITY: f32 = 0.001;
const FPS_EMA_ALPHA: f32 = 0.9;

struct GeometryApp {
    window: Option<Arc<Window>>,
    engine: Option<Engine>,
    orbit_pressing: bool,
    last_cursor: Option<(f64, f64)>,
    close_requested: bool,
    last_frame_time: Option<Instant>,
    fps_ema: f32,
}

impl Default for GeometryApp {
    fn default() -> Self {
        Self {
            window: None,
            engine: None,
            orbit_pressing: false,
            last_cursor: None,
            close_requested: false,
            last_frame_time: None,
            fps_ema: 60.0,
        }
    }
}

impl ApplicationHandler for GeometryApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("geometry-demo")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600));
        let window = Arc::new(event_loop.create_window(attrs).expect("create window"));
        FORTE_POOL.populate();
        let mut engine = FORTE_POOL
            .block_on(Engine::new(Arc::clone(&window)))
            .expect("init engine");
        engine.camera_mut().projection = Projection::Perspective {
            fov_y_rad: std::f32::consts::FRAC_PI_4,
        };
        let _ = build_geometry_scene(engine.world_mut());
        let _ = engine.enable_ui();
        if let Some(ui) = engine.ui.as_mut() {
            ui.set_theme(render::Theme::dark());
        }
        self.window = Some(window);
        self.engine = Some(engine);
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
                    ui.update_springs(1.0 / 60.0);
                    ui.windows_mut().retain(|w| w.id != STATS_WINDOW_ID);
                    ui.add_window(stats_window);
                }
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
                self.orbit_pressing = state == ElementState::Pressed;
                self.last_cursor = None;
            }
            WindowEvent::CursorLeft { .. } => {
                self.orbit_pressing = false;
                self.last_cursor = None;
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x, position.y);
                if self.orbit_pressing {
                    if let Some((lx, ly)) = self.last_cursor {
                        let dyaw = (x - lx) as f32 * ORBIT_SENSITIVITY;
                        let dpitch = (ly - y) as f32 * ORBIT_SENSITIVITY;
                        engine.orbit(dyaw, dpitch);
                    }
                    self.last_cursor = Some((x, y));
                }
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
    let mut app = GeometryApp::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}
