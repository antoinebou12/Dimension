//! Screenshot integration test: render a frame and capture pixels.

#![cfg(not(target_arch = "wasm32"))]

#[cfg(target_os = "linux")]
use winit::platform::x11::EventLoopBuilderExtX11;

use render::{Engine, Primitive, Primitive3D};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::Window;
use winit::window::WindowId;

struct ScreenshotTestApp {
    result: Option<Result<(), String>>,
}

impl ApplicationHandler for ScreenshotTestApp {
    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.result.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("screenshot-test")
            .with_inner_size(winit::dpi::PhysicalSize::new(64, 64))
            .with_visible(false);
        let window = match event_loop.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                self.result = Some(Err(format!("create window: {e}")));
                event_loop.exit();
                return;
            }
        };
        let engine = match pollster::block_on(Engine::new(Arc::clone(&window))) {
            Ok(e) => e,
            Err(e) => {
                self.result = Some(Err(format!("engine init: {e}")));
                event_loop.exit();
                return;
            }
        };
        let mut engine = engine;
        let root = engine.world().root_entity();
        let e = engine.world_mut().spawn(root);
        engine
            .world_mut()
            .set_primitive(e, Primitive::ThreeD(Primitive3D::Cube));
        if engine.render_frame().is_err() {
            self.result = Some(Err("render_frame failed".into()));
            event_loop.exit();
            return;
        }
        let mut pixels = Vec::new();
        engine.snap(&mut pixels);
        self.result = Some(Ok(()));
        if !(pixels.len() >= 3 && pixels.len() % 3 == 0 && pixels.iter().any(|&b| b != 0)) {
            self.result = Some(Err(format!(
                "snap failed: len={}, mod3={}, any_nonzero={}",
                pixels.len(),
                pixels.len() % 3,
                pixels.iter().any(|&b| b != 0)
            )));
        }
        event_loop.exit();
    }
}

/// Requires a display (DISPLAY or WAYLAND_DISPLAY). Run with `cargo test --test screenshot -- --ignored`.
#[test]
#[ignore = "requires display (DISPLAY or WAYLAND_DISPLAY); run with --ignored when display available"]
fn screenshot_snap_after_render() {
    let event_loop = EventLoop::builder()
        .with_any_thread(true)
        .build()
        .expect("event loop");
    let mut app = ScreenshotTestApp { result: None };
    let _ = event_loop.run_app(&mut app);
    match app.result {
        Some(Ok(())) => {}
        Some(Err(e)) => panic!("screenshot test failed: {e}"),
        None => panic!("screenshot test did not run (resumed not called)"),
    }
}
