//! GPU context initialization and storage.
//!
//! [`GpuContext`] holds the wgpu device, queue, and pipelines. Use [`init_blocking`](super::init_blocking)
//! (native) or [`init_async`](super::init_async) (wasm) to create and store the context.

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

use super::config::GpuConfig;
use super::pipelines::{
    create_add_pipeline, create_add_vec4_pipeline, create_axpy_pipeline, create_dot_pipeline,
    create_elementwise_pipeline, create_matmul_pipeline, create_matvec_pipeline,
    create_scale_pipeline, create_scale_vec4_pipeline, create_spmv_pipeline, create_sub_pipeline,
    create_unary_pipeline,
};

/// Workgroup size used for 1D kernels (dot, elementwise, matvec, SpMV). Clamped to device limits.
pub(crate) const ELEMENTWISE_WORKGROUP_SIZE: u32 = 256;

#[allow(dead_code)]
pub(crate) struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub max_compute_workgroup_size_x: u32,
    pub max_compute_workgroup_size_y: u32,
    pub max_compute_workgroup_size_z: u32,
    pub elementwise_workgroup_size: u32,
    pub matmul_pipeline: wgpu::ComputePipeline,
    pub matmul_pipeline_16: wgpu::ComputePipeline,
    pub matmul_tiled_pipeline_16: wgpu::ComputePipeline,
    pub matmul_workgroup_size: u32,
    pub matmul_bind_group_layout: wgpu::BindGroupLayout,
    pub dot_pipeline: wgpu::ComputePipeline,
    pub dot_bind_group_layout: wgpu::BindGroupLayout,
    /// 256-thread matvec pipeline; None when max_compute_workgroup_size_x < 256.
    pub matvec_pipeline_256: Option<wgpu::ComputePipeline>,
    pub matvec_pipeline_32: wgpu::ComputePipeline,
    pub matvec_bind_group_layout: wgpu::BindGroupLayout,
    pub add_pipeline: wgpu::ComputePipeline,
    pub add_vec4_pipeline: wgpu::ComputePipeline,
    pub add_bind_group_layout: wgpu::BindGroupLayout,
    pub sub_pipeline: wgpu::ComputePipeline,
    pub sub_bind_group_layout: wgpu::BindGroupLayout,
    pub mul_pipeline: wgpu::ComputePipeline,
    pub mul_bind_group_layout: wgpu::BindGroupLayout,
    pub scale_pipeline: wgpu::ComputePipeline,
    pub scale_vec4_pipeline: wgpu::ComputePipeline,
    pub scale_bind_group_layout: wgpu::BindGroupLayout,
    pub axpy_pipeline: wgpu::ComputePipeline,
    pub axpy_bind_group_layout: wgpu::BindGroupLayout,
    pub abs_pipeline: wgpu::ComputePipeline,
    pub abs_bind_group_layout: wgpu::BindGroupLayout,
    pub sqrt_pipeline: wgpu::ComputePipeline,
    pub sqrt_bind_group_layout: wgpu::BindGroupLayout,
    pub div_pipeline: wgpu::ComputePipeline,
    pub div_bind_group_layout: wgpu::BindGroupLayout,
    pub spmv_pipeline: wgpu::ComputePipeline,
    pub spmv_bind_group_layout: wgpu::BindGroupLayout,
    #[cfg(target_arch = "wasm32")]
    pub(crate) staging_pool: RefCell<Vec<(u64, wgpu::Buffer)>>,
}

#[cfg(target_arch = "wasm32")]
impl GpuContext {
    /// Take a staging buffer from the pool (size >= n_bytes) or create a new one.
    pub(crate) fn get_staging_buffer(&self, n_bytes: u64, label: &str) -> wgpu::Buffer {
        let mut pool = self.staging_pool.borrow_mut();
        if let Some(pos) = pool.iter().position(|(s, _)| *s >= n_bytes) {
            pool.remove(pos).1
        } else {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: n_bytes,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        }
    }

    /// Return a staging buffer to the pool for reuse.
    pub(crate) fn return_staging(&self, size: u64, buffer: wgpu::Buffer) {
        self.staging_pool.borrow_mut().push((size, buffer));
    }
}

