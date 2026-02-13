//! GPU compute backend for f32 matrix multiplication, vector dot product, and norm (WebGPU/wgpu).
//!
//! Enable with `--features gpu`. When initialized, `Matrix<f32>` × `Matrix<f32>` in
//! [`crate::operators`] and `Vector<f32>::dot` / `Vector<f32>::norm` will use the GPU
//! when applicable; otherwise CPU is used. Call [`init_blocking`] (native) or
//! [`init_async`] (wasm) before relying on GPU acceleration.
//!
//! # Module layout
//!
//! - [`config`] — GpuConfig, PowerPreference
//! - [`shaders`] — WGSL source strings (modify for kernel tuning)
//! - [`pipelines`] — Pipeline creation
//! - [`context`] — GpuContext and init

mod config;
mod context;
mod pipelines;
mod shaders;

pub use config::{GpuConfig, PowerPreference};

use crate::matrix::Matrix;
use crate::types::Storage;
use crate::vector::Vector;

use context::{ELEMENTWISE_WORKGROUP_SIZE, GpuContext};

/// Maximum vector length for GPU dot/norm (conservative for buffer limits).
const MAX_DOT_LENGTH: usize = 1 << 24;

/// Default minimum vector length for GPU dot/norm. Below this, CPU (SIMD) is used.
/// Matches [`ExecutorThresholds::dot_len_min`] default; use [`AutoExecutor`] for custom thresholds.
pub const MIN_LEN_GPU_DOT: usize = 64 * 1024;

/// Default minimum elements (M*K*N) for GPU matmul. Use [`AutoExecutor`] for custom thresholds.
pub const MIN_ELEMENTS_GPU_MATMUL: usize = 128 * 128 * 128;

/// Default minimum elements (rows*cols) for GPU matvec. Use [`AutoExecutor`] for custom thresholds.
pub const MIN_ELEMENTS_GPU_MATVEC: usize = 256 * 256;

/// Default minimum length for GPU elementwise (add, sub, scale, axpy). Use [`AutoExecutor`] for custom thresholds.
pub const MIN_LEN_GPU_ELEMENTWISE: usize = 64 * 1024;

#[cfg(not(target_arch = "wasm32"))]
use std::mem;
#[cfg(not(target_arch = "wasm32"))]
use wgpu::util::DeviceExt;

#[cfg(feature = "gpu")]
use futures::channel::oneshot;

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

#[cfg(target_arch = "wasm32")]
fn set_last_init_error(msg: String) {
    let _ = LAST_GPU_INIT_ERROR.with(|c| *c.borrow_mut() = Some(msg));
}

#[cfg(target_arch = "wasm32")]
fn return_staging_to_pool(size: u64, buffer: wgpu::Buffer) {
    GPU_CONTEXT.with(|c| {
        if let Some(ref ctx) = *c.borrow() {
            ctx.return_staging(size, buffer);
        }
    });
}

/// Get a staging buffer (from pool on wasm32, create on native).
fn get_staging_buffer(ctx: &GpuContext, n_bytes: u64, label: &str) -> wgpu::Buffer {
    #[cfg(target_arch = "wasm32")]
    return ctx.get_staging_buffer(n_bytes, label);
    #[cfg(not(target_arch = "wasm32"))]
    ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: n_bytes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

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

#[cfg(target_arch = "wasm32")]
fn set_context(ctx: GpuContext) {
    GPU_CONTEXT.with(|c| *c.borrow_mut() = Some(ctx));
}

/// Initialize the GPU context (native). Blocks until the device is ready.
/// Call once before using GPU-accelerated matmul. Returns `true` if initialization succeeded.
/// Pass `Some(config)` to customize adapter selection and limits.
#[cfg(not(target_arch = "wasm32"))]
pub fn init_blocking(config: Option<&GpuConfig>) -> bool {
    if GPU_CONTEXT.get().is_some() {
        return true;
    }
    let ctx = match context::init_inner(config) {
        Some(c) => c,
        None => return false,
    };
    GPU_CONTEXT.set(ctx).is_ok()
}

/// Initialize the GPU context (native). Blocks until the device is ready.
/// Convenience overload that uses default config. Prefer [`init_blocking`] with explicit config.
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::missing_panics_doc)]
pub fn init_blocking_default() -> bool {
    init_blocking(None)
}

