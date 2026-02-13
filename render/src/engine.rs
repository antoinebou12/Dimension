//! Engine: window/canvas, wgpu, world, camera, optional UI layer.

use crate::backend::{Camera, Framebuffer, GpuRenderer, ReadbackGuard, RenderTarget};
use crate::error::RenderError;
use crate::gizmo::{GizmoAxis, GizmoMode};
use crate::grid::GridCubeDescriptor;
use crate::material::MaterialRegistry;
use crate::pick::screen_ray_to_world;
use crate::scene::{EntityId, World};
use crate::slice_plane::SlicePlane;
use crate::ui::UiLayer;
use crate::view_mode::ViewMode;
use mathlib::math3d::matrix4f_inverse;
use wgpu::Backends;
use wgpu::InstanceDescriptor;
use wgpu::PollType;
use wgpu::RequestAdapterOptions;

#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

#[cfg(not(target_arch = "wasm32"))]
use {std::sync::Arc, winit::window::Window};

/// State for an active gizmo drag (translate, rotate, or scale along one axis).
#[derive(Clone, Debug)]
pub struct GizmoDragState {
    pub entity_id: EntityId,
    pub axis: GizmoAxis,
    pub mode: GizmoMode,
    pub start_screen: (f32, f32),
    pub start_world_pos: [f32; 3],
    /// Ray-plane hit on the view plane at drag start (for rotate angle reference).
    pub start_plane_hit: [f32; 3],
    /// World-space axis direction (normalized) at drag start.
    pub axis_direction: [f32; 3],
    pub start_rotation: [f32; 3],
    pub start_scale: [f32; 3],
    /// For rotate: entity world rotation at drag start (so we apply delta from start to current).
    pub start_world_rot_quat: mathlib::Quat4f,
    /// For rotate: parent's inverse rotation (world to local).
    pub parent_inv_rot_quat: mathlib::Quat4f,
}

/// World-space view direction (from camera toward scene). Normalized.
fn view_forward_world(camera: &Camera) -> [f32; 3] {
    let view = camera.view_matrix();
    let view_inv = matrix4f_inverse(&view);
    let x = -view_inv.get(0, 2);
    let y = -view_inv.get(1, 2);
    let z = -view_inv.get(2, 2);
    let len = (x * x + y * y + z * z).sqrt().max(1e-9);
    [x / len, y / len, z / len]
}

/// Ray-plane intersection. Returns t such that origin + t*dir lies on the plane. Returns None if parallel.
fn ray_plane_intersection(
    origin: &[f32; 3],
    dir: &[f32; 3],
    plane_pt: &[f32; 3],
    plane_normal: &[f32; 3],
) -> Option<f32> {
    let denom = plane_normal[0] * dir[0] + plane_normal[1] * dir[1] + plane_normal[2] * dir[2];
    if denom.abs() < 1e-9 {
        return None;
    }
    let diff = [
        plane_pt[0] - origin[0],
        plane_pt[1] - origin[1],
        plane_pt[2] - origin[2],
    ];
    let t =
        (diff[0] * plane_normal[0] + diff[1] * plane_normal[1] + diff[2] * plane_normal[2]) / denom;
    Some(t)
}

/// Rotation part of world matrix column for the given axis (X=0, Y=1, Z=2). Normalized.
fn world_axis_direction(world_mat: &mathlib::math3d::Matrix4f, axis: GizmoAxis) -> [f32; 3] {
    let col = match axis {
        GizmoAxis::X => 0,
        GizmoAxis::Y => 1,
        GizmoAxis::Z => 2,
    };
    let x = world_mat.get(0, col);
    let y = world_mat.get(1, col);
    let z = world_mat.get(2, col);
    let len = (x * x + y * y + z * z).sqrt().max(1e-9);
    [x / len, y / len, z / len]
}

/// Project point onto plane through origin with given normal. Returns point on plane.
fn project_onto_plane(point: &[f32; 3], origin: &[f32; 3], normal: &[f32; 3]) -> [f32; 3] {
    let dx = point[0] - origin[0];
    let dy = point[1] - origin[1];
    let dz = point[2] - origin[2];
    let d = dx * normal[0] + dy * normal[1] + dz * normal[2];
    [
        point[0] - d * normal[0],
        point[1] - d * normal[1],
        point[2] - d * normal[2],
    ]
}

/// Angle in radians from start to current in the ring plane (around axis). Returns None if vectors too short.
fn gizmo_rotate_delta_angle(
    origin: &[f32; 3],
    start_plane: &[f32; 3],
    current_plane: &[f32; 3],
    axis: &[f32; 3],
) -> Option<f32> {
    let start_proj = project_onto_plane(start_plane, origin, axis);
    let current_proj = project_onto_plane(current_plane, origin, axis);
    let start_vec = [
        start_proj[0] - origin[0],
        start_proj[1] - origin[1],
        start_proj[2] - origin[2],
    ];
    let current_vec = [
        current_proj[0] - origin[0],
        current_proj[1] - origin[1],
        current_proj[2] - origin[2],
    ];
    let start_len =
        (start_vec[0] * start_vec[0] + start_vec[1] * start_vec[1] + start_vec[2] * start_vec[2])
            .sqrt();
    let current_len = (current_vec[0] * current_vec[0]
        + current_vec[1] * current_vec[1]
        + current_vec[2] * current_vec[2])
        .sqrt();
    if start_len < 1e-6 || current_len < 1e-6 {
        return None;
    }
    let cross = [
        start_vec[1] * current_vec[2] - start_vec[2] * current_vec[1],
        start_vec[2] * current_vec[0] - start_vec[0] * current_vec[2],
        start_vec[0] * current_vec[1] - start_vec[1] * current_vec[0],
    ];
    let cross_dot_axis = cross[0] * axis[0] + cross[1] * axis[1] + cross[2] * axis[2];
    let dot = start_vec[0] * current_vec[0]
        + start_vec[1] * current_vec[1]
        + start_vec[2] * current_vec[2];
    let dot = (dot / (start_len * current_len)).clamp(-1.0, 1.0);
    Some(cross_dot_axis.signum() * dot.acos())
}

