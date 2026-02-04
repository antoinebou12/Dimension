//! WASM bindings for GPU init and availability (requires both `wasm` and `gpu` features).

use crate::gpu;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

/// Returns whether the GPU backend is initialized and available for f32 matmul.
#[wasm_bindgen(js_name = gpuAvailable)]
pub fn gpu_available() -> bool {
    gpu::is_available()
}

/// Returns whether f32 matmul actually uses the GPU. On wasm32 this is always false (WebGPU matmul
/// readback is async and not used); on native it matches gpuAvailable().
#[wasm_bindgen(js_name = gpuMatmulAvailable)]
pub fn gpu_matmul_available() -> bool {
    #[cfg(target_arch = "wasm32")]
    return false;
    #[cfg(not(target_arch = "wasm32"))]
    gpu::is_available()
}

/// Initializes the GPU backend asynchronously (WebGPU). Resolve with `true` if initialization
/// succeeded, `false` otherwise. Call this before relying on GPU-accelerated matmul in the demo.
#[wasm_bindgen(js_name = initGpuAsync)]
pub fn init_gpu_async() -> js_sys::Promise {
    future_to_promise(async move {
        let ok = gpu::init_async().await;
        Ok(JsValue::from(ok))
    })
}

/// Returns the last GPU init failure message, if any. Call after initGpuAsync() resolves to false for a clearer error.
#[wasm_bindgen(js_name = gpuLastError)]
pub fn gpu_last_error() -> Option<String> {
    gpu::last_init_error()
}