pub(crate) async fn init_inner_async(config: Option<&GpuConfig>) -> Result<GpuContext, String> {
    use super::shaders::{WGSL_ABS, WGSL_DIV, WGSL_MUL, WGSL_SQRT};

    let default_config = GpuConfig::default();
    let cfg = config.unwrap_or(&default_config);
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: cfg.power_preference.to_wgpu(),
            force_fallback_adapter: cfg.force_fallback_adapter,
            compatible_surface: None,
        })
        .await
        .map_err(|_| {
            "No WebGPU adapter. Use HTTPS or localhost and ensure WebGPU is enabled in the browser."
                .to_string()
        })?;
    let required_limits = {
        #[cfg(target_arch = "wasm32")]
        {
            adapter.limits()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if cfg.relaxed_limits {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            }
        }
    };
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("mathlib gpu"),
            required_features: wgpu::Features::empty(),
            required_limits,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::default(),
        })
        .await
        .map_err(|e| format!("WebGPU device creation failed: {}", e))?;
    let (matmul_pipeline, matmul_pipeline_16, matmul_tiled_pipeline_16, matmul_bind_group_layout) =
        create_matmul_pipeline(&device);
    let limits = device.limits();
    let max_x = limits.max_compute_workgroup_size_x;
    let max_y = limits.max_compute_workgroup_size_y;
    let max_z = limits.max_compute_workgroup_size_z;
    let matmul_workgroup_size = if max_x >= 16 && max_y >= 16 {
        16u32
    } else {
        8u32
    };
    let elementwise_workgroup_size = max_x.min(ELEMENTWISE_WORKGROUP_SIZE);
    let (dot_pipeline, dot_bind_group_layout) = create_dot_pipeline(&device);
    let (matvec_pipeline_256, matvec_pipeline_32, matvec_bind_group_layout) =
        create_matvec_pipeline(&device, max_x);
    let (add_pipeline, add_bind_group_layout) = create_add_pipeline(&device);
    let add_vec4_pipeline = create_add_vec4_pipeline(&device, &add_bind_group_layout);
    let (sub_pipeline, sub_bind_group_layout) = create_sub_pipeline(&device);
    let (mul_pipeline, mul_bind_group_layout) =
        create_elementwise_pipeline(&device, WGSL_MUL, "mul");
    let (scale_pipeline, scale_bind_group_layout) = create_scale_pipeline(&device);
    let scale_vec4_pipeline = create_scale_vec4_pipeline(&device, &scale_bind_group_layout);
    let (axpy_pipeline, axpy_bind_group_layout) = create_axpy_pipeline(&device);
    let (abs_pipeline, abs_bind_group_layout) = create_unary_pipeline(&device, WGSL_ABS, "abs");
    let (sqrt_pipeline, sqrt_bind_group_layout) = create_unary_pipeline(&device, WGSL_SQRT, "sqrt");
    let (div_pipeline, div_bind_group_layout) =
        create_elementwise_pipeline(&device, WGSL_DIV, "div");
    let (spmv_pipeline, spmv_bind_group_layout) = create_spmv_pipeline(&device);
    Ok(GpuContext {
        device,
        queue,
        max_compute_workgroup_size_x: max_x,
        max_compute_workgroup_size_y: max_y,
        max_compute_workgroup_size_z: max_z,
        elementwise_workgroup_size,
        matmul_pipeline,
        matmul_pipeline_16,
        matmul_tiled_pipeline_16,
        matmul_workgroup_size,
        matmul_bind_group_layout,
        dot_pipeline,
        dot_bind_group_layout,
        matvec_pipeline_256,
        matvec_pipeline_32,
        matvec_bind_group_layout,
        add_pipeline,
        add_vec4_pipeline,
        add_bind_group_layout,
        sub_pipeline,
        sub_bind_group_layout,
        mul_pipeline,
        mul_bind_group_layout,
        scale_pipeline,
        scale_vec4_pipeline,
        scale_bind_group_layout,
        axpy_pipeline,
        axpy_bind_group_layout,
        abs_pipeline,
        abs_bind_group_layout,
        sqrt_pipeline,
        sqrt_bind_group_layout,
        div_pipeline,
        div_bind_group_layout,
        spmv_pipeline,
        spmv_bind_group_layout,
        #[cfg(target_arch = "wasm32")]
        staging_pool: RefCell::new(Vec::new()),
    })
}

#[cfg(not(target_arch = "wasm32"))]
static FORTE_POOL: forte::ThreadPool = forte::ThreadPool::new();

#[allow(dead_code)]
pub(crate) fn init_inner(_config: Option<&GpuConfig>) -> Option<GpuContext> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        FORTE_POOL.populate();
        return FORTE_POOL.block_on(init_inner_async(_config)).ok();
    }
    #[cfg(target_arch = "wasm32")]
    return None;
}
