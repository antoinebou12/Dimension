# render — Architecture and usage

**render** is a 2D/3D rendering engine in the Dimension repo. It uses wgpu for GPU rendering and mathlib for all CPU math (transforms, MVP, world matrix chain, orthographic or perspective camera). WASM-first; platform code is separated for web (canvas, requestAnimationFrame) and native (winit, pollster).

For project context see [AGENTS.md](../AGENTS.md). Quick build/run: [render/README.md](../render/README.md). Public API docs follow the Rust skill doc-* rules; see [AGENTS.md#documentation](../AGENTS.md#documentation).

## Architecture

- **Platform**: Entry point and event loop. `wasm` — canvas element and `request_animation_frame`. `native` — winit window; `forte::block_on` for async (or `pollster::block_on` when `sdl3` feature is enabled). `sdl3` — optional SDL3 platform via `run_sdl3()` (requires SDL3 installed).
- **Scene**: Hierarchical world built on mathlib `Tree`. `World` holds the scene tree; entities have `Transform`, `Primitive`, and `NodeData`. Parent/child structure is exposed via [`World::root_entity`], [`World::children`], [`World::parent`], and [`World::entities_dfs`]. World matrix chain and transforms use mathlib (MVP, orthographic). The same structure can be serialized when the `serde` feature is enabled (see Serialization below).
- **Backend**: wgpu-only. `GpuRenderer` owns pipeline and buffers; `Camera` for view/projection (mathlib orthographic or perspective via `Projection`); `Vertex` and mesh handling; math prep for CPU-side data uploaded to GPU. The scene pass uses **view-frustum culling** (only entities whose world AABB intersects the frustum are drawn) and **GPU instancing** by (mesh, material): one instanced draw call per “island” of visible entities sharing the same primitive and material. The **cull** module provides frustum and AABB types and tests; the **spatial** module provides a BSP tree for spatial queries; **pick** uses BSP for ray traversal and ray–segment for line/curve primitives.
- **UI**: Optional wgpu-rendered 2D layer (`UiLayer`) with semi-transparent windows, buttons, sliders, checkboxes; light/dark theme. Rendered in a second pass with alpha blending after the scene. Platform forwards mouse to the engine for hit-test and interaction.

## Main types

| Type | Role |
|------|------|
| `Engine` | Holds surface, `GpuRenderer`, multiple `World`s with an active index, `Camera`, optional `UiLayer`. Use `world()`, `world_mut()` for the active world; `set_active_world`, `selected_entity` for scene UI. |
| `run` | Platform entry: takes window/canvas and runs the engine loop. |
| `World` | Scene graph (mathlib tree); entities with `Transform`, `Primitive`, `NodeData`. Use `root_entity()`, `children(id)`, `parent(id)`, `entities_dfs()` to traverse. Serializable with the `serde` feature. |
| `GpuRenderer` | wgpu device/queue, pipeline, render pass. |
| `Camera` | View and projection (mathlib). Use `Camera::new` for orthographic or `Camera::new_perspective` for perspective; see `Projection` enum. |
| `UiLayer` | Optional UI layer: windows, buttons, sliders, checkboxes; theme (light/dark); hit-test and input. |
| `Theme` | Light or dark semi-transparent color set for UI. |
| `Transform`, `Primitive` | Scene components. |
| `Transform::rotation_quat` | Optional quaternion rotation; when `Some`, used for the model matrix instead of Euler `rotation` (avoids gimbal lock). Prefer when syncing from kinematics or when rotating via gizmo. |
| `Vertex` | Vertex format for mesh data. |
| `ShaderSources`, `ShaderConfig` | Default embedded WGSL and optional custom shader override. |
| `FrameStats` | Per-frame statistics (FPS, CPU/GPU time, element count). Set by platform after each frame; see [Stats overlay](#stats-overlay). |
| `build_stats_panel`, `STATS_WINDOW_ID` | Build the in-engine Stats window; use with `engine.last_frame_stats()` to show FPS, CPU, GPU, elements. |
| `ViewMode` | How scene geometry is drawn: `Solid`, `Wireframe`, `VertexPoints`, `ColorMap`. Use `engine.set_view_mode()` and `engine.view_mode()`. |
| `GizmoMode` | Gizmo overlay mode: `Translate`, `Rotate`, `Scale`. Use `engine.set_gizmo_mode()`; gizmo is drawn when an entity is selected. |
| `pick_entity` | Ray cast from screen to world; returns `Option<EntityId>`. Perspective only; use with `engine.world()` and `engine.camera()`. |
| `Material`, `MaterialRegistry` | Polyscope-style matcaps. Static (matcap), blendable (R/G/B/K), or UV-diffuse (single texture at vertex UV, bilinear). Use `engine.materials`, `load_static_material`, `load_blendable_material` (native, requires `material` feature). Built-in **procedural** (checkerboard) is UV-mapped. |
| `build_material_panel`, `MATERIAL_WINDOW_ID` | Build the Material window for per-entity material selection. |
| `Framebuffer`, `RenderTarget` | Offscreen render target (FBO) and render destination (surface or FBO). Use [`Engine::create_framebuffer`], [`Engine::render_frame_to`]. |
| `ReadbackGuard`, `ReadbackLayout` | Async pixel readback (PBO-style). Use [`Engine::read_pixels_async`] or [`Engine::read_pixels_async_from`]; call [`Engine::poll_device`] on WASM so the callback runs. |

### Transform

`Transform` has `position`, `rotation` (Euler roll, pitch, yaw in radians), `scale`, and optional `rotation_quat: Option<Quat4f>`. When `rotation_quat` is `Some`, it is used to build the rotation part of the model matrix; otherwise `rotation` is used. Use `rotation_quat` when you have a quaternion (e.g. from kinematics armature sync or after gizmo rotate) to avoid gimbal lock. Construct with `Transform::from_position_quat(position, q)` or set via `set_rotation_quat`.

### Materials

With the **`material`** feature enabled, the engine supports Polyscope-style matcaps:

- **Static materials**: Single texture; matcap using view-space normal for UV (optionally with lighting).
- **Blendable materials**: Four basis textures (R, G, B, K). Entity color drives blending: `output = r*R + g*G + b*B + (1-r-g-b)*K`.
- **UV-diffuse materials**: Single texture sampled at vertex UV with bilinear (e.g. built-in procedural checkerboard).

`NodeData.material` holds the material name (`None` = vertex color mode). Use `world.set_material(id, Some("flat"))`, `Some("procedural")`, or `None` for vertex color. The built-in **flat** and **procedural** (checkerboard, UV-mapped) materials are registered when the feature is on. Load custom materials with `engine.load_static_material(name, path)` or `engine.load_blendable_material(name, paths)` (native only). Use [`build_material_panel`] for the top-level Material window.

### Gizmo

When an entity is selected (`engine.selected_entity()`), a 3D transform gizmo is drawn at that entity’s world position. Use `engine.set_gizmo_mode(GizmoMode::Translate | Rotate | Scale)` to switch handles. The gizmo is rendered as an overlay after the scene pass. Mesh generation is in `gizmo::gizmo_mesh`; axis colors are `GIZMO_X_COLOR`, `GIZMO_Y_COLOR`, `GIZMO_Z_COLOR`.

### Picking

`pick_entity(world, camera, screen_x, screen_y)` casts a ray from the given pixel (in viewport coordinates) and returns the first hit entity, if any. Requires perspective projection (`Camera::perspective_params()`); returns `None` for orthographic. Picking uses a BSP tree over world AABBs for candidate entities, then ray–triangle (Möller–Trumbore) for mesh primitives and ray–segment for line and curve primitives (LineSegment, Bézier, Hermite, B-spline). The native platform uses picking on left-click (when not over UI) to set the selected entity.

### View modes

`Engine::view_mode()` and `set_view_mode(ViewMode)` control how scene geometry is drawn:

- **Solid** (default): Filled triangles.
- **Wireframe**: Polygon edges (same vertex data; line pipeline). Not all wgpu backends support line mode.
- **VertexPoints**: Vertices as points (point-list pipeline).
- **ColorMap**: Scalar-to-color mapping via a 1D viridis colormap. Uses `vertex.color.r` as the scalar (0–1); applications set R when building meshes for ColorMap mode. Includes tone mapping (exposure 1.0, gamma 2.2) for HDR-friendly output.

### Spring UI

Sliders support optional spring animation: `value` moves toward `target_value` each frame. Call `engine.update_ui_springs(dt)` from the platform each frame (e.g. before `render_frame`). Use `Slider::set_target_value()` to animate to a new value. Constants `SLIDER_SPRING_STIFFNESS` and `SLIDER_SPRING_DAMPING` tune the motion.

### Scrollable windows and layout

Set [`Window::content_height`] larger than the window body height to make a window scrollable. [`Window::scroll_y`] is the current offset (clamped to 0..=max); the platform calls `engine.ui_scroll(delta)` on mouse wheel when the cursor is over UI, so the topmost scrollable window under the cursor scrolls. Use [`VerticalLayout`] or [`vertical_stack`] to lay out rows (e.g. dynamic lists of sliders) without hand-calculating positions. To repopulate a window (e.g. after switching content), find it by id with `ui.windows_mut().iter_mut().find(|w| w.id == id)` then call [`Window::clear_children`] and add new controls. Use [`Engine::reset_camera`] to restore default orbit (e.g. wire a "Reset" button via [`SceneAction::ResetCamera`] in the demo).

### Shaders

Shaders live in `render/shaders/`:

- **scene.wgsl** — Scene pass (MVP transform, vertex color). Used for all scene primitives (2D and 3D). Entry points: `vs_main`, `fs_main`. Vertex layout: buffer 0 — position, uv, color; buffer 1 (instance) — mvp, model_view, material_mode, entity_color, selected. Per-object data is supplied via the instance-rate vertex buffer only (no per-draw object uniform). Material modes 0–3 (vertex color, matcap, blendable, UV-diffuse). Supports ViewMode::ColorMap (colormap lookup at `vertex.color.r`) and tone mapping (exposure, gamma).
- **ui.wgsl** — UI pass (screen-space ortho, vertex color with alpha). The same shader is used for all UI quads: window panel, title bar, button, slider track, slider thumb, checkbox, and optional label background. Label **text** is rendered separately via wgpu_text (glyph atlas); only the optional label background quad uses this shader. Entry points: `vs_main`, `fs_main`.
- **compute_example.wgsl** — Template compute shader. Entry point: `cs_main`.

Use [`ShaderSources`] for default embedded WGSL: `ShaderSources::scene_vertex_fragment()`, `ShaderSources::ui_vertex_fragment()`, `ShaderSources::compute_example()`. Use [`ShaderConfig`] to override with custom WGSL (e.g. loaded from disk on native). Pass `Some(&config)` to `GpuRenderer::new` and `UiRenderPass::new` (via `UiLayer::new`); pass `None` for defaults.

### Primitives (2D and 3D)

Primitives are split at the type level: use `Primitive::TwoD(Primitive2D::…)` for 2D shapes (z = 0) and `Primitive::ThreeD(Primitive3D::…)` for 3D.

- **Primitive2D**: `Quad` (fullscreen), `Square`, `Circle`, `Ellipse`, `Triangle`.
- **Primitive3D**: `Quad`, `Triangle`, `Cube`, `Tetrahedron`, `Cylinder`, `Sphere`, `Cone`, `Capsule`; and line/curve: `LineSegment`, `Bezier`, `Hermite`, `BSpline`.

### Line and curve primitives

Line and curve primitives are drawn as **line lists** (not triangles). Control points use [`CurvePoint`] (a wrapper over `[f32; 3]` with `Eq`/`Hash` for mesh caching).

- **LineSegment** — `start` and `end`; two vertices.
- **Bezier** — cubic Bézier with 4 control points; sampled at 32 segments by default.
- **Hermite** — cubic Hermite with 2 points and 2 tangent vectors.
- **BSpline** — cubic B-spline segment with 4 control points.

Curve evaluation uses mathlib’s `math::curve` (`linear_curve`, `bezier_curve`, `hermite_curve`, `bspline_curve`). The transform gizmo moves the whole entity; control points are in local space. Example: `cargo run -p render --example curves_gizmo` (uses `run_demo(RunDemo::Curves)` to build a curve scene).

Mesh generation (in `backend::mesh`) uses mathlib for all geometry: `vector3`, `sin_scalar`/`cos_scalar` for circles, ellipses, and cylinders; `math::curve` for line/curve primitives. When the `simd` feature is enabled, batch color fill uses mathlib’s SIMD (`add_f32`, `set_zero_f32`) for circle, ellipse, and cylinder vertices.

### UI (optional)

The `ui` module provides a GPU-rendered 2D UI layer with semi-transparent panels and controls:

- **UiLayer**: Create with `UiLayer::new(device, format, width, height)`, or call `engine.enable_ui()` after creating the engine. Holds a list of `Window`s, each with children: `Button`, `Slider`, `Checkbox`. Uses `Theme::light()` or `Theme::dark()` (semi-transparent RGBA). Rendered in a second pass with alpha blending over the scene. On WASM the same wgpu UI works; font is embedded; use build_*_panel in the demo for in-canvas panels.
- **Theme**: `Theme::light()` and `Theme::dark()`; use `ui_layer.set_theme(Theme::dark())` to switch.
- **Components**: `Window` (rect, optional title bar, children), `Button`, `Slider` (value in [0, 1]), `Checkbox` (checked state), `Label` (non-interactive text line for stats). Layout is manual (set `Rect` on each control). Add controls with `window.add_button(…)`, `add_slider`, `add_checkbox`, `add_label`; add windows with `ui_layer.add_window(window)`.
- **Input**: The platform layer forwards cursor and mouse buttons to the engine. Call `engine.ui_mouse_move(x, y)`, `engine.ui_mouse_down()`, `engine.ui_mouse_up()` from platform (already wired in native and WASM). When the cursor is over UI, orbit is not started (`engine.is_cursor_over_ui()`). Slider value updates on drag; checkbox toggles on click. Use `engine.take_clicked_control()` after mouse up to react to button clicks (e.g. scene panel: [`build_scene_panel`], [`apply_scene_action`]).

### Stats overlay

Per-frame statistics (FPS, CPU time, optional GPU time, element count) are collected by the platform and stored on the engine via `engine.set_last_frame_stats(stats)`; the engine fills `element_count` from the active world.

- **WASM**: The default demo ([render/demo/wasm-demo/index.html](../render/demo/wasm-demo/index.html)) shows a stats overlay in the top-right (HTML div) updated each frame from Rust. No in-engine text required.
- **Native**: The default run shows an in-engine **Stats** window (built with [`build_stats_panel`]) with FPS, CPU ms, GPU ms, and element count. The platform replaces the stats window each frame using `STATS_WINDOW_ID` and `engine.last_frame_stats()`. GPU time is only available when the backend supports timestamp queries; the platform must call `engine.poll_device()` after each `render_frame()` so that timestamp readback callbacks run and [`engine.last_gpu_time_ms()`] can update.
- **API**: [`FrameStats`], `engine.last_frame_stats()`, `engine.set_last_frame_stats()`, [`build_stats_panel`], `STATS_WINDOW_ID`.

### Screenshot

Capture the current framebuffer as raw RGB pixels or as an image:

- **`engine.snap(out)`** — Writes full-screen RGB data (3 bytes per pixel) into the provided `Vec<u8>`. Rows are ordered from bottom to top.
- **`engine.snap_rect(out, x, y, width, height)`** — Captures a rectangular region. Call [`render_frame`] at least once before snapping.
- **`engine.snap_image()`** — Returns an `image::ImageBuffer<Rgb<u8>, Vec<u8>>` (requires the `screenshot` feature). Use `image.save("screenshot.png")` to write to disk.

### Offscreen rendering (FBO) and pixel readback (async)

- **FBO**: Create an offscreen framebuffer with [`Engine::create_framebuffer(width, height)`]. FBOs are fixed at creation; create a new one and drop the old when you need a different size (e.g. after window resize).
- **Render to FBO**: Call [`Engine::render_frame_to(target)`] to render the same scene (active world, points overlay, optional UI) to an offscreen target. Nothing is presented to the surface.
- **Async readback (PBO-style)**: Use [`Engine::read_pixels_async(x, y, width, height, callback)`] to read from the internal readback texture (last frame rendered to the surface). Use [`Engine::read_pixels_async_from(framebuffer, x, y, width, height, callback)`] to read from an FBO (e.g. after `render_frame_to`). The callback receives `Result<(), RenderError>` and an optional [`ReadbackGuard`]: when `Ok`, use `guard.get_mapped_range()` to read raw RGBA bytes (see [`ReadbackLayout`] for row stride and format), then drop the guard to unmap and return the buffer to the pool. Staging buffers are reused to avoid per-call allocation.
- **WASM**: On WebAssembly, use `read_pixels_async` (or `read_pixels_async_from`) and call [`Engine::poll_device`] after each frame so the callback runs; do not block the main thread.

### Serialization

With the **`serde`** feature enabled, scene and world can be serialized to a **compact, shareable binary format** (bincode). The full tree structure (parent/child) and all per-entity data (transform, primitive, color) are preserved.

- **Types**: `World`, `NodeData`, `Transform`, `Primitive`, `EntityId` implement `Serialize`/`Deserialize` when the feature is on (mathlib `Tree` is used and must be built with `mathlib/serde`).
- **API**: [`world_to_bytes`] and [`world_from_bytes`] in the `serialization` module (re-exported from the crate root). Use them to save/load layouts or exchange scenes.
- **Example**: Use [`world_to_bytes`] and [`world_from_bytes`] to serialize a world, then run the render loop via `render_demo::run_native()`.
- **Tests**: `cargo test -p render --test serialization --features serde` runs the round-trip test (tree structure and node data).

Run `cd render && cargo doc --open` for full API docs.

## Build and run

From repo root (see [justfile](../justfile)):

- **Native**: `just build-render`, `just run-render` (or `cargo run -p render-demo --example render_native` from repo root). Run `cargo run -p render-demo --example curves_native` for the curves demo.
- **SDL3** (optional): `cargo run -p render-demo --example sdl3_quad --features sdl3`. Requires SDL3 installed (e.g. vcpkg: `vcpkg install sdl3:x64-windows`; set `SDL3_DIR` or `VCPKG_ROOT`).
- **WASM**: `just render-wasm` (build + serve). Open http://localhost:3000/wasm-demo/. Requires **wasm-bindgen** on PATH (e.g. `cargo install wasm-bindgen-cli`) and **Rust (stable, see rust-toolchain.toml)**; on **Linux/WSL**, a C compiler (**clang** or **build-essential**) may be required. **wasm-opt** (binaryen) is optional.

See [render/README.md](../render/README.md) and [render/demo/README.md](../render/demo/README.md) for exact commands and examples.

## Conventions

Same as mathlib: run `cargo fmt` and `cargo clippy` in the render crate; add or update tests when changing behavior; document public API per the Rust doc conventions in [AGENTS.md](../AGENTS.md#documentation).

Same as mathlib: run `cargo fmt` and `cargo clippy` in the render crate; add or update tests when changing behavior; document public API per the Rust doc conventions in [AGENTS.md](../AGENTS.md#documentation).
