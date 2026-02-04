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
    }

    // —— Matrix 2D ——
    const MATRIX_EXAMPLES = [
      { A: [1, 0, 0, 0, 1, 0, 0, 0, 1], B: [1, 2, 3, 4, 5, 6, 7, 8, 9], rows: 3, cols: 3 },
      { A: [1, 0, 0, 1], B: [2, 1, 0, 2], rows: 2, cols: 2 },
      { A: [1, 2, 0, 1], B: [0, 1, 1, 0], rows: 2, cols: 2 },
    ];
    const matrixResults = MATRIX_EXAMPLES.map((ex) => {
      const A = WasmMatrix.fromArray(ex.rows, ex.cols, ex.A);
      const B = WasmMatrix.fromArray(ex.rows, ex.cols, ex.B);
      const C = A.mul(B);
      return { ...ex, A, B, C };
    });
    bindExampleSelector("matrix-examples", ["Example 1", "Example 2", "Example 3"], (i) => {
      const r = matrixResults[i];
      byId("out-matrix").innerHTML =
        "A:" + renderMatrixHTML(r.rows, r.cols, r.A.toArray()) +
        "B:" + renderMatrixHTML(r.rows, r.cols, r.B.toArray()) +
        "C = A×B:" + renderMatrixHTML(r.rows, r.cols, r.C.toArray());
    });
    byId("out-matrix").innerHTML =
      "A (identity):" + renderMatrixHTML(3, 3, matrixResults[0].A.toArray()) +
      "B:" + renderMatrixHTML(3, 3, matrixResults[0].B.toArray()) +
      "C = A×B:" + renderMatrixHTML(3, 3, matrixResults[0].C.toArray());

    if (typeof initGpuAsync === "function" && typeof gpuAvailable === "function") {
      const gpuStatusEl = byId("gpu-matrix-status");
      const gpuBackendEl = byId("gpu-matrix-backend");
      const gpuInitBtn = byId("gpu-init-btn");
      if (gpuStatusEl && gpuBackendEl && gpuInitBtn) {
        gpuStatusEl.style.display = "block";
        function updateGpuBackendLabel() {
          gpuBackendEl.textContent = gpuAvailable() ? "Backend: GPU" : "Backend: CPU";
          gpuInitBtn.disabled = gpuAvailable();
          const gv = byId("gpu-vector-backend");
          if (gv) gv.textContent = gpuAvailable() ? "Backend: GPU" : "Backend: CPU";
        }
        updateGpuBackendLabel();
        gpuInitBtn.addEventListener("click", () => {
          gpuInitBtn.disabled = true;
          gpuInitBtn.textContent = "Initializing…";
          initGpuAsync().then((ok) => {
            updateGpuBackendLabel();
            gpuInitBtn.textContent = "Init GPU";
            gpuInitBtn.disabled = ok;
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
        const gpuBackendEl = byId("gpu-matrix-backend");
        const gv = byId("gpu-vector-backend");
        if (gpuBackendEl) gpuBackendEl.textContent = gpuAvailable() ? "Backend: GPU" : "Backend: CPU";
        if (gv) gv.textContent = gpuAvailable() ? "Backend: GPU" : "Backend: CPU";
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
      const { n, numRuns, cpu64Ms, cpu32Ms, gpu32Ms, sample, gpuMatmulUnavailable, gpuUnavailableMessage } = payload;
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
          const total = n * n;
          const numRuns = n >= LARGE_SIZE_THRESHOLD ? NUM_MATMUL_RUNS_LARGE : NUM_MATMUL_RUNS;
          outLargeMatrix.textContent = n + "×" + n + " matmul\nPreparing…";
          await yieldToEventLoop();
          const data64 = new Float64Array(total * 2);
          const data32 = new Float32Array(total * 2);
          for (let i = 0; i < total * 2; i++) {
            const v = (i % 100) * 0.01;
            data64[i] = v;
            data32[i] = v;
          }
          const A64 = WasmMatrix.fromArray(n, n, Array.from(data64.subarray(0, total)));
          const B64 = WasmMatrix.fromArray(n, n, Array.from(data64.subarray(total, total * 2)));
          outLargeMatrix.textContent = n + "×" + n + " matmul\nWarmup…";
          await yieldToEventLoop();
          const hasGpuBuild = typeof WasmMatrix32 !== "undefined" && typeof gpuAvailable === "function";
          const matmulF32Cpu = typeof lib.matmulF32Cpu === "function" ? lib.matmulF32Cpu : null;
          const gpuLastError = typeof lib.gpuLastError === "function" ? lib.gpuLastError : null;

          if (hasGpuBuild && typeof initGpuAsync === "function" && !gpuAvailable()) {
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
          if (A32 && B32 && gpuAvailable()) {
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
          const sample = C64 && C64.toArray ? C64.toArray()[0].toFixed(4) : "—";

          let cpu32Ms = 0;
          let gpu32Ms = 0;
          const canMeasureCpu32 = A32 && B32 && (matmulF32Cpu || !gpuAvailable());
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
          if (hasGpuBuild && gpuAvailable() && A32 && B32) {
            const gpu32Times = [];
            for (let r = 0; r < numRuns; r++) {
              outLargeMatrix.textContent = n + "×" + n + " matmul\nRunning… GPU (f32) " + (r + 1) + "/" + numRuns;
              await yieldToEventLoop();
              const t0 = performance.now();
              A32.mul(B32);
              gpu32Times.push(performance.now() - t0);
              await yieldToEventLoop();
            }
            gpu32Ms = median(gpu32Times);
          }

          renderMatmulResults({
            n, numRuns, cpu64Ms, cpu32Ms, gpu32Ms, sample,
            gpuMatmulUnavailable: gpuMatmulAvailable && !gpuMatmulAvailable(),
            gpuUnavailableMessage: (hasGpuBuild && !gpuAvailable()) ? (gpuLastError ? gpuLastError().trim() : "unavailable (init WebGPU first)") : (!hasGpuBuild ? "build with wasm-build-gpu, then Init GPU" : null),
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
        "A:" + renderMatrixHTML(r.rows, r.cols, r.A.toArray()) +
        "v: [" + r.v.join(", ") + "]<sup>T</sup><br>y = A×v: [" + resStr + "]<sup>T</sup>";
      drawVectorBarsGeneric("canvas-matrix-vector", r.result);
    });
    (() => {
      const r = matrixVectorResults[0];
      const resStr = r.result.map((x) => Number(x).toFixed(2)).join(", ");
      byId("out-matrix-vector").innerHTML =
        "A:" + renderMatrixHTML(r.rows, r.cols, r.A.toArray()) +
        "v: [" + r.v.join(", ") + "]<sup>T</sup><br>y = A×v: [" + resStr + "]<sup>T</sup>";
      drawVectorBarsGeneric("canvas-matrix-vector", r.result);
    })();

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
    const svdResults = SVD_EXAMPLES.map((ex) => {
      const M = WasmMatrix.fromArray(ex.rows, ex.cols, ex.data);
      const svd = M.svdEcon();
      return { sigma: svd.getSigma().toArray(), u: svd.getU(), v: svd.getV() };
    });
    function updateSvdOutput(i) {
      const r = svdResults[i];
      byId("out-svd").innerHTML =
        "<div class=\"vector-inline\"><strong>σ:</strong> [" + r.sigma.map((x) => x.toFixed(4)).join(", ") + "]</div>" +
        "<div class=\"dims-inline\">U: " + r.u.rows + "×" + r.u.cols + ", V: " + r.v.rows + "×" + r.v.cols + "</div>";
      drawVectorBarsGeneric("canvas-svd-sigma", r.sigma);
    }
    bindExampleSelector("svd-examples", ["Example 1", "Example 2"], updateSvdOutput);
    updateSvdOutput(0);

    // —— LU ——
    const LU_EXAMPLES = [
      { A: [1, 1, 1, -1], b: [2, 0], rows: 2, cols: 2 },
      { A: [2, 1, 1, 1], b: [4, 3], rows: 2, cols: 2 },
      { A: [1, 0, 1, 2, 2, 0, 0, 1, 1], b: [1, 2, 2], rows: 3, cols: 3 },
      { A: [2, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1], b: [1, 2, 3, 4], rows: 4, cols: 4 },
    ];
    const luResults = LU_EXAMPLES.map((ex) => {
      const rows = ex.rows ?? 2;
      const cols = ex.cols ?? 2;
      const A = WasmMatrix.fromArray(rows, cols, ex.A);
      const b = WasmVector.fromArray(ex.b);
      const lu = new WasmLu(A);
      return { A, b: ex.b, x: lu.solve(b).toArray(), rows, cols };
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
    drawSparseCSRDiagram("canvas-sparse-csr", 340, 220, 3, 3, SPARSE_TRIPLETS);

  } catch (e) {
    const msg = (e.message || "").toLowerCase();
    const out = byId("out-vector");
    if (out) { out.className = "error"; out.textContent = "Error: " + (e.message || String(e)); }
    showError((msg.includes("fetch") || msg.includes("import") ? needBuild + "\n\n" : "") + (e.message || String(e)));
  }
}