/// Apply a world-space rotation (delta_angle around axis from start rotation) to the entity.
fn apply_gizmo_rotate_delta_from_start(
    world: &mut World,
    entity_id: EntityId,
    axis_direction: &[f32; 3],
    delta_angle: f32,
    start_world_rot_quat: &mathlib::Quat4f,
    parent_inv_rot_quat: &mathlib::Quat4f,
) {
    use mathlib::cg::vector3;

    let axis_v = vector3(axis_direction[0], axis_direction[1], axis_direction[2]);
    let r_delta = mathlib::Quat4f::from_axis_angle(&axis_v, delta_angle);
    let new_world_quat = r_delta * *start_world_rot_quat;
    let new_local_quat = *parent_inv_rot_quat * new_world_quat;
    let (r, p, y) = new_local_quat.to_euler_angles();
    if let Some(data) = world.get_mut(entity_id) {
        data.transform.rotation = [r, p, y];
        data.transform.rotation_quat = Some(new_local_quat);
    }
}

/// Scene lighting parameters (ambient intensity, directional lights).
#[derive(Clone, Debug)]
pub struct SceneLighting {
    /// Ambient intensity in [0, 1]. Default 0.45.
    pub ambient_intensity: f32,
    /// Direction toward the main directional light in view space (normalized). Default top-right-front.
    pub light_direction: [f32; 3],
    /// Directional light strength (0 = no diffuse). Default 1.0.
    pub lighting_strength: f32,
    /// Optional second light: [x, y, z] direction in view space (normalized), w = strength. None = single light.
    pub second_light: Option<[f32; 4]>,
}

impl Default for SceneLighting {
    fn default() -> Self {
        Self {
            ambient_intensity: 0.45,
            light_direction: [0.577_350_27_f32; 3], // 1/sqrt(3) each axis
            lighting_strength: 1.0,
            second_light: None,
        }
    }
}

/// Per-frame statistics for overlay (FPS, CPU/GPU time, element count).
#[derive(Clone, Debug, Default)]
pub struct FrameStats {
    /// Frames per second (smoothed).
    pub fps: f32,
    /// CPU time for the last frame in milliseconds.
    pub cpu_time_ms: f32,
    /// GPU time for the last frame in milliseconds, if available.
    pub gpu_time_ms: Option<f32>,
    /// Number of entities in the active world that have a primitive (drawn elements).
    pub element_count: usize,
}

/// Count entities in `world` that have a primitive (drawn elements).
fn element_count(world: &World) -> usize {
    world
        .entities_dfs()
        .iter()
        .filter(|id| world.get(**id).and_then(|n| n.primitive).is_some())
        .count()
}

/// Shared wgpu init: request device, build config, configure surface.
async fn wgpu_init(
    surface: &wgpu::Surface<'_>,
    adapter: &wgpu::Adapter,
    width: u32,
    height: u32,
) -> Result<(wgpu::Device, wgpu::Queue, wgpu::SurfaceConfiguration), RenderError> {
    // Disable on WASM: timestamp readback uses map_async and double-buffered staging; callbacks
    // are deferred on WASM so buffers can be reused before unmapped, causing "Buffer is already mapped".
    let timestamp_query = adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY)
        && !cfg!(target_arch = "wasm32");
    let required_features = if timestamp_query {
        wgpu::Features::TIMESTAMP_QUERY
    } else {
        wgpu::Features::empty()
    };
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features,
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            trace: wgpu::Trace::default(),
        })
        .await
        .map_err(|e| RenderError::WgpuInit(e.to_string()))?;

    let caps = surface.get_capabilities(adapter);
    let format = caps
        .formats
        .iter()
        .find(|f: &&wgpu::TextureFormat| f.is_srgb())
        .copied()
        .unwrap_or(caps.formats[0]);

    let present_mode = caps
        .present_modes
        .iter()
        .find(|m| matches!(m, wgpu::PresentMode::Mailbox))
        .copied()
        .unwrap_or(caps.present_modes[0]);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        format,
        width,
        height,
        present_mode,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    surface.configure(&device, &config);

    Ok((device, queue, config))
}

