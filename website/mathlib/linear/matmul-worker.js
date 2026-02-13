/**
 * Web Worker: runs matmul benchmark off the main thread so the UI stays responsive.
 * Loads WASM once, then on 'run' runs setup, warmup, and timed runs; posts progress and results.
 */
import init, * as lib from "../pkg/mathlib.js";

let initialized = false;
let WasmMatrix, WasmMatrix32, WasmVector;
let gpuAvailable, initGpuAsync, gpuMatmulAvailable, matmulF32Cpu, matmulF32CpuWithProgress, matmulF32GpuAsync, gpuLastError;

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
    const v = Math.sin(i * 0.1) * 0.5 + 0.5;
    data64[i] = v;
    data32[i] = v;
  }

  postProgress(n + "×" + n + " matmul\nWarmup…");
  const A64 = WasmMatrix.fromArray(n, n, Array.from(data64.subarray(0, total)));
  const B64 = WasmMatrix.fromArray(n, n, Array.from(data64.subarray(total, total * 2)));

  const hasGpuBuild = typeof WasmMatrix32 !== "undefined" && typeof gpuAvailable === "function";
  if (hasGpuBuild && typeof initGpuAsync === "function" && typeof gpuAvailable === "function" && !gpuAvailable()) {
    postProgress(n + "×" + n + " matmul\nInitializing GPU…");
    try {
      await initGpuAsync();
      if (typeof gpuAvailable === "function" && gpuAvailable() && WasmMatrix32 && matmulF32GpuAsync) {
        const warmA = WasmMatrix32.fromArray(4, 4, [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]);
        const warmB = WasmMatrix32.fromArray(4, 4, Array(16).fill(1));
        await matmulF32GpuAsync(warmA, warmB);
      }
    } catch (_) {}
  }

  const A32 = hasGpuBuild ? WasmMatrix32.fromArray(n, n, Array.from(data32.subarray(0, total))) : null;
  const B32 = hasGpuBuild ? WasmMatrix32.fromArray(n, n, Array.from(data32.subarray(total, total * 2))) : null;

  postProgress(n + "×" + n + " matmul\nWarmup… CPU (f64)");
  A64.mul(B64);
  if (A32 && B32 && matmulF32Cpu) {
    postProgress(n + "×" + n + " matmul\nWarmup… CPU (f32)");
    const warmupCpu32 = typeof matmulF32CpuWithProgress === "function"
      ? () => matmulF32CpuWithProgress(A32, B32, () => {})
      : () => matmulF32Cpu(A32, B32);
    warmupCpu32();
  }
  if (A32 && B32 && (typeof gpuAvailable === "function" && gpuAvailable())) {
    postProgress(n + "×" + n + " matmul\nWarmup… GPU (f32)");
    if (matmulF32GpuAsync) {
      await matmulF32GpuAsync(A32, B32);
    } else {
      A32.mul(B32);
    }
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
  const arr = C64 && C64.toArray ? C64.toArray() : [];
  const sample = arr.length > 0 ? arr[0].toFixed(4) : "—";
  const mid = Math.floor(n / 2);
  const sampleCenter = arr.length > mid + mid * n ? arr[mid + mid * n].toFixed(4) : null;

  let cpu32Ms = 0;
  let gpu32Ms = 0;
  const canMeasureCpu32 = A32 && B32 && (matmulF32Cpu || !(typeof gpuAvailable === "function" && gpuAvailable()));
  if (canMeasureCpu32) {
    const cpu32Times = [];
    const useProgress = n >= 256 && typeof matmulF32CpuWithProgress === "function";
    const runCpu32 = useProgress
      ? () => matmulF32CpuWithProgress(A32, B32, (p) => {
          self.postMessage({ type: "progress", text: n + "×" + n + " matmul (CPU f32)…", progress: p });
        })
      : matmulF32Cpu
        ? () => matmulF32Cpu(A32, B32)
        : () => A32.mul(B32);
    for (let r = 0; r < numRuns; r++) {
      postProgress(n + "×" + n + " matmul\nRunning… CPU (f32) " + (r + 1) + "/" + numRuns);
      const t0 = performance.now();
      runCpu32();
      cpu32Times.push(performance.now() - t0);
    }
    cpu32Ms = median(cpu32Times);
  }
  if (hasGpuBuild && (typeof gpuAvailable === "function" && gpuAvailable()) && A32 && B32 && matmulF32GpuAsync) {
    const gpu32Times = [];
    for (let r = 0; r < numRuns; r++) {
      postProgress(n + "×" + n + " matmul\nRunning… GPU (f32) " + (r + 1) + "/" + numRuns);
      const t0 = performance.now();
      const result = await matmulF32GpuAsync(A32, B32);
      if (result != null) gpu32Times.push(performance.now() - t0);
    }
    if (gpu32Times.length > 0) gpu32Ms = median(gpu32Times);
  }

  const gpuMatmulUnavailable = typeof gpuMatmulAvailable === "function" && !gpuMatmulAvailable();
  let gpuUnavailableMessage = null;
  if (hasGpuBuild && !(typeof gpuAvailable === "function" && gpuAvailable())) {
    const err = typeof gpuLastError === "function" ? gpuLastError() : null;
    gpuUnavailableMessage = (err && typeof err === "string" && err.trim()) ? err.trim() : "unavailable (init WebGPU first)";
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
    sampleCenter,
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
    matmulF32CpuWithProgress = lib.matmulF32CpuWithProgress;
    matmulF32GpuAsync = lib.matmulF32GpuAsync;
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
