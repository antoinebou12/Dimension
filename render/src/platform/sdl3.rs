//! SDL3 platform: optional alternative to winit for native demos.
//!
//! Requires the `sdl3` feature. Uses pollster for async Engine init (SDL3 Window is not Send).

use crate::demo::build_demo_scene;
use crate::engine::Engine;
use crate::platform::input_constants::{ORBIT_SENSITIVITY, PAN_SENSITIVITY, ZOOM_SENSITIVITY};
use crate::platform::RunDemo;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::video::WindowBuilder;

/// Run the render loop with SDL3.
///
/// Creates an 800x600 window, initializes wgpu, and runs the event loop with orbit/zoom
/// and optional UI interaction. Requires the `sdl3` feature.
///
/// # Errors
/// Returns error if SDL init, window creation, or engine init fails.
pub fn run_sdl3() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let sdl = sdl3::init()?;
    let video = sdl.video()?;

    let window = WindowBuilder::new(&video, "render (SDL3)", 800, 600)
        .build()
        .map_err(|e| format!("SDL3 window creation: {e}"))?;

    let mut engine =
        pollster::block_on(Engine::new_sdl3(window)).map_err(|e| format!("Engine init: {e}"))?;

    let _ = engine.enable_ui();
    build_demo_scene(&mut engine, RunDemo::Default);

    let mut event_pump = sdl.event_pump()?;
    let mut orbit_pressing = false;
    let mut last_cursor: Option<(f32, f32)> = None;
    let mut ctrl_held = false;
    let mut shift_held = false;
    let mut running = true;

    while running {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => running = false,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => running = false,
                Event::KeyDown {
                    keycode: Some(Keycode::LCtrl | Keycode::RCtrl),
                    ..
                } => ctrl_held = true,
                Event::KeyUp {
                    keycode: Some(Keycode::LCtrl | Keycode::RCtrl),
                    ..
                } => ctrl_held = false,
                Event::KeyDown {
                    keycode: Some(Keycode::LShift | Keycode::RShift),
                    ..
                } => shift_held = true,
                Event::KeyUp {
                    keycode: Some(Keycode::LShift | Keycode::RShift),
                    ..
                } => shift_held = false,
                Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => {
                    if mouse_btn == sdl3::mouse::MouseButton::Left {
                        engine.ui_mouse_down();
                        orbit_pressing = !engine.is_cursor_over_ui();
                        last_cursor = Some((x as f32, y as f32));
                    }
                }
                Event::MouseButtonUp { mouse_btn, .. } => {
                    if mouse_btn == sdl3::mouse::MouseButton::Left {
                        engine.ui_mouse_up();
                        orbit_pressing = false;
                        last_cursor = None;
                    }
                }
                Event::MouseMotion { x, y, .. } => {
                    let (x, y) = (x as f32, y as f32);
                    engine.ui_mouse_move(x, y);
                    if orbit_pressing {
                        if !shift_held {
                            if let Some((lx, ly)) = last_cursor {
                                if ctrl_held {
                                    let dx = (x - lx) * PAN_SENSITIVITY;
                                    let dy = (y - ly) * PAN_SENSITIVITY;
                                    engine.pan(dx, dy);
                                } else {
                                    let dyaw = (x - lx) * ORBIT_SENSITIVITY;
                                    let dpitch = (ly - y) * ORBIT_SENSITIVITY;
                                    engine.orbit(dyaw, dpitch);
                                }
                            }
                        }
                        last_cursor = Some((x, y));
                    }
                }
                Event::MouseWheel { y, .. } => {
                    engine.zoom(-y as f32 * ZOOM_SENSITIVITY);
                }
                Event::Window { win_event, .. } => {
                    use sdl3::event::WindowEvent;
                    match win_event {
                        WindowEvent::Resized(w, h) | WindowEvent::PixelSizeChanged(w, h) => {
                            engine.resize(w.max(0) as u32, h.max(0) as u32);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        engine.set_gizmo_hidden(ctrl_held);
        if let Err(e) = engine.render_frame() {
            let _ = e; // SurfaceLost would need window size; WindowEvent::Resized handles normal resize
        }
    }

    Ok(())
}
