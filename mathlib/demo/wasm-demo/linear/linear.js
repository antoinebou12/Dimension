/**
 * mathlib WASM demo — Linear algebra (vector, matrix, storage, Cholesky, SVD, LU).
 */
import {
  initLib, byId, showError, needHttp, needBuild,
  bindExampleSelector, scaleToCanvas, renderMatrixHTML, renderMatrixHTMLWithColors, drawVectorBarsGeneric,
  drawStorageGrid, drawSparseCSRDiagram,
} from "../shared.js";

function colMajorFlat(rows, cols, values) {
  const out = [];
  for (let j = 0; j < cols; j++)
    for (let i = 0; i < rows; i++) out.push(values[i * cols + j]);
  return out;
}

function rowMajorFlat(rows, cols, values) {
  return values.slice();
}

function renderStorageFlatArray(containerId, flatArr, storage) {
  const el = byId(containerId);
  if (!el) return;
  const palette = ["#e3f2fd", "#bbdefb", "#90caf9", "#ffebee", "#ffcdd2", "#ef9a9a", "#e8f5e9", "#c8e6c9", "#a5d6a7"];
  let html = storage === "column" ? "Column-major flat: [" : "Row-major flat: [";
  flatArr.forEach((val, idx) => {
    html += '<span style="background:' + palette[idx % palette.length] + ';padding:2px 6px;margin:0 1px;border-radius:4px">' + val + "</span>";
    if (idx < flatArr.length - 1) html += ", ";
  });
  html += "]";
  el.innerHTML = html;
}

