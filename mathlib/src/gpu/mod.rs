//! GPU compute backend for f32 matrix multiplication, vector dot product, and norm (WebGPU/wgpu).
//!
//! Enable with `--features gpu`. When initialized, `Matrix<f32>` × `Matrix<f32>` in
//! [`crate::operators`] and `Vector<f32>::dot` / `Vector<f32>::norm` will use the GPU
//! when applicable; otherwise CPU is used. Call [`init_blocking`] (native) or
//! [`init_async`] (wasm) before relying on GPU acceleration.

use crate::matrix::Matrix;
use crate::vector::Vector;

#[cfg(not(target_arch = "wasm32"))]
use crate::types::Storage;
#[cfg(not(target_arch = "wasm32"))]
use std::mem;
#[cfg(not(target_arch = "wasm32"))]
use wgpu::util::DeviceExt;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::OnceLock;

#[cfg(not(target_arch = "wasm32"))]
static GPU_CONTEXT: OnceLock<GpuContext> = OnceLock::new();

#[cfg(target_arch = "wasm32")]
std::thread_local! {
    static GPU_CONTEXT: std::cell::RefCell<Option<GpuContext>> = std::cell::RefCell::new(None);
}

#[cfg(target_arch = "wasm32")]
std::thread_local! {
    static LAST_GPU_INIT_ERROR: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
}

#[allow(dead_code)]
struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    matmul_pipeline: wgpu::ComputePipeline,
    matmul_pipeline_16: wgpu::ComputePipeline,
    matmul_workgroup_size: u32,
    matmul_bind_group_layout: wgpu::BindGroupLayout,
    dot_pipeline: wgpu::ComputePipeline,
    dot_bind_group_layout: wgpu::BindGroupLayout,
    matvec_pipeline: wgpu::ComputePipeline,
    matvec_bind_group_layout: wgpu::BindGroupLayout,
    add_pipeline: wgpu::ComputePipeline,
    add_bind_group_layout: wgpu::BindGroupLayout,
    sub_pipeline: wgpu::ComputePipeline,
    sub_bind_group_layout: wgpu::BindGroupLayout,
    mul_pipeline: wgpu::ComputePipeline,
    mul_bind_group_layout: wgpu::BindGroupLayout,
    scale_pipeline: wgpu::ComputePipeline,
    scale_bind_group_layout: wgpu::BindGroupLayout,
    axpy_pipeline: wgpu::ComputePipeline,
    axpy_bind_group_layout: wgpu::BindGroupLayout,
    abs_pipeline: wgpu::ComputePipeline,
    abs_bind_group_layout: wgpu::BindGroupLayout,
    sqrt_pipeline: wgpu::ComputePipeline,
    sqrt_bind_group_layout: wgpu::BindGroupLayout,
    div_pipeline: wgpu::ComputePipeline,
    div_bind_group_layout: wgpu::BindGroupLayout,
    spmv_pipeline: wgpu::ComputePipeline,
    spmv_bind_group_layout: wgpu::BindGroupLayout,
}

const WGSL_MATMUL: &str = r"
struct Params {
    M: u32,
    K: u32,
    N: u32,
}

@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read> B: array<f32>;
@group(0) @binding(2) var<storage, read_write> C: array<f32>;
@group(0) @binding(3) var<uniform> params: Params;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let j = gid.y;
    if (i >= params.M || j >= params.N) {
        return;
    }
    var sum = 0.0;
    for (var k = 0u; k < params.K; k++) {
        sum += A[k * params.M + i] * B[j * params.K + k];
    }
    C[j * params.M + i] = sum;
}
";

const WGSL_DOT: &str = r"
struct DotParams {
    n: u32,
}

@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read> B: array<f32>;
@group(0) @binding(2) var<storage, read_write> result: array<f32>;
@group(0) @binding(3) var<uniform> params: DotParams;

var<workgroup> partial: array<f32, 256>;

@compute @workgroup_size(256, 1, 1)
fn main(@builtin(local_invocation_id) lid: vec3<u32>) {
    let i = lid.x;
    var sum = 0.0;
    var idx = i;
    while (idx < params.n) {
        sum += A[idx] * B[idx];
        idx += 256u;
    }
    partial[i] = sum;
    workgroupBarrier();
    var stride = 128u;
    for (var s = 0; s < 8; s++) {
        if (i < stride) {
            partial[i] += partial[i + stride];
        }
        workgroupBarrier();
        stride = stride / 2u;
    }
    if (i == 0u) {
        result[0] = partial[0];
    }
}
";

const WGSL_MATVEC: &str = r"
struct MatvecParams {
    m: u32,
    n: u32,
}

@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read> v: array<f32>;
@group(0) @binding(2) var<storage, read_write> y: array<f32>;
@group(0) @binding(3) var<uniform> params: MatvecParams;

@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.m) {
        return;
    }
    var sum = 0.0;
    for (var k = 0u; k < params.n; k++) {
        sum += A[i + k * params.m] * v[k];
    }
    y[i] = sum;
}
";

const WGSL_ADD: &str = r"
struct AddParams {
    n: u32,
}

@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read> B: array<f32>;
@group(0) @binding(2) var<storage, read_write> C: array<f32>;
@group(0) @binding(3) var<uniform> params: AddParams;

@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) {
        return;
    }
    C[i] = A[i] + B[i];
}
";