/// Engine: holds window/canvas, wgpu state, worlds, active world, camera, optional UI layer.
pub struct Engine {
    surface: wgpu::Surface<'static>,
    renderer: GpuRenderer,
    /// Scene worlds; at least one. [`Self::active_world`] indexes the one used for rendering.
    worlds: Vec<World>,
    /// Index into [`Self::worlds`] for the current scene.
    active_world: usize,
    /// Entity selected in the scene UI (e.g. for primitive selector).
    selected_entity: Option<EntityId>,
    camera: Camera,
    config: wgpu::SurfaceConfiguration,
    /// Optional UI layer (windows, buttons, sliders, checkboxes); rendered after the scene.
    pub ui: Option<UiLayer>,
    /// How scene geometry is drawn (solid, wireframe, vertex points, color map).
    view_mode: ViewMode,
    /// Gizmo mode when an entity is selected (translate, rotate, scale).
    gizmo_mode: GizmoMode,
    /// When true, the gizmo is not drawn (e.g. when Ctrl is held for camera-only mode).
    gizmo_hidden: bool,
    /// Active gizmo drag state (axis, start screen, start world position, axis direction).
    gizmo_drag: Option<GizmoDragState>,
    /// Optional slice plane overlay (checker quad with lighting).
    slice_plane: Option<SlicePlane>,
    /// Optional grid cube overlay (instanced cells).
    grid_overlay: Option<GridCubeDescriptor>,
    /// Batched 3D points to draw this frame (position, color, size). Cleared after each render.
    points: Vec<([f32; 3], [f32; 4], f32)>,
    /// Batched polylines to draw this frame (vertices, color, width). Cleared after each render.
    polylines: Vec<(Vec<[f32; 3]>, [f32; 4], f32)>,
    /// Scene lighting (ambient and directional light).
    pub scene_lighting: SceneLighting,
    /// Last frame stats (FPS, CPU/GPU time, element count); set by platform after each frame.
    last_frame_stats: Option<FrameStats>,
    /// Material registry (static and blendable matcaps).
    pub materials: MaterialRegistry,
    #[cfg(target_arch = "wasm32")]
    _canvas: HtmlCanvasElement,
    #[cfg(not(target_arch = "wasm32"))]
    _window: Option<Arc<Window>>,
    #[cfg(all(not(target_arch = "wasm32"), feature = "sdl3"))]
    _sdl_window: Option<sdl3::video::Window>,
}