if (window.location.protocol === "file:") {
  showError(needHttp);
  const out = byId("out-vector");
  if (out) { out.className = "error"; out.textContent = needHttp; }
} else {
  try {
    const lib = await initLib();
    const { WasmMatrix, WasmMatrix32, WasmVector, WasmCholesky, WasmLu } = lib;
    const initGpuAsync = lib.initGpuAsync;
    const gpuAvailable = lib.gpuAvailable;
    const gpuMatmulAvailable = typeof lib.gpuMatmulAvailable === "function" ? lib.gpuMatmulAvailable : null;

    // —— Vector add, dot, norm ——
    const VECTOR_EXAMPLES = [
      { a: [1, 2, 3], b: [4, 5, 6] },
      { a: [0, 1], b: [1, 0] },
      { a: [1, 0, 0], b: [0, 1, 0] },
    ];
    const vectorResults = VECTOR_EXAMPLES.map((ex) => {
      const a = WasmVector.fromArray(ex.a);
      const b = WasmVector.fromArray(ex.b);
      const c = a.add(b).toArray();
      let dot = "—";
      try {
        const d = a.dot(b);
        if (typeof d === "number") dot = d.toFixed(4);
      } catch (_) {}
      return { a: ex.a, b: ex.b, c, dot, normA: a.norm().toFixed(4), normB: b.norm().toFixed(4) };
    });
    function updateVectorOutput(i) {
      const r = vectorResults[i];
      let text = "a = [" + r.a.join(", ") + "]\nb = [" + r.b.join(", ") + "]\na + b = [" + r.c.join(", ") + "]";
      text += "\n\ndot(a, b) = " + r.dot + ", norm(a) = " + r.normA + ", norm(b) = " + r.normB;
      byId("out-vector").textContent = text;
      drawVectorBarsGeneric("canvas-vector", r.c);
    }
    bindExampleSelector("vector-examples", ["Example 1", "Example 2", "Example 3"], updateVectorOutput);
    updateVectorOutput(0);

    if (typeof gpuAvailable === "function") {
      const gpuVectorStatus = byId("gpu-vector-status");
      const gpuVectorBackend = byId("gpu-vector-backend");
      if (gpuVectorStatus && gpuVectorBackend) {
        gpuVectorStatus.style.display = "block";
        gpuVectorBackend.textContent = gpuAvailable() ? "Backend: GPU" : "Backend: CPU";
      }
      const gpuVectorCpuGpuStatus = byId("gpu-vector-cpu-gpu-status");
      const gpuVectorCpuGpuBackend = byId("gpu-vector-cpu-gpu-backend");
      if (gpuVectorCpuGpuStatus && gpuVectorCpuGpuBackend) {
        gpuVectorCpuGpuStatus.style.display = "block";
        gpuVectorCpuGpuBackend.textContent = gpuAvailable() ? "Backend: GPU" : "Backend: CPU";
      }
    }

    // —— Matrix 2D ——
    const MATRIX_EXAMPLES = [
      { A: [1, 0, 0, 0, 1, 0, 0, 0, 1], B: [1, 2, 3, 4, 5, 6, 7, 8, 9], rows: 3, cols: 3 },
      { A: [1, 0, 0, 1], B: [2, 1, 0, 2], rows: 2, cols: 2 },
      { A: [1, 2, 0, 1], B: [0, 1, 1, 0], rows: 2, cols: 2 },
      { A: [1, 0, 1, 0, 1, 1, 1, 0, 0, 0, 1, 0], B: [1, 0, 1, 0, 0, 1, 0, 1], rowsA: 3, colsA: 4, rowsB: 4, colsB: 2 },
    ];
    const matrixResults = MATRIX_EXAMPLES.map((ex) => {
      const rowsA = ex.rowsA ?? ex.rows;
      const colsA = ex.colsA ?? ex.cols;
      const rowsB = ex.rowsB ?? ex.rows;
      const colsB = ex.colsB ?? ex.cols;
      const A = WasmMatrix.fromArray(rowsA, colsA, ex.A);
      const B = WasmMatrix.fromArray(rowsB, colsB, ex.B);
      const C = A.mul(B);
      return { ...ex, rowsA, colsA, rowsB, colsB, rowsC: rowsA, colsC: colsB, A, B, C };
    });
    function getMatrixFormulaHTML(r) {
      const rowsA = r.rowsA;
      const colsA = r.colsA;
      const rowsB = r.rowsB;
      const rowsC = r.rowsC;
      const colsC = r.colsC;
      let html = "<p class=\"matrix-formula-desc\"><strong>Formula:</strong> (C)<sub>i,j</sub> = row<sub>i</sub>(A)·col<sub>j</sub>(B). Column j of C = A × (column j of B).</p>";
      const highlightColors = ["#b3d7ff", "#b8e6b8", "#ffd9b3", "#e6b3ff"];
      const aArr = r.A.toArray();
      const bArr = r.B.toArray();
      const cArr = r.C.toArray();
      const parts = [];
      for (let i = 0; i < Math.min(rowsC, 4); i++) {
        for (let j = 0; j < Math.min(colsC, 4); j++) {
          const color = highlightColors[(i * colsC + j) % highlightColors.length];
          let sumStr = "";
          let sum = 0;
          for (let k = 0; k < colsA; k++) {
            const a = aArr[k * rowsA + i];
            const b = bArr[j * rowsB + k];
            if (k > 0) sumStr += " + ";
            sumStr += a + "×" + b;
            sum += a * b;
          }
          const cVal = cArr[j * rowsC + i];
          parts.push("<span style=\"background:" + color + ";padding:2px 6px;border-radius:4px;margin:0 2px\">C[" + i + "," + j + "] = " + sumStr + " = " + cVal + "</span>");
        }
      }
      html += "<p class=\"matrix-formula-numbers\">" + parts.join(" ") + "</p>";
      html += "<p class=\"matrix-formula-storage\"><strong>Column-major:</strong> <code>fromArray(rows, cols, data)</code> expects flat = [col<sub>0</sub>, col<sub>1</sub>, …]; index (i,j) → <code>j*rows + i</code>.</p>";
      return html;
    }
    bindExampleSelector("matrix-examples", ["Example 1", "Example 2", "Example 3", "Example 4 (3×4 × 4×2)"], (i) => {
      const r = matrixResults[i];
      const outMatrix = byId("out-matrix");
      const outFormula = byId("out-matrix-formula");
      outMatrix.innerHTML =
        "A:" + renderMatrixHTML(r.rowsA, r.colsA, r.A.toArray()) +
        "B:" + renderMatrixHTML(r.rowsB, r.colsB, r.B.toArray()) +
        "C = A×B:" + renderMatrixHTML(r.rowsC, r.colsC, r.C.toArray());
      if (outFormula) outFormula.innerHTML = getMatrixFormulaHTML(r);
    });
    const outMatrix = byId("out-matrix");
    const outMatrixFormula = byId("out-matrix-formula");
    if (outMatrix) {
      const r0 = matrixResults[0];
      outMatrix.innerHTML =
        "A (identity):" + renderMatrixHTML(r0.rowsA, r0.colsA, r0.A.toArray()) +
        "B:" + renderMatrixHTML(r0.rowsB, r0.colsB, r0.B.toArray()) +
        "C = A×B:" + renderMatrixHTML(r0.rowsC, r0.colsC, r0.C.toArray());
    }
    if (outMatrixFormula) outMatrixFormula.innerHTML = getMatrixFormulaHTML(matrixResults[0]);

    if (typeof initGpuAsync === "function" && typeof gpuAvailable === "function") {
      const gpuStatusEl = byId("gpu-matrix-status");
      const gpuBackendEl = byId("gpu-matrix-backend");
      const gpuInitBtn = byId("gpu-init-btn");
      if (gpuStatusEl && gpuBackendEl && gpuInitBtn) {
        gpuStatusEl.style.display = "block";
        function updateGpuBackendLabel() {
          const backend = gpuAvailable() ? "Backend: GPU" : "Backend: CPU";
          gpuBackendEl.textContent = backend;
          gpuInitBtn.disabled = gpuAvailable();
          const gv = byId("gpu-vector-backend");
          if (gv) gv.textContent = backend;
          const gvc = byId("gpu-vector-cpu-gpu-backend");
          if (gvc) gvc.textContent = backend;
          const gb = byId("gpu-basic-f32-backend");
          if (gb) gb.textContent = backend;
        }
        updateGpuBackendLabel();
        gpuInitBtn.addEventListener("click", () => {
          gpuInitBtn.disabled = true;
          gpuInitBtn.textContent = "Initializing…";
          initGpuAsync().then(async (ok) => {
            updateGpuBackendLabel();
            gpuInitBtn.textContent = "Init GPU";
            gpuInitBtn.disabled = ok;
            if (ok && typeof lib.dotF32GpuAsync === "function") {
              try {
                const a = new Float32Array(256);
                const b = new Float32Array(256);
                await lib.dotF32GpuAsync(a, b);
              } catch (_) {}
            }
          }).catch(() => {
            gpuInitBtn.textContent = "Init GPU";
            gpuInitBtn.disabled = false;
          });
        });
      }
    }

    // —— Large matrix ——
    const largeMatrixRun = byId("large-matrix-run");
    const largeMatrixSize = byId("large-matrix-size");
    const outLargeMatrix = byId("out-large-matrix");
    const matmulPlotWrap = byId("matmul-plot-wrap");
    const canvasMatmul = byId("canvas-matmul");
    const gpuInitBtn = byId("gpu-init-btn");
    function updateGpuBackendLabel() {
      if (typeof gpuAvailable === "function") {
        const backend = gpuAvailable() ? "Backend: GPU" : "Backend: CPU";
        const gpuBackendEl = byId("gpu-matrix-backend");
        const gv = byId("gpu-vector-backend");
        const gb = byId("gpu-basic-f32-backend");
        if (gpuBackendEl) gpuBackendEl.textContent = backend;
        if (gv) gv.textContent = backend;
        if (gb) gb.textContent = backend;
        if (gpuInitBtn) gpuInitBtn.disabled = gpuAvailable();
      }
    }
    function median(arr) {
      if (arr.length === 0) return 0;
      const s = arr.slice().sort((a, b) => a - b);
      const m = Math.floor(s.length / 2);
      return s.length % 2 ? s[m] : (s[m - 1] + s[m]) / 2;
    }

    function yieldToEventLoop() {
      return new Promise((r) => setTimeout(r, 0));
    }

    const NUM_MATMUL_RUNS = 5;
    const NUM_MATMUL_RUNS_LARGE = 3;
    const LARGE_SIZE_THRESHOLD = 1024;

    function renderMatmulResults(payload) {
      const { n, numRuns, cpu64Ms, cpu32Ms, gpu32Ms, sample, sampleCenter, gpuMatmulUnavailable, gpuUnavailableMessage } = payload;
      let text = n + "×" + n + " matmul (median of " + numRuns + " runs)\n";
      text += "CPU (f64): " + cpu64Ms.toFixed(2) + " ms\n";
      if (cpu32Ms > 0) text += "CPU (f32): " + cpu32Ms.toFixed(2) + " ms\n";
      if (gpu32Ms > 0) {
        text += "GPU (f32): " + gpu32Ms.toFixed(2) + " ms" + (gpuMatmulUnavailable ? " (CPU in browser)" : "") + "\n";
      } else if (gpuUnavailableMessage) {
        text += "GPU (f32): " + gpuUnavailableMessage + "\n";
      }
      if (gpuMatmulUnavailable) text += "Note: GPU (f32) runs on CPU in the browser (WebGPU matmul not yet supported).\n";
      text += "Sample C[0,0] = " + sample;
      if (sampleCenter != null) {
        const mid = Math.floor(n / 2);
        text += ", C[" + mid + "," + mid + "] = " + sampleCenter;
      }
      outLargeMatrix.textContent = text;

      if (matmulPlotWrap && canvasMatmul && (cpu64Ms > 0 || cpu32Ms > 0 || gpu32Ms > 0)) {
        matmulPlotWrap.style.display = "block";
        const ctx = canvasMatmul.getContext("2d");
        const w = canvasMatmul.width;
        const h = canvasMatmul.height;
        const maxMs = Math.max(cpu64Ms, cpu32Ms, gpu32Ms || 1, 1);
        const barH = Math.min(40, (h - 24) / 3);
        const gap = 10;
        const top = 16;
        const labelWidth = 160;
        const barLeft = labelWidth + 12;
        const maxBarW = w - barLeft - 24;
        ctx.fillStyle = "#fff";
        ctx.fillRect(0, 0, w, h);
        ctx.font = "14px \"DM Sans\", system-ui, sans-serif";
        ctx.textAlign = "left";
        ctx.textBaseline = "middle";
        ctx.fillStyle = "#212529";
        let y = top;
        if (cpu64Ms > 0) {
          ctx.fillStyle = "#0d6efd";
          ctx.fillRect(barLeft, y, (cpu64Ms / maxMs) * maxBarW, barH);
          ctx.fillStyle = "#212529";
          ctx.fillText("CPU (f64) " + cpu64Ms.toFixed(0) + " ms", 12, y + barH / 2);
          y += barH + gap;
        }
        if (cpu32Ms > 0) {
          ctx.fillStyle = "#20c997";
          ctx.fillRect(barLeft, y, (cpu32Ms / maxMs) * maxBarW, barH);
          ctx.fillStyle = "#212529";
          ctx.fillText("CPU (f32) " + cpu32Ms.toFixed(0) + " ms", 12, y + barH / 2);
          y += barH + gap;
        }
        if (gpu32Ms > 0) {
          ctx.fillStyle = "#fd7e14";
          ctx.fillRect(barLeft, y, (gpu32Ms / maxMs) * maxBarW, barH);
          ctx.fillStyle = "#212529";
          const gpuLabel = gpuMatmulUnavailable ? "GPU (f32) " + gpu32Ms.toFixed(0) + " ms (CPU)" : "GPU (f32) " + gpu32Ms.toFixed(0) + " ms";
          ctx.fillText(gpuLabel, 12, y + barH / 2);
        }
      }
    }

    let matmulWorker = null;
    let matmulWorkerReady = false;
    try {
      matmulWorker = new Worker(new URL("matmul-worker.js", import.meta.url), { type: "module" });
      matmulWorker.addEventListener("message", (ev) => {
        if (ev.data && ev.data.type === "ready") matmulWorkerReady = true;
      });
    } catch (_) {}

    if (largeMatrixRun && largeMatrixSize && outLargeMatrix) {
      largeMatrixRun.addEventListener("click", async () => {
        largeMatrixRun.disabled = true;
        const n = parseInt(largeMatrixSize.value, 10);
        const numRuns = n >= LARGE_SIZE_THRESHOLD ? NUM_MATMUL_RUNS_LARGE : NUM_MATMUL_RUNS;

        if (matmulWorker && matmulWorkerReady) {
          const onMessage = (ev) => {
            const msg = ev.data;
            if (!msg || !msg.type) return;
            if (msg.type === "progress") {
              outLargeMatrix.textContent = msg.text;
            } else if (msg.type === "done") {
              matmulWorker.removeEventListener("message", onMessage);
              matmulWorker.removeEventListener("error", onError);
              renderMatmulResults(msg);
              largeMatrixRun.disabled = false;
            } else if (msg.type === "error") {
              matmulWorker.removeEventListener("message", onMessage);
              matmulWorker.removeEventListener("error", onError);
              outLargeMatrix.textContent = "Error: " + (msg.message || "worker error");
              largeMatrixRun.disabled = false;
            }
          };
          const onError = () => {
            matmulWorker.removeEventListener("message", onMessage);
            matmulWorker.removeEventListener("error", onError);
            outLargeMatrix.textContent = "Worker failed; run again or refresh.";
            largeMatrixRun.disabled = false;
          };
          matmulWorker.addEventListener("message", onMessage);
          matmulWorker.addEventListener("error", onError);
          outLargeMatrix.textContent = n + "×" + n + " matmul\nRunning in background…";
          matmulWorker.postMessage({ type: "run", n, numRuns });
          return;
        }

        try {
          const total = n * n;
          const numRuns = n >= LARGE_SIZE_THRESHOLD ? NUM_MATMUL_RUNS_LARGE : NUM_MATMUL_RUNS;
          outLargeMatrix.textContent = n + "×" + n + " matmul\nPreparing…";
          await yieldToEventLoop();
          const data64 = new Float64Array(total * 2);
          const data32 = new Float32Array(total * 2);
          for (let i = 0; i < total * 2; i++) {
            const v = Math.sin(i * 0.1) * 0.5 + 0.5;
            data64[i] = v;
            data32[i] = v;
          }
          const A64 = WasmMatrix.fromArray(n, n, Array.from(data64.subarray(0, total)));
          const B64 = WasmMatrix.fromArray(n, n, Array.from(data64.subarray(total, total * 2)));
          outLargeMatrix.textContent = n + "×" + n + " matmul\nWarmup…";
          await yieldToEventLoop();
          const hasGpuBuild = typeof WasmMatrix32 !== "undefined" && typeof gpuAvailable === "function";
          const matmulF32Cpu = typeof lib.matmulF32Cpu === "function" ? lib.matmulF32Cpu : null;
          const matmulF32GpuAsync = typeof lib.matmulF32GpuAsync === "function" ? lib.matmulF32GpuAsync : null;
          const gpuLastError = typeof lib.gpuLastError === "function" ? lib.gpuLastError : null;

          if (hasGpuBuild && typeof initGpuAsync === "function" && typeof gpuAvailable === "function" && !gpuAvailable()) {
            outLargeMatrix.textContent = n + "×" + n + " matmul\nInitializing GPU…";
            try {
              await initGpuAsync();
              updateGpuBackendLabel();
            } catch (_) {}
            await yieldToEventLoop();
          }

          const A32 = hasGpuBuild ? WasmMatrix32.fromArray(n, n, Array.from(data32.subarray(0, total))) : null;
          const B32 = hasGpuBuild ? WasmMatrix32.fromArray(n, n, Array.from(data32.subarray(total, total * 2))) : null;

          outLargeMatrix.textContent = n + "×" + n + " matmul\nWarmup… CPU (f64)";
          await yieldToEventLoop();
          A64.mul(B64);
          await yieldToEventLoop();
          if (A32 && B32 && matmulF32Cpu) {
            outLargeMatrix.textContent = n + "×" + n + " matmul\nWarmup… CPU (f32)";
            await yieldToEventLoop();
            matmulF32Cpu(A32, B32);
            await yieldToEventLoop();
          }
          if (A32 && B32 && (typeof gpuAvailable === "function" && gpuAvailable())) {
            outLargeMatrix.textContent = n + "×" + n + " matmul\nWarmup… GPU (f32)";
            await yieldToEventLoop();
            A32.mul(B32);
            await yieldToEventLoop();
          }

          const cpu64Times = [];
          for (let r = 0; r < numRuns; r++) {
            outLargeMatrix.textContent = n + "×" + n + " matmul\nRunning… CPU (f64) " + (r + 1) + "/" + numRuns;
            await yieldToEventLoop();
            const t0 = performance.now();
            A64.mul(B64);
            cpu64Times.push(performance.now() - t0);
            await yieldToEventLoop();
          }
          const cpu64Ms = median(cpu64Times);
          const C64 = A64.mul(B64);
          const cArr = C64 && C64.toArray ? C64.toArray() : [];
          const sample = cArr.length > 0 ? cArr[0].toFixed(4) : "—";
          const mid = Math.floor(n / 2);
          const sampleCenter = cArr.length > mid + mid * n ? cArr[mid + mid * n].toFixed(4) : null;

          let cpu32Ms = 0;
          let gpu32Ms = 0;
          const canMeasureCpu32 = A32 && B32 && (matmulF32Cpu || !(typeof gpuAvailable === "function" && gpuAvailable()));
          if (canMeasureCpu32) {
            const cpu32Times = [];
            const runCpu32 = matmulF32Cpu ? () => matmulF32Cpu(A32, B32) : () => A32.mul(B32);
            for (let r = 0; r < numRuns; r++) {
              outLargeMatrix.textContent = n + "×" + n + " matmul\nRunning… CPU (f32) " + (r + 1) + "/" + numRuns;
              await yieldToEventLoop();
              const t0 = performance.now();
              runCpu32();
              cpu32Times.push(performance.now() - t0);
              await yieldToEventLoop();
            }
            cpu32Ms = median(cpu32Times);
          }
          let gpuMatmulUsedRealGpu = false;
          if (hasGpuBuild && (typeof gpuAvailable === "function" && gpuAvailable()) && A32 && B32) {
            const gpu32Times = [];
            for (let r = 0; r < numRuns; r++) {
              outLargeMatrix.textContent = n + "×" + n + " matmul\nRunning… GPU (f32) " + (r + 1) + "/" + numRuns;
              await yieldToEventLoop();
              const t0 = performance.now();
              if (matmulF32GpuAsync) {
                const result = await matmulF32GpuAsync(A32, B32);
                if (result != null) {
                  gpu32Times.push(performance.now() - t0);
                  gpuMatmulUsedRealGpu = true;
                }
              } else {
                A32.mul(B32);
                gpu32Times.push(performance.now() - t0);
              }
              await yieldToEventLoop();
            }
            gpu32Ms = median(gpu32Times);
          }

          renderMatmulResults({
            n, numRuns, cpu64Ms, cpu32Ms, gpu32Ms, sample, sampleCenter,
            gpuMatmulUnavailable: (gpu32Ms > 0) && !gpuMatmulUsedRealGpu,
            gpuUnavailableMessage: (hasGpuBuild && !(typeof gpuAvailable === "function" && gpuAvailable())) ? (gpuLastError ? gpuLastError().trim() : "unavailable (init WebGPU first)") : (!hasGpuBuild ? "build with wasm-build-gpu, then Init GPU" : null),
          });
        } finally {
          largeMatrixRun.disabled = false;
        }
      });
    }

    // —— Matrix × Vector ——
    const MATRIX_VECTOR_EXAMPLES = [
      { label: "Identity × v", A: [1, 0, 0, 0, 1, 0, 0, 0, 1], rows: 3, cols: 3, v: [1, 2, 3] },
      { label: "Scale 2×", A: [2, 0, 0, 2], rows: 2, cols: 2, v: [3, 4] },
      { label: "Rotate 90°", A: [0, 1, -1, 0], rows: 2, cols: 2, v: [1, 0] },
      { label: "Linear combo", A: [1, 0, 1, 0, 1, 1, 0, 0, 1], rows: 3, cols: 3, v: [1, 1, 1] },
    ];
    const matrixVectorResults = MATRIX_VECTOR_EXAMPLES.map((ex) => {
      const A = WasmMatrix.fromArray(ex.rows, ex.cols, ex.A);
      const v = WasmVector.fromArray(ex.v);
      const result = A.mulVector(v);
      return { ...ex, A, result: result ? result.toArray() : [] };
    });
    bindExampleSelector("matrix-vector-examples", MATRIX_VECTOR_EXAMPLES.map((e) => e.label), (i) => {
      const r = matrixVectorResults[i];
      const resStr = r.result.map((x) => Number(x).toFixed(2)).join(", ");
      byId("out-matrix-vector").innerHTML =
        "A:" + renderMatrixHTMLWithColors(r.rows, r.cols, r.A.toArray(), { colorBy: "value" }) +
        "v: [" + r.v.join(", ") + "]<sup>T</sup><br>y = A×v: [" + resStr + "]<sup>T</sup>";
      drawVectorBarsGeneric("canvas-matrix-vector", r.result);
    });
    (() => {
      const r = matrixVectorResults[0];
      const resStr = r.result.map((x) => Number(x).toFixed(2)).join(", ");
      byId("out-matrix-vector").innerHTML =
        "A:" + renderMatrixHTMLWithColors(r.rows, r.cols, r.A.toArray(), { colorBy: "value" }) +
        "v: [" + r.v.join(", ") + "]<sup>T</sup><br>y = A×v: [" + resStr + "]<sup>T</sup>";
      drawVectorBarsGeneric("canvas-matrix-vector", r.result);
    })();

    // —— Solve Ax = b (general) ——
    const SOLVE_EXAMPLES = [
      { A: [2, 1, 1, 2], b: [3, 3], rows: 2, cols: 2 },
      { A: [1, 1, 1, -1], b: [2, 0], rows: 2, cols: 2 },
      { A: [3, 0, 0, 0, 2, 0, 0, 0, 1], b: [6, 4, 1], rows: 3, cols: 3 },
    ];
    const solveResults = SOLVE_EXAMPLES.map((ex) => {
      try {
        const A = WasmMatrix.fromArray(ex.rows, ex.cols, ex.A);
        const b = WasmVector.fromArray(ex.b);
        const x = A.solve(b);
        return { ...ex, A, b: ex.b, x: x ? x.toArray() : [], err: null };
      } catch (e) {
        return { ...ex, A: null, b: ex.b, x: [], err: e.message || String(e) };
      }
    });
    function updateSolveOutput(i) {
      const r = solveResults[i];
      const outSolve = byId("out-solve");
      const canvasSolveX = byId("canvas-solve-x");
      if (!outSolve) return;
      if (r.err) {
        outSolve.textContent = "Error: " + r.err;
        if (canvasSolveX) drawVectorBarsGeneric("canvas-solve-x", []);
        return;
      }
      const xStr = r.x.map((v) => Number(v).toFixed(4)).join(", ");
      outSolve.innerHTML =
        "<div class=\"matrix-block\"><strong>A (" + r.rows + "×" + r.cols + "):</strong>" +
        renderMatrixHTMLWithColors(r.rows, r.cols, r.A.toArray(), { colorBy: "value" }) + "</div>" +
        "<div class=\"vector-inline\"><strong>b:</strong> [" + r.b.map((v) => Number(v).toFixed(2)).join(", ") + "]</div>" +
        "<div class=\"vector-inline\"><strong>x (solution):</strong> [" + xStr + "]</div>";
      if (canvasSolveX) drawVectorBarsGeneric("canvas-solve-x", r.x);
    }
    bindExampleSelector("solve-examples", ["2×2 (x=[1,1])", "2×2 (x=[1,1], LU)", "3×3 diagonal"], updateSolveOutput);
    updateSolveOutput(0);

    // —— Damped least squares ——
    const DAMPED_EXAMPLES = [
      { A: [1, 0, 1, 0, 1, 1], rows: 3, cols: 2, b: [1, 2, 3] },
      { A: [1, 0, 0, 1, 1, 1], rows: 3, cols: 2, b: [1, 1, 2] },
      { A: [1, 1, 1, 1, 2, 3], rows: 3, cols: 2, b: [1, 2, 2], lineFit: true },
    ];
    const dampedLambdaEl = byId("damped-lambda");
    const dampedLambdaValueEl = byId("damped-lambda-value");
    const outDamped = byId("out-damped");
    let currentDampedExampleIndex = 0;
    function updateDampedOutput() {
      const lambdaSq = parseFloat(dampedLambdaEl?.value || "1", 10);
      if (dampedLambdaValueEl) dampedLambdaValueEl.textContent = lambdaSq.toFixed(1);
      const ex = DAMPED_EXAMPLES[currentDampedExampleIndex];
      if (!ex || !outDamped) return;
      try {
        const A = WasmMatrix.fromArray(ex.rows, ex.cols, ex.A);
        const b = WasmVector.fromArray(ex.b);
        const x = typeof A.dampedLeastSquares === "function"
          ? A.dampedLeastSquares(b, lambdaSq)
          : null;
        if (!x) {
          outDamped.textContent = "dampedLeastSquares not available (rebuild with wasm).";
          drawVectorBarsGeneric("canvas-damped-x", []);
          return;
        }
        const xArr = x.toArray();
        const xStr = xArr.map((v) => Number(v).toFixed(4)).join(", ");
        outDamped.innerHTML =
          "<div class=\"matrix-block\"><strong>A (" + ex.rows + "×" + ex.cols + "):</strong>" +
          renderMatrixHTMLWithColors(ex.rows, ex.cols, ex.A, { colorBy: "value" }) + "</div>" +
          "<div class=\"vector-inline\"><strong>b:</strong> [" + ex.b.map((v) => Number(v).toFixed(2)).join(", ") + "]</div>" +
          "<div class=\"vector-inline\"><strong>λ²:</strong> " + lambdaSq.toFixed(2) + "</div>" +
          "<div class=\"vector-inline\"><strong>x:</strong> [" + xStr + "]</div>";
        drawVectorBarsGeneric("canvas-damped-x", xArr);
        const linefitWrap = byId("damped-linefit-wrap");
        const linefitCanvas = byId("canvas-damped-linefit");
        if (ex.lineFit && linefitWrap && linefitCanvas && xArr.length >= 2) {
          linefitWrap.style.display = "block";
          const ctx = linefitCanvas.getContext("2d");
          const w = linefitCanvas.width;
          const h = linefitCanvas.height;
          const pad = 24;
          const intercept = xArr[0];
          const slope = xArr[1];
          const t = [1, 2, 3];
          const y = ex.b;
          const xMin = 0.5, xMax = 3.5, yMin = Math.min(0, ...y) - 0.5, yMax = Math.max(...y) + 0.5;
          const scaleX = (v) => pad + ((v - xMin) / (xMax - xMin)) * (w - 2 * pad);
          const scaleY = (v) => h - pad - ((v - yMin) / (yMax - yMin)) * (h - 2 * pad);
          ctx.fillStyle = "#fff";
          ctx.fillRect(0, 0, w, h);
          ctx.strokeStyle = "#333";
          ctx.beginPath();
          ctx.moveTo(scaleX(xMin), scaleY(intercept + slope * xMin));
          ctx.lineTo(scaleX(xMax), scaleY(intercept + slope * xMax));
          ctx.stroke();
          ctx.fillStyle = "#0d6efd";
          for (let i = 0; i < t.length; i++) {
            ctx.beginPath();
            ctx.arc(scaleX(t[i]), scaleY(y[i]), 5, 0, Math.PI * 2);
            ctx.fill();
          }
        } else if (linefitWrap) {
          linefitWrap.style.display = "none";
        }
      } catch (e) {
        outDamped.textContent = "Error: " + (e.message || e);
        drawVectorBarsGeneric("canvas-damped-x", []);
      }
    }
    if (dampedLambdaEl) {
      dampedLambdaEl.addEventListener("input", updateDampedOutput);
    }
    bindExampleSelector("damped-examples", ["3×2 overdetermined", "3×2 (1,1,2)", "Line fit (t=1,2,3)"], (i) => {
      currentDampedExampleIndex = i;
      updateDampedOutput();
    });
    updateDampedOutput();

    // —— Basic f32 ops (add, scale, dot, matvec) with backend indicator ——
    const outBasicF32 = byId("out-basic-f32");
    const gpuBasicF32Status = byId("gpu-basic-f32-status");
    const gpuBasicF32Backend = byId("gpu-basic-f32-backend");
    if (outBasicF32 && typeof WasmMatrix32 !== "undefined") {
      gpuBasicF32Status.style.display = "block";
      gpuBasicF32Backend.textContent = (typeof gpuAvailable === "function" && gpuAvailable()) ? "Backend: GPU" : "Backend: CPU";
      try {
        const A = WasmMatrix32.fromArray(2, 2, [1, 2, 3, 4]);
        const B = WasmMatrix32.fromArray(2, 2, [0.5, 1, 1.5, 2]);
        const addResult = A.add(B);
        const scaleResult = A.scale(2);
        const dotVal = typeof lib.dotF32 === "function"
          ? lib.dotF32(new Float32Array([1, 2, 3]), new Float32Array([4, 5, 6]))
          : null;
        const matvecY = A.mulVectorF32(new Float32Array([1, 0]));
        let html = "A + B (2×2): " + (addResult && addResult.toArray ? addResult.toArray().map((x) => x.toFixed(2)).join(", ") : "—") + "<br>";
        html += "2×A: " + (scaleResult && scaleResult.toArray ? scaleResult.toArray().map((x) => x.toFixed(2)).join(", ") : "—") + "<br>";
        html += "dot([1,2,3], [4,5,6]) = " + (dotVal != null ? dotVal.toFixed(4) : "—") + "<br>";
        html += "A×[1,0]<sup>T</sup> = [" + (matvecY && matvecY.length ? matvecY.map((x) => x.toFixed(2)).join(", ") : "—") + "]<sup>T</sup>";
        outBasicF32.innerHTML = html;
      } catch (e) {
        outBasicF32.textContent = "Error: " + (e.message || e);
      }
    } else if (outBasicF32) {
      outBasicF32.textContent = "WasmMatrix32 not available (build with wasm + gpu features for f32 ops).";
    }

    // —— Basic f32 CPU vs GPU benchmark ——
    const basicF32BenchRun = byId("basic-f32-bench-run");
    const outBasicF32Bench = byId("out-basic-f32-bench");
    const basicF32VecSize = byId("basic-f32-vec-size");
    if (basicF32BenchRun && outBasicF32Bench && typeof WasmMatrix32 !== "undefined") {
      const dotF32 = typeof lib.dotF32 === "function" ? lib.dotF32 : null;
      const dotF32GpuAsync = typeof lib.dotF32GpuAsync === "function" ? lib.dotF32GpuAsync : null;
      const normF32GpuAsync = typeof lib.normF32GpuAsync === "function" ? lib.normF32GpuAsync : null;
      const matvecF32GpuAsync = typeof lib.matvecF32GpuAsync === "function" ? lib.matvecF32GpuAsync : null;
      const addF32GpuAsync = typeof lib.addF32GpuAsync === "function" ? lib.addF32GpuAsync : null;
      const scaleF32GpuAsync = typeof lib.scaleF32GpuAsync === "function" ? lib.scaleF32GpuAsync : null;
      const NUM_BASIC_RUNS = 5;
      const MATVEC_SIZE = 256;
      basicF32BenchRun.addEventListener("click", async () => {
        basicF32BenchRun.disabled = true;
        outBasicF32Bench.textContent = "Running…";
        await yieldToEventLoop();
        try {
          const n = Math.max(1000, Math.min(10_000_000, parseInt(basicF32VecSize.value, 10) || 100_000));
          const vecA = new Float32Array(n);
          const vecB = new Float32Array(n);
          for (let i = 0; i < n; i++) {
            vecA[i] = Math.sin(i * 0.1) * 0.5 + 0.5;
            vecB[i] = Math.cos(i * 0.07) * 0.5 + 0.5;
          }
          if (typeof initGpuAsync === "function" && typeof gpuAvailable === "function" && !gpuAvailable()) {
            outBasicF32Bench.textContent = "Initializing GPU…";
            try { await initGpuAsync(); updateGpuBackendLabel(); } catch (_) {}
            await yieldToEventLoop();
          }
          let addCpuMs = 0, addGpuMs = 0, scaleCpuMs = 0, scaleGpuMs = 0;
          let dotCpuMs = 0, dotGpuMs = 0, normCpuMs = 0, normGpuMs = 0, matvecCpuMs = 0, matvecGpuMs = 0;
          const addOut = new Float32Array(n);
          const scaleAlpha = 2.5;
          const addCpuTimes = [];
          for (let r = 0; r < NUM_BASIC_RUNS; r++) {
            const t0 = performance.now();
            for (let i = 0; i < n; i++) addOut[i] = vecA[i] + vecB[i];
            addCpuTimes.push(performance.now() - t0);
          }
          addCpuMs = median(addCpuTimes);
          if (addF32GpuAsync && (typeof gpuAvailable === "function" && gpuAvailable())) {
            await addF32GpuAsync(vecA, vecB);
            await yieldToEventLoop();
            const times = [];
            for (let r = 0; r < NUM_BASIC_RUNS; r++) {
              const t0 = performance.now();
              const res = await addF32GpuAsync(vecA, vecB);
              if (res != null) times.push(performance.now() - t0);
            }
            if (times.length > 0) addGpuMs = median(times);
          }
          const scaleCpuTimes = [];
          for (let r = 0; r < NUM_BASIC_RUNS; r++) {
            const t0 = performance.now();
            for (let i = 0; i < n; i++) addOut[i] = scaleAlpha * vecA[i];
            scaleCpuTimes.push(performance.now() - t0);
          }
          scaleCpuMs = median(scaleCpuTimes);
          if (scaleF32GpuAsync && (typeof gpuAvailable === "function" && gpuAvailable())) {
            await scaleF32GpuAsync(scaleAlpha, vecA);
            await yieldToEventLoop();
            const times = [];
            for (let r = 0; r < NUM_BASIC_RUNS; r++) {
              const t0 = performance.now();
              const res = await scaleF32GpuAsync(scaleAlpha, vecA);
              if (res != null) times.push(performance.now() - t0);
            }
            if (times.length > 0) scaleGpuMs = median(times);
          }
          if (dotF32) {
            dotF32(vecA, vecB);
            const times = [];
            for (let r = 0; r < NUM_BASIC_RUNS; r++) {
              const t0 = performance.now();
              dotF32(vecA, vecB);
              times.push(performance.now() - t0);
            }
            dotCpuMs = median(times);
          }
          if (dotF32GpuAsync && (typeof gpuAvailable === "function" && gpuAvailable())) {
            await dotF32GpuAsync(vecA, vecB);
            await yieldToEventLoop();
            const times = [];
            for (let r = 0; r < NUM_BASIC_RUNS; r++) {
              const t0 = performance.now();
              const res = await dotF32GpuAsync(vecA, vecB);
              if (res != null) times.push(performance.now() - t0);
            }
            if (times.length > 0) dotGpuMs = median(times);
          }
          const vecForNorm = WasmVector.fromArray(Array.from(vecA));
          vecForNorm.norm();
          await yieldToEventLoop();
          const normTimes = [];
          for (let r = 0; r < NUM_BASIC_RUNS; r++) {
            const t0 = performance.now();
            vecForNorm.norm();
            normTimes.push(performance.now() - t0);
          }
          normCpuMs = median(normTimes);
          if (normF32GpuAsync && (typeof gpuAvailable === "function" && gpuAvailable())) {
            await normF32GpuAsync(vecA);
            await yieldToEventLoop();
            const times = [];
            for (let r = 0; r < NUM_BASIC_RUNS; r++) {
              const t0 = performance.now();
              const res = await normF32GpuAsync(vecA);
              if (res != null) times.push(performance.now() - t0);
            }
            if (times.length > 0) normGpuMs = median(times);
          }
          const matSize = MATVEC_SIZE * MATVEC_SIZE;
          const matData = new Float32Array(matSize);
          const vec256 = new Float32Array(MATVEC_SIZE);
          for (let i = 0; i < matSize; i++) matData[i] = Math.sin(i * 0.1) * 0.5 + 0.5;
          for (let i = 0; i < MATVEC_SIZE; i++) vec256[i] = Math.sin(i * 0.1) * 0.5 + 0.5;
          const A256 = WasmMatrix32.fromArray(MATVEC_SIZE, MATVEC_SIZE, Array.from(matData));
          A256.mulVectorF32(vec256);
          await yieldToEventLoop();
          const matvecCpuTimes = [];
          for (let r = 0; r < NUM_BASIC_RUNS; r++) {
            const t0 = performance.now();
            A256.mulVectorF32(vec256);
            matvecCpuTimes.push(performance.now() - t0);
          }
          matvecCpuMs = median(matvecCpuTimes);
          if (matvecF32GpuAsync && (typeof gpuAvailable === "function" && gpuAvailable())) {
            await matvecF32GpuAsync(A256, vec256);
            await yieldToEventLoop();
            const times = [];
            for (let r = 0; r < NUM_BASIC_RUNS; r++) {
              const t0 = performance.now();
              const res = await matvecF32GpuAsync(A256, vec256);
              if (res != null) times.push(performance.now() - t0);
            }
            if (times.length > 0) matvecGpuMs = median(times);
          }
          let text = "Basic f32 (median of " + NUM_BASIC_RUNS + " runs)\n";
          text += "add (n=" + n + "): CPU " + addCpuMs.toFixed(2) + " ms";
          if (addGpuMs > 0) text += ", GPU " + addGpuMs.toFixed(2) + " ms";
          text += "\n";
          text += "scale (n=" + n + "): CPU " + scaleCpuMs.toFixed(2) + " ms";
          if (scaleGpuMs > 0) text += ", GPU " + scaleGpuMs.toFixed(2) + " ms";
          text += "\n";
          text += "dot (n=" + n + "): CPU " + dotCpuMs.toFixed(2) + " ms";
          if (dotGpuMs > 0) text += ", GPU " + dotGpuMs.toFixed(2) + " ms";
          text += "\n";
          text += "norm (n=" + n + "): CPU " + normCpuMs.toFixed(2) + " ms";
          if (normGpuMs > 0) text += ", GPU " + normGpuMs.toFixed(2) + " ms";
          text += "\n";
          text += "matvec (" + MATVEC_SIZE + "×" + MATVEC_SIZE + "): CPU " + matvecCpuMs.toFixed(2) + " ms";
          if (matvecGpuMs > 0) text += ", GPU " + matvecGpuMs.toFixed(2) + " ms";
          text += "\n";
          if (typeof gpuAvailable === "function" && !gpuAvailable()) text += "Init GPU for GPU times.";
          outBasicF32Bench.textContent = text;
        } catch (e) {
          outBasicF32Bench.textContent = "Error: " + (e.message || e);
        } finally {
          basicF32BenchRun.disabled = false;
        }
      });
    } else if (outBasicF32Bench) {
      outBasicF32Bench.textContent = "WasmMatrix32 not available.";
    }

    // —— Vector CPU vs GPU (top section) ——
    const vectorCpuGpuRun = byId("vector-cpu-gpu-run");
    const outVectorCpuGpu = byId("out-vector-cpu-gpu");
    const vectorCpuGpuVecSize = byId("vector-cpu-gpu-vec-size");
    const NUM_VECTOR_RUNS = 5;
    if (vectorCpuGpuRun && outVectorCpuGpu) {
      const dotF32 = typeof lib.dotF32 === "function" ? lib.dotF32 : null;
      const dotF32GpuAsync = typeof lib.dotF32GpuAsync === "function" ? lib.dotF32GpuAsync : null;
      const normF32GpuAsync = typeof lib.normF32GpuAsync === "function" ? lib.normF32GpuAsync : null;
      const addF32GpuAsync = typeof lib.addF32GpuAsync === "function" ? lib.addF32GpuAsync : null;
      vectorCpuGpuRun.addEventListener("click", async () => {
        vectorCpuGpuRun.disabled = true;
        outVectorCpuGpu.textContent = "Running…";
        await yieldToEventLoop();
        try {
          const n = Math.max(1000, Math.min(100_000_000, parseInt(vectorCpuGpuVecSize?.value, 10) || 100_000));
          const vecA = new Float32Array(n);
          const vecB = new Float32Array(n);
          for (let i = 0; i < n; i++) {
            vecA[i] = Math.sin(i * 0.1) * 0.5 + 0.5;
            vecB[i] = Math.cos(i * 0.07) * 0.5 + 0.5;
          }
          if (typeof initGpuAsync === "function" && typeof gpuAvailable === "function" && !gpuAvailable()) {
            outVectorCpuGpu.textContent = "Initializing GPU…";
            try { await initGpuAsync(); updateGpuBackendLabel(); } catch (_) {}
            await yieldToEventLoop();
          }
          let addCpuMs = 0, addGpuMs = 0, dotCpuMs = 0, dotGpuMs = 0, normCpuMs = 0, normGpuMs = 0;
          const addOut = new Float32Array(n);
          const addCpuTimes = [];
          for (let r = 0; r < NUM_VECTOR_RUNS; r++) {
            const t0 = performance.now();
            for (let i = 0; i < n; i++) addOut[i] = vecA[i] + vecB[i];
            addCpuTimes.push(performance.now() - t0);
          }
          addCpuMs = median(addCpuTimes);
          if (addF32GpuAsync && (typeof gpuAvailable === "function" && gpuAvailable())) {
            await addF32GpuAsync(vecA, vecB);
            await yieldToEventLoop();
            const times = [];
            for (let r = 0; r < NUM_VECTOR_RUNS; r++) {
              const t0 = performance.now();
              const res = await addF32GpuAsync(vecA, vecB);
              if (res != null) times.push(performance.now() - t0);
            }
            if (times.length > 0) addGpuMs = median(times);
          }
          if (dotF32) {
            dotF32(vecA, vecB);
            const times = [];
            for (let r = 0; r < NUM_VECTOR_RUNS; r++) {
              const t0 = performance.now();
              dotF32(vecA, vecB);
              times.push(performance.now() - t0);
            }
            dotCpuMs = median(times);
          }
          if (dotF32GpuAsync && (typeof gpuAvailable === "function" && gpuAvailable())) {
            await dotF32GpuAsync(vecA, vecB);
            await yieldToEventLoop();
            const times = [];
            for (let r = 0; r < NUM_VECTOR_RUNS; r++) {
              const t0 = performance.now();
              const res = await dotF32GpuAsync(vecA, vecB);
              if (res != null) times.push(performance.now() - t0);
            }
            if (times.length > 0) dotGpuMs = median(times);
          }
          const vecForNorm = WasmVector.fromArray(Array.from(vecA));
          vecForNorm.norm();
          await yieldToEventLoop();
          const normTimes = [];
          for (let r = 0; r < NUM_VECTOR_RUNS; r++) {
            const t0 = performance.now();
            vecForNorm.norm();
            normTimes.push(performance.now() - t0);
          }
          normCpuMs = median(normTimes);
          if (normF32GpuAsync && (typeof gpuAvailable === "function" && gpuAvailable())) {
            await normF32GpuAsync(vecA);
            await yieldToEventLoop();
            const times = [];
            for (let r = 0; r < NUM_VECTOR_RUNS; r++) {
              const t0 = performance.now();
              const res = await normF32GpuAsync(vecA);
              if (res != null) times.push(performance.now() - t0);
            }
            if (times.length > 0) normGpuMs = median(times);
          }
          let text = "Vector f32 (median of " + NUM_VECTOR_RUNS + " runs)\n";
          text += "add (n=" + n + "): CPU " + addCpuMs.toFixed(2) + " ms";
          if (addGpuMs > 0) text += ", GPU " + addGpuMs.toFixed(2) + " ms";
          text += "\n";
          text += "dot (n=" + n + "): CPU " + dotCpuMs.toFixed(2) + " ms";
          if (dotGpuMs > 0) text += ", GPU " + dotGpuMs.toFixed(2) + " ms";
          text += "\n";
          text += "norm (n=" + n + "): CPU " + normCpuMs.toFixed(2) + " ms";
          if (normGpuMs > 0) text += ", GPU " + normGpuMs.toFixed(2) + " ms";
          text += "\n";
          if (typeof gpuAvailable === "function" && !gpuAvailable()) text += "Init GPU (Matrix section) for GPU times.";
          outVectorCpuGpu.textContent = text;
        } catch (e) {
          outVectorCpuGpu.textContent = "Error: " + (e.message || e);
        } finally {
          vectorCpuGpuRun.disabled = false;
        }
      });
    }

    // —— Storage ——
    const STORAGE_VALUES = [1, 2, 3, 4, 5, 6, 7, 8, 9];
    const STORAGE_ROWS = 3, STORAGE_COLS = 3;
    const colMajorData = colMajorFlat(STORAGE_ROWS, STORAGE_COLS, STORAGE_VALUES);
    const rowMajorData = rowMajorFlat(STORAGE_ROWS, STORAGE_COLS, STORAGE_VALUES);
    bindExampleSelector("storage-examples", ["Column-major", "Row-major"], (i) => {
      const storage = i === 0 ? "column" : "row";
      const data = i === 0 ? colMajorData : rowMajorData;
      const canvas = byId("canvas-storage");
      const ctx = canvas.getContext("2d");
      drawStorageGrid(ctx, canvas.width, canvas.height, STORAGE_ROWS, STORAGE_COLS, storage, data);
      renderStorageFlatArray("storage-flat-array", data, storage);
    });
    (() => {
      const canvas = byId("canvas-storage");
      const ctx = canvas.getContext("2d");
      drawStorageGrid(ctx, canvas.width, canvas.height, STORAGE_ROWS, STORAGE_COLS, "column", colMajorData);
      renderStorageFlatArray("storage-flat-array", colMajorData, "column");
    })();

    // —— Cholesky ——
    const CHOLESKY_EXAMPLES = [
      { A: [4, 2, 2, 3], b: [6, 5], rows: 2, cols: 2 },
      { A: [1, 0, 0, 4], b: [1, 8], rows: 2, cols: 2 },
      { A: [1, 1, 1, 1, 1, 5, 5, 5, 1, 5, 14, 14, 1, 5, 14, 15], b: [4, 14, 32, 33], rows: 4, cols: 4 },
    ];
    const choleskyResults = CHOLESKY_EXAMPLES.map((ex) => {
      const rows = ex.rows ?? 2;
      const cols = ex.cols ?? 2;
      const A = WasmMatrix.fromArray(rows, cols, ex.A);
      const b = WasmVector.fromArray(ex.b);
      const chol = new WasmCholesky(A);
      return { A, b: ex.b, L: chol.getL(), x: chol.solve(b).toArray(), rows, cols };
    });
    function updateCholeskyOutput(i) {
      const r = choleskyResults[i];
      const rows = r.rows;
      const cols = r.cols;
      const aArr = r.A.toArray();
      const lArr = r.L.toArray();
      let ltBlock = "";
      if (rows === 4 && cols === 4) {
        const ltArr = [];
        for (let j = 0; j < cols; j++)
          for (let i = 0; i < rows; i++)
            ltArr.push(lArr[i * rows + j]);
        ltBlock = "<div class=\"matrix-block\"><strong>L<sup>T</sup>:</strong>" +
          renderMatrixHTMLWithColors(rows, cols, ltArr, { colorBy: "structure", structure: "upper", decimals: 0 }) + "</div>";
      }
      byId("out-cholesky").innerHTML =
        "<div class=\"matrix-block\"><strong>A (" + rows + "×" + cols + " SPD):</strong>" +
          renderMatrixHTMLWithColors(rows, cols, aArr, { colorBy: "value", decimals: 0 }) + "</div>" +
        "<div class=\"matrix-block\"><strong>L (Cholesky):</strong>" +
          renderMatrixHTMLWithColors(rows, cols, lArr, { colorBy: "structure", structure: "lower", decimals: 0 }) + "</div>" +
        ltBlock +
        "<div class=\"vector-inline\"><strong>b:</strong> [" + r.b.map((x) => x.toFixed(2)).join(", ") + "]</div>" +
        "<div class=\"vector-inline\"><strong>x (solution):</strong> [" + r.x.map((x) => x.toFixed(4)).join(", ") + "]</div>";
      drawVectorBarsGeneric("canvas-cholesky-x", r.x);
    }
    bindExampleSelector("cholesky-examples", ["Example 1", "Example 2", "Example 3 (4×4)"], updateCholeskyOutput);
    updateCholeskyOutput(0);

    // —— SVD ——
    const SVD_EXAMPLES = [
      { data: [1, 0, 0, 1, 1, 1], rows: 3, cols: 2 },
      { data: [1, 2, 3, 4, 5, 6], rows: 2, cols: 3 },
    ];
    const outSvdEl = byId("out-svd");
    if (outSvdEl) outSvdEl.textContent = "Computing…";
    const svdResults = await Promise.all(SVD_EXAMPLES.map(async (ex) => {
      const M = WasmMatrix.fromArray(ex.rows, ex.cols, ex.data);
      const svd = typeof M.svdEconAsync === "function" ? await M.svdEconAsync() : M.svdEcon();
      return { sigma: svd.getSigma().toArray(), u: svd.getU(), v: svd.getV(), M, rows: ex.rows, cols: ex.cols };
    }));
    function updateSvdOutput(i) {
      const r = svdResults[i];
      const mArr = r.M.toArray();
      const uArr = r.u.toArray();
      const vArr = r.v.toArray();
      byId("out-svd").innerHTML =
        "<div class=\"matrix-block\"><strong>M (input " + r.rows + "×" + r.cols + "):</strong>" +
          renderMatrixHTMLWithColors(r.rows, r.cols, mArr, { colorBy: "value" }) + "</div>" +
        "<div class=\"matrix-block\"><strong>U (" + r.u.rows + "×" + r.u.cols + "):</strong>" +
          renderMatrixHTMLWithColors(r.u.rows, r.u.cols, uArr, { colorBy: "value" }) + "</div>" +
        "<div class=\"vector-inline\"><strong>σ:</strong> [" + r.sigma.map((x) => x.toFixed(4)).join(", ") + "]</div>" +
        "<div class=\"matrix-block\"><strong>V (" + r.v.rows + "×" + r.v.cols + "):</strong>" +
          renderMatrixHTMLWithColors(r.v.rows, r.v.cols, vArr, { colorBy: "value" }) + "</div>";
      drawVectorBarsGeneric("canvas-svd-sigma", r.sigma);
    }
    bindExampleSelector("svd-examples", ["Example 1", "Example 2"], updateSvdOutput);
    updateSvdOutput(0);

    // —— LU ——
    const LU_EXAMPLES = [
      { A: [1, 1, 1, -1], b: [2, 0], rows: 2, cols: 2 },
      { A: [2, 1, 1, 1], b: [4, 3], rows: 2, cols: 2 },
      { A: [1, 0, 1, 2, 2, 0, 0, 1, 1], b: [1, 2, 2], rows: 3, cols: 3 },
      { A: [2, 1, 0, 0, 1, 2, 1, 0, 0, 1, 2, 1, 0, 0, 1, 2], b: [1, 2, 3, 4], rows: 4, cols: 4 },
    ];
    const luResults = LU_EXAMPLES.map((ex) => {
      const rows = ex.rows ?? 2;
      const cols = ex.cols ?? 2;
      const A = WasmMatrix.fromArray(rows, cols, ex.A);
      const b = WasmVector.fromArray(ex.b);
      const lu = new WasmLu(A);
      const x = lu.solve(b).toArray();
      return { A, b: ex.b, x, rows, cols };
    });
    function updateLuOutput(i) {
      const r = luResults[i];
      const rows = r.rows;
      const cols = r.cols;
      byId("out-lu").innerHTML =
        "<div class=\"matrix-block\"><strong>A (" + rows + "×" + cols + "):</strong>" +
          renderMatrixHTMLWithColors(rows, cols, r.A.toArray(), { colorBy: "value" }) + "</div>" +
        "<div class=\"vector-inline\"><strong>b:</strong> [" + r.b.map((x) => Number(x).toFixed(2)).join(", ") + "]</div>" +
        "<div class=\"vector-inline\"><strong>x (solution):</strong> [" + r.x.map((x) => x.toFixed(4)).join(", ") + "]</div>";
      drawVectorBarsGeneric("canvas-lu-x", r.x);
    }
    bindExampleSelector("lu-examples", ["Example 1", "Example 2", "Example 3 (3×3)", "Example 4 (4×4)"], updateLuOutput);
    updateLuOutput(0);

    // —— Sparse (CSR diagram) ——
    const SPARSE_TRIPLETS = [[0, 0, 1], [0, 2, 2], [1, 1, 3], [2, 0, 4]];
    const sparseRows = 3, sparseCols = 3;
    const sparseGrid = [];
    for (let i = 0; i < sparseRows; i++) {
      sparseGrid[i] = [];
      for (let j = 0; j < sparseCols; j++) sparseGrid[i][j] = 0;
    }
    SPARSE_TRIPLETS.forEach(([i, j, val]) => {
      if (i < sparseRows && j < sparseCols) sparseGrid[i][j] = val;
    });
    const sparseColMajor = [];
    for (let j = 0; j < sparseCols; j++)
      for (let i = 0; i < sparseRows; i++)
        sparseColMajor.push(sparseGrid[i][j]);
    const outSparse = byId("out-sparse-matrix");
    if (outSparse) {
      outSparse.innerHTML = "<div class=\"matrix-block\"><strong>Matrix (3×3) — zeros gray, non-zeros colored:</strong>" +
        renderMatrixHTMLWithColors(sparseRows, sparseCols, sparseColMajor, { colorBy: "value", decimals: 0 }) + "</div>";
    }
    drawSparseCSRDiagram("canvas-sparse-csr", 340, 220, 3, 3, SPARSE_TRIPLETS);

  } catch (e) {
    const msg = (e.message || "").toLowerCase();
    const out = byId("out-vector");
    if (out) { out.className = "error"; out.textContent = "Error: " + (e.message || String(e)); }
    showError((msg.includes("fetch") || msg.includes("import") ? needBuild + "\n\n" : "") + (e.message || String(e)));
  }
}