const WGSL_SUB: &str = r"
struct SubParams {
    n: u32,
}

@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read> B: array<f32>;
@group(0) @binding(2) var<storage, read_write> C: array<f32>;
@group(0) @binding(3) var<uniform> params: SubParams;

@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) {
        return;
    }
    C[i] = A[i] - B[i];
}
";

const WGSL_MUL: &str = r"
struct MulParams { n: u32, }
@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read> B: array<f32>;
@group(0) @binding(2) var<storage, read_write> C: array<f32>;
@group(0) @binding(3) var<uniform> params: MulParams;
@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) { return; }
    C[i] = A[i] * B[i];
}
";

const WGSL_SCALE: &str = r"
struct ScaleParams { n: u32, alpha: f32, }
@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read_write> C: array<f32>;
@group(0) @binding(2) var<uniform> params: ScaleParams;
@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) { return; }
    C[i] = params.alpha * A[i];
}
";

const WGSL_AXPY: &str = r"
struct AxpyParams { n: u32, alpha: f32, }
@group(0) @binding(0) var<storage, read> x: array<f32>;
@group(0) @binding(1) var<storage, read> y: array<f32>;
@group(0) @binding(2) var<storage, read_write> z: array<f32>;
@group(0) @binding(3) var<uniform> params: AxpyParams;
@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) { return; }
    z[i] = params.alpha * x[i] + y[i];
}
";

const WGSL_ABS: &str = r"
struct AbsParams { n: u32, }
@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read_write> C: array<f32>;
@group(0) @binding(2) var<uniform> params: AbsParams;
@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) { return; }
    C[i] = abs(A[i]);
}
";

const WGSL_SQRT: &str = r"
struct SqrtParams { n: u32, }
@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read_write> C: array<f32>;
@group(0) @binding(2) var<uniform> params: SqrtParams;
@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) { return; }
    C[i] = sqrt(A[i]);
}
";

const WGSL_DIV: &str = r"
struct DivParams { n: u32, }
@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read> B: array<f32>;
@group(0) @binding(2) var<storage, read_write> C: array<f32>;
@group(0) @binding(3) var<uniform> params: DivParams;
@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) { return; }
    C[i] = select(0.0, A[i] / B[i], B[i] != 0.0);
}
";

const WGSL_SPMV: &str = r"
struct SpmvParams { m: u32, n: u32, nnz: u32, }
@group(0) @binding(0) var<storage, read> row_ptr: array<u32>;
@group(0) @binding(1) var<storage, read> col_ind: array<u32>;
@group(0) @binding(2) var<storage, read> values: array<f32>;
@group(0) @binding(3) var<storage, read> x: array<f32>;
@group(0) @binding(4) var<storage, read_write> y: array<f32>;
@group(0) @binding(5) var<uniform> params: SpmvParams;
@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.m) { return; }
    var sum = 0.0;
    let start = row_ptr[i];
    let end = row_ptr[i + 1u];
    for (var k = start; k < end; k++) {
        sum += values[k] * x[col_ind[k]];
    }
    y[i] = sum;
}
";

/// Returns whether the GPU context is initialized and usable.
#[must_use]
pub fn is_available() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    return GPU_CONTEXT.get().is_some();
    #[cfg(target_arch = "wasm32")]
    return GPU_CONTEXT.with(|c| c.borrow().is_some());
}

/// Returns the last GPU init failure message, if any (WASM only). Clears the stored message after read.
#[must_use]
pub fn last_init_error() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    return LAST_GPU_INIT_ERROR.with(|c| c.borrow_mut().take());
    #[cfg(not(target_arch = "wasm32"))]
    None
}

/// Initialize the GPU context (native). Blocks until the device is ready.
/// Call once before using GPU-accelerated matmul. Returns `true` if initialization succeeded.
///
/// # Errors
///
/// Fails if no WebGPU/Vulkan/Metal/D3D adapter is available or device creation fails.
#[cfg(not(target_arch = "wasm32"))]
pub fn init_blocking() -> bool {
    if GPU_CONTEXT.get().is_some() {
        return true;
    }
    let ctx = match init_inner() {
        Some(c) => c,
        None => return false,
    };
    GPU_CONTEXT.set(ctx).is_ok()
}

#[cfg(target_arch = "wasm32")]
fn set_last_init_error(msg: String) {
    let _ = LAST_GPU_INIT_ERROR.with(|c| *c.borrow_mut() = Some(msg));
}