impl Engine {
    /// Create engine from winit window (native only).
    ///
    /// # Errors
    /// Returns error if wgpu init fails.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new(window: Arc<Window>) -> Result<Self, RenderError> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance
            .create_surface(Arc::clone(&window))
            .map_err(|e| RenderError::WgpuInit(e.to_string()))?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| RenderError::WgpuInit(e.to_string()))?;

        let (device, queue, config) =
            wgpu_init(&surface, &adapter, size.width, size.height).await?;

        let renderer = GpuRenderer::new(device, queue, config.clone(), None)?;
        let camera = Camera::new_perspective(
            size.width as f32,
            size.height as f32,
            std::f32::consts::FRAC_PI_4,
        );

        // mut needed when feature "material" is enabled
        #[allow(unused_mut)]
        let mut engine = Self {
            surface,
            renderer,
            worlds: vec![World::new()],
            active_world: 0,
            selected_entity: None,
            camera,
            config,
            ui: None,
            view_mode: ViewMode::default(),
            gizmo_mode: GizmoMode::default(),
            gizmo_hidden: false,
            gizmo_drag: None,
            slice_plane: None,
            grid_overlay: None,
            points: Vec::new(),
            polylines: Vec::new(),
            scene_lighting: SceneLighting::default(),
            last_frame_stats: None,
            materials: MaterialRegistry::new(),
            _window: Some(window),
            #[cfg(feature = "sdl3")]
            _sdl_window: None,
        };
        #[cfg(feature = "material")]
        crate::material::register_builtin_flat(
            engine.renderer.device(),
            engine.renderer.queue(),
            &mut engine.materials,
        );
        #[cfg(feature = "material")]
        crate::material::register_builtin_procedural(
            engine.renderer.device(),
            engine.renderer.queue(),
            &mut engine.materials,
        );
        Ok(engine)
    }

    /// Create engine from SDL3 window (native only, requires sdl3 feature).
    ///
    /// # Errors
    /// Returns error if wgpu init fails.
    #[cfg(all(not(target_arch = "wasm32"), feature = "sdl3"))]
    pub async fn new_sdl3(window: sdl3::video::Window) -> Result<Self, RenderError> {
        let (width, height) = window.size_in_pixels();
        let instance = wgpu::Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });

        let raw = unsafe { wgpu::SurfaceTargetUnsafe::from_window(&window) }
            .map_err(|e| RenderError::WgpuInit(e.to_string()))?;
        let surface = unsafe { instance.create_surface_unsafe(raw) }
            .map_err(|e| RenderError::WgpuInit(e.to_string()))?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| RenderError::WgpuInit(e.to_string()))?;

        let (device, queue, config) = wgpu_init(&surface, &adapter, width, height).await?;

        let renderer = GpuRenderer::new(device, queue, config.clone(), None)?;
        let camera =
            Camera::new_perspective(width as f32, height as f32, std::f32::consts::FRAC_PI_4);

        let engine = Self {
            surface,
            renderer,
            worlds: vec![World::new()],
            active_world: 0,
            selected_entity: None,
            camera,
            config,
            ui: None,
            view_mode: ViewMode::default(),
            gizmo_mode: GizmoMode::default(),
            gizmo_hidden: false,
            gizmo_drag: None,
            slice_plane: None,
            grid_overlay: None,
            points: Vec::new(),
            polylines: Vec::new(),
            scene_lighting: SceneLighting::default(),
            last_frame_stats: None,
            materials: MaterialRegistry::new(),
            _window: None,
            _sdl_window: Some(window),
        };
        #[cfg(feature = "material")]
        crate::material::register_builtin_flat(
            engine.renderer.device(),
            engine.renderer.queue(),
            &mut engine.materials,
        );
        #[cfg(feature = "material")]
        crate::material::register_builtin_procedural(
            engine.renderer.device(),
            engine.renderer.queue(),
            &mut engine.materials,
        );
        Ok(engine)
    }

    /// Create engine from canvas (WASM only).
    ///
    /// # Errors
    /// Returns error if wgpu init fails or canvas has zero dimensions.
    #[cfg(target_arch = "wasm32")]
    pub async fn new_wasm(canvas: HtmlCanvasElement) -> Result<Self, RenderError> {
        let width = canvas.width();
        let height = canvas.height();
        if width == 0 || height == 0 {
            return Err(RenderError::WgpuInit(format!(
                "Canvas has zero dimensions ({}x{}). Ensure the canvas has explicit width/height or is visible.",
                width, height
            )));
        }
        let instance = wgpu::Instance::new(&InstanceDescriptor {
            backends: Backends::BROWSER_WEBGPU,
            ..Default::default()
        });

        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
            .map_err(|e| RenderError::WgpuInit(e.to_string()))?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| RenderError::WgpuInit(e.to_string()))?;

        let (device, queue, config) = wgpu_init(&surface, &adapter, width, height).await?;

        let renderer = GpuRenderer::new(device, queue, config.clone(), None)?;
        let camera =
            Camera::new_perspective(width as f32, height as f32, std::f32::consts::FRAC_PI_4);

        #[allow(unused_mut)]
        let mut engine = Self {
            surface,
            renderer,
            worlds: vec![World::new()],
            active_world: 0,
            selected_entity: None,
            camera,
            config,
            ui: None,
            view_mode: ViewMode::default(),
            gizmo_mode: GizmoMode::default(),
            gizmo_hidden: false,
            gizmo_drag: None,
            slice_plane: None,
            grid_overlay: None,
            points: Vec::new(),
            polylines: Vec::new(),
            scene_lighting: SceneLighting::default(),
            last_frame_stats: None,
            materials: MaterialRegistry::new(),
            _canvas: canvas,
        };
        #[cfg(feature = "material")]
        crate::material::register_builtin_flat(
            engine.renderer.device(),
            engine.renderer.queue(),
            &mut engine.materials,
        );
        #[cfg(feature = "material")]
        crate::material::register_builtin_procedural(
            engine.renderer.device(),
            engine.renderer.queue(),
            &mut engine.materials,
        );
        Ok(engine)
    }

    /// Enable the UI layer (semi-transparent windows, buttons, sliders, checkboxes). Call after construction.
    ///
    /// # Errors
    /// Returns error if UI pipeline creation fails.
    pub fn enable_ui(&mut self) -> Result<(), RenderError> {
        let width = self.config.width as f32;
        let height = self.config.height as f32;
        let layer = UiLayer::new(
            self.renderer.device(),
            self.renderer.queue(),
            self.config.format,
            width,
            height,
        )?;
        self.ui = Some(layer);
        Ok(())
    }

    /// Resize viewport.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(self.renderer.device(), &self.config);
            self.renderer.resize(width, height);
            self.camera.resize(width, height);
            if let Some(ref mut ui) = self.ui {
                ui.resize(self.renderer.queue(), width as f32, height as f32);
            }
        }
    }

    /// Current viewport width in pixels (for UI layout, e.g. stats panel position).
    #[must_use]
    pub fn viewport_width(&self) -> u32 {
        self.config.width
    }

    /// Queue a 3D point to be drawn this frame. Cleared after [`Self::render_frame`].
    pub fn draw_point(&mut self, position: [f32; 3], color: [f32; 4], size: f32) {
        self.points.push((position, color, size));
    }

    /// Queue a polyline to be drawn this frame. Cleared after [`Self::render_frame`].
    ///
    /// Draws a thick line through the given `vertices` with the given `color` and `width`.
    /// Requires at least two vertices; single-point or empty polylines are ignored.
    pub fn draw_polyline(
        &mut self,
        vertices: impl Into<Vec<[f32; 3]>>,
        color: [f32; 4],
        width: f32,
    ) {
        let v = vertices.into();
        if v.len() >= 2 && width > 0.0 {
            self.polylines.push((v, color, width));
        }
    }

    /// Creates an offscreen framebuffer (FBO) for render-to-texture and readback.
    ///
    /// Uses the surface format. The FBO is fixed at creation; create a new one when you need a
    /// different size (e.g. after window resize).
    ///
    /// # Errors
    /// Returns error if width or height is zero or if texture creation fails.
    pub fn create_framebuffer(&self, width: u32, height: u32) -> Result<Framebuffer, RenderError> {
        self.renderer.create_framebuffer(width, height, None)
    }

    /// Renders one frame to an offscreen framebuffer (no present).
    ///
    /// Same scene, points overlay, and optional UI as [`Self::render_frame`], but the result
    /// is written to `target` and nothing is presented to the surface. Uses the active world.
    /// Use [`Self::read_pixels_async_from`] to read pixels from the framebuffer.
    ///
    /// # Errors
    /// Returns error on render failure.
    pub fn render_frame_to(&mut self, target: &Framebuffer) -> Result<(), RenderError> {
        let world = &self.worlds[self.active_world];
        let points = std::mem::take(&mut self.points);
        let polylines = std::mem::take(&mut self.polylines);
        let effective_selected = if self.gizmo_hidden {
            None
        } else {
            self.selected_entity
        };
        self.renderer.render(
            RenderTarget::Framebuffer(target),
            world,
            &self.materials,
            &mut self.camera,
            self.scene_lighting.ambient_intensity,
            self.scene_lighting.light_direction,
            self.scene_lighting.lighting_strength,
            self.scene_lighting.second_light,
            self.view_mode,
            effective_selected,
            self.gizmo_mode,
            self.slice_plane.as_ref(),
            self.grid_overlay.as_ref(),
            &points,
            &polylines,
            self.ui.as_mut(),
        )
    }

    /// Render one frame (scene, points overlay, then UI if enabled).
    ///
    /// # Errors
    /// Returns error on render failure.
    pub fn render_frame(&mut self) -> Result<(), RenderError> {
        let world = &self.worlds[self.active_world];
        let points = std::mem::take(&mut self.points);
        let polylines = std::mem::take(&mut self.polylines);
        let effective_selected = if self.gizmo_hidden {
            None
        } else {
            self.selected_entity
        };
        self.renderer.render(
            RenderTarget::Surface(&self.surface),
            world,
            &self.materials,
            &mut self.camera,
            self.scene_lighting.ambient_intensity,
            self.scene_lighting.light_direction,
            self.scene_lighting.lighting_strength,
            self.scene_lighting.second_light,
            self.view_mode,
            effective_selected,
            self.gizmo_mode,
            self.slice_plane.as_ref(),
            self.grid_overlay.as_ref(),
            &points,
            &polylines,
            self.ui.as_mut(),
        )
    }

    /// Reads pixels asynchronously from the readback texture (last frame rendered to surface).
    ///
    /// On WASM, call [`Self::poll_device`] after each frame so the callback runs; do not block
    /// the main thread.
    ///
    /// # Errors
    /// Returns error if the readback texture is not available.
    pub fn read_pixels_async(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        callback: impl FnOnce(Result<(), RenderError>, Option<ReadbackGuard>) + Send + 'static,
    ) -> Result<(), RenderError> {
        self.renderer
            .read_pixels_async(x, y, width, height, callback)
    }

    /// Reads pixels asynchronously from a framebuffer (e.g. after [`Self::render_frame_to`]).
    ///
    /// Use this to read back from an offscreen target without going through the surface readback
    /// texture. On WASM, call [`Self::poll_device`] so the callback runs.
    ///
    /// # Errors
    /// Returns error if the copy or map fails.
    pub fn read_pixels_async_from(
        &self,
        framebuffer: &Framebuffer,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        callback: impl FnOnce(Result<(), RenderError>, Option<ReadbackGuard>) + Send + 'static,
    ) -> Result<(), RenderError> {
        self.renderer
            .read_pixels_async_from(framebuffer, x, y, width, height, callback)
    }

    /// Set last frame stats. Call from platform after each frame with FPS and CPU time;
    /// [`Self`] fills `element_count` from the active world.
    pub fn set_last_frame_stats(&mut self, mut stats: FrameStats) {
        stats.element_count = element_count(&self.worlds[self.active_world]);
        self.last_frame_stats = Some(stats);
    }

    /// Last frame stats (FPS, CPU/GPU time, element count), if set by the platform.
    #[must_use]
    pub fn last_frame_stats(&self) -> Option<&FrameStats> {
        self.last_frame_stats.as_ref()
    }

    /// Last completed GPU time in milliseconds (from timestamp queries), if available.
    #[must_use]
    pub fn last_gpu_time_ms(&self) -> Option<f32> {
        self.renderer.last_gpu_time_ms()
    }

    /// Number of entities in the active world that have a primitive (drawn elements).
    #[must_use]
    pub fn active_world_element_count(&self) -> usize {
        element_count(&self.worlds[self.active_world])
    }

    /// Process wgpu async callbacks (e.g. timestamp readback for GPU timing).
    /// Call after [`Self::render_frame`] on native so [`Self::last_gpu_time_ms`] can update.
    /// No-op on WebGPU for blocking behavior; GPU time remains unavailable on WASM.
    pub fn poll_device(&self) {
        let _ = self.renderer.device().poll(PollType::Poll);
    }

    /// Captures the current framebuffer as raw RGB pixel data.
    ///
    /// Reads all pixels currently displayed on the screen into a buffer.
    /// The buffer is automatically resized to fit the screen dimensions.
    /// Pixels are stored in RGB format (3 bytes per pixel), row by row from bottom to top.
    ///
    /// Call [`Self::render_frame`] at least once before snapping to capture the current view.
    ///
    /// # Arguments
    /// * `out` - The output buffer. It will be resized to width × height × 3 bytes.
    ///
    /// # Example
    /// ```ignore
    /// let mut pixels = Vec::new();
    /// engine.snap(&mut pixels);
    /// // pixels now contains RGB data (width × height × 3 bytes)
    /// ```
    pub fn snap(&self, out: &mut Vec<u8>) {
        let (width, height) = (self.config.width, self.config.height);
        self.snap_rect(out, 0, 0, width as usize, height as usize);
    }

    /// Captures a rectangular region of the framebuffer as raw RGB pixel data.
    ///
    /// Reads a specific rectangular region of pixels from the screen.
    /// Pixels are stored in RGB format (3 bytes per pixel), rows from bottom to top.
    ///
    /// # Arguments
    /// * `out` - The output buffer. It will be resized to width × height × 3 bytes.
    /// * `x` - The x-coordinate of the rectangle's bottom-left corner.
    /// * `y` - The y-coordinate of the rectangle's bottom-left corner.
    /// * `width` - The width of the rectangle in pixels.
    /// * `height` - The height of the rectangle in pixels.
    pub fn snap_rect(&self, out: &mut Vec<u8>, x: usize, y: usize, width: usize, height: usize) {
        self.renderer.read_pixels(out, x, y, width, height);
    }

    /// Captures the current framebuffer as an image.
    ///
    /// Returns an [`ImageBuffer`] containing the current screen content.
    /// The image is automatically flipped vertically to match the expected orientation
    /// (wgpu's top-left origin is converted to top-left image layout; the raw readback
    /// uses bottom-to-top rows for OpenGL-style compatibility).
    ///
    /// Requires the `screenshot` feature.
    ///
    /// # Returns
    /// An [`image::ImageBuffer`]`<`[`image::Rgb`]`<u8>, Vec<u8>>` containing the screen pixels.
    ///
    /// # Example
    /// ```ignore
    /// let image = engine.snap_image();
    /// image.save("screenshot.png").unwrap();
    /// ```
    #[cfg(feature = "screenshot")]
    #[must_use]
    pub fn snap_image(&self) -> image::ImageBuffer<image::Rgb<u8>, Vec<u8>> {
        use image::imageops;
        let (width, height) = (self.config.width, self.config.height);
        let mut buf = Vec::new();
        self.snap(&mut buf);
        let img = image::ImageBuffer::from_vec(width, height, buf)
            .expect("buffer size must match width×height×3 for image");
        imageops::flip_vertical(&img)
    }

    /// Reference to the active world.
    #[must_use]
    pub fn world(&self) -> &World {
        &self.worlds[self.active_world]
    }

    /// Mutable reference to the active world.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.worlds[self.active_world]
    }

    /// All worlds. Use [`Self::active_world`] to index.
    #[must_use]
    pub fn worlds(&self) -> &[World] {
        &self.worlds
    }

    /// Mutable slice of all worlds.
    pub fn worlds_mut(&mut self) -> &mut [World] {
        &mut self.worlds
    }

    /// Index of the active world in [`Self::worlds`].
    #[must_use]
    pub fn active_world(&self) -> usize {
        self.active_world
    }

    /// Set the active world by index. No-op if index is out of bounds.
    pub fn set_active_world(&mut self, index: usize) {
        if index < self.worlds.len() {
            self.active_world = index;
        }
    }

    /// Currently selected entity (for scene UI primitive selector).
    #[must_use]
    pub fn selected_entity(&self) -> Option<EntityId> {
        self.selected_entity
    }

    /// Set the selected entity.
    pub fn set_selected_entity(&mut self, id: Option<EntityId>) {
        self.selected_entity = id;
    }

    /// Returns and clears the last clicked UI button's control id, if any. Call after [`Self::ui_mouse_up`].
    #[must_use]
    pub fn take_clicked_control(&mut self) -> Option<crate::ui::ControlId> {
        self.ui.as_mut().and_then(|u| u.take_clicked_control())
    }

    /// Reference to the camera (for unproject, etc.).
    #[must_use]
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Mutable reference to the camera.
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Orbit camera by delta (radians). Use for mouse drag navigation.
    pub fn orbit(&mut self, dyaw: f32, dpitch: f32) {
        self.camera.orbit(dyaw, dpitch);
    }

    /// Zoom camera (scroll wheel).
    pub fn zoom(&mut self, delta: f32) {
        self.camera.zoom(delta);
    }

    /// Pan camera (move look-at target in view plane). Use for Ctrl+drag camera control.
    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.camera.pan(dx, dy);
    }

    /// Reset camera orbit to default (yaw 0, pitch 0, distance 2.5). Use when the user clicks a "Reset" button.
    pub fn reset_camera(&mut self) {
        self.camera.reset_orbit();
    }

    /// Forward cursor position to UI; call from platform on cursor move.
    pub fn ui_mouse_move(&mut self, x: f32, y: f32) {
        if let Some(ref mut ui) = self.ui {
            ui.ui_mouse_move(x, y);
        }
    }

    /// Forward mouse down to UI; call from platform when left button is pressed.
    pub fn ui_mouse_down(&mut self) {
        if let Some(ref mut ui) = self.ui {
            ui.ui_mouse_down();
        }
    }

    /// Forward mouse up to UI; call from platform when left button is released.
    pub fn ui_mouse_up(&mut self) {
        if let Some(ref mut ui) = self.ui {
            ui.ui_mouse_up();
        }
    }

    /// Advance UI spring animations by `dt` seconds. Call each frame from the platform (e.g. before [`Self::render_frame`]).
    pub fn update_ui_springs(&mut self, dt: f32) {
        if let Some(ref mut ui) = self.ui {
            ui.update_springs(dt);
        }
    }

    /// Scroll the topmost scrollable window under the cursor by `delta` (positive = content up).
    /// Call from platform on mouse wheel when cursor is over UI; otherwise use [`Self::zoom`] for camera.
    pub fn ui_scroll(&mut self, delta: f32) {
        if let Some(ref mut ui) = self.ui {
            ui.scroll_window_at_cursor(delta);
        }
    }

    /// Returns true if the cursor is over any UI control (e.g. skip orbit when true).
    #[must_use]
    pub fn is_cursor_over_ui(&self) -> bool {
        self.ui.as_ref().is_some_and(|ui| ui.is_cursor_over_ui())
    }

    /// Current view mode (solid, wireframe, vertex points, color map).
    #[must_use]
    pub fn view_mode(&self) -> ViewMode {
        self.view_mode
    }

    /// Set view mode for scene rendering.
    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    /// Current gizmo mode (translate, rotate, scale).
    #[must_use]
    pub fn gizmo_mode(&self) -> GizmoMode {
        self.gizmo_mode
    }

    /// Set gizmo mode for the overlay when an entity is selected.
    pub fn set_gizmo_mode(&mut self, mode: GizmoMode) {
        self.gizmo_mode = mode;
    }

    /// Whether the gizmo is hidden (e.g. when Ctrl is held for camera-only mode).
    #[must_use]
    pub fn gizmo_hidden(&self) -> bool {
        self.gizmo_hidden
    }

    /// Set whether the gizmo is hidden. When true, the gizmo is not drawn even if an entity is selected.
    pub fn set_gizmo_hidden(&mut self, hidden: bool) {
        self.gizmo_hidden = hidden;
    }

    /// Returns true if a gizmo drag is currently active.
    #[must_use]
    pub fn is_gizmo_dragging(&self) -> bool {
        self.gizmo_drag.is_some()
    }

    /// Start a gizmo drag. Call when the user picks a gizmo handle. Returns true if started.
    pub fn gizmo_drag_start(
        &mut self,
        entity_id: EntityId,
        axis: GizmoAxis,
        screen_x: f32,
        screen_y: f32,
    ) -> bool {
        let world = &self.worlds[self.active_world];
        let Some(world_mat) = world.entity_world_matrix(entity_id) else {
            return false;
        };
        let Some(start_world_pos) = world.entity_world_position(entity_id) else {
            return false;
        };
        let Some(node) = world.get(entity_id) else {
            return false;
        };
        let axis_direction = world_axis_direction(&world_mat, axis);
        let view_forward = view_forward_world(&self.camera);
        let start_plane_hit = match screen_ray_to_world(&self.camera, screen_x, screen_y) {
            Some((origin, dir)) => {
                match ray_plane_intersection(&origin, &dir, &start_world_pos, &view_forward) {
                    Some(t) if t > 0.0 => [
                        origin[0] + dir[0] * t,
                        origin[1] + dir[1] * t,
                        origin[2] + dir[2] * t,
                    ],
                    _ => start_world_pos,
                }
            }
            None => start_world_pos,
        };
        let mut start_rot =
            mathlib::math3d::Matrix3f::with_storage(3, 3, mathlib::types::Storage::Column);
        for i in 0..3 {
            for j in 0..3 {
                start_rot.set(i, j, world_mat.get(i, j));
            }
        }
        let start_world_rot_quat = mathlib::Quat4f::from_rotation_matrix3(&start_rot);
        let parent_inv_rot_quat = world
            .parent(entity_id)
            .and_then(|pid| {
                world.entity_world_matrix(pid).map(|parent_world| {
                    let parent_inv = matrix4f_inverse(&parent_world);
                    let mut pr = mathlib::math3d::Matrix3f::with_storage(
                        3,
                        3,
                        mathlib::types::Storage::Column,
                    );
                    for i in 0..3 {
                        for j in 0..3 {
                            pr.set(i, j, parent_inv.get(i, j));
                        }
                    }
                    mathlib::Quat4f::from_rotation_matrix3(&pr)
                })
            })
            .unwrap_or(mathlib::Quat4f::identity());
        self.gizmo_drag = Some(GizmoDragState {
            entity_id,
            axis,
            mode: self.gizmo_mode,
            start_screen: (screen_x, screen_y),
            start_world_pos,
            start_plane_hit,
            axis_direction,
            start_rotation: node.transform.rotation,
            start_scale: node.transform.scale,
            start_world_rot_quat,
            parent_inv_rot_quat,
        });
        true
    }

    /// Update entity transform during a gizmo drag. No-op if not dragging.
    pub fn gizmo_drag_move(&mut self, screen_x: f32, screen_y: f32) {
        let Some(ref drag) = self.gizmo_drag else {
            return;
        };
        let world = &mut self.worlds[self.active_world];
        if world.get(drag.entity_id).is_none() {
            self.gizmo_drag = None;
            return;
        }
        let (origin, dir) = match screen_ray_to_world(&self.camera, screen_x, screen_y) {
            Some(r) => r,
            None => return,
        };
        let view_forward = view_forward_world(&self.camera);
        let plane_t =
            match ray_plane_intersection(&origin, &dir, &drag.start_world_pos, &view_forward) {
                Some(t) if t > 0.0 => t,
                _ => return,
            };
        let current_plane = [
            origin[0] + dir[0] * plane_t,
            origin[1] + dir[1] * plane_t,
            origin[2] + dir[2] * plane_t,
        ];
        match drag.mode {
            GizmoMode::Translate => {
                let delta = [
                    current_plane[0] - drag.start_world_pos[0],
                    current_plane[1] - drag.start_world_pos[1],
                    current_plane[2] - drag.start_world_pos[2],
                ];
                let t = delta[0] * drag.axis_direction[0]
                    + delta[1] * drag.axis_direction[1]
                    + delta[2] * drag.axis_direction[2];
                let new_pos = [
                    drag.start_world_pos[0] + t * drag.axis_direction[0],
                    drag.start_world_pos[1] + t * drag.axis_direction[1],
                    drag.start_world_pos[2] + t * drag.axis_direction[2],
                ];
                world.set_entity_world_position(drag.entity_id, new_pos[0], new_pos[1], new_pos[2]);
            }
            GizmoMode::Rotate => {
                if let Some(delta_angle) = gizmo_rotate_delta_angle(
                    &drag.start_world_pos,
                    &drag.start_plane_hit,
                    &current_plane,
                    &drag.axis_direction,
                ) {
                    apply_gizmo_rotate_delta_from_start(
                        world,
                        drag.entity_id,
                        &drag.axis_direction,
                        delta_angle,
                        &drag.start_world_rot_quat,
                        &drag.parent_inv_rot_quat,
                    );
                }
            }
            GizmoMode::Scale => {
                let delta = [
                    current_plane[0] - drag.start_world_pos[0],
                    current_plane[1] - drag.start_world_pos[1],
                    current_plane[2] - drag.start_world_pos[2],
                ];
                let t = delta[0] * drag.axis_direction[0]
                    + delta[1] * drag.axis_direction[1]
                    + delta[2] * drag.axis_direction[2];
                let scale_factor = (1.0 + t * 2.0_f32).max(0.01);
                let new_scale = match drag.axis {
                    GizmoAxis::X => [
                        (drag.start_scale[0] * scale_factor).max(0.01),
                        drag.start_scale[1],
                        drag.start_scale[2],
                    ],
                    GizmoAxis::Y => [
                        drag.start_scale[0],
                        (drag.start_scale[1] * scale_factor).max(0.01),
                        drag.start_scale[2],
                    ],
                    GizmoAxis::Z => [
                        drag.start_scale[0],
                        drag.start_scale[1],
                        (drag.start_scale[2] * scale_factor).max(0.01),
                    ],
                };
                if let Some(data) = world.get_mut(drag.entity_id) {
                    data.transform.scale = new_scale;
                }
            }
        }
    }

    /// End the current gizmo drag.
    pub fn gizmo_drag_end(&mut self) {
        self.gizmo_drag = None;
    }

    /// Optional slice plane overlay. `None` = disabled.
    #[must_use]
    pub fn slice_plane(&self) -> Option<&SlicePlane> {
        self.slice_plane.as_ref()
    }

    /// Set the slice plane overlay. Pass `None` to disable.
    pub fn set_slice_plane(&mut self, plane: Option<SlicePlane>) {
        self.slice_plane = plane;
    }

    /// Optional grid cube overlay. `None` = disabled.
    #[must_use]
    pub fn grid_overlay(&self) -> Option<&GridCubeDescriptor> {
        self.grid_overlay.as_ref()
    }

    /// Set the grid cube overlay. Pass `None` to disable.
    pub fn set_grid_overlay(&mut self, overlay: Option<GridCubeDescriptor>) {
        self.grid_overlay = overlay;
    }

    /// Load a static material from bytes. Requires `material` feature.
    ///
    /// # Errors
    /// Returns error if image parsing or texture creation fails.
    #[cfg(feature = "material")]
    pub fn load_static_material_from_bytes(
        &mut self,
        name: impl Into<String>,
        data: &[u8],
        as_png: bool,
    ) -> Result<(), RenderError> {
        use crate::backend::create_texture_from_image;
        let image = if as_png {
            parse::image::parse_png(data).map_err(|e| RenderError::MaterialLoad(e.to_string()))?
        } else {
            parse::image::parse_jpeg(data).map_err(|e| RenderError::MaterialLoad(e.to_string()))?
        };
        let (texture, _) =
            create_texture_from_image(self.renderer.device(), self.renderer.queue(), &image, None)?;
        let mat = crate::material::Material::static_mat(name.into(), texture);
        self.materials.insert(mat.name.clone(), mat);
        Ok(())
    }

    /// Load a static material from a file path. Native only. Requires `material` feature.
    ///
    /// # Errors
    /// Returns error if file read, image parsing, or texture creation fails.
    #[cfg(all(feature = "material", not(target_arch = "wasm32")))]
    pub fn load_static_material(
        &mut self,
        name: impl Into<String>,
        path: &std::path::Path,
    ) -> Result<(), RenderError> {
        let data = std::fs::read(path)
            .map_err(|e| RenderError::MaterialLoad(format!("read file: {e}")))?;
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        let as_png = ext == "png" || ext.is_empty();
        self.load_static_material_from_bytes(name, &data, as_png)
    }

    /// Load a blendable material from four image file paths. Native only. Requires `material` feature.
    ///
    /// Files should be R, G, B, K basis matcaps.
    ///
    /// # Errors
    /// Returns error if any file read, image parsing, or texture creation fails.
    #[cfg(all(feature = "material", not(target_arch = "wasm32")))]
    pub fn load_blendable_material(
        &mut self,
        name: impl Into<String>,
        paths: [&std::path::Path; 4],
    ) -> Result<(), RenderError> {
        use crate::backend::create_texture_from_image;
        let name = name.into();
        let device = self.renderer.device();
        let queue = self.renderer.queue();
        let mut images = Vec::with_capacity(4);
        for p in &paths {
            let data = std::fs::read(p)
                .map_err(|e| RenderError::MaterialLoad(format!("read {}: {e}", p.display())))?;
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("png");
            let img = if ext.eq_ignore_ascii_case("png") {
                parse::image::parse_png(&data)
                    .map_err(|e| RenderError::MaterialLoad(e.to_string()))?
            } else {
                parse::image::parse_jpeg(&data)
                    .map_err(|e| RenderError::MaterialLoad(e.to_string()))?
            };
            images.push(img);
        }
        let (tex_r, _) = create_texture_from_image(device, queue, &images[0], Some("mat_r"))?;
        let (tex_g, _) = create_texture_from_image(device, queue, &images[1], Some("mat_g"))?;
        let (tex_b, _) = create_texture_from_image(device, queue, &images[2], Some("mat_b"))?;
        let (tex_k, _) = create_texture_from_image(device, queue, &images[3], Some("mat_k"))?;
        let mat = crate::material::Material::blendable(name.clone(), tex_r, tex_g, tex_b, tex_k);
        self.materials.insert(name, mat);
        Ok(())
    }

    /// Load a blendable material from base path and extension. Native only. Requires `material` feature.
    ///
    /// Loads `base_r.ext`, `base_g.ext`, `base_b.ext`, `base_k.ext`.
    /// Extension should include the dot (e.g. `".png"`) or will be added.
    #[cfg(all(feature = "material", not(target_arch = "wasm32")))]
    pub fn load_blendable_material_from_base(
        &mut self,
        name: impl Into<String>,
        base_path: &std::path::Path,
        filename_ext: &str,
    ) -> Result<(), RenderError> {
        let base = base_path.to_string_lossy();
        let ext = if filename_ext.starts_with('.') {
            filename_ext.to_string()
        } else {
            format!(".{filename_ext}")
        };
        let paths = [
            std::path::PathBuf::from(format!("{base}_r{ext}")),
            std::path::PathBuf::from(format!("{base}_g{ext}")),
            std::path::PathBuf::from(format!("{base}_b{ext}")),
            std::path::PathBuf::from(format!("{base}_k{ext}")),
        ];
        self.load_blendable_material(name, [&paths[0], &paths[1], &paths[2], &paths[3]])
    }
}
