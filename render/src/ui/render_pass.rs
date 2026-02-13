//! UI render pass: pipeline with alpha blending and screen-space ortho.
//!
//! Quad mesh for panels/controls (with optional rounded corners from theme); text via wgpu_text (glyph_brush + ab_glyph). Label text is laid out within the label rect with bounds and horizontal/vertical alignment. Uses wgpu 28 API: `write_texture(dest, data, TexelCopyBufferLayout, Extent3d)` with `texture.as_image_copy()`; sampler `mipmap_filter` is `MipmapFilterMode`, not `FilterMode`.

use crate::backend::{ShaderConfig, Vertex};
use crate::error::RenderError;
use crate::ui::components::{LabelTextAlign, WindowChild};
use crate::ui::mesh::{build_ui_mesh, WindowDrawRange};
use crate::ui::{Theme, Window};
use ab_glyph::PxScale;
use mathlib::cg::matrix4f_to_array;
use mathlib::math3d::Matrix4f;
use wgpu::util::DeviceExt;
use wgpu_text::glyph_brush::{HorizontalAlign, Layout, Section, Text};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

/// Screen-space orthographic matrix: pixel (0,0) top-left, (width, height) bottom-right -> NDC.
fn ui_ortho_matrix(width: f32, height: f32) -> Matrix4f {
    mathlib::cg::new_orthographic(0.0, width, height, 0.0, -1.0, 1.0)
}

/// UI pipeline and buffers; builds mesh from windows + theme and draws with alpha blend.
/// Quad mesh for panels/controls; text via wgpu_text TextBrush.
/// Vertex/index buffers are reused when the mesh fits; new buffers are created only when the mesh grows.
pub struct UiRenderPass {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    index_count: u32,
    /// Capacity of the current vertex buffer (vertex count); 0 when no buffer.
    vertex_capacity: u32,
    /// Capacity of the current index buffer (index count); 0 when no buffer.
    index_capacity: u32,
    /// Text renderer (glyph_brush + ab_glyph).
    text_brush: wgpu_text::TextBrush<ab_glyph::FontRef<'static>>,
    /// Per-window draw ranges for scissor (set each frame in update_mesh).
    window_ranges: Vec<WindowDrawRange>,
    /// Viewport size (for resetting scissor before text).
    viewport_width: u32,
    viewport_height: u32,
}