async fn init_inner_async() -> Option<GpuContext> {
    let instance = wgpu::Instance::default();
    let adapter = match instance
        // Use Default to avoid "high performance" hint; Chrome on Windows still may log that powerPreference is ignored (crbug.com/369219127).
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
    {
        Ok(a) => a,
        Err(_) => {
            #[cfg(target_arch = "wasm32")]
            set_last_init_error(
                "No WebGPU adapter. Use HTTPS or localhost and ensure WebGPU is enabled in the browser.".to_string(),
            );
            return None;
        }
    };
    let (device, queue) = match adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("mathlib gpu"),
            required_features: wgpu::Features::empty(),
            required_limits: {
                #[cfg(target_arch = "wasm32")]
                {
                    adapter.limits()
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    wgpu::Limits::default()
                }
            },
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::default(),
        })
        .await
    {
        Ok(pair) => pair,
        Err(_e) => {
            #[cfg(target_arch = "wasm32")]
            set_last_init_error(format!("WebGPU device creation failed: {}", _e));
            return None;
        }
    };
    let (matmul_pipeline, matmul_pipeline_16, matmul_bind_group_layout) = create_matmul_pipeline(&device);
    let limits = device.limits();
    let matmul_workgroup_size = if limits.max_compute_workgroup_size_x >= 16
        && limits.max_compute_workgroup_size_y >= 16
    {
        16u32
    } else {
        8u32
    };
    let (dot_pipeline, dot_bind_group_layout) = create_dot_pipeline(&device);
    let (matvec_pipeline, matvec_bind_group_layout) = create_matvec_pipeline(&device);
    let (add_pipeline, add_bind_group_layout) = create_add_pipeline(&device);
    let (sub_pipeline, sub_bind_group_layout) = create_sub_pipeline(&device);
    let (mul_pipeline, mul_bind_group_layout) =
        create_elementwise_pipeline(&device, WGSL_MUL, "mul");
    let (scale_pipeline, scale_bind_group_layout) = create_scale_pipeline(&device);
    let (axpy_pipeline, axpy_bind_group_layout) = create_axpy_pipeline(&device);
    let (abs_pipeline, abs_bind_group_layout) = create_unary_pipeline(&device, WGSL_ABS, "abs");
    let (sqrt_pipeline, sqrt_bind_group_layout) = create_unary_pipeline(&device, WGSL_SQRT, "sqrt");
    let (div_pipeline, div_bind_group_layout) =
        create_elementwise_pipeline(&device, WGSL_DIV, "div");
    let (spmv_pipeline, spmv_bind_group_layout) = create_spmv_pipeline(&device);
    Some(GpuContext {
        device,
        queue,
        matmul_pipeline,
        matmul_pipeline_16,
        matmul_workgroup_size,
        matmul_bind_group_layout,
        dot_pipeline,
        dot_bind_group_layout,
        matvec_pipeline,
        matvec_bind_group_layout,
        add_pipeline,
        add_bind_group_layout,
        sub_pipeline,
        sub_bind_group_layout,
        mul_pipeline,
        mul_bind_group_layout,
        scale_pipeline,
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
    })
}

#[allow(dead_code)]
fn init_inner() -> Option<GpuContext> {
    #[cfg(not(target_arch = "wasm32"))]
    return pollster::block_on(init_inner_async());
    #[cfg(target_arch = "wasm32")]
    return None;
}

#[cfg(target_arch = "wasm32")]
fn set_context(ctx: GpuContext) {
    GPU_CONTEXT.with(|c| *c.borrow_mut() = Some(ctx));
}

/// Initialize the GPU context asynchronously (for wasm). Returns a future that resolves when
/// the device is ready. On native, prefer [`init_blocking`].
#[cfg(target_arch = "wasm32")]
pub fn init_async() -> impl std::future::Future<Output = bool> {
    async move {
        if GPU_CONTEXT.with(|c| c.borrow().is_some()) {
            return true;
        }
        let ctx = init_inner_async().await;
        match ctx {
            Some(c) => {
                set_context(c);
                true
            }
            None => false,
        }
    }
}

fn create_matmul_pipeline(
    device: &wgpu::Device,
) -> (wgpu::ComputePipeline, wgpu::ComputePipeline, wgpu::BindGroupLayout) {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("mathlib matmul layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("mathlib matmul pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        immediate_size: 0,
    });
    let shader_8 = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("mathlib matmul shader 8x8"),
        source: wgpu::ShaderSource::Wgsl(WGSL_MATMUL.into()),
    });
    let pipeline_8 = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("mathlib matmul pipeline 8x8"),
        layout: Some(&pipeline_layout),
        module: &shader_8,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let shader_16 = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("mathlib matmul shader 16x16"),
        source: wgpu::ShaderSource::Wgsl(WGSL_MATMUL_16.into()),
    });
    let pipeline_16 = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("mathlib matmul pipeline 16x16"),
        layout: Some(&pipeline_layout),
        module: &shader_16,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    (pipeline_8, pipeline_16, bind_group_layout)
}