/// Initialize the GPU context asynchronously. Returns a future that resolves when the device is ready.
/// On wasm, call this before GPU ops; on native, prefer [`init_blocking`].
/// Pass `Some(config)` to customize adapter selection and limits.
pub fn init_async(config: Option<&GpuConfig>) -> impl std::future::Future<Output = bool> {
    let config = config.cloned();
    async move {
        #[cfg(target_arch = "wasm32")]
        {
            if GPU_CONTEXT.with(|c| c.borrow().is_some()) {
                return true;
            }
            match context::init_inner_async(config.as_ref()).await {
                Ok(c) => {
                    set_context(c);
                    true
                }
                Err(e) => {
                    set_last_init_error(e);
                    false
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if GPU_CONTEXT.get().is_some() {
                return true;
            }
            context::init_inner_async(config.as_ref())
                .await
                .map_or(false, |c| GPU_CONTEXT.set(c).is_ok())
        }
    }
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
                &ctx.matmul_tiled_pipeline_16
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

/// Tries to compute `a * b` on the GPU with async readback. Returns `None` if GPU is not
/// initialized, matrices are invalid, or readback fails. Works on both native and wasm32;
/// on wasm32 the callback from map_async runs on the event loop, so this must be awaited.
#[cfg(feature = "gpu")]
pub async fn try_matmul_f32_async(a: &Matrix<f32>, b: &Matrix<f32>) -> Option<Matrix<f32>> {
    let (staging, rx) = run_matmul_submit_and_map_async(a, b)?;
    rx.await.ok().and_then(|r| r.ok())?;
    let buffer_slice = staging.slice(..);
    let view = buffer_slice.get_mapped_range();
    let size_c = a.rows() * b.cols();
    let data: &[f32] = unsafe { std::slice::from_raw_parts(view.as_ptr().cast(), size_c) };
    let mut out = Matrix::with_storage(a.rows(), b.cols(), Storage::Column);
    out.data_mut().copy_from_slice(data);
    drop(view);
    staging.unmap();
    Some(out)
}

/// Runs the matmul compute and copy-to-staging, then map_async; returns (staging buffer, oneshot
/// receiver). Used by try_matmul_f32_async. Returns None if no context or validation fails.
#[cfg(feature = "gpu")]
fn run_matmul_submit_and_map_async(
    a: &Matrix<f32>,
    b: &Matrix<f32>,
) -> Option<(
    wgpu::Buffer,
    oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
)> {
    if a.cols() != b.rows() || a.rows() == 0 || a.cols() == 0 || b.cols() == 0 {
        return None;
    }
    if a.storage != Storage::Column || b.storage != Storage::Column {
        return None;
    }
    let m = a.rows() as u32;
    let k = a.cols() as u32;
    let n = b.cols() as u32;
    let size_c = (m as usize).checked_mul(n as usize)?;
    let size_a_bytes = (m as usize).checked_mul(k as usize)? * std::mem::size_of::<f32>();
    let size_b_bytes = (k as usize).checked_mul(n as usize)? * std::mem::size_of::<f32>();

    #[cfg(not(target_arch = "wasm32"))]
    let ctx_ref = GPU_CONTEXT.get()?;
    #[cfg(target_arch = "wasm32")]
    let (staging, rx) = GPU_CONTEXT.with(|c| {
        let ctx_ref = c.borrow();
        let ctx_ref = ctx_ref.as_ref()?;
        run_matmul_submit_and_map_async_inner(
            ctx_ref,
            a,
            b,
            m,
            n,
            size_c,
            size_a_bytes,
            size_b_bytes,
        )
    })?;
    #[cfg(target_arch = "wasm32")]
    return Some((staging, rx));

    #[cfg(not(target_arch = "wasm32"))]
    run_matmul_submit_and_map_async_inner(ctx_ref, a, b, m, n, size_c, size_a_bytes, size_b_bytes)
}

#[cfg(feature = "gpu")]
fn run_matmul_submit_and_map_async_inner(
    ctx_ref: &GpuContext,
    a: &Matrix<f32>,
    b: &Matrix<f32>,
    m: u32,
    n: u32,
    size_c: usize,
    size_a_bytes: usize,
    size_b_bytes: usize,
) -> Option<(
    wgpu::Buffer,
    oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
)> {
    let k = a.cols() as u32;

    let buf_a = ctx_ref.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("matmul A"),
        size: size_a_bytes as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let buf_b = ctx_ref.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("matmul B"),
        size: size_b_bytes as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx_ref.queue.write_buffer(&buf_a, 0, unsafe {
        std::slice::from_raw_parts(a.data().as_ptr().cast(), size_a_bytes)
    });
    ctx_ref.queue.write_buffer(&buf_b, 0, unsafe {
        std::slice::from_raw_parts(b.data().as_ptr().cast(), size_b_bytes)
    });

    let buf_c = ctx_ref.device.create_buffer(&wgpu::BufferDescriptor {
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
        std::slice::from_raw_parts(
            (&params as *const Params).cast(),
            std::mem::size_of::<Params>(),
        )
    };
    let buf_params = ctx_ref.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("matmul params"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx_ref.queue.write_buffer(&buf_params, 0, param_bytes);

    let bind_group = ctx_ref
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("matmul bind group"),
            layout: &ctx_ref.matmul_bind_group_layout,
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
    let mut encoder = ctx_ref
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("matmul encoder"),
        });
    let ws = ctx_ref.matmul_workgroup_size;
    let pipeline = if ws >= 16 {
        &ctx_ref.matmul_tiled_pipeline_16
    } else {
        &ctx_ref.matmul_pipeline
    };
    {
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
    let staging = ctx_ref.device.create_buffer(&wgpu::BufferDescriptor {
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
    ctx_ref.queue.submit(std::iter::once(encoder.finish()));

    let buffer_slice = staging.slice(..);
    let (tx, rx) = oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    Some((staging, rx))
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
        if n > MAX_DOT_LENGTH {
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

/// Tries to compute dot product on the GPU with async readback. Works on both native and wasm.
#[cfg(feature = "gpu")]
pub async fn try_dot_f32_async(a: &Vector<f32>, b: &Vector<f32>) -> Option<f32> {
    let (staging, rx) = run_dot_submit_and_map_async(a, b)?;
    rx.await.ok().and_then(|r| r.ok())?;
    let buffer_slice = staging.slice(..);
    let view = buffer_slice.get_mapped_range();
    let result = unsafe { *view.as_ptr().cast::<f32>() };
    drop(view);
    staging.unmap();
    #[cfg(target_arch = "wasm32")]
    return_staging_to_pool(std::mem::size_of::<f32>() as u64, staging);
    Some(result)
}

/// Tries to compute norm on the GPU with async readback.
#[cfg(feature = "gpu")]
pub async fn try_norm_f32_async(v: &Vector<f32>) -> Option<f32> {
    try_dot_f32_async(v, v).await.map(|d| d.sqrt())
}

#[cfg(feature = "gpu")]
fn run_dot_submit_and_map_async(
    a: &Vector<f32>,
    b: &Vector<f32>,
) -> Option<(
    wgpu::Buffer,
    oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
)> {
    let n = a.rows();
    if n != b.rows() || n == 0 || n > MAX_DOT_LENGTH {
        return None;
    }
    #[cfg(not(target_arch = "wasm32"))]
    return run_dot_submit_and_map_async_inner(GPU_CONTEXT.get()?, a, b, n);
    #[cfg(target_arch = "wasm32")]
    GPU_CONTEXT.with(|c| {
        let ctx_ref = c.borrow();
        let ctx_ref = ctx_ref.as_ref()?;
        run_dot_submit_and_map_async_inner(ctx_ref, a, b, n)
    })
}

#[cfg(feature = "gpu")]
fn run_dot_submit_and_map_async_inner(
    ctx: &GpuContext,
    a: &Vector<f32>,
    b: &Vector<f32>,
    n: usize,
) -> Option<(
    wgpu::Buffer,
    oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
)> {
    let n_bytes = n * std::mem::size_of::<f32>();
    let buf_a = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("dot A"),
        size: n_bytes as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let buf_b = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("dot B"),
        size: n_bytes as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&buf_a, 0, unsafe {
        std::slice::from_raw_parts(a.data().as_ptr().cast(), n_bytes)
    });
    ctx.queue.write_buffer(&buf_b, 0, unsafe {
        std::slice::from_raw_parts(b.data().as_ptr().cast(), n_bytes)
    });
    let buf_result = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("dot result"),
        size: std::mem::size_of::<f32>() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    #[repr(C)]
    struct DotParams {
        n: u32,
    }
    let params = DotParams { n: n as u32 };
    let param_bytes: &[u8] =
        unsafe { std::slice::from_raw_parts((&params as *const DotParams).cast(), 4) };
    let buf_params = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("dot params"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&buf_params, 0, param_bytes);
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
    let staging = get_staging_buffer(ctx, std::mem::size_of::<f32>() as u64, "dot staging");
    encoder.copy_buffer_to_buffer(
        &buf_result,
        0,
        &staging,
        0,
        std::mem::size_of::<f32>() as u64,
    );
    ctx.queue.submit(std::iter::once(encoder.finish()));
    let buffer_slice = staging.slice(..);
    let (tx, rx) = oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    Some((staging, rx))
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
            let (matvec_pipeline, wg_x) = match &ctx.matvec_pipeline_256 {
                Some(p) => (p, (m as u32 + 255) / 256),
                None => (&ctx.matvec_pipeline_32, (m as u32 + 31) / 32),
            };
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("matvec pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(matvec_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
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

/// Tries to compute matrix-vector product on the GPU with async readback. Works on both native and wasm.
#[cfg(feature = "gpu")]
pub async fn try_matvec_f32_async(a: &Matrix<f32>, v: &Vector<f32>) -> Option<Vector<f32>> {
    let (staging, rx) = run_matvec_submit_and_map_async(a, v)?;
    rx.await.ok().and_then(|r| r.ok())?;
    let m = a.rows();
    let buffer_slice = staging.slice(..);
    let view = buffer_slice.get_mapped_range();
    let data: &[f32] = unsafe { std::slice::from_raw_parts(view.as_ptr().cast(), m) };
    let mut out = Vector::with_capacity(m);
    out.data_mut().copy_from_slice(data);
    drop(view);
    staging.unmap();
    #[cfg(target_arch = "wasm32")]
    return_staging_to_pool((m * std::mem::size_of::<f32>()) as u64, staging);
    Some(out)
}

#[cfg(feature = "gpu")]
fn run_matvec_submit_and_map_async(
    a: &Matrix<f32>,
    v: &Vector<f32>,
) -> Option<(
    wgpu::Buffer,
    oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
)> {
    let m = a.rows();
    let n = a.cols();
    if n != v.rows() || m == 0 || n == 0 || a.storage != Storage::Column {
        return None;
    }
    #[cfg(not(target_arch = "wasm32"))]
    return run_matvec_submit_and_map_async_inner(GPU_CONTEXT.get()?, a, v, m, n);
    #[cfg(target_arch = "wasm32")]
    GPU_CONTEXT.with(|c| {
        let ctx_ref = c.borrow();
        let ctx_ref = ctx_ref.as_ref()?;
        run_matvec_submit_and_map_async_inner(ctx_ref, a, v, m, n)
    })
}

#[cfg(feature = "gpu")]
fn run_matvec_submit_and_map_async_inner(
    ctx: &GpuContext,
    a: &Matrix<f32>,
    v: &Vector<f32>,
    m: usize,
    n: usize,
) -> Option<(
    wgpu::Buffer,
    oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
)> {
    let buf_a = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("matvec A"),
        size: (m * n * std::mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let buf_v = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("matvec v"),
        size: (n * std::mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&buf_a, 0, unsafe {
        std::slice::from_raw_parts(a.data().as_ptr().cast(), m * n * std::mem::size_of::<f32>())
    });
    ctx.queue.write_buffer(&buf_v, 0, unsafe {
        std::slice::from_raw_parts(v.data().as_ptr().cast(), n * std::mem::size_of::<f32>())
    });
    let buf_y = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("matvec y"),
        size: (m * std::mem::size_of::<f32>()) as u64,
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
    let param_bytes: &[u8] =
        unsafe { std::slice::from_raw_parts((&params as *const MatvecParams).cast(), 8) };
    let buf_params = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("matvec params"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&buf_params, 0, param_bytes);
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
        let (matvec_pipeline, wg_x) = match &ctx.matvec_pipeline_256 {
            Some(p) => (p, (m as u32 + 255) / 256),
            None => (&ctx.matvec_pipeline_32, (m as u32 + 31) / 32),
        };
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("matvec pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(matvec_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(wg_x, 1, 1);
    }
    let m_bytes = (m * std::mem::size_of::<f32>()) as u64;
    let staging = get_staging_buffer(ctx, m_bytes, "matvec staging");
    encoder.copy_buffer_to_buffer(&buf_y, 0, &staging, 0, m_bytes);
    ctx.queue.submit(std::iter::once(encoder.finish()));
    let buffer_slice = staging.slice(..);
    let (tx, rx) = oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    Some((staging, rx))
}

/// Tries to compute element-wise add `C = A + B` on the GPU with async readback.
#[cfg(feature = "gpu")]
pub async fn try_add_f32_async(a: &[f32], b: &[f32]) -> Option<Vec<f32>> {
    let (staging, rx) = run_elementwise_submit_and_map_async(a, b, ElemOp::Add)?;
    rx.await.ok().and_then(|r| r.ok())?;
    let n = a.len();
    let buffer_slice = staging.slice(..);
    let view = buffer_slice.get_mapped_range();
    let result: Vec<f32> = unsafe { std::slice::from_raw_parts(view.as_ptr().cast(), n) }.to_vec();
    drop(view);
    staging.unmap();
    #[cfg(target_arch = "wasm32")]
    return_staging_to_pool((n * std::mem::size_of::<f32>()) as u64, staging);
    Some(result)
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

/// Tries to compute element-wise sub on the GPU with async readback.
#[cfg(feature = "gpu")]
pub async fn try_sub_f32_async(a: &[f32], b: &[f32]) -> Option<Vec<f32>> {
    let (staging, rx) = run_elementwise_submit_and_map_async(a, b, ElemOp::Sub)?;
    rx.await.ok().and_then(|r| r.ok())?;
    let n = a.len();
    let buffer_slice = staging.slice(..);
    let view = buffer_slice.get_mapped_range();
    let result: Vec<f32> = unsafe { std::slice::from_raw_parts(view.as_ptr().cast(), n) }.to_vec();
    drop(view);
    staging.unmap();
    Some(result)
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

/// Tries to compute scalar multiply on the GPU with async readback.
#[cfg(feature = "gpu")]
pub async fn try_scale_f32_async(alpha: f32, a: &[f32]) -> Option<Vec<f32>> {
    let (staging, rx) = run_scale_submit_and_map_async(alpha, a)?;
    rx.await.ok().and_then(|r| r.ok())?;
    let n = a.len();
    let buffer_slice = staging.slice(..);
    let view = buffer_slice.get_mapped_range();
    let result: Vec<f32> = unsafe { std::slice::from_raw_parts(view.as_ptr().cast(), n) }.to_vec();
    drop(view);
    staging.unmap();
    #[cfg(target_arch = "wasm32")]
    return_staging_to_pool((n * std::mem::size_of::<f32>()) as u64, staging);
    Some(result)
}

#[cfg(feature = "gpu")]
fn run_scale_submit_and_map_async(
    alpha: f32,
    a: &[f32],
) -> Option<(
    wgpu::Buffer,
    oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
)> {
    let n = a.len();
    if n == 0 {
        return None;
    }
    #[cfg(not(target_arch = "wasm32"))]
    return run_scale_submit_and_map_async_inner(GPU_CONTEXT.get()?, alpha, a, n);
    #[cfg(target_arch = "wasm32")]
    GPU_CONTEXT.with(|c| {
        let ctx_ref = c.borrow();
        let ctx_ref = ctx_ref.as_ref()?;
        run_scale_submit_and_map_async_inner(ctx_ref, alpha, a, n)
    })
}

#[cfg(feature = "gpu")]
fn run_scale_submit_and_map_async_inner(
    ctx: &GpuContext,
    alpha: f32,
    a: &[f32],
    n: usize,
) -> Option<(
    wgpu::Buffer,
    oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
)> {
    let buf_a = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scale A"),
        size: (n * std::mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&buf_a, 0, unsafe {
        std::slice::from_raw_parts(a.as_ptr().cast(), n * std::mem::size_of::<f32>())
    });
    let buf_c = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scale C"),
        size: (n * std::mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    #[repr(C)]
    struct ScaleParams {
        n: u32,
        alpha: f32,
    }
    let params = ScaleParams { n: n as u32, alpha };
    let param_bytes: &[u8] =
        unsafe { std::slice::from_raw_parts((&params as *const ScaleParams).cast(), 8) };
    let buf_params = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scale params"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&buf_params, 0, param_bytes);
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
        pass.set_pipeline(&ctx.scale_vec4_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        let wg_x = (n as u32 + 1023) / 1024;
        pass.dispatch_workgroups(wg_x, 1, 1);
    }
    let n_bytes = (n * std::mem::size_of::<f32>()) as u64;
    let staging = get_staging_buffer(ctx, n_bytes, "scale staging");
    encoder.copy_buffer_to_buffer(&buf_c, 0, &staging, 0, n_bytes);
    ctx.queue.submit(std::iter::once(encoder.finish()));
    let buffer_slice = staging.slice(..);
    let (tx, rx) = oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    Some((staging, rx))
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
        pass.set_pipeline(&ctx.scale_vec4_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        let wg_x = (n as u32 + 1023) / 1024;
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

/// Tries to compute AXPY on the GPU with async readback.
#[cfg(feature = "gpu")]
pub async fn try_axpy_f32_async(alpha: f32, x: &[f32], y: &[f32]) -> Option<Vec<f32>> {
    let (staging, rx) = run_axpy_submit_and_map_async(alpha, x, y)?;
    rx.await.ok().and_then(|r| r.ok())?;
    let n = x.len();
    let buffer_slice = staging.slice(..);
    let view = buffer_slice.get_mapped_range();
    let result: Vec<f32> = unsafe { std::slice::from_raw_parts(view.as_ptr().cast(), n) }.to_vec();
    drop(view);
    staging.unmap();
    Some(result)
}

#[cfg(feature = "gpu")]
fn run_axpy_submit_and_map_async(
    alpha: f32,
    x: &[f32],
    y: &[f32],
) -> Option<(
    wgpu::Buffer,
    oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
)> {
    let n = x.len();
    if n != y.len() || n == 0 {
        return None;
    }
    #[cfg(not(target_arch = "wasm32"))]
    return run_axpy_submit_and_map_async_inner(GPU_CONTEXT.get()?, alpha, x, y, n);
    #[cfg(target_arch = "wasm32")]
    GPU_CONTEXT.with(|c| {
        let ctx_ref = c.borrow();
        let ctx_ref = ctx_ref.as_ref()?;
        run_axpy_submit_and_map_async_inner(ctx_ref, alpha, x, y, n)
    })
}

#[cfg(feature = "gpu")]
fn run_axpy_submit_and_map_async_inner(
    ctx: &GpuContext,
    alpha: f32,
    x: &[f32],
    y: &[f32],
    n: usize,
) -> Option<(
    wgpu::Buffer,
    oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
)> {
    let n_bytes = n * std::mem::size_of::<f32>();
    let buf_x = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("axpy x"),
        size: n_bytes as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let buf_y = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("axpy y"),
        size: n_bytes as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&buf_x, 0, unsafe {
        std::slice::from_raw_parts(x.as_ptr().cast(), n_bytes)
    });
    ctx.queue.write_buffer(&buf_y, 0, unsafe {
        std::slice::from_raw_parts(y.as_ptr().cast(), n_bytes)
    });
    let buf_z = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("axpy z"),
        size: n_bytes as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    #[repr(C)]
    struct AxpyParams {
        n: u32,
        alpha: f32,
    }
    let params = AxpyParams { n: n as u32, alpha };
    let param_bytes: &[u8] =
        unsafe { std::slice::from_raw_parts((&params as *const AxpyParams).cast(), 8) };
    let buf_params = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("axpy params"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&buf_params, 0, param_bytes);
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
        let wg_x = (n as u32 + ELEMENTWISE_WORKGROUP_SIZE - 1) / ELEMENTWISE_WORKGROUP_SIZE;
        pass.dispatch_workgroups(wg_x, 1, 1);
    }
    let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("axpy staging"),
        size: n_bytes as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(&buf_z, 0, &staging, 0, n_bytes as u64);
    ctx.queue.submit(std::iter::once(encoder.finish()));
    let buffer_slice = staging.slice(..);
    let (tx, rx) = oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    Some((staging, rx))
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
        let wg_x = (n as u32 + ELEMENTWISE_WORKGROUP_SIZE - 1) / ELEMENTWISE_WORKGROUP_SIZE;
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
        let wg_x = (n as u32 + ELEMENTWISE_WORKGROUP_SIZE - 1) / ELEMENTWISE_WORKGROUP_SIZE;
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
            let wg_x = (m as u32 + ELEMENTWISE_WORKGROUP_SIZE - 1) / ELEMENTWISE_WORKGROUP_SIZE;
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

/// Tries to compute sparse matrix-vector product on the GPU with async readback. Works on both native and wasm.
#[cfg(feature = "gpu")]
pub async fn try_spmv_f32_async(
    sparse: &crate::SparseMatrixCRS<f32>,
    x: &Vector<f32>,
) -> Option<Vector<f32>> {
    let (staging, rx) = run_spmv_submit_and_map_async(sparse, x)?;
    rx.await.ok().and_then(|r| r.ok())?;
    let m = sparse.rows();
    let buffer_slice = staging.slice(..);
    let view = buffer_slice.get_mapped_range();
    let data: &[f32] = unsafe { std::slice::from_raw_parts(view.as_ptr().cast(), m) };
    let mut out = Vector::with_capacity(m);
    out.data_mut().copy_from_slice(data);
    drop(view);
    staging.unmap();
    Some(out)
}

#[cfg(feature = "gpu")]
fn run_spmv_submit_and_map_async(
    sparse: &crate::SparseMatrixCRS<f32>,
    x: &Vector<f32>,
) -> Option<(
    wgpu::Buffer,
    oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
)> {
    let m = sparse.rows();
    let n = sparse.cols();
    if n != x.rows() || m == 0 || n == 0 {
        return None;
    }
    let row_ptr = sparse.row_ptr();
    if row_ptr.len() < m + 1 {
        return None;
    }
    #[cfg(not(target_arch = "wasm32"))]
    return run_spmv_submit_and_map_async_inner(GPU_CONTEXT.get()?, sparse, x, m, n);
    #[cfg(target_arch = "wasm32")]
    GPU_CONTEXT.with(|c| {
        let ctx_ref = c.borrow();
        let ctx_ref = ctx_ref.as_ref()?;
        run_spmv_submit_and_map_async_inner(ctx_ref, sparse, x, m, n)
    })
}

#[cfg(feature = "gpu")]
fn run_spmv_submit_and_map_async_inner(
    ctx: &GpuContext,
    sparse: &crate::SparseMatrixCRS<f32>,
    x: &Vector<f32>,
    m: usize,
    n: usize,
) -> Option<(
    wgpu::Buffer,
    oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
)> {
    let row_ptr = sparse.row_ptr();
    let col_ind = sparse.col_ind();
    let vals = sparse.values();
    let nnz = vals.len();
    let buf_row = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("spmv row_ptr"),
        size: (row_ptr.len() * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let buf_col = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("spmv col_ind"),
        size: (col_ind.len() * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let buf_vals = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("spmv values"),
        size: (nnz * std::mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let buf_x = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("spmv x"),
        size: (n * std::mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&buf_row, 0, unsafe {
        std::slice::from_raw_parts(
            row_ptr.as_ptr().cast(),
            row_ptr.len() * std::mem::size_of::<u32>(),
        )
    });
    ctx.queue.write_buffer(&buf_col, 0, unsafe {
        std::slice::from_raw_parts(
            col_ind.as_ptr().cast(),
            col_ind.len() * std::mem::size_of::<u32>(),
        )
    });
    ctx.queue.write_buffer(&buf_vals, 0, unsafe {
        std::slice::from_raw_parts(vals.as_ptr().cast(), nnz * std::mem::size_of::<f32>())
    });
    ctx.queue.write_buffer(&buf_x, 0, unsafe {
        std::slice::from_raw_parts(x.data().as_ptr().cast(), n * std::mem::size_of::<f32>())
    });
    let buf_y = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("spmv y"),
        size: (m * std::mem::size_of::<f32>()) as u64,
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
    let param_bytes: &[u8] =
        unsafe { std::slice::from_raw_parts((&params as *const SpmvParams).cast(), 12) };
    let buf_params = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("spmv params"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&buf_params, 0, param_bytes);
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
        let wg_x = (m as u32 + ELEMENTWISE_WORKGROUP_SIZE - 1) / ELEMENTWISE_WORKGROUP_SIZE;
        pass.dispatch_workgroups(wg_x, 1, 1);
    }
    let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("spmv staging"),
        size: (m * std::mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(
        &buf_y,
        0,
        &staging,
        0,
        (m * std::mem::size_of::<f32>()) as u64,
    );
    ctx.queue.submit(std::iter::once(encoder.finish()));
    let buffer_slice = staging.slice(..);
    let (tx, rx) = oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    Some((staging, rx))
}

/// Tries to compute sparse matrix-vector product `y = A * x` on the GPU for CCS format.
/// Converts CCS to CRS and dispatches to the CRS SpMV pipeline.
#[must_use]
pub fn try_spmv_ccs_f32(
    sparse: &crate::SparseMatrixCCS<f32>,
    x: &Vector<f32>,
) -> Option<Vector<f32>> {
    let crs = crate::SparseMatrixCRS::from_sparse(sparse);
    try_spmv_f32(&crs, x)
}

#[cfg(feature = "gpu")]
#[allow(dead_code)] // Mul/Div not constructed on wasm32 (try_mul_f32/try_div_f32 return None)
enum ElemOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[cfg(feature = "gpu")]
fn run_elementwise_submit_and_map_async(
    a: &[f32],
    b: &[f32],
    op: ElemOp,
) -> Option<(
    wgpu::Buffer,
    oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
)> {
    let n = a.len();
    if n != b.len() || n == 0 {
        return None;
    }
    #[cfg(not(target_arch = "wasm32"))]
    return run_elementwise_submit_and_map_async_inner(GPU_CONTEXT.get()?, a, b, n, op);
    #[cfg(target_arch = "wasm32")]
    GPU_CONTEXT.with(|c| {
        let ctx_ref = c.borrow();
        let ctx_ref = ctx_ref.as_ref()?;
        run_elementwise_submit_and_map_async_inner(ctx_ref, a, b, n, op)
    })
}

#[cfg(feature = "gpu")]
fn run_elementwise_submit_and_map_async_inner(
    ctx: &GpuContext,
    a: &[f32],
    b: &[f32],
    n: usize,
    op: ElemOp,
) -> Option<(
    wgpu::Buffer,
    oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
)> {
    let n_bytes = n * std::mem::size_of::<f32>();
    let buf_a = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("elem A"),
        size: n_bytes as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let buf_b = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("elem B"),
        size: n_bytes as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&buf_a, 0, unsafe {
        std::slice::from_raw_parts(a.as_ptr().cast(), n_bytes)
    });
    ctx.queue.write_buffer(&buf_b, 0, unsafe {
        std::slice::from_raw_parts(b.as_ptr().cast(), n_bytes)
    });
    let buf_c = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("elem C"),
        size: n_bytes as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    #[repr(C)]
    struct ElemParams {
        n: u32,
    }
    let params = ElemParams { n: n as u32 };
    let param_bytes: &[u8] =
        unsafe { std::slice::from_raw_parts((&params as *const ElemParams).cast(), 4) };
    let buf_params = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("elem params"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&buf_params, 0, param_bytes);
    let (pipeline, layout, use_vec4) = match op {
        ElemOp::Add => (&ctx.add_vec4_pipeline, &ctx.add_bind_group_layout, true),
        ElemOp::Sub => (&ctx.sub_pipeline, &ctx.sub_bind_group_layout, false),
        ElemOp::Mul => (&ctx.mul_pipeline, &ctx.mul_bind_group_layout, false),
        ElemOp::Div => (&ctx.div_pipeline, &ctx.div_bind_group_layout, false),
    };
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
        let wg_x = if use_vec4 {
            (n as u32 + 1023) / 1024
        } else {
            (n as u32 + ELEMENTWISE_WORKGROUP_SIZE - 1) / ELEMENTWISE_WORKGROUP_SIZE
        };
        pass.dispatch_workgroups(wg_x, 1, 1);
    }
    let staging = get_staging_buffer(ctx, n_bytes as u64, "elem staging");
    encoder.copy_buffer_to_buffer(&buf_c, 0, &staging, 0, n_bytes as u64);
    ctx.queue.submit(std::iter::once(encoder.finish()));
    let buffer_slice = staging.slice(..);
    let (tx, rx) = oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    Some((staging, rx))
}

#[cfg(not(target_arch = "wasm32"))]
fn try_elementwise_f32_impl(a: &[f32], b: &[f32], op: ElemOp) -> Option<Vec<f32>> {
    let ctx = GPU_CONTEXT.get()?;
    let n = a.len();
    if n != b.len() || n == 0 {
        return None;
    }
    let (pipeline, layout, use_vec4) = match op {
        ElemOp::Add => (&ctx.add_vec4_pipeline, &ctx.add_bind_group_layout, true),
        ElemOp::Sub => (&ctx.sub_pipeline, &ctx.sub_bind_group_layout, false),
        ElemOp::Mul => (&ctx.mul_pipeline, &ctx.mul_bind_group_layout, false),
        ElemOp::Div => (&ctx.div_pipeline, &ctx.div_bind_group_layout, false),
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
        let wg_x = if use_vec4 {
            (n as u32 + 1023) / 1024
        } else {
            (n as u32 + ELEMENTWISE_WORKGROUP_SIZE - 1) / ELEMENTWISE_WORKGROUP_SIZE
        };
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