impl UiRenderPass {
    /// Create the UI pipeline and buffers. Call `resize` and `update_mesh` before first draw.
    ///
    /// # Errors
    /// Returns error if pipeline creation fails.
    pub fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        width: f32,
        height: f32,
        shader_config: Option<&ShaderConfig>,
    ) -> Result<Self, RenderError> {
        let shader = if shader_config.and_then(|c| c.ui_wgsl.as_ref()).is_some() {
            let wgsl = shader_config.unwrap().ui_wgsl();
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ui_shader"),
                source: wgpu::ShaderSource::Wgsl(wgsl.into()),
            })
        } else {
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/ui.wgsl"))
        };

        let ortho = ui_ortho_matrix(width, height);
        let ortho_arr = matrix4f_to_array(&ortho);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI uniform buffer"),
            contents: bytemuck::cast_slice(&ortho_arr),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("UI uniform bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("UI bind group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let font_bytes = include_bytes!("../../fonts/DejaVuSans.ttf");
        let text_brush = wgpu_text::BrushBuilder::using_font_bytes(font_bytes)
            .map_err(|e| RenderError::WgpuInit(format!("font load failed: {e}")))?
            .initial_cache_size((1024, 1024))
            .draw_cache_align_4x4(true)
            .build(device, width as u32, height as u32, format);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI pipeline layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Ok(Self {
            pipeline,
            uniform_buffer,
            bind_group,
            vertex_buffer: None,
            index_buffer: None,
            index_count: 0,
            vertex_capacity: 0,
            index_capacity: 0,
            text_brush,
            window_ranges: Vec::new(),
            viewport_width: width as u32,
            viewport_height: height as u32,
        })
    }

    /// Update ortho matrix for new viewport size. Also resizes text brush view.
    pub fn resize(&mut self, queue: &wgpu::Queue, width: f32, height: f32) {
        self.viewport_width = width as u32;
        self.viewport_height = height as u32;
        let ortho = ui_ortho_matrix(width, height);
        let ortho_arr = matrix4f_to_array(&ortho);
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&ortho_arr));
        self.text_brush.resize_view(width, height, queue);
    }

    /// Upload UI mesh from windows and theme; call before draw.
    /// Reuses existing vertex/index buffers when the new mesh fits; creates new buffers only when the mesh grows.
    /// Also queues text sections for TextBrush rendering (with scroll offset for scrollable windows).
    pub fn update_mesh(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        windows: &[Window],
        theme: &Theme,
        viewport_width: f32,
    ) {
        let (vertices, indices, ranges) = build_ui_mesh(windows, theme, viewport_width);
        let vertex_count = vertices.len() as u32;
        self.index_count = indices.len() as u32;
        self.window_ranges = ranges;

        if self.index_count == 0 {
            self.vertex_buffer = None;
            self.index_buffer = None;
            self.vertex_capacity = 0;
            self.index_capacity = 0;
        } else {
            let fits =
                vertex_count <= self.vertex_capacity && self.index_count <= self.index_capacity;

            if fits {
                if let (Some(ref vb), Some(ref ib)) = (&self.vertex_buffer, &self.index_buffer) {
                    queue.write_buffer(vb, 0, bytemuck::cast_slice(&vertices));
                    queue.write_buffer(ib, 0, bytemuck::cast_slice(&indices));
                }
            } else {
                self.vertex_buffer = Some(device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("UI vertex buffer"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    },
                ));
                self.index_buffer = Some(device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("UI index buffer"),
                        contents: bytemuck::cast_slice(&indices),
                        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                    },
                ));
                self.vertex_capacity = vertex_count;
                self.index_capacity = self.index_count;
            }
        }

        // Queue text sections for wgpu_text (apply scroll offset for scrollable windows).
        // Bounds constrain layout to the label rect; layout aligns text horizontally and vertically.
        let h_align = |a: LabelTextAlign| match a {
            LabelTextAlign::Left => HorizontalAlign::Left,
            LabelTextAlign::Center => HorizontalAlign::Center,
            LabelTextAlign::Right => HorizontalAlign::Right,
        };
        let sections: Vec<Section> = windows
            .iter()
            .flat_map(|w| {
                let scroll_y = if w.is_scrollable() { w.scroll_y } else { 0.0 };
                w.children.iter().filter_map(move |c| {
                    if let WindowChild::Label(l) = c {
                        let y = l.rect.y - scroll_y;
                        let scale = PxScale::from(l.rect.h);
                        let layout = Layout::default_single_line()
                            .h_align(h_align(l.alignment))
                            .v_align(wgpu_text::glyph_brush::VerticalAlign::Center);
                        let section = Section::default()
                            .add_text(
                                Text::new(&l.text)
                                    .with_scale(scale)
                                    .with_color(theme.label_text),
                            )
                            .with_screen_position((l.rect.x, y))
                            .with_bounds((l.rect.w, l.rect.h))
                            .with_layout(layout);
                        Some(section)
                    } else {
                        None
                    }
                })
            })
            .collect();

        if let Err(e) = self.text_brush.queue(device, queue, &sections) {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::error_1(&JsValue::from_str(&format!("TextBrush queue failed: {e}")));
            #[cfg(not(target_arch = "wasm32"))]
            eprintln!("TextBrush queue failed: {e}");
        }
    }

    /// Encode UI draw into the given render pass. Call after `update_mesh` in the same frame.
    /// Draw order: quads first (with per-window scissor), then text (both with alpha blending; text on top).
    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if let (Some(ref vb), Some(ref ib)) = (&self.vertex_buffer, &self.index_buffer) {
            if self.index_count > 0 {
                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_bind_group(0, &self.bind_group, &[]);
                render_pass.set_vertex_buffer(0, vb.slice(..));
                render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint16);
                for r in &self.window_ranges {
                    if r.index_count > 0 {
                        let x = r.body_rect.x as u32;
                        let y = r.body_rect.y as u32;
                        let w = r.body_rect.w as u32;
                        let h = r.body_rect.h as u32;
                        render_pass.set_scissor_rect(x, y, w, h);
                        render_pass.draw_indexed(
                            r.index_start..(r.index_start + r.index_count),
                            0,
                            0..1,
                        );
                    }
                }
                // Reset scissor to full viewport so text draws everywhere
                render_pass.set_scissor_rect(0, 0, self.viewport_width, self.viewport_height);
            }
        }
        self.text_brush.draw(render_pass);
    }
}