fn create_dot_pipeline(device: &wgpu::Device) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("mathlib dot layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("mathlib dot pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("mathlib dot shader"),
        source: wgpu::ShaderSource::Wgsl(WGSL_DOT.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("mathlib dot pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    (pipeline, bind_group_layout)
}

fn create_matvec_pipeline(device: &wgpu::Device) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("mathlib matvec layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("mathlib matvec pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("mathlib matvec shader"),
        source: wgpu::ShaderSource::Wgsl(WGSL_MATVEC.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("mathlib matvec pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    (pipeline, bind_group_layout)
}

fn create_add_pipeline(device: &wgpu::Device) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
    create_elementwise_pipeline(device, WGSL_ADD, "add")
}

fn create_sub_pipeline(device: &wgpu::Device) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
    create_elementwise_pipeline(device, WGSL_SUB, "sub")
}

fn create_scale_pipeline(device: &wgpu::Device) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("mathlib scale layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("mathlib scale pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("mathlib scale shader"),
        source: wgpu::ShaderSource::Wgsl(WGSL_SCALE.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("mathlib scale pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    (pipeline, bind_group_layout)
}

fn create_axpy_pipeline(device: &wgpu::Device) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("mathlib axpy layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("mathlib axpy pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("mathlib axpy shader"),
        source: wgpu::ShaderSource::Wgsl(WGSL_AXPY.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("mathlib axpy pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    (pipeline, bind_group_layout)
}

fn create_spmv_pipeline(device: &wgpu::Device) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("mathlib spmv layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("mathlib spmv pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("mathlib spmv shader"),
        source: wgpu::ShaderSource::Wgsl(WGSL_SPMV.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("mathlib spmv pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    (pipeline, bind_group_layout)
}

fn create_unary_pipeline(
    device: &wgpu::Device,
    wgsl: &str,
    label: &str,
) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(&format!("mathlib {} layout", label)),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(&format!("mathlib {} pipeline layout", label)),
        bind_group_layouts: &[&bind_group_layout],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&format!("mathlib {} shader", label)),
        source: wgpu::ShaderSource::Wgsl(wgsl.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(&format!("mathlib {} pipeline", label)),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    (pipeline, bind_group_layout)
}

fn create_elementwise_pipeline(
    device: &wgpu::Device,
    wgsl: &str,
    label: &str,
) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(&format!("mathlib {} layout", label)),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(&format!("mathlib {} pipeline layout", label)),
        bind_group_layouts: &[&bind_group_layout],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&format!("mathlib {} shader", label)),
        source: wgpu::ShaderSource::Wgsl(wgsl.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(&format!("mathlib {} pipeline", label)),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    (pipeline, bind_group_layout)
}

/// Tries to compute `a * b` on the GPU. Returns `None` if GPU is not initialized,
/// matrices are not column-major, any dimension is zero, or on wasm (readback is async there).
#[must_use]
pub fn try_matmul_f32(_a: &Matrix<f32>, _b: &Matrix<f32>) -> Option<Matrix<f32>> {
    #[cfg(target_arch = "wasm32")]
    return None;
    #[cfg(not(target_arch = "wasm32"))]
    {
        let ctx = GPU_CONTEXT.get()?;
        let (a, b) = (_a, _b);
        if a.cols() != b.rows() || a.rows() == 0 || a.cols() == 0 || b.cols() == 0 {
            return None;
        }
        if a.storage != Storage::Column || b.storage != Storage::Column {
            return None;
        }
        let m = a.rows() as u32;
        let k = a.cols() as u32;
        let n = b.cols() as u32;
        let _size_a = (m as usize).checked_mul(k as usize)?;
        let _size_b = (k as usize).checked_mul(n as usize)?;
        let size_c = (m as usize).checked_mul(n as usize)?;

        let buf_a = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("matmul A"),
                contents: unsafe {
                    std::slice::from_raw_parts(
                        a.data().as_ptr().cast(),
                        a.data().len() * mem::size_of::<f32>(),
                    )
                },
                usage: wgpu::BufferUsages::STORAGE,
            });
        let buf_b = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("matmul B"),
                contents: unsafe {
                    std::slice::from_raw_parts(
                        b.data().as_ptr().cast(),
                        b.data().len() * mem::size_of::<f32>(),
                    )
                },
                usage: wgpu::BufferUsages::STORAGE,
            });
        let buf_c = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("matmul C"),
            size: (size_c * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        #[repr(C)]
        struct Params {
            m: u32,
            k: u32,
            n: u32,
        }
        let params = Params { m, k, n };
        let param_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts((&params as *const Params).cast(), mem::size_of::<Params>())
        };
        let buf_params = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("matmul params"),
                contents: param_bytes,
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("matmul bind group"),
            layout: &ctx.matmul_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buf_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buf_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buf_c.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buf_params.as_entire_binding(),
                },
            ],
        });
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("matmul encoder"),
            });
        {
            let ws = ctx.matmul_workgroup_size;
            let pipeline = if ws >= 16 {
                &ctx.matmul_pipeline_16
            } else {
                &ctx.matmul_pipeline
            };
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("matmul pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            let wg_x = (m + ws - 1) / ws;
            let wg_y = (n + ws - 1) / ws;
            pass.dispatch_workgroups(wg_x, wg_y, 1);
        }
        let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("matmul staging"),
            size: (size_c * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(
            &buf_c,
            0,
            &staging,
            0,
            (size_c * std::mem::size_of::<f32>()) as u64,
        );
        ctx.queue.submit(std::iter::once(encoder.finish()));

        let buffer_slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
        let _ = ctx.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
        rx.recv().ok().and_then(|r| r.ok())?;
        let view = buffer_slice.get_mapped_range();
        let data: &[f32] = unsafe { std::slice::from_raw_parts(view.as_ptr().cast(), size_c) };
        let mut out = Matrix::with_storage(a.rows(), b.cols(), Storage::Column);
        out.data_mut().copy_from_slice(data);
        drop(view);
        staging.unmap();
        Some(out)
    }
}

