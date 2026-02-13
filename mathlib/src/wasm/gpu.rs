//! WASM bindings for GPU init and availability (requires both `wasm` and `gpu` features).

use crate::Vector;
use crate::gpu;
use crate::wasm::WasmMatrix32;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

/// Returns whether the GPU backend is initialized and available for f32 ops.
#[wasm_bindgen(js_name = gpuAvailable)]
pub fn gpu_available() -> bool {
    gpu::is_available()
}

/// Returns whether f32 matmul actually uses the GPU. When true, async matmul is available.
#[wasm_bindgen(js_name = gpuMatmulAvailable)]
pub fn gpu_matmul_available() -> bool {
    gpu::is_available()
}

/// Initializes the GPU backend asynchronously (WebGPU). Resolve with `true` if initialization
/// succeeded, `false` otherwise. Call this before relying on GPU-accelerated matmul in the demo.
#[wasm_bindgen(js_name = initGpuAsync)]
pub fn init_gpu_async() -> js_sys::Promise {
    future_to_promise(async move {
        let ok = gpu::init_async(None).await;
        Ok(JsValue::from(ok))
    })
}

/// Returns the last GPU init failure message, if any. Call after initGpuAsync() resolves to false for a clearer error.
#[wasm_bindgen(js_name = gpuLastError)]
pub fn gpu_last_error() -> Option<String> {
    gpu::last_init_error()
}

/// Multiplies two f32 matrices on the GPU (async readback). Returns a Promise that resolves to
/// the result matrix or null if GPU is not initialized or the operation fails. Call initGpuAsync()
/// first. On the browser this uses WebGPU; on native uses the same GPU backend.
#[wasm_bindgen(js_name = matmulF32GpuAsync)]
pub fn matmul_f32_gpu_async(a: &WasmMatrix32, b: &WasmMatrix32) -> js_sys::Promise {
    let a_inner = a.clone_inner();
    let b_inner = b.clone_inner();
    future_to_promise(async move {
        let result = gpu::try_matmul_f32_async(&a_inner, &b_inner).await;
        Ok(match result {
            Some(m) => JsValue::from(WasmMatrix32::from_inner(m)),
            None => JsValue::NULL,
        })
    })
}

/// Dot product of two f32 vectors on the GPU (async). Returns a Promise that resolves to
/// the scalar result or null if GPU is not initialized or the operation fails.
#[wasm_bindgen(js_name = dotF32GpuAsync)]
pub fn dot_f32_gpu_async(a: &[f32], b: &[f32]) -> js_sys::Promise {
    let va = Vector::<f32>::from_slice(a);
    let vb = Vector::<f32>::from_slice(b);
    future_to_promise(async move {
        let result = gpu::try_dot_f32_async(&va, &vb).await;
        Ok(match result {
            Some(x) => JsValue::from_f64(x as f64),
            None => JsValue::NULL,
        })
    })
}

/// Euclidean norm of an f32 vector on the GPU (async). Returns a Promise that resolves to
/// the scalar result or null if GPU is not initialized or the operation fails.
#[wasm_bindgen(js_name = normF32GpuAsync)]
pub fn norm_f32_gpu_async(v: &[f32]) -> js_sys::Promise {
    let v_inner = Vector::<f32>::from_slice(v);
    future_to_promise(async move {
        let result = gpu::try_norm_f32_async(&v_inner).await;
        Ok(match result {
            Some(x) => JsValue::from_f64(x as f64),
            None => JsValue::NULL,
        })
    })
}

/// Element-wise add of two f32 buffers (vectors or flat matrices) on the GPU (async).
/// Returns a Promise that resolves to Float32Array or null if GPU is not initialized or lengths differ.
#[wasm_bindgen(js_name = addF32GpuAsync)]
pub fn add_f32_gpu_async(a: &[f32], b: &[f32]) -> js_sys::Promise {
    let a = a.to_vec();
    let b = b.to_vec();
    future_to_promise(async move {
        let result = gpu::try_add_f32_async(&a, &b).await;
        Ok(match result {
            Some(vec) => {
                let arr = js_sys::Float32Array::new_with_length(vec.len() as u32);
                for (i, &x) in vec.iter().enumerate() {
                    arr.set_index(i as u32, x);
                }
                arr.into()
            }
            None => JsValue::NULL,
        })
    })
}

/// Scalar multiply (scale) of an f32 buffer on the GPU (async).
/// Returns a Promise that resolves to Float32Array or null if GPU is not initialized.
#[wasm_bindgen(js_name = scaleF32GpuAsync)]
pub fn scale_f32_gpu_async(alpha: f32, a: &[f32]) -> js_sys::Promise {
    let a = a.to_vec();
    future_to_promise(async move {
        let result = gpu::try_scale_f32_async(alpha, &a).await;
        Ok(match result {
            Some(vec) => {
                let arr = js_sys::Float32Array::new_with_length(vec.len() as u32);
                for (i, &x) in vec.iter().enumerate() {
                    arr.set_index(i as u32, x);
                }
                arr.into()
            }
            None => JsValue::NULL,
        })
    })
}

/// Matrix-vector product A×v on the GPU (async). Returns a Promise that resolves to
/// the result vector as Float32Array or null if GPU is not initialized or the operation fails.
#[wasm_bindgen(js_name = matvecF32GpuAsync)]
pub fn matvec_f32_gpu_async(a: &WasmMatrix32, v: &[f32]) -> js_sys::Promise {
    let a_inner = a.clone_inner();
    let v_inner = Vector::<f32>::from_slice(v);
    future_to_promise(async move {
        let result = gpu::try_matvec_f32_async(&a_inner, &v_inner).await;
        Ok(match result {
            Some(vec) => {
                let data = vec.data();
                let arr = js_sys::Float32Array::new_with_length(data.len() as u32);
                for (i, &x) in data.iter().enumerate() {
                    arr.set_index(i as u32, x);
                }
                arr.into()
            }
            None => JsValue::NULL,
        })
    })
}
