/**
 * mathlib WASM demo — GPU (WebGPU) section.
 * Requires build with: just wasm-build-gpu
 * Exposes: initGpuAsync, matmulF32GpuAsync, dotF32GpuAsync, matvecF32GpuAsync.
 */

import { initLib, byId, showError, needHttp, needBuild } from "../shared.js";

if (window.location.protocol === "file:") {
  showError(needHttp);
} else {
  try {
    const lib = await initLib();
    const initGpuAsync = lib.initGpuAsync;
    const gpuAvailable = lib.gpuAvailable;
    const gpuLastError = lib.gpuLastError;
    const matmulF32GpuAsync = lib.matmulF32GpuAsync;
    const dotF32GpuAsync = lib.dotF32GpuAsync;
    const matvecF32GpuAsync = lib.matvecF32GpuAsync;
    const WasmMatrix32 = lib.WasmMatrix32;

    const gpuInitBtn = byId("gpu-init-btn");
    const gpuStatus = byId("gpu-status");
    const matmulBtn = byId("matmul-btn");
    const matmulOut = byId("matmul-out");
    const dotBtn = byId("dot-btn");
    const dotOut = byId("dot-out");
    const matvecBtn = byId("matvec-btn");
    const matvecOut = byId("matvec-out");

    if (typeof initGpuAsync !== "function" || typeof gpuAvailable !== "function") {
      gpuStatus.textContent = "GPU bindings not available (build with just wasm-build-gpu)";
      gpuInitBtn.disabled = true;
    } else {
      function updateStatus() {
        const available = gpuAvailable();
        gpuStatus.textContent = available ? "Backend: GPU" : "Backend: CPU (init first)";
        gpuInitBtn.disabled = available;
      }
      updateStatus();

      gpuInitBtn.addEventListener("click", async () => {
        gpuInitBtn.disabled = true;
        gpuStatus.textContent = "Initializing…";
        try {
          const ok = await initGpuAsync();
          updateStatus();
          if (!ok) {
            const err = gpuLastError ? gpuLastError() : "";
            gpuStatus.textContent = "Init failed" + (err ? ": " + err : "");
          }
        } catch (e) {
          gpuStatus.textContent = "Init error: " + (e.message || e);
        }
        gpuInitBtn.disabled = typeof gpuAvailable === "function" && gpuAvailable();
      });
    }

    if (matmulBtn && matmulF32GpuAsync && WasmMatrix32) {
      matmulBtn.addEventListener("click", async () => {
        matmulOut.textContent = "Running…";
        const a = WasmMatrix32.fromArray(4, 4, [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]);
        const b = WasmMatrix32.fromArray(4, 4, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        try {
          const c = await matmulF32GpuAsync(a, b);
          matmulOut.textContent = c ? "C = A×B (4×4)\n" + (c.toArray ? c.toArray().map((x) => x.toFixed(2)).join(", ") : String(c)) : "null (GPU not init or failed)";
        } catch (e) {
          matmulOut.textContent = "Error: " + (e.message || e);
        }
      });
    }

    if (dotBtn && dotF32GpuAsync) {
      dotBtn.addEventListener("click", async () => {
        dotOut.textContent = "Running…";
        const a = new Float32Array([1, 2, 3]);
        const b = new Float32Array([4, 5, 6]);
        try {
          const d = await dotF32GpuAsync(a, b);
          dotOut.textContent = d != null ? "dot = " + d : "null (GPU not init or failed)";
        } catch (e) {
          dotOut.textContent = "Error: " + (e.message || e);
        }
      });
    }

    if (matvecBtn && matvecF32GpuAsync && WasmMatrix32) {
      matvecBtn.addEventListener("click", async () => {
        matvecOut.textContent = "Running…";
        const a = WasmMatrix32.fromArray(3, 2, [1, 0, 0, 0, 1, 0]);
        const v = new Float32Array([1, 2]);
        try {
          const y = await matvecF32GpuAsync(a, v);
          matvecOut.textContent = y ? "y = A×v = [" + Array.from(y).map((x) => x.toFixed(2)).join(", ") + "]" : "null (GPU not init or failed)";
        } catch (e) {
          matvecOut.textContent = "Error: " + (e.message || e);
        }
      });
    }
  } catch (e) {
    showError((e.message || "").toLowerCase().includes("fetch") ? needBuild + "\n\n" : "" + (e.message || e));
  }
}