/// Tries to compute dot product of `a` and `b` on the GPU. Returns `None` if GPU is not
/// initialized, vectors are empty or length-mismatched, or on wasm (readback is async there).
#[must_use]
pub fn try_dot_f32(_a: &Vector<f32>, _b: &Vector<f32>) -> Option<f32> {
    #[cfg(target_arch = "wasm32")]
    return None;
    #[cfg(not(target_arch = "wasm32"))]
    {
        let ctx = GPU_CONTEXT.get()?;
        let (a, b) = (_a, _b);
        let n = a.rows();
        if n != b.rows() || n == 0 {
            return None;
        }
        if n > 256 * 256 {
            return None;
        }
        let buf_a = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("dot A"),
                contents: unsafe {
                    std::slice::from_raw_parts(a.data().as_ptr().cast(), n * mem::size_of::<f32>())
                },
                usage: wgpu::BufferUsages::STORAGE,
            });
        let buf_b = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("dot B"),
                contents: unsafe {
                    std::slice::from_raw_parts(b.data().as_ptr().cast(), n * mem::size_of::<f32>())
                },
                usage: wgpu::BufferUsages::STORAGE,
            });
        let buf_result = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("dot result"),
            size: mem::size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        #[repr(C)]
        struct DotParams {
            n: u32,
        }
        let params = DotParams { n: n as u32 };
        let param_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                (&params as *const DotParams).cast(),
                mem::size_of::<DotParams>(),
            )
        };
        let buf_params = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("dot params"),
                contents: param_bytes,
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("dot bind group"),
            layout: &ctx.dot_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buf_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buf_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buf_result.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buf_params.as_entire_binding(),
                },
            ],
        });
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("dot encoder"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("dot pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&ctx.dot_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("dot staging"),
            size: mem::size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(&buf_result, 0, &staging, 0, mem::size_of::<f32>() as u64);
        ctx.queue.submit(std::iter::once(encoder.finish()));
        let buffer_slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
        let _ = ctx.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
        rx.recv().ok().and_then(|r| r.ok())?;
        let view = buffer_slice.get_mapped_range();
        let result = unsafe { *view.as_ptr().cast::<f32>() };
        drop(view);
        staging.unmap();
        Some(result)
    }
}

/// Tries to compute `norm(v) = sqrt(dot(v, v))` on the GPU.
#[must_use]
pub fn try_norm_f32(v: &Vector<f32>) -> Option<f32> {
    try_dot_f32(v, v).map(|d| d.sqrt())
}

/// Tries to compute matrix-vector product `y = A * v` on the GPU. Returns `None` if GPU is not
/// initialized, dimensions mismatch, A is not column-major, or on wasm.
#[must_use]
pub fn try_matvec_f32(_a: &Matrix<f32>, _v: &Vector<f32>) -> Option<Vector<f32>> {
    #[cfg(target_arch = "wasm32")]
    return None;
    #[cfg(not(target_arch = "wasm32"))]
    {
        let ctx = GPU_CONTEXT.get()?;
        let (a, v) = (_a, _v);
        let m = a.rows();
        let n = a.cols();
        if n != v.rows() || m == 0 || n == 0 {
            return None;
        }
        if a.storage != Storage::Column {
            return None;
        }
        let buf_a = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("matvec A"),
                contents: unsafe {
                    std::slice::from_raw_parts(
                        a.data().as_ptr().cast(),
                        m * n * mem::size_of::<f32>(),
                    )
                },
                usage: wgpu::BufferUsages::STORAGE,
            });
        let buf_v = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("matvec v"),
                contents: unsafe {
                    std::slice::from_raw_parts(v.data().as_ptr().cast(), n * mem::size_of::<f32>())
                },
                usage: wgpu::BufferUsages::STORAGE,
            });
        let buf_y = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("matvec y"),
            size: (m * mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        #[repr(C)]
        struct MatvecParams {
            m: u32,
            n: u32,
        }
        let params = MatvecParams {
            m: m as u32,
            n: n as u32,
        };
        let param_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                (&params as *const MatvecParams).cast(),
                mem::size_of::<MatvecParams>(),
            )
        };
        let buf_params = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("matvec params"),
                contents: param_bytes,
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("matvec bind group"),
            layout: &ctx.matvec_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buf_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buf_v.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buf_y.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buf_params.as_entire_binding(),
                },
            ],
        });
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("matvec encoder"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("matvec pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&ctx.matvec_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            let wg_x = (m as u32 + 255) / 256;
            pass.dispatch_workgroups(wg_x, 1, 1);
        }
        let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("matvec staging"),
            size: (m * mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(&buf_y, 0, &staging, 0, (m * mem::size_of::<f32>()) as u64);
        ctx.queue.submit(std::iter::once(encoder.finish()));
        let buffer_slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
        let _ = ctx.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
        rx.recv().ok().and_then(|r| r.ok())?;
        let view = buffer_slice.get_mapped_range();
        let data: &[f32] = unsafe { std::slice::from_raw_parts(view.as_ptr().cast(), m) };
        let mut out = Vector::with_capacity(m);
        out.data_mut().copy_from_slice(data);
        drop(view);
        staging.unmap();
        Some(out)
    }
}

/// Tries to compute element-wise add `C = A + B` on the GPU. Works for matrices and vectors
/// (same layout). Returns `None` if GPU is not initialized, size mismatch, or on wasm.
#[must_use]
pub fn try_add_f32(_a: &[f32], _b: &[f32]) -> Option<Vec<f32>> {
    #[cfg(target_arch = "wasm32")]
    return None;
    #[cfg(not(target_arch = "wasm32"))]
    try_elementwise_f32_impl(_a, _b, ElemOp::Add)
}

