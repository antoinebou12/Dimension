/**
 * Web Worker: runs matmul benchmark off the main thread so the UI stays responsive.
 * Loads WASM once, then on 'run' runs setup, warmup, and timed runs; posts progress and results.
 */
import init, * as lib from "../pkg/mathlib.js";

let initialized = false;
let WasmMatrix, WasmMatrix32, WasmVector;
let gpuAvailable, initGpuAsync, gpuMatmulAvailable, matmulF32Cpu, gpuLastError;

function median(arr) {
  if (arr.length === 0) return 0;
  const s = arr.slice().sort((a, b) => a - b);
  const m = Math.floor(s.length / 2);
  return s.length % 2 ? s[m] : (s[m - 1] + s[m]) / 2;
}

function postProgress(text) {
  self.postMessage({ type: "progress", text });
}

async function runBenchmark(n, numRuns) {
  const total = n * n;
  postProgress(n + "×" + n + " matmul\nPreparing…");

  const data64 = new Float64Array(total * 2);
  const data32 = new Float32Array(total * 2);
  for (let i = 0; i < total * 2; i++) {
    const v = (i % 100) * 0.01;
    data64[i] = v;
    data32[i] = v;
  }

  postProgress(n + "×" + n + " matmul\nWarmup…");
  const A64 = WasmMatrix.fromArray(n, n, Array.from(data64.subarray(0, total)));
  const B64 = WasmMatrix.fromArray(n, n, Array.from(data64.subarray(total, total * 2)));

  const hasGpuBuild = typeof WasmMatrix32 !== "undefined" && typeof gpuAvailable === "function";
  if (hasGpuBuild && typeof initGpuAsync === "function" && !gpuAvailable()) {
    postProgress(n + "×" + n + " matmul\nInitializing GPU…");
    try {
      await initGpuAsync();
    } catch (_) {}
  }

  const A32 = hasGpuBuild ? WasmMatrix32.fromArray(n, n, Array.from(data32.subarray(0, total))) : null;
  const B32 = hasGpuBuild ? WasmMatrix32.fromArray(n, n, Array.from(data32.subarray(total, total * 2))) : null;

  postProgress(n + "×" + n + " matmul\nWarmup… CPU (f64)");
  A64.mul(B64);
  if (A32 && B32 && matmulF32Cpu) {
    postProgress(n + "×" + n + " matmul\nWarmup… CPU (f32)");
    matmulF32Cpu(A32, B32);
  }
  if (A32 && B32 && gpuAvailable()) {
    postProgress(n + "×" + n + " matmul\nWarmup… GPU (f32)");
    A32.mul(B32);
  }

  const cpu64Times = [];
  for (let r = 0; r < numRuns; r++) {
    postProgress(n + "×" + n + " matmul\nRunning… CPU (f64) " + (r + 1) + "/" + numRuns);
    const t0 = performance.now();
    A64.mul(B64);
    cpu64Times.push(performance.now() - t0);
  }
  const cpu64Ms = median(cpu64Times);
  const C64 = A64.mul(B64);
  const sample = C64 && C64.toArray ? C64.toArray()[0].toFixed(4) : "—";

  let cpu32Ms = 0;
  let gpu32Ms = 0;
  const canMeasureCpu32 = A32 && B32 && (matmulF32Cpu || !gpuAvailable());
  if (canMeasureCpu32) {
    const cpu32Times = [];
    const runCpu32 = matmulF32Cpu ? () => matmulF32Cpu(A32, B32) : () => A32.mul(B32);
    for (let r = 0; r < numRuns; r++) {
      postProgress(n + "×" + n + " matmul\nRunning… CPU (f32) " + (r + 1) + "/" + numRuns);
      const t0 = performance.now();
      runCpu32();
      cpu32Times.push(performance.now() - t0);
    }
    cpu32Ms = median(cpu32Times);
  }
  if (hasGpuBuild && gpuAvailable() && A32 && B32) {
    const gpu32Times = [];
    for (let r = 0; r < numRuns; r++) {
      postProgress(n + "×" + n + " matmul\nRunning… GPU (f32) " + (r + 1) + "/" + numRuns);
      const t0 = performance.now();
      A32.mul(B32);
      gpu32Times.push(performance.now() - t0);
    }
    gpu32Ms = median(gpu32Times);
  }

  const gpuMatmulUnavailable = typeof gpuMatmulAvailable === "function" && !gpuMatmulAvailable();
  let gpuUnavailableMessage = null;
  if (hasGpuBuild && !gpuAvailable()) {
    gpuUnavailableMessage = (typeof gpuLastError === "function" && gpuLastError()) ? gpuLastError().trim() : "unavailable (init WebGPU first)";
  } else if (!hasGpuBuild) {
    gpuUnavailableMessage = "build with wasm-build-gpu, then Init GPU";
  }

  self.postMessage({
    type: "done",
    n,
    numRuns,
    cpu64Ms,
    cpu32Ms,
    gpu32Ms,
    sample,
    gpuMatmulUnavailable: gpuMatmulUnavailable || false,
    gpuUnavailableMessage,
  });
}

(async () => {
  try {
    await init();
    WasmMatrix = lib.WasmMatrix;
    WasmMatrix32 = lib.WasmMatrix32;
    WasmVector = lib.WasmVector;
    gpuAvailable = lib.gpuAvailable;
    initGpuAsync = lib.initGpuAsync;
    gpuMatmulAvailable = lib.gpuMatmulAvailable;
    matmulF32Cpu = lib.matmulF32Cpu;
    gpuLastError = lib.gpuLastError;
    initialized = true;
    self.postMessage({ type: "ready" });
  } catch (e) {
    self.postMessage({ type: "error", message: (e && e.message) || String(e) });
  }
})();

self.onmessage = (ev) => {
  const msg = ev.data;
  if (msg && msg.type === "run" && initialized) {
    const n = Math.max(1, Math.min(4096, parseInt(msg.n, 10) || 256));
    const numRuns = Math.max(1, Math.min(10, parseInt(msg.numRuns, 10) || 5));
    runBenchmark(n, numRuns).catch((e) => {
      self.postMessage({ type: "error", message: (e && e.message) || String(e) });
    });
  }
};