/// Tries to compute element-wise sub `C = A - B` on the GPU.
#[must_use]
pub fn try_sub_f32(_a: &[f32], _b: &[f32]) -> Option<Vec<f32>> {
    #[cfg(target_arch = "wasm32")]
    return None;
    #[cfg(not(target_arch = "wasm32"))]
    try_elementwise_f32_impl(_a, _b, ElemOp::Sub)
}

/// Tries to compute element-wise multiply `C = A .* B` on the GPU.
#[must_use]
pub fn try_mul_f32(_a: &[f32], _b: &[f32]) -> Option<Vec<f32>> {
    #[cfg(target_arch = "wasm32")]
    return None;
    #[cfg(not(target_arch = "wasm32"))]
    try_elementwise_f32_impl(_a, _b, ElemOp::Mul)
}

/// Tries to compute element-wise divide `C = A ./ B` on the GPU. Zero in B yields 0 in C.
#[must_use]
pub fn try_div_f32(_a: &[f32], _b: &[f32]) -> Option<Vec<f32>> {
    #[cfg(target_arch = "wasm32")]
    return None;
    #[cfg(not(target_arch = "wasm32"))]
    try_elementwise_f32_impl(_a, _b, ElemOp::Div)
}

/// Tries to compute scalar multiply `C = alpha * A` on the GPU.
#[must_use]
pub fn try_scale_f32(_alpha: f32, _a: &[f32]) -> Option<Vec<f32>> {
    #[cfg(target_arch = "wasm32")]
    return None;
    #[cfg(not(target_arch = "wasm32"))]
    try_scale_f32_impl(_alpha, _a)
}

#[cfg(not(target_arch = "wasm32"))]
fn try_scale_f32_impl(alpha: f32, a: &[f32]) -> Option<Vec<f32>> {
    let ctx = GPU_CONTEXT.get()?;
    let n = a.len();
    if n == 0 {
        return None;
    }
    let buf_a = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("scale A"),
            contents: unsafe {
                std::slice::from_raw_parts(a.as_ptr().cast(), n * mem::size_of::<f32>())
            },
            usage: wgpu::BufferUsages::STORAGE,
        });
    let buf_c = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scale C"),
        size: (n * mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    #[repr(C)]
    struct ScaleParams {
        n: u32,
        alpha: f32,
    }
    let params = ScaleParams { n: n as u32, alpha };
    let param_bytes: &[u8] = unsafe {
        std::slice::from_raw_parts(
            (&params as *const ScaleParams).cast(),
            mem::size_of::<ScaleParams>(),
        )
    };
    let buf_params = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("scale params"),
            contents: param_bytes,
            usage: wgpu::BufferUsages::UNIFORM,
        });
    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("scale bind group"),
        layout: &ctx.scale_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buf_a.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buf_c.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: buf_params.as_entire_binding(),
            },
        ],
    });
    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("scale encoder"),
        });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("scale pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&ctx.scale_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        let wg_x = (n as u32 + 255) / 256;
        pass.dispatch_workgroups(wg_x, 1, 1);
    }
    let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scale staging"),
        size: (n * mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(&buf_c, 0, &staging, 0, (n * mem::size_of::<f32>()) as u64);
    ctx.queue.submit(std::iter::once(encoder.finish()));
    let buffer_slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
    let _ = ctx.device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    rx.recv().ok().and_then(|r| r.ok())?;
    let view = buffer_slice.get_mapped_range();
    let result: Vec<f32> = unsafe { std::slice::from_raw_parts(view.as_ptr().cast(), n) }.to_vec();
    drop(view);
    staging.unmap();
    Some(result)
}

/// Tries to compute AXPY `z = alpha * x + y` on the GPU.
#[must_use]
pub fn try_axpy_f32(_alpha: f32, _x: &[f32], _y: &[f32]) -> Option<Vec<f32>> {
    #[cfg(target_arch = "wasm32")]
    return None;
    #[cfg(not(target_arch = "wasm32"))]
    try_axpy_f32_impl(_alpha, _x, _y)
}

#[cfg(not(target_arch = "wasm32"))]
fn try_axpy_f32_impl(alpha: f32, x: &[f32], y: &[f32]) -> Option<Vec<f32>> {
    let ctx = GPU_CONTEXT.get()?;
    let n = x.len();
    if n != y.len() || n == 0 {
        return None;
    }
    let buf_x = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("axpy x"),
            contents: unsafe {
                std::slice::from_raw_parts(x.as_ptr().cast(), n * mem::size_of::<f32>())
            },
            usage: wgpu::BufferUsages::STORAGE,
        });
    let buf_y = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("axpy y"),
            contents: unsafe {
                std::slice::from_raw_parts(y.as_ptr().cast(), n * mem::size_of::<f32>())
            },
            usage: wgpu::BufferUsages::STORAGE,
        });
    let buf_z = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("axpy z"),
        size: (n * mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    #[repr(C)]
    struct AxpyParams {
        n: u32,
        alpha: f32,
    }
    let params = AxpyParams { n: n as u32, alpha };
    let param_bytes: &[u8] = unsafe {
        std::slice::from_raw_parts(
            (&params as *const AxpyParams).cast(),
            mem::size_of::<AxpyParams>(),
        )
    };
    let buf_params = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("axpy params"),
            contents: param_bytes,
            usage: wgpu::BufferUsages::UNIFORM,
        });
    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("axpy bind group"),
        layout: &ctx.axpy_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buf_x.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buf_y.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: buf_z.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: buf_params.as_entire_binding(),
            },
        ],
    });
    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("axpy encoder"),
        });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("axpy pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&ctx.axpy_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        let wg_x = (n as u32 + 255) / 256;
        pass.dispatch_workgroups(wg_x, 1, 1);
    }
    let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("axpy staging"),
        size: (n * mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(&buf_z, 0, &staging, 0, (n * mem::size_of::<f32>()) as u64);
    ctx.queue.submit(std::iter::once(encoder.finish()));
    let buffer_slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
    let _ = ctx.device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    rx.recv().ok().and_then(|r| r.ok())?;
    let view = buffer_slice.get_mapped_range();
    let result: Vec<f32> = unsafe { std::slice::from_raw_parts(view.as_ptr().cast(), n) }.to_vec();
    drop(view);
    staging.unmap();
    Some(result)
}

/// Tries to compute element-wise abs `C = |A|` on the GPU.
#[must_use]
pub fn try_abs_f32(_a: &[f32]) -> Option<Vec<f32>> {
    #[cfg(target_arch = "wasm32")]
    return None;
    #[cfg(not(target_arch = "wasm32"))]
    try_unary_f32_impl(_a, false)
}

/// Tries to compute element-wise sqrt `C = sqrt(A)` on the GPU.
#[must_use]
pub fn try_sqrt_f32(_a: &[f32]) -> Option<Vec<f32>> {
    #[cfg(target_arch = "wasm32")]
    return None;
    #[cfg(not(target_arch = "wasm32"))]
    try_unary_f32_impl(_a, true)
}

#[cfg(not(target_arch = "wasm32"))]
fn try_unary_f32_impl(a: &[f32], is_sqrt: bool) -> Option<Vec<f32>> {
    let ctx = GPU_CONTEXT.get()?;
    let n = a.len();
    if n == 0 {
        return None;
    }
    let (pipeline, layout) = if is_sqrt {
        (&ctx.sqrt_pipeline, &ctx.sqrt_bind_group_layout)
    } else {
        (&ctx.abs_pipeline, &ctx.abs_bind_group_layout)
    };
    let buf_a = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("unary A"),
            contents: unsafe {
                std::slice::from_raw_parts(a.as_ptr().cast(), n * mem::size_of::<f32>())
            },
            usage: wgpu::BufferUsages::STORAGE,
        });
    let buf_c = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("unary C"),
        size: (n * mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    #[repr(C)]
    struct UnaryParams {
        n: u32,
    }
    let params = UnaryParams { n: n as u32 };
    let param_bytes: &[u8] = unsafe {
        std::slice::from_raw_parts(
            (&params as *const UnaryParams).cast(),
            mem::size_of::<UnaryParams>(),
        )
    };
    let buf_params = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("unary params"),
            contents: param_bytes,
            usage: wgpu::BufferUsages::UNIFORM,
        });
    let op_label = if is_sqrt { "sqrt" } else { "abs" };
    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&format!("{} bind group", op_label)),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buf_a.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buf_c.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: buf_params.as_entire_binding(),
            },
        ],
    });
    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&format!("{} encoder", op_label)),
        });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(&format!("{} pass", op_label)),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        let wg_x = (n as u32 + 255) / 256;
        pass.dispatch_workgroups(wg_x, 1, 1);
    }
    let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("unary staging"),
        size: (n * mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(&buf_c, 0, &staging, 0, (n * mem::size_of::<f32>()) as u64);
    ctx.queue.submit(std::iter::once(encoder.finish()));
    let buffer_slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
    let _ = ctx.device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    rx.recv().ok().and_then(|r| r.ok())?;
    let view = buffer_slice.get_mapped_range();
    let result: Vec<f32> = unsafe { std::slice::from_raw_parts(view.as_ptr().cast(), n) }.to_vec();
    drop(view);
    staging.unmap();
    Some(result)
}

/// Tries to compute squared norm `dot(v, v)` on the GPU.
#[must_use]
pub fn try_squared_norm_f32(v: &Vector<f32>) -> Option<f32> {
    try_dot_f32(v, v)
}

/// Tries to compute sparse matrix-vector product `y = A * x` on the GPU (CRS format).
#[must_use]
pub fn try_spmv_f32(
    _sparse: &crate::SparseMatrixCRS<f32>,
    _x: &Vector<f32>,
) -> Option<Vector<f32>> {
    #[cfg(target_arch = "wasm32")]
    return None;
    #[cfg(not(target_arch = "wasm32"))]
    {
        let ctx = GPU_CONTEXT.get()?;
        let (sparse, x) = (_sparse, _x);
        let m = sparse.rows();
        let n = sparse.cols();
        if n != x.rows() || m == 0 || n == 0 {
            return None;
        }
        let row_ptr = sparse.row_ptr();
        let col_ind = sparse.col_ind();
        let vals = sparse.values();
        let nnz = vals.len();
        if row_ptr.len() < m + 1 {
            return None;
        }
        let buf_row = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("spmv row_ptr"),
                contents: unsafe {
                    std::slice::from_raw_parts(
                        row_ptr.as_ptr().cast(),
                        row_ptr.len() * mem::size_of::<u32>(),
                    )
                },
                usage: wgpu::BufferUsages::STORAGE,
            });
        let buf_col = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("spmv col_ind"),
                contents: unsafe {
                    std::slice::from_raw_parts(
                        col_ind.as_ptr().cast(),
                        col_ind.len() * mem::size_of::<u32>(),
                    )
                },
                usage: wgpu::BufferUsages::STORAGE,
            });
        let buf_vals = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("spmv values"),
                contents: unsafe {
                    std::slice::from_raw_parts(vals.as_ptr().cast(), nnz * mem::size_of::<f32>())
                },
                usage: wgpu::BufferUsages::STORAGE,
            });
        let buf_x = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("spmv x"),
                contents: unsafe {
                    std::slice::from_raw_parts(x.data().as_ptr().cast(), n * mem::size_of::<f32>())
                },
                usage: wgpu::BufferUsages::STORAGE,
            });
        let buf_y = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("spmv y"),
            size: (m * mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        #[repr(C)]
        struct SpmvParams {
            m: u32,
            n: u32,
            nnz: u32,
        }
        let params = SpmvParams {
            m: m as u32,
            n: n as u32,
            nnz: nnz as u32,
        };
        let param_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                (&params as *const SpmvParams).cast(),
                mem::size_of::<SpmvParams>(),
            )
        };
        let buf_params = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("spmv params"),
                contents: param_bytes,
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("spmv bind group"),
            layout: &ctx.spmv_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buf_row.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buf_col.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buf_vals.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buf_x.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: buf_y.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: buf_params.as_entire_binding(),
                },
            ],
        });
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("spmv encoder"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("spmv pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&ctx.spmv_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            let wg_x = (m as u32 + 255) / 256;
            pass.dispatch_workgroups(wg_x, 1, 1);
        }
        let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("spmv staging"),
            size: (m * mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(&buf_y, 0, &staging, 0, (m * mem::size_of::<f32>()) as u64);
        ctx.queue.submit(std::iter::once(encoder.finish()));
        let buffer_slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
        let _ = ctx.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
        rx.recv().ok().and_then(|r| r.ok())?;
        let view = buffer_slice.get_mapped_range();
        let data: &[f32] = unsafe { std::slice::from_raw_parts(view.as_ptr().cast(), m) };
        let mut out = Vector::with_capacity(m);
        out.data_mut().copy_from_slice(data);
        drop(view);
        staging.unmap();
        Some(out)
    }
}

#[cfg(not(target_arch = "wasm32"))]
enum ElemOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[cfg(not(target_arch = "wasm32"))]
fn try_elementwise_f32_impl(a: &[f32], b: &[f32], op: ElemOp) -> Option<Vec<f32>> {
    let ctx = GPU_CONTEXT.get()?;
    let n = a.len();
    if n != b.len() || n == 0 {
        return None;
    }
    let (pipeline, layout) = match op {
        ElemOp::Add => (&ctx.add_pipeline, &ctx.add_bind_group_layout),
        ElemOp::Sub => (&ctx.sub_pipeline, &ctx.sub_bind_group_layout),
        ElemOp::Mul => (&ctx.mul_pipeline, &ctx.mul_bind_group_layout),
        ElemOp::Div => (&ctx.div_pipeline, &ctx.div_bind_group_layout),
    };
    let buf_a = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("elem A"),
            contents: unsafe {
                std::slice::from_raw_parts(a.as_ptr().cast(), n * mem::size_of::<f32>())
            },
            usage: wgpu::BufferUsages::STORAGE,
        });
    let buf_b = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("elem B"),
            contents: unsafe {
                std::slice::from_raw_parts(b.as_ptr().cast(), n * mem::size_of::<f32>())
            },
            usage: wgpu::BufferUsages::STORAGE,
        });
    let buf_c = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("elem C"),
        size: (n * mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    #[repr(C)]
    struct ElemParams {
        n: u32,
    }
    let params = ElemParams { n: n as u32 };
    let param_bytes: &[u8] = unsafe {
        std::slice::from_raw_parts(
            (&params as *const ElemParams).cast(),
            mem::size_of::<ElemParams>(),
        )
    };
    let buf_params = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("elem params"),
            contents: param_bytes,
            usage: wgpu::BufferUsages::UNIFORM,
        });
    let op_label = match op {
        ElemOp::Add => "add",
        ElemOp::Sub => "sub",
        ElemOp::Mul => "mul",
        ElemOp::Div => "div",
    };
    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&format!("{} bind group", op_label)),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buf_a.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buf_b.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: buf_c.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: buf_params.as_entire_binding(),
            },
        ],
    });
    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(&format!("{} encoder", op_label)),
        });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(&format!("{} pass", op_label)),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        let wg_x = (n as u32 + 255) / 256;
        pass.dispatch_workgroups(wg_x, 1, 1);
    }
    let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("elem staging"),
        size: (n * mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(&buf_c, 0, &staging, 0, (n * mem::size_of::<f32>()) as u64);
    ctx.queue.submit(std::iter::once(encoder.finish()));
    let buffer_slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
    let _ = ctx.device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    rx.recv().ok().and_then(|r| r.ok())?;
    let view = buffer_slice.get_mapped_range();
    let result: Vec<f32> = unsafe { std::slice::from_raw_parts(view.as_ptr().cast(), n) }.to_vec();
    drop(view);
    staging.unmap();
    Some(result)
}
