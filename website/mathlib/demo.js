/**
 * mathlib WASM demo — Demos: Vector, Matrix, K-means, PCA, SVM, Distance, Cholesky, SVD, Simplex, Camera.
 * Import pkg/mathlib.js and run each demo, update DOM.
 */

const needHttp =
  "This page must be served over HTTP. Do not open the HTML file directly (file://).\n\nFrom the mathlib folder run:  npx serve .\nThen open:  /wasm-demo/";
const needBuild =
  "Cannot load pkg/mathlib.js — build first (from repo root):  just wasm-build\nThen refresh.";
const needRebuild =
  "This demo requires a rebuild. From repo root run:  just wasm-build\nThen refresh the page.";

function byId(id) {
  return document.getElementById(id);
}

/** Create example selector buttons; onSelect(i) is called when user picks example i. */
function bindExampleSelector(containerId, labels, onSelect) {
  const container = byId(containerId);
  if (!container) return;
  container.innerHTML = "";
  labels.forEach((label, i) => {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.textContent = label;
    btn.addEventListener("click", () => {
      container.querySelectorAll("button").forEach((b, j) => b.classList.toggle("active", j === i));
      onSelect(i);
    });
    container.appendChild(btn);
  });
  if (labels.length > 0) container.querySelector("button").classList.add("active");
}

function showError(message) {
  const callout = byId("error-callout");
  const pre = byId("error-message");
  if (callout && pre) {
    pre.textContent = message;
    callout.style.display = "block";
  }
}

/** Scale 2D points to canvas coordinates (y flipped). */
function scaleToCanvas(points, width, height, padding) {
  if (points.length === 0) return [];
  let minX = points[0][0],
    maxX = minX,
    minY = points[0][1],
    maxY = minY;
  for (const p of points) {
    minX = Math.min(minX, p[0]);
    maxX = Math.max(maxX, p[0]);
    minY = Math.min(minY, p[1]);
    maxY = Math.max(maxY, p[1]);
  }
  const rangeX = maxX - minX || 1;
  const rangeY = maxY - minY || 1;
  const pad = padding ?? 20;
  const w = width - 2 * pad;
  const h = height - 2 * pad;
  return points.map((p) => [
    pad + ((p[0] - minX) / rangeX) * w,
    height - pad - ((p[1] - minY) / rangeY) * h,
  ]);
}

/** Render column-major f64 matrix as HTML table (rows x cols). */
function renderMatrixHTML(rows, cols, data) {
  let html = '<table class="matrix-table"><tbody>';
  for (let i = 0; i < rows; i++) {
    html += "<tr>";
    for (let j = 0; j < cols; j++)
      html += "<td>" + (data[j * rows + i].toFixed(2)) + "</td>";
    html += "</tr>";
  }
  html += "</tbody></table>";
  return html;
}

/** Render 4×4 column-major float array as HTML table (for WasmMatrix32). */
function renderMatrix4x4Float(data) {
  const rows = 4;
  const cols = 4;
  const arr = Array.isArray(data) ? data : Array.from(data);
  let html = '<table class="matrix-table"><tbody>';
  for (let i = 0; i < rows; i++) {
    html += "<tr>";
    for (let j = 0; j < cols; j++)
      html += "<td>" + (arr[j * rows + i].toFixed(4)) + "</td>";
    html += "</tr>";
  }
  html += "</tbody></table>";
  return html;
}

/** Circular layout: return array of [x, y] in pixel coords for n nodes. */
function graphNodePositions(n, width, height, padding) {
  const cx = width / 2;
  const cy = height / 2;
  const r = Math.min(width, height) / 2 - (padding ?? 24);
  const out = [];
  for (let i = 0; i < n; i++) {
    const angle = (2 * Math.PI * i) / n - Math.PI / 2;
    out.push([cx + r * Math.cos(angle), cy + r * Math.sin(angle)]);
  }
  return out;
}

/** Draw weighted directed graph on canvas. pathNodes = ordered list of node ids on shortest path. */
function drawGraphOnCanvas(ctx, width, height, n, edges, pathNodes, distances, source) {
  const positions = graphNodePositions(n, width, height, 28);
  const pathSet = new Set();
  for (let k = 0; k < pathNodes.length - 1; k++)
    pathSet.add(pathNodes[k] + "," + pathNodes[k + 1]);

  ctx.fillStyle = "#fff";
  ctx.fillRect(0, 0, width, height);

  const edgeColor = "#adb5bd";
  const pathColor = "#0d6efd";
  const nodeFill = "#fff";
  const nodeStroke = "#212529";
  const nodeRadius = 14;
  const pathRadius = 16;

  for (let i = 0; i < edges.length; i += 3) {
    const u = edges[i];
    const v = edges[i + 1];
    const w = edges[i + 2];
    const onPath = pathSet.has(u + "," + v);
    const [x1, y1] = positions[u];
    const [x2, y2] = positions[v];

    ctx.strokeStyle = onPath ? pathColor : edgeColor;
    ctx.lineWidth = onPath ? 3 : 1.5;
    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x2, y2);
    ctx.stroke();
  }

  for (let i = 0; i < n; i++) {
    const [x, y] = positions[i];
    const isSource = i === source;
    const r = isSource ? pathRadius : nodeRadius;
    ctx.fillStyle = nodeFill;
    ctx.strokeStyle = nodeStroke;
    ctx.lineWidth = isSource ? 2.5 : 1.5;
    ctx.beginPath();
    ctx.arc(x, y, r, 0, 2 * Math.PI);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = nodeStroke;
    ctx.font = "12px \"DM Sans\", system-ui, sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(String(i), x, y);
    const dist = distances[i];
    if (dist != null && dist !== Infinity) {
      ctx.font = "10px \"DM Sans\", system-ui, sans-serif";
      ctx.fillStyle = "#6c757d";
      ctx.fillText("d=" + Number(dist).toFixed(0), x, y + 20);
    }
  }

  ctx.font = "11px \"DM Sans\", system-ui, sans-serif";
  ctx.fillStyle = "#495057";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  for (let i = 0; i < edges.length; i += 3) {
    const u = edges[i];
    const v = edges[i + 1];
    const w = edges[i + 2];
    const [x1, y1] = positions[u];
    const [x2, y2] = positions[v];
    const mx = (x1 + x2) / 2;
    const my = (y1 + y2) / 2;
    ctx.fillText(Number(w) === w && w % 1 === 0 ? String(w) : w.toFixed(1), mx, my);
  }
}

const COLOR_PALETTE = [
  "#0d6efd", "#dc3545", "#198754", "#fd7e14", "#6f42c1",
  "#20c997", "#e83e8c", "#ffc107", "#0dcaf0",
];

/** Draw graph with vertex colors. colors = array of color indices. */
function drawGraphColoringOnCanvas(ctx, width, height, n, edges, colors) {
  const positions = graphNodePositions(n, width, height, 28);
  ctx.fillStyle = "#fff";
  ctx.fillRect(0, 0, width, height);
  const edgeColor = "#adb5bd";
  const nodeStroke = "#212529";
  const nodeRadius = 14;
  for (let i = 0; i < edges.length; i += 3) {
    const u = edges[i], v = edges[i + 1], w = edges[i + 2];
    const [x1, y1] = positions[u], [x2, y2] = positions[v];
    ctx.strokeStyle = edgeColor;
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x2, y2);
    ctx.stroke();
  }
  for (let i = 0; i < n; i++) {
    const [x, y] = positions[i];
    const ci = (colors && colors[i] != null) ? colors[i] % COLOR_PALETTE.length : 0;
    ctx.fillStyle = COLOR_PALETTE[ci];
    ctx.strokeStyle = nodeStroke;
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.arc(x, y, nodeRadius, 0, 2 * Math.PI);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = "#fff";
    ctx.font = "12px \"DM Sans\", system-ui, sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(String(i), x, y);
  }
  ctx.font = "11px \"DM Sans\", system-ui, sans-serif";
  ctx.fillStyle = "#495057";
  for (let i = 0; i < edges.length; i += 3) {
    const u = edges[i], v = edges[i + 1], w = edges[i + 2];
    const [x1, y1] = positions[u], [x2, y2] = positions[v];
    const mx = (x1 + x2) / 2, my = (y1 + y2) / 2;
    ctx.fillText(Number(w) === w && w % 1 === 0 ? String(w) : w.toFixed(1), mx, my);
  }
}

/** Draw storage layout grid: 3×3 matrix with cells colored by flat-array index. storage = "column" | "row". */
function drawStorageGrid(ctx, width, height, rows, cols, storage, values) {
  const palette = [
    "#e3f2fd", "#bbdefb", "#90caf9",
    "#ffebee", "#ffcdd2", "#ef9a9a",
    "#e8f5e9", "#c8e6c9", "#a5d6a7",
  ];
  const cellW = Math.min(60, (width - 40) / cols);
  const cellH = Math.min(50, (height - 40) / rows);
  const startX = (width - cellW * cols) / 2;
  const startY = 20;
  ctx.fillStyle = "#fff";
  ctx.fillRect(0, 0, width, height);
  for (let i = 0; i < rows; i++) {
    for (let j = 0; j < cols; j++) {
      const idx = storage === "column" ? j * rows + i : i * cols + j;
      const val = storage === "column" ? values[j * rows + i] : values[i * cols + j];
      const x = startX + j * cellW;
      const y = startY + i * cellH;
      ctx.fillStyle = palette[idx % palette.length];
      ctx.strokeStyle = "#495057";
      ctx.lineWidth = 1;
      ctx.fillRect(x, y, cellW, cellH);
      ctx.strokeRect(x, y, cellW, cellH);
      ctx.fillStyle = "#212529";
      ctx.font = "14px \"DM Sans\", system-ui, sans-serif";
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText(String(val), x + cellW / 2, y + cellH / 2 - 6);
      ctx.font = "10px \"DM Sans\", system-ui, sans-serif";
      ctx.fillStyle = "#6c757d";
      ctx.fillText("[" + idx + "]", x + cellW / 2, y + cellH / 2 + 8);
    }
  }
}

/** Draw vector bar chart (reusable). */
function drawVectorBarsGeneric(canvasId, c) {
  const canvas = byId(canvasId);
  if (!canvas) return;
  const w = canvas.width, h = canvas.height;
  const ctx = canvas.getContext("2d");
  ctx.fillStyle = "#fff";
  ctx.fillRect(0, 0, w, h);
  const n = c.length;
  if (n === 0) return;
  const pad = 20;
  const barW = Math.max(2, (w - 2 * pad) / n - 4);
  const maxAbs = Math.max(1e-10, ...c.map((x) => Math.abs(x)));
  const scale = (h - 2 * pad) / 2 / maxAbs;
  const midY = h / 2;
  for (let i = 0; i < n; i++) {
    const x = pad + i * ((w - 2 * pad) / n) + 2;
    const val = c[i];
    const bh = val * scale;
    ctx.fillStyle = val >= 0 ? "#0d6efd" : "#fd7e14";
    ctx.fillRect(x, val >= 0 ? midY - bh : midY, barW, Math.abs(bh));
  }
  ctx.strokeStyle = "#212529";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(0, midY);
  ctx.lineTo(w, midY);
  ctx.stroke();
}

/** Draw graph with BFS/DFS order labels. order = visit order, depth = optional depth array. */
function drawGraphTreeOnCanvas(ctx, width, height, n, edges, order, depth, source) {
  const positions = graphNodePositions(n, width, height, 28);
  ctx.fillStyle = "#fff";
  ctx.fillRect(0, 0, width, height);
  const edgeColor = "#adb5bd";
  const nodeFill = "#fff";
  const nodeStroke = "#212529";
  const nodeRadius = 14;
  const orderRank = {};
  if (order) for (let i = 0; i < order.length; i++) orderRank[order[i]] = i;
  for (let i = 0; i < edges.length; i += 3) {
    const u = edges[i], v = edges[i + 1], w = edges[i + 2];
    const [x1, y1] = positions[u], [x2, y2] = positions[v];
    ctx.strokeStyle = edgeColor;
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x2, y2);
    ctx.stroke();
  }
  for (let i = 0; i < n; i++) {
    const [x, y] = positions[i];
    ctx.fillStyle = nodeFill;
    ctx.strokeStyle = nodeStroke;
    ctx.lineWidth = i === source ? 2.5 : 1.5;
    ctx.beginPath();
    ctx.arc(x, y, nodeRadius, 0, 2 * Math.PI);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = nodeStroke;
    ctx.font = "12px \"DM Sans\", system-ui, sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(String(i), x, y - 6);
    if (orderRank[i] != null) {
      ctx.font = "10px \"DM Sans\", system-ui, sans-serif";
      ctx.fillStyle = "#6c757d";
      ctx.fillText("ord=" + orderRank[i], x, y + 8);
    }
    if (depth && depth[i] != null && depth[i] !== 4294967295) {
      ctx.fillText("d=" + depth[i], x, y + 20);
    }
  }
  ctx.font = "11px \"DM Sans\", system-ui, sans-serif";
  ctx.fillStyle = "#495057";
  for (let i = 0; i < edges.length; i += 3) {
    const u = edges[i], v = edges[i + 1], w = edges[i + 2];
    const [x1, y1] = positions[u], [x2, y2] = positions[v];
    const mx = (x1 + x2) / 2, my = (y1 + y2) / 2;
    ctx.fillText(Number(w) === w && w % 1 === 0 ? String(w) : w.toFixed(1), mx, my);
  }
}

if (window.location.protocol === "file:") {
  showError(needHttp);
  byId("out-vector").className = "error";
  byId("out-vector").textContent = needHttp;
} else {
  try {
    const lib = await import("./pkg/mathlib.js");
    await lib.default();

    const {
      WasmMatrix,
      WasmMatrix32,
      WasmVector,
      WasmKmeans,
      WasmPca,
      WasmSvm,
      WasmDistance,
      WasmCholesky,
      WasmCg,
      WasmSimplexResult,
    } = lib;
    const WasmDbscan = lib.WasmDbscan;
    const WasmLu = lib.WasmLu;
    const WasmGraph = lib.WasmGraph;
    const WasmSvmRbf = lib.WasmSvmRbf;
    /** Build undirected graph from edge list [u, v, w, ...]. */
    function buildUndirectedGraph(n, edges) {
      const g = new WasmGraph(n);
      for (let i = 0; i < edges.length; i += 3) {
        g.addEdgeUndirected(edges[i], edges[i + 1], edges[i + 2]);
      }
      return g;
    }
    const psoMinimize = lib.psoMinimize;
    const lineSearchBacktracking = lib.lineSearchBacktracking;
    const wave2d = lib.wave2d;
    const wave2dParams = lib.wave2dParams;
    const perlin2d = lib.perlin2d;
    const fbm2dPerlin = lib.fbm2dPerlin;
    const NOISE_LABEL = lib.NOISE_LABEL;
    const initGpuAsync = lib.initGpuAsync;
    const gpuAvailable = lib.gpuAvailable;

    // —— 1. Vector add ——
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
      return {
        a: ex.a,
        b: ex.b,
        c,
        dot,
        normA: a.norm().toFixed(4),
        normB: b.norm().toFixed(4),
      };
    });
    function updateVectorOutput(i) {
      const r = vectorResults[i];
      let text = "a = [" + r.a.join(", ") + "]\nb = [" + r.b.join(", ") + "]\na + b = [" + r.c.join(", ") + "]";
      text += "\n\ndot(a, b) = " + r.dot + ", norm(a) = " + r.normA + ", norm(b) = " + r.normB;
      byId("out-vector").textContent = text;
      drawVectorBarsGeneric("canvas-vector", r.c);
    }
    bindExampleSelector("vector-examples", ["Example 1", "Example 2", "Example 3"], (i) => {
      updateVectorOutput(i);
    });
    updateVectorOutput(0);

    if (typeof gpuAvailable === "function") {
      const gpuVectorStatus = byId("gpu-vector-status");
      const gpuVectorBackend = byId("gpu-vector-backend");
      if (gpuVectorStatus && gpuVectorBackend) {
        gpuVectorStatus.style.display = "block";
        gpuVectorBackend.textContent = gpuAvailable() ? "Backend: GPU" : "Backend: CPU";
      }
    }

    // —— 2. Matrix 2D (column-major) ——
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

    const largeMatrixSec = byId("sec-matrix-large");
    const largeMatrixRun = byId("large-matrix-run");
    const largeMatrixSize = byId("large-matrix-size");
    const outLargeMatrix = byId("out-large-matrix");
    const matmulPlotWrap = byId("matmul-plot-wrap");
    const canvasMatmul = byId("canvas-matmul");
    const gpuInitBtn = byId("gpu-init-btn");
    function updateGpuBackendLabelForMatmul() {
      if (typeof gpuAvailable === "function") {
        const gpuBackendEl = byId("gpu-matrix-backend");
        const gv = byId("gpu-vector-backend");
        if (gpuBackendEl) gpuBackendEl.textContent = gpuAvailable() ? "Backend: GPU" : "Backend: CPU";
        if (gv) gv.textContent = gpuAvailable() ? "Backend: GPU" : "Backend: CPU";
        if (gpuInitBtn) gpuInitBtn.disabled = gpuAvailable();
      }
    }
    if (largeMatrixSec && largeMatrixRun && largeMatrixSize && outLargeMatrix && typeof WasmMatrix !== "undefined") {
      largeMatrixRun.addEventListener("click", async () => {
        const n = parseInt(largeMatrixSize.value, 10);
        const total = n * n;
        const data64 = new Float64Array(total * 2);
        const data32 = new Float32Array(total * 2);
        for (let i = 0; i < total * 2; i++) {
          const v = (i % 100) * 0.01;
          data64[i] = v;
          data32[i] = v;
        }
        let cpuMs = 0;
        let gpuMs = 0;
        const A64 = WasmMatrix.fromArray(n, n, Array.from(data64.subarray(0, total)));
        const B64 = WasmMatrix.fromArray(n, n, Array.from(data64.subarray(total, total * 2)));
        const t0 = performance.now();
        const C64 = A64.mul(B64);
        const t1 = performance.now();
        cpuMs = t1 - t0;
        const sample = C64 && C64.toArray ? C64.toArray()[0].toFixed(4) : "—";
        let text = n + "×" + n + " matmul\nCPU (f64): " + cpuMs.toFixed(2) + " ms";

        const hasGpuBuild = typeof WasmMatrix32 !== "undefined" && typeof gpuAvailable === "function";
        const gpuLastError = typeof lib.gpuLastError === "function" ? lib.gpuLastError : null;
        if (hasGpuBuild && typeof initGpuAsync === "function" && typeof gpuAvailable === "function" && !gpuAvailable()) {
          outLargeMatrix.textContent = text + "\nGPU: initializing…";
          try {
            const ok = await initGpuAsync();
            updateGpuBackendLabelForMatmul();
            if (!ok) {
              const errMsg = gpuLastError ? gpuLastError() : null;
              text += "\nGPU: " + (errMsg && errMsg.trim() ? errMsg.trim() : "unavailable (WebGPU not supported or init failed)");
            }
          } catch (_) {
            const errMsg = gpuLastError ? gpuLastError() : null;
            text += "\nGPU: " + (errMsg && errMsg.trim() ? errMsg.trim() : "init failed");
          }
        }

        if (hasGpuBuild && typeof gpuAvailable === "function" && gpuAvailable()) {
          const A32 = WasmMatrix32.fromArray(n, n, Array.from(data32.subarray(0, total)));
          const B32 = WasmMatrix32.fromArray(n, n, Array.from(data32.subarray(total, total * 2)));
          const t2 = performance.now();
          const C32 = A32.mul(B32);
          const t3 = performance.now();
          gpuMs = t3 - t2;
          text += "\nGPU (f32): " + gpuMs.toFixed(2) + " ms";
        } else if (!hasGpuBuild) {
          text += "\nGPU: build with just wasm-build-gpu, then click Init GPU";
        } else if (text.indexOf("GPU:") === -1) {
          text += "\nGPU: click \"Init GPU\" above first";
        }
        text += "\nSample C[0,0] = " + sample;
        outLargeMatrix.textContent = text;
        if (matmulPlotWrap && canvasMatmul && (cpuMs > 0 || gpuMs > 0)) {
          matmulPlotWrap.style.display = "block";
          const ctx = canvasMatmul.getContext("2d");
          const w = canvasMatmul.width, h = canvasMatmul.height;
          const maxMs = Math.max(cpuMs, gpuMs || 1);
          ctx.fillStyle = "#fff";
          ctx.fillRect(0, 0, w, h);
          const barH = 24;
          const maxBarW = w - 80;
          ctx.fillStyle = "#0d6efd";
          ctx.fillRect(60, 10, (cpuMs / maxMs) * maxBarW, barH);
          ctx.fillStyle = "#212529";
          ctx.font = "12px \"DM Sans\", system-ui, sans-serif";
          ctx.textAlign = "right";
          ctx.fillText("CPU " + cpuMs.toFixed(0) + " ms", 55, 26);
          if (gpuMs > 0) {
            ctx.fillStyle = "#fd7e14";
            ctx.fillRect(60, 44, (gpuMs / maxMs) * maxBarW, barH);
            ctx.fillStyle = "#212529";
            ctx.fillText("GPU " + gpuMs.toFixed(0) + " ms", 55, 60);
          }
        }
      });
    }

    // —— 2a. Matrix × Vector (A × v) ——
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
      return { ...ex, result: result ? result.toArray() : [] };
    });
    bindExampleSelector("matrix-vector-examples", MATRIX_VECTOR_EXAMPLES.map((e) => e.label), (i) => {
      const r = matrixVectorResults[i];
      const resStr = r.result.map((x) => Number(x).toFixed(2)).join(", ");
      byId("out-matrix-vector").innerHTML =
        "A:" + renderMatrixHTML(r.rows, r.cols, r.A) +
        "v: [" + r.v.join(", ") + "]<sup>T</sup><br>y = A×v: [" + resStr + "]<sup>T</sup>";
      drawVectorBarsGeneric("canvas-matrix-vector", r.result);
    });
    (() => {
      const r = matrixVectorResults[0];
      const resStr = r.result.map((x) => Number(x).toFixed(2)).join(", ");
      byId("out-matrix-vector").innerHTML =
        "A:" + renderMatrixHTML(r.rows, r.cols, r.A) +
        "v: [" + r.v.join(", ") + "]<sup>T</sup><br>y = A×v: [" + resStr + "]<sup>T</sup>";
      drawVectorBarsGeneric("canvas-matrix-vector", r.result);
    })();

    // —— 2b. Storage (column vs row major) ——
    const STORAGE_VALUES = [1, 2, 3, 4, 5, 6, 7, 8, 9];
    const STORAGE_ROWS = 3;
    const STORAGE_COLS = 3;
    function colMajorFlat(rows, cols, values) {
      const out = [];
      for (let j = 0; j < cols; j++)
        for (let i = 0; i < rows; i++) out.push(values[i * cols + j]);
      return out;
    }
    function rowMajorFlat(rows, cols, values) {
      return values.slice();
    }
    const colMajorData = colMajorFlat(STORAGE_ROWS, STORAGE_COLS, STORAGE_VALUES);
    const rowMajorData = rowMajorFlat(STORAGE_ROWS, STORAGE_COLS, STORAGE_VALUES);
    function renderStorageFlatArray(containerId, flatArr, storage) {
      const el = byId(containerId);
      if (!el) return;
      const palette = [
        "#e3f2fd", "#bbdefb", "#90caf9",
        "#ffebee", "#ffcdd2", "#ef9a9a",
        "#e8f5e9", "#c8e6c9", "#a5d6a7",
      ];
      let html = storage === "column" ? "Column-major flat: [" : "Row-major flat: [";
      flatArr.forEach((val, idx) => {
        html += '<span style="background:' + palette[idx % palette.length] + ';padding:2px 6px;margin:0 1px;border-radius:4px">' + val + "</span>";
        if (idx < flatArr.length - 1) html += ", ";
      });
      html += "]";
      el.innerHTML = html;
    }
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

    // —— 3. K-means 2D / 3D ——
    const KMEANS_EXAMPLES = [
      { data: [0, 0, 1, 0, 0.5, 0.5, 5, 5, 6, 5, 5.5, 5.5, 10, 0, 11, 0, 10.5, 0.5], rows: 9, cols: 2, k: 3 },
      { data: [0, 1, 2, 10, 11, 12, 0, 0, 0, 10, 10, 10], rows: 6, cols: 2, k: 2 },
      { data: [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1], rows: 8, cols: 3, k: 2 },
    ];
    const kmeansResults = KMEANS_EXAMPLES.map((ex) => {
      const dataK = WasmMatrix.fromArray(ex.rows, ex.cols, ex.data);
      const km = new WasmKmeans(dataK, ex.k, 100);
      const labelsK = km.getLabels();
      const centroidsK = km.getCentroids();
      const cents = [];
      for (let i = 0; i < ex.k; i++) {
        if (ex.cols === 3) cents.push([centroidsK.get(i, 0), centroidsK.get(i, 1), centroidsK.get(i, 2)]);
        else cents.push([centroidsK.get(i, 0), centroidsK.get(i, 1)]);
      }
      const pointsK = [];
      for (let i = 0; i < ex.rows; i++) {
        if (ex.cols === 3) pointsK.push([dataK.get(i, 0), dataK.get(i, 1), dataK.get(i, 2)]);
        else pointsK.push([dataK.get(i, 0), dataK.get(i, 1)]);
      }
      return { labelsK, cents, pointsK, n: ex.rows, k: ex.k, cols: ex.cols };
    });
    function project3dTo2d(points) {
      return points.map((p) => [p[0] + 0.3 * (p[2] || 0), p[1] + 0.2 * (p[2] || 0)]);
    }
    function showKmeans(i) {
      const r = kmeansResults[i];
      let centStr = "";
      for (let j = 0; j < r.k; j++) centStr += (j ? ", " : "") + "(" + r.cents[j].map((x) => Number(x).toFixed(2)).join(", ") + ")";
      byId("out-kmeans").textContent = "Labels: [" + r.labelsK.join(", ") + "]\nCentroids: " + centStr;
      const pts2d = r.cols === 3 ? project3dTo2d(r.pointsK) : r.pointsK;
      const cents2d = r.cols === 3 ? project3dTo2d(r.cents) : r.cents;
      const allK = [...pts2d, ...cents2d];
      const scaledK = scaleToCanvas(allK, 320, 280);
      const ctxK = byId("canvas-kmeans").getContext("2d");
      ctxK.fillStyle = "#fff";
      ctxK.fillRect(0, 0, 320, 280);
      const pal = ["#0d6efd", "#fd7e14", "#198754"];
      for (let i = 0; i < r.n; i++) {
        ctxK.fillStyle = pal[r.labelsK[i] % pal.length] || "#0d6efd";
        ctxK.beginPath();
        ctxK.arc(scaledK[i][0], scaledK[i][1], 5, 0, 6.28);
        ctxK.fill();
      }
      ctxK.strokeStyle = "#000";
      ctxK.lineWidth = 2;
      for (let i = r.n; i < r.n + r.k; i++) {
        ctxK.beginPath();
        ctxK.arc(scaledK[i][0], scaledK[i][1], 8, 0, 6.28);
        ctxK.stroke();
      }
    }
    bindExampleSelector("kmeans-examples", ["3 blobs 2D", "2 clusters", "Cube 3D"], showKmeans);
    showKmeans(0);

    // —— 4. PCA 2D ——
    const PCA_EXAMPLES = [
      (() => { const d = []; for (let i = 0; i < 10; i++) for (let j = 0; j < 4; j++) d.push(i * 0.5 + j); return { data: d, rows: 10, cols: 4 }; })(),
      (() => { const d = []; for (let i = 0; i < 8; i++) for (let j = 0; j < 3; j++) d.push(i + j * 2); return { data: d, rows: 8, cols: 3 }; })(),
    ];
    const pcaResults = PCA_EXAMPLES.map((ex) => {
      const matP = WasmMatrix.fromArray(ex.rows, ex.cols, ex.data);
      const pca = new WasmPca(matP, 2);
      const meanP = pca.getMean().toArray();
      const evP = pca.getExplainedVariance().toArray();
      const proj = pca.transform(matP);
      const pointsP = [];
      for (let i = 0; i < ex.rows; i++) pointsP.push([proj.get(i, 0), proj.get(i, 1)]);
      return { meanP, evP, pointsP, rows: ex.rows };
    });
    function drawPcaCanvas(r) {
      const scaledP = scaleToCanvas(r.pointsP, 320, 280);
      const ctxP = byId("canvas-pca").getContext("2d");
      ctxP.fillStyle = "#fff";
      ctxP.fillRect(0, 0, 320, 280);
      let cx = 0, cy = 0;
      for (const p of scaledP) { cx += p[0]; cy += p[1]; }
      cx /= scaledP.length; cy /= scaledP.length;
      const pad = 20;
      const halfW = 0.4 * (320 - 2 * pad);
      const halfH = 0.4 * (280 - 2 * pad);
      ctxP.strokeStyle = "#dc3545";
      ctxP.lineWidth = 2;
      ctxP.setLineDash([4, 4]);
      ctxP.beginPath();
      ctxP.moveTo(cx - halfW, cy);
      ctxP.lineTo(cx + halfW, cy);
      ctxP.stroke();
      ctxP.strokeStyle = "#0d6efd";
      ctxP.beginPath();
      ctxP.moveTo(cx, cy - halfH);
      ctxP.lineTo(cx, cy + halfH);
      ctxP.stroke();
      ctxP.setLineDash([]);
      ctxP.font = "10px \"DM Sans\", system-ui, sans-serif";
      ctxP.fillStyle = "#dc3545";
      ctxP.fillText("PC1", cx + halfW + 4, cy + 4);
      ctxP.fillStyle = "#0d6efd";
      ctxP.fillText("PC2", cx + 4, cy - halfH - 4);
      ctxP.fillStyle = "#0d6efd";
      for (const p of scaledP) { ctxP.beginPath(); ctxP.arc(p[0], p[1], 5, 0, 6.28); ctxP.fill(); }
    }
    bindExampleSelector("pca-examples", ["Example 1", "Example 2"], (i) => {
      const r = pcaResults[i];
      byId("out-pca").textContent =
        "Mean: [" + r.meanP.map((x) => x.toFixed(3)).join(", ") + "]\nExplained variance (2): [" + r.evP.map((x) => x.toFixed(4)).join(", ") + "]";
      drawPcaCanvas(r);
    });
    byId("out-pca").textContent =
      "Mean (4): [" + pcaResults[0].meanP.map((x) => x.toFixed(3)).join(", ") + "]\nExplained variance (2): [" + pcaResults[0].evP.map((x) => x.toFixed(4)).join(", ") + "]";
    drawPcaCanvas(pcaResults[0]);

    // —— 5. SVM 2D (Linear / RBF) ——
    const SVM_LINEAR_EXAMPLES = [
      { data: [1, 2, 1, 2, 3, 4, 3, 4, 6, 7, 6, 7, 8, 9, 8, 9], labels: [1, 1, 1, 1, -1, -1, -1, -1], n: 8 },
      { data: [0, 0, 1, 0, 0, 1, 1, 1, 3, 3, 4, 3, 3, 4, 4, 4], labels: [1, 1, 1, 1, -1, -1, -1, -1], n: 8 },
    ];
    const SVM_RBF_EXAMPLES = [
      { data: [0, 0, 0.5, 0, 0.35, 0.35, -0.5, 0, -0.35, -0.35, 2, 0, 2, 2, 0, 2, -2, 0, -1, -1.5], labels: [1, 1, 1, 1, 1, -1, -1, -1, -1, -1], n: 10, gamma: 0.5 },
      { data: [0.5, 0, 0.35, 0.35, 0, 0.5, -0.35, 0.35, -0.5, 0, 1.5, 0, 1, 1, 0, 1.5, -1, 0, -1, -1], labels: [1, 1, 1, 1, 1, -1, -1, -1, -1, -1], n: 10, gamma: 0.8 },
    ];
    let svmKernelIndex = 0;
    let svmExampleIndex = 0;
    function getSvmExample() {
      const exs = svmKernelIndex === 0 ? SVM_LINEAR_EXAMPLES : SVM_RBF_EXAMPLES;
      return exs[svmExampleIndex];
    }
    function drawSvmCanvas(r, kernel, predictFn) {
      const canvas = byId("canvas-svm");
      const w = canvas.width, h = canvas.height;
      const ctxS = canvas.getContext("2d");
      const n = r.pointsS.length;
      let minX = r.pointsS[0][0], maxX = minX, minY = r.pointsS[0][1], maxY = minY;
      for (const p of r.pointsS) {
        minX = Math.min(minX, p[0]); maxX = Math.max(maxX, p[0]);
        minY = Math.min(minY, p[1]); maxY = Math.max(maxY, p[1]);
      }
      const pad = 0.3 * Math.max(maxX - minX || 1, maxY - minY || 1) || 1;
      minX -= pad; maxX += pad; minY -= pad; maxY += pad;
      if (kernel === "rbf" && predictFn) {
        const gridRes = 40;
        for (let gi = 0; gi < gridRes; gi++) {
          for (let gj = 0; gj < gridRes; gj++) {
            const x = minX + (gj / (gridRes - 1)) * (maxX - minX);
            const y = maxY - (gi / (gridRes - 1)) * (maxY - minY);
            const pred = predictFn([x, y]);
            ctxS.fillStyle = pred >= 0 ? "rgba(13,110,253,0.2)" : "rgba(253,126,20,0.2)";
            ctxS.fillRect((gj / gridRes) * w, (gi / gridRes) * h, Math.ceil(w / gridRes) + 1, Math.ceil(h / gridRes) + 1);
          }
        }
      }
      const allS = r.pointsS.slice();
      if (kernel === "linear" && r.w && Math.abs(r.w[1]) > 1e-10) {
        allS.push([minX, -(r.w[0] * minX + r.bias) / r.w[1]], [maxX, -(r.w[0] * maxX + r.bias) / r.w[1]]);
      }
      const scaledS = scaleToCanvas(allS, w, h, 15);
      if (kernel === "linear" && r.w && Math.abs(r.w[1]) > 1e-10) {
        ctxS.strokeStyle = "#000";
        ctxS.lineWidth = 2;
        ctxS.beginPath();
        ctxS.moveTo(scaledS[n][0], scaledS[n][1]);
        ctxS.lineTo(scaledS[n + 1][0], scaledS[n + 1][1]);
        ctxS.stroke();
      }
      for (let i = 0; i < n; i++) {
        ctxS.fillStyle = r.labels[i] === 1 ? "#0d6efd" : "#fd7e14";
        ctxS.strokeStyle = "#212529";
        ctxS.lineWidth = 1;
        ctxS.beginPath();
        ctxS.arc(scaledS[i][0], scaledS[i][1], 6, 0, 6.28);
        ctxS.fill();
        ctxS.stroke();
      }
    }
    function updateSvmDemo() {
      const ex = getSvmExample();
      try {
        const dataS = WasmMatrix.fromArray(ex.n, 2, ex.data);
        if (svmKernelIndex === 0) {
          const res = WasmSvm.train(dataS, ex.labels);
          const w = res.getWeights().toArray();
          const bias = res.getBias();
          const preds = res.predictAll(dataS);
          const pointsS = [];
          for (let i = 0; i < ex.n; i++) pointsS.push([dataS.get(i, 0), dataS.get(i, 1)]);
          const r = { w, bias, preds, pointsS, labels: ex.labels };
          byId("out-svm").textContent = "Linear: weights [" + r.w.map((x) => Number(x).toFixed(4)).join(", ") + "], bias " + r.bias.toFixed(4) + "\nPredictions: [" + r.preds.map((x) => Number(x)).join(", ") + "]";
          drawSvmCanvas(r, "linear");
        } else {
          const gamma = ex.gamma || 0.5;
          const res = WasmSvmRbf.train(dataS, ex.labels, gamma);
          const preds = res.predictAll(dataS);
          const pointsS = [];
          for (let i = 0; i < ex.n; i++) pointsS.push([dataS.get(i, 0), dataS.get(i, 1)]);
          const r = { preds, pointsS, labels: ex.labels };
          const predFn = (p) => res.predict(p);
          byId("out-svm").textContent = "RBF γ=" + gamma + ", n_sv=" + res.getSupportVectors().rows + "\nPredictions: [" + r.preds.map((x) => Number(x)).join(", ") + "]";
          drawSvmCanvas(r, "rbf", predFn);
        }
      } catch (err) {
        byId("out-svm").textContent = "Error: " + (err.message || err);
      }
    }
    bindExampleSelector("svm-kernel", ["Linear", "RBF"], (i) => {
      svmKernelIndex = i;
      svmExampleIndex = 0;
      byId("svm-examples").querySelectorAll("button").forEach((b, j) => b.classList.toggle("active", j === 0));
      updateSvmDemo();
    });
    bindExampleSelector("svm-examples", ["Example 1", "Example 2"], (i) => {
      svmExampleIndex = i;
      updateSvmDemo();
    });
    updateSvmDemo();

    // —— 6. Distance ——
    const DISTANCE_EXAMPLES = [
      { a: [1, 0, 0], b: [0.6, 0.8, 0] },
      { a: [1, 1], b: [0, 0] },
      { a: [3, 4], b: [0, 0] },
      { a: [1, 2, -1], b: [2, 0, 1] },
      { a: [0.5, 0.5], b: [1, 0] },
    ];
    const distanceResults = DISTANCE_EXAMPLES.map((ex) => {
      const va = WasmVector.fromArray(ex.a);
      const vb = WasmVector.fromArray(ex.b);
      return {
        a: ex.a, b: ex.b,
        eucl: va.euclideanDistance(vb),
        manh: WasmDistance.manhattan(va, vb),
        cosSim: WasmDistance.cosineSimilarity(va, vb),
        cosDist: WasmDistance.cosineDistance(va, vb),
        cheb: WasmDistance.chebyshev(va, vb),
        mink3: WasmDistance.minkowski(va, vb, 3),
      };
    });
    function drawDistanceCanvas(r) {
      const wrap = byId("distance-plot-wrap");
      const canvas = byId("canvas-distance");
      if (r.a.length !== 2 || r.b.length !== 2) {
        wrap.style.display = "none";
        return;
      }
      wrap.style.display = "block";
      const w = canvas.width, h = canvas.height, cx = w / 2, cy = h / 2;
      const scale = 0.9 * Math.min(w, h) / 2 / Math.max(1e-10, Math.sqrt(Math.max(r.a[0]**2 + r.a[1]**2, r.b[0]**2 + r.b[1]**2)));
      const ax = cx + r.a[0] * scale, ay = cy - r.a[1] * scale;
      const bx = cx + r.b[0] * scale, by = cy - r.b[1] * scale;
      const ctx = canvas.getContext("2d");
      ctx.fillStyle = "#fff";
      ctx.fillRect(0, 0, w, h);
      ctx.strokeStyle = "#0d6efd";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(cx, cy);
      ctx.lineTo(ax, ay);
      ctx.stroke();
      ctx.strokeStyle = "#fd7e14";
      ctx.beginPath();
      ctx.moveTo(cx, cy);
      ctx.lineTo(bx, by);
      ctx.stroke();
      ctx.font = "12px \"DM Sans\", system-ui, sans-serif";
      ctx.fillStyle = "#0d6efd";
      ctx.fillText("a", ax + 5, ay);
      ctx.fillStyle = "#fd7e14";
      ctx.fillText("b", bx + 5, by);
      ctx.fillStyle = "#495057";
      ctx.fillText("euclidean: " + r.eucl.toFixed(3), 10, h - 10);
    }
    function showDistance(i) {
      const r = distanceResults[i];
      byId("out-distance").textContent =
        "a = [" + r.a.join(", ") + "], b = [" + r.b.join(", ") + "]\n" +
        "euclidean: " + r.eucl.toFixed(4) + "\nmanhattan: " + r.manh.toFixed(4) +
        "\ncosineSimilarity: " + r.cosSim.toFixed(4) + "\ncosineDistance: " + r.cosDist.toFixed(4) +
        "\nchebyshev: " + r.cheb.toFixed(4) + "\nminkowski(p=3): " + r.mink3.toFixed(4);
      drawDistanceCanvas(r);
    }
    bindExampleSelector("distance-examples", ["Ex 1", "Ex 2", "Ex 3", "Ex 4", "Ex 5"], showDistance);
    showDistance(0);

    // —— 7. Cholesky solve ——
    const CHOLESKY_EXAMPLES = [
      { A: [4, 2, 2, 3], b: [6, 5] },
      { A: [1, 0, 0, 4], b: [1, 8] },
    ];
    const choleskyResults = CHOLESKY_EXAMPLES.map((ex) => {
      const A = WasmMatrix.fromArray(2, 2, ex.A);
      const b = WasmVector.fromArray(ex.b);
      const chol = new WasmCholesky(A);
      return { A, b: ex.b, L: chol.getL(), x: chol.solve(b).toArray() };
    });
    bindExampleSelector("cholesky-examples", ["Example 1", "Example 2"], (i) => {
      const r = choleskyResults[i];
      byId("out-cholesky").innerHTML =
        "A (2×2 SPD):" + renderMatrixHTML(2, 2, r.A.toArray()) +
        "L (Cholesky):" + renderMatrixHTML(2, 2, r.L.toArray()) +
        "b: [" + r.b.map((x) => x.toFixed(2)).join(", ") + "]\nx (solution): [" + r.x.map((x) => x.toFixed(4)).join(", ") + "]";
    });
    byId("out-cholesky").innerHTML =
      "A (2×2 SPD):" + renderMatrixHTML(2, 2, choleskyResults[0].A.toArray()) +
      "L (Cholesky):" + renderMatrixHTML(2, 2, choleskyResults[0].L.toArray()) +
      "b: [" + choleskyResults[0].b.map((x) => x.toFixed(2)).join(", ") + "]\nx (solution): [" + choleskyResults[0].x.map((x) => x.toFixed(4)).join(", ") + "]";

    // —— 8. SVD ——
    const SVD_EXAMPLES = [
      { data: [1, 0, 0, 1, 1, 1], rows: 3, cols: 2 },
      { data: [1, 2, 3, 4, 5, 6], rows: 2, cols: 3 },
    ];
    const svdResults = SVD_EXAMPLES.map((ex) => {
      const M = WasmMatrix.fromArray(ex.rows, ex.cols, ex.data);
      const svd = M.svdEcon();
      return { sigma: svd.getSigma().toArray(), u: svd.getU(), v: svd.getV() };
    });
    bindExampleSelector("svd-examples", ["Example 1", "Example 2"], (i) => {
      const r = svdResults[i];
      byId("out-svd").textContent =
        "Economical SVD\nSingular values σ: [" + r.sigma.map((x) => x.toFixed(4)).join(", ") + "]\n" +
        "U: " + r.u.rows + "×" + r.u.cols + ", V: " + r.v.rows + "×" + r.v.cols;
    });
    byId("out-svd").textContent =
      "Matrix 3×2, economical SVD\nSingular values σ: [" + svdResults[0].sigma.map((x) => x.toFixed(4)).join(", ") + "]\n" +
      "U: " + svdResults[0].u.rows + "×" + svdResults[0].u.cols + ", V: " + svdResults[0].v.rows + "×" + svdResults[0].v.cols;

    // —— 9. Simplex LP ——
    const SIMPLEX_EXAMPLES = [
      { c: [1, 1], A: [1, 1, 2, 0], b: [4, 2] },
      { c: [2, 1], A: [1, 1, 1, 0], b: [6, 4] },
    ];
    const simplexResults = SIMPLEX_EXAMPLES.map((ex) => {
      const c = WasmVector.fromArray(ex.c);
      const A = WasmMatrix.fromArray(2, 2, ex.A);
      const b = WasmVector.fromArray(ex.b);
      const s = new WasmSimplexResult(c, A, b);
      return { status: s.getStatus(), obj: s.getObjective(), x: s.getX().toArray() };
    });
    bindExampleSelector("simplex-examples", ["Example 1", "Example 2"], (i) => {
      const r = simplexResults[i];
      byId("out-simplex").textContent =
        "min c′x, Ax = b, x ≥ 0\nStatus: " + r.status + "\nObjective: " + r.obj.toFixed(4) + "\nx: [" + r.x.map((x) => x.toFixed(4)).join(", ") + "]";
    });
    byId("out-simplex").textContent =
      "min c′x, Ax = b, x ≥ 0\nStatus: " + simplexResults[0].status + "\nObjective: " + simplexResults[0].obj.toFixed(4) + "\nx: [" + simplexResults[0].x.map((x) => x.toFixed(4)).join(", ") + "]";

    // —— 10. Camera matrices ——
    const CAMERA_EXAMPLES = [
      { aspect: 16 / 9, fovY: Math.PI / 4, near: 0.1, far: 100 },
      { aspect: 1, fovY: Math.PI / 6, near: 0.5, far: 50 },
    ];
    const cameraResults = CAMERA_EXAMPLES.map((ex) => {
      const persp = WasmCg.newPerspective(ex.aspect, ex.fovY, ex.near, ex.far);
      const lookAt = WasmCg.lookAtRh(0, 0, 5, 0, 0, 0, 0, 1, 0);
      return { ...ex, persp, lookAt };
    });
    bindExampleSelector("camera-examples", ["Example 1", "Example 2"], (i) => {
      const r = cameraResults[i];
      byId("out-camera").innerHTML =
        "<strong>Perspective</strong> (aspect=" + r.aspect.toFixed(2) + ", fov=" + (r.fovY === Math.PI / 4 ? "π/4" : "π/6") + ", near=" + r.near + ", far=" + r.far + "):" +
        renderMatrix4x4Float(r.persp.toArray()) +
        "<strong>Look-at RH</strong> (eye 0,0,5 → target 0,0,0, up 0,1,0):" +
        renderMatrix4x4Float(r.lookAt.toArray());
    });
    byId("out-camera").innerHTML =
      "<strong>Perspective</strong> (aspect=1.78, fov=π/4, near=0.1, far=100):" +
      renderMatrix4x4Float(cameraResults[0].persp.toArray()) +
      "<strong>Look-at RH</strong> (eye 0,0,5 → target 0,0,0, up 0,1,0):" +
      renderMatrix4x4Float(cameraResults[0].lookAt.toArray());

    // —— 11. DBSCAN ——
    try {
      if (typeof WasmDbscan !== "function") {
        byId("out-dbscan").textContent = needRebuild;
      } else {
        const noiseVal = typeof NOISE_LABEL !== "undefined" && typeof NOISE_LABEL === "function" ? NOISE_LABEL() : 4294967295;
        const DBSCAN_EXAMPLES = [
          { data: [0, 1, 0, 10, 0, 0, 1, 10], n: 4, eps: 2.0, minPts: 2 },
          { data: [0, 0, 1, 0, 2, 0, 0, 1, 1, 1, 10, 10], n: 6, eps: 1.5, minPts: 2 },
        ];
        const dbscanResults = DBSCAN_EXAMPLES.map((ex) => {
          const dataDb = WasmMatrix.fromArray(ex.n, 2, ex.data);
          const db = new WasmDbscan(dataDb, ex.eps, ex.minPts);
          const labelsDb = db.getLabels();
          const pointsDb = [];
          for (let i = 0; i < ex.n; i++) pointsDb.push([dataDb.get(i, 0), dataDb.get(i, 1)]);
          return { labelsDb, pointsDb, nClusters: db.nClusters(), n: ex.n, eps: ex.eps, minPts: ex.minPts, noiseVal };
        });
        function showDbscan(i) {
          const r = dbscanResults[i];
          byId("out-dbscan").textContent =
            r.n + " points; eps=" + r.eps + ", min_pts=" + r.minPts + "\nlabels: [" + r.labelsDb.join(", ") + "]\nn_clusters: " + r.nClusters;
          const scaledDb = scaleToCanvas(r.pointsDb, 320, 280);
          const ctxDb = byId("canvas-dbscan").getContext("2d");
          ctxDb.fillStyle = "#fff";
          ctxDb.fillRect(0, 0, 320, 280);
          for (let j = 0; j < r.n; j++) {
            const isNoise = r.labelsDb[j] === r.noiseVal || r.labelsDb[j] > 1000;
            ctxDb.fillStyle = isNoise ? "#fff" : r.labelsDb[j] === 0 ? "#0d6efd" : "#fd7e14";
            ctxDb.strokeStyle = "#333";
            ctxDb.lineWidth = 1;
            ctxDb.beginPath();
            ctxDb.arc(scaledDb[j][0], scaledDb[j][1], 8, 0, 6.28);
            if (isNoise) ctxDb.stroke();
            else ctxDb.fill();
          }
        }
        bindExampleSelector("dbscan-examples", ["Example 1", "Example 2"], showDbscan);
        showDbscan(0);
      }
    } catch (err) {
      byId("out-dbscan").textContent = "Error: " + (err.message || err);
    }

    // —— 12. LU solve ——
    try {
      if (typeof WasmLu !== "function") {
        byId("out-lu").textContent = needRebuild;
      } else {
        const LU_EXAMPLES = [
          { A: [1, 1, 1, -1], b: [2, 0] },
          { A: [2, 1, 1, 1], b: [4, 3] },
        ];
        const luResults = LU_EXAMPLES.map((ex) => {
          const A = WasmMatrix.fromArray(2, 2, ex.A);
          const b = WasmVector.fromArray(ex.b);
          const lu = new WasmLu(A);
          return { A, b: ex.b, x: lu.solve(b).toArray() };
        });
        bindExampleSelector("lu-examples", ["Example 1", "Example 2"], (i) => {
          const r = luResults[i];
          byId("out-lu").innerHTML =
            "A (2×2):" + renderMatrixHTML(2, 2, r.A.toArray()) +
            "b: [" + r.b.join(", ") + "]\nx (solution): [" + r.x.map((x) => x.toFixed(4)).join(", ") + "]";
        });
        byId("out-lu").innerHTML =
          "A (2×2):" + renderMatrixHTML(2, 2, luResults[0].A.toArray()) +
          "b: [" + luResults[0].b.join(", ") + "]\nx (solution): [" + luResults[0].x.map((x) => x.toFixed(4)).join(", ") + "]";
      }
    } catch (err) {
      byId("out-lu").textContent = "Error: " + (err.message || err);
    }

    // —— 13. Graph (Dijkstra, A*, D* Lite) ——
    try {
      if (typeof WasmGraph !== "function" || typeof WasmGraph.fromEdges !== "function") {
        byId("out-graph").textContent = needRebuild;
      } else {
        const GRAPH_EXAMPLES = [
          { title: "4-node", n: 4, edges: [0, 1, 1, 0, 2, 4, 1, 2, 2, 1, 3, 6, 2, 3, 1], source: 0, target: 3 },
          { title: "5-node", n: 5, edges: [0, 1, 2, 0, 2, 5, 1, 2, 1, 1, 3, 3, 2, 3, 1, 2, 4, 2, 3, 4, 4], source: 0, target: 4 },
          { title: "4-node (0→2)", n: 4, edges: [0, 1, 1, 0, 2, 4, 1, 2, 2, 1, 3, 6, 2, 3, 1], source: 0, target: 2 },
          { title: "6-node", n: 6, edges: [0, 1, 2, 0, 2, 1, 1, 3, 1, 1, 4, 3, 2, 1, 1, 2, 4, 2, 3, 5, 2, 4, 5, 1], source: 0, target: 5 },
          { title: "Chain 5", n: 5, edges: [0, 1, 1, 1, 2, 1, 2, 3, 1, 3, 4, 1], source: 0, target: 4 },
          { title: "Star", n: 6, edges: [0, 1, 1, 0, 2, 1, 0, 3, 1, 0, 4, 1, 0, 5, 1], source: 0, target: 5 },
          { title: "Grid 6", n: 6, edges: [0, 1, 1, 0, 2, 1, 1, 3, 1, 2, 3, 1, 2, 4, 1, 3, 5, 1, 4, 5, 1], source: 0, target: 5 },
          { title: "8-node", n: 8, edges: [0, 1, 1, 0, 2, 2, 1, 3, 1, 1, 4, 2, 2, 3, 1, 2, 5, 1, 3, 6, 1, 4, 6, 1, 5, 6, 1, 6, 7, 1], source: 0, target: 7 },
        ];
        const GRAPH_ALGOS = [
          { id: "dijkstra", title: "Dijkstra" },
          { id: "astar", title: "A* (zero h)" },
          { id: "astarCoords", title: "A* (Euclidean)" },
          { id: "dstar", title: "D* Lite" },
        ];
        function graphCoordsForLayout(n) {
          const out = [];
          for (let i = 0; i < n; i++) {
            const angle = (2 * Math.PI * i) / n - Math.PI / 2;
            out.push(Math.cos(angle), Math.sin(angle));
          }
          return out;
        }
        function edgesToText(edges) {
          const parts = [];
          for (let i = 0; i < edges.length; i += 3)
            parts.push(edges[i] + "→" + edges[i + 1] + "(" + edges[i + 2] + ")");
          return parts.join(", ");
        }
        function getPathAndDists(ex, algoId) {
          const gDists = WasmGraph.fromEdges(ex.n, ex.edges);
          const dres = gDists.dijkstra(ex.source);
          const dists = dres.getDistances();
          let path = [];
          let distVal = 0;
          if (algoId === "dijkstra") {
            path = dres.pathTo(ex.target);
            distVal = dists[ex.target] ?? Infinity;
          } else {
            const gPath = WasmGraph.fromEdges(ex.n, ex.edges);
            if (algoId === "astar") {
              const ares = gPath.astar(ex.source, ex.target);
              path = ares.getPath();
              distVal = ares.getDist();
            } else if (algoId === "astarCoords") {
              const coordsData = graphCoordsForLayout(ex.n);
              const coords = WasmMatrix.fromArray(ex.n, 2, coordsData);
              const ares = gPath.astarWithCoords(ex.source, ex.target, coords);
              path = ares.getPath();
              distVal = ares.getDist();
            } else if (algoId === "dstar") {
              const dres2 = gPath.dstarLite(ex.source, ex.target);
              path = dres2.getPath();
              distVal = dres2.getDist();
            }
          }
          return { path, dists, distVal };
        }
        let graphExampleIndex = 0;
        let graphAlgoIndex = 0;
        function showGraphExample(algoIndex) {
          graphAlgoIndex = algoIndex ?? graphAlgoIndex;
          const ex = GRAPH_EXAMPLES[graphExampleIndex];
          const algoId = GRAPH_ALGOS[graphAlgoIndex].id;
          const algoTitle = GRAPH_ALGOS[graphAlgoIndex].title;
          const { path, dists, distVal } = getPathAndDists(ex, algoId);
          const canvas = byId("canvas-graph");
          const ctx = canvas.getContext("2d");
          drawGraphOnCanvas(
            ctx,
            canvas.width,
            canvas.height,
            ex.n,
            ex.edges,
            path,
            dists,
            ex.source
          );
          byId("out-graph").textContent =
            "Graph: " +
            ex.n +
            " nodes, edges " +
            edgesToText(ex.edges) +
            "\n" +
            algoTitle +
            " from " +
            ex.source +
            " to " +
            ex.target +
            ":\ndistances (Dijkstra): [" +
            dists.map((x) => (x === 1 / 0 || x === Infinity ? "∞" : x.toFixed(2))).join(", ") +
            "]\npath: [" +
            path.join(", ") +
            "]\ndist: " +
            (Number.isFinite(distVal) ? distVal.toFixed(2) : "∞");
          byId("graph-algorithms").querySelectorAll("button").forEach((b, i) => b.classList.toggle("active", i === graphAlgoIndex));
        }
        function setGraphExample(index) {
          graphExampleIndex = index;
          byId("graph-examples").querySelectorAll("button").forEach((b, i) => b.classList.toggle("active", i === index));
          showGraphExample();
        }
        const graphExamplesDiv = byId("graph-examples");
        GRAPH_EXAMPLES.forEach((_, i) => {
          const btn = document.createElement("button");
          btn.type = "button";
          btn.textContent = GRAPH_EXAMPLES[i].title;
          btn.addEventListener("click", () => setGraphExample(i));
          graphExamplesDiv.appendChild(btn);
        });
        const graphAlgosDiv = byId("graph-algorithms");
        GRAPH_ALGOS.forEach((_, i) => {
          const btn = document.createElement("button");
          btn.type = "button";
          btn.textContent = GRAPH_ALGOS[i].title;
          btn.addEventListener("click", () => showGraphExample(i));
          graphAlgosDiv.appendChild(btn);
        });
        setGraphExample(0);
      }
    } catch (err) {
      byId("out-graph").textContent = "Error: " + (err.message || err);
    }

    // —— 13b. Graph coloring ——
    try {
      if (typeof WasmGraph !== "function" || typeof WasmGraph.prototype.greedyVertexColoring !== "function") {
        byId("out-coloring").textContent = needRebuild;
      } else {
        const COLORING_EXAMPLES = [
          { title: "4-node", n: 4, edges: [0, 1, 1, 0, 2, 1, 1, 2, 1, 1, 3, 1, 2, 3, 1] },
          { title: "Triangle", n: 3, edges: [0, 1, 1, 1, 2, 1, 2, 0, 1] },
          { title: "Path 4", n: 4, edges: [0, 1, 1, 1, 2, 1, 2, 3, 1] },
          { title: "Star 5", n: 5, edges: [0, 1, 1, 0, 2, 1, 0, 3, 1, 0, 4, 1] },
          { title: "Bipartite", n: 4, edges: [0, 2, 1, 0, 3, 1, 1, 2, 1, 1, 3, 1] },
        ];
        const COLORING_ALGOS = [
          { id: "greedy", title: "Greedy" },
          { id: "dsatur", title: "DSatur" },
          { id: "bipartite", title: "Bipartite" },
        ];
        let coloringExampleIndex = 0;
        let coloringAlgoIndex = 0;
        function showColoringExample(algoIndex) {
          coloringAlgoIndex = algoIndex ?? coloringAlgoIndex;
          const ex = COLORING_EXAMPLES[coloringExampleIndex];
          const g = buildUndirectedGraph(ex.n, ex.edges);
          const algoId = COLORING_ALGOS[coloringAlgoIndex].id;
          let colors = [];
          let text = "Graph: " + ex.n + " nodes\n";
          if (algoId === "greedy") {
            colors = g.greedyVertexColoring();
            text += "Greedy: " + (colors.length ? Math.max(...colors) + 1 : 0) + " colors\ncolors: [" + colors.join(", ") + "]";
          } else if (algoId === "dsatur") {
            colors = g.dsaturColoring();
            text += "DSatur: " + (colors.length ? Math.max(...colors) + 1 : 0) + " colors\ncolors: [" + colors.join(", ") + "]";
          } else {
            const bip = g.isBipartite();
            if (bip != null) {
              colors = bip;
              text += "Bipartite: yes (2-coloring)\ncolors: [" + colors.join(", ") + "]";
            } else {
              text += "Bipartite: no (odd cycle)";
            }
          }
          drawGraphColoringOnCanvas(
            byId("canvas-coloring").getContext("2d"),
            byId("canvas-coloring").width,
            byId("canvas-coloring").height,
            ex.n,
            ex.edges,
            colors.length ? colors : null
          );
          byId("out-coloring").textContent = text;
          byId("coloring-algos").querySelectorAll("button").forEach((b, i) => b.classList.toggle("active", i === coloringAlgoIndex));
        }
        function setColoringExample(index) {
          coloringExampleIndex = index;
          byId("coloring-examples").querySelectorAll("button").forEach((b, i) => b.classList.toggle("active", i === index));
          showColoringExample();
        }
        COLORING_EXAMPLES.forEach((_, i) => {
          const btn = document.createElement("button");
          btn.type = "button";
          btn.textContent = COLORING_EXAMPLES[i].title;
          btn.addEventListener("click", () => setColoringExample(i));
          byId("coloring-examples").appendChild(btn);
        });
        COLORING_ALGOS.forEach((_, i) => {
          const btn = document.createElement("button");
          btn.type = "button";
          btn.textContent = COLORING_ALGOS[i].title;
          btn.addEventListener("click", () => showColoringExample(i));
          byId("coloring-algos").appendChild(btn);
        });
        setColoringExample(0);
      }
    } catch (err) {
      byId("out-coloring").textContent = "Error: " + (err.message || err);
    }

    // —— 13c. Graph tree (BFS/DFS) ——
    try {
      if (typeof WasmGraph !== "function" || typeof WasmGraph.prototype.bfs !== "function") {
        byId("out-tree").textContent = needRebuild;
      } else {
        const TREE_EXAMPLES = [
          { title: "4-node", n: 4, edges: [0, 1, 1, 0, 2, 1, 1, 3, 1], source: 0 },
          { title: "Path 5", n: 5, edges: [0, 1, 1, 1, 2, 1, 2, 3, 1, 3, 4, 1], source: 0 },
          { title: "Star 5", n: 5, edges: [0, 1, 1, 0, 2, 1, 0, 3, 1, 0, 4, 1], source: 0 },
          { title: "Grid 6", n: 6, edges: [0, 1, 1, 0, 2, 1, 1, 3, 1, 2, 3, 1, 2, 4, 1, 3, 5, 1, 4, 5, 1], source: 0 },
        ];
        const TREE_ALGOS = [
          { id: "bfs", title: "BFS" },
          { id: "dfsPreorder", title: "DFS preorder" },
          { id: "dfsPostorder", title: "DFS postorder" },
        ];
        let treeExampleIndex = 0;
        let treeAlgoIndex = 0;
        function showTreeExample(algoIndex) {
          treeAlgoIndex = algoIndex ?? treeAlgoIndex;
          const ex = TREE_EXAMPLES[treeExampleIndex];
          const g = buildUndirectedGraph(ex.n, ex.edges);
          const algoId = TREE_ALGOS[treeAlgoIndex].id;
          let order = [];
          let depth = [];
          let text = "Graph: " + ex.n + " nodes, source " + ex.source + "\n";
          if (algoId === "bfs") {
            const res = g.bfs(ex.source);
            order = res.getOrder();
            depth = res.getDepth();
            text += "BFS order: [" + order.join(", ") + "]\ndepth: [" + depth.map((d) => d === 4294967295 ? "∞" : d).join(", ") + "]";
          } else if (algoId === "dfsPreorder") {
            order = g.dfsPreorder(ex.source);
            text += "DFS preorder: [" + order.join(", ") + "]";
          } else {
            order = g.dfsPostorder(ex.source);
            text += "DFS postorder: [" + order.join(", ") + "]";
          }
          drawGraphTreeOnCanvas(
            byId("canvas-tree").getContext("2d"),
            byId("canvas-tree").width,
            byId("canvas-tree").height,
            ex.n,
            ex.edges,
            order,
            depth.length ? depth : null,
            ex.source
          );
          byId("out-tree").textContent = text;
          byId("tree-algos").querySelectorAll("button").forEach((b, i) => b.classList.toggle("active", i === treeAlgoIndex));
        }
        function setTreeExample(index) {
          treeExampleIndex = index;
          byId("tree-examples").querySelectorAll("button").forEach((b, i) => b.classList.toggle("active", i === index));
          showTreeExample();
        }
        TREE_EXAMPLES.forEach((_, i) => {
          const btn = document.createElement("button");
          btn.type = "button";
          btn.textContent = TREE_EXAMPLES[i].title;
          btn.addEventListener("click", () => setTreeExample(i));
          byId("tree-examples").appendChild(btn);
        });
        TREE_ALGOS.forEach((_, i) => {
          const btn = document.createElement("button");
          btn.type = "button";
          btn.textContent = TREE_ALGOS[i].title;
          btn.addEventListener("click", () => showTreeExample(i));
          byId("tree-algos").appendChild(btn);
        });
        setTreeExample(0);
      }
    } catch (err) {
      byId("out-tree").textContent = "Error: " + (err.message || err);
    }

    // —— 14. Line search (backtracking) ——
    try {
      if (typeof lineSearchBacktracking !== "function") {
        byId("out-linesearch").textContent = needRebuild;
      } else {
        const LINESEARCH_EXAMPLES = [
          { x0: [-1], d: [1], f0: 4, gDotD: -4, costFn: (pt) => (pt[0] - 1) ** 2, fAt: (a, x0, d) => (x0[0] + a * d[0] - 1) ** 2, alphaMax: 2 },
          { x0: [0], d: [1], f0: 1, gDotD: -2, costFn: (pt) => pt[0] ** 2, fAt: (a, x0, d) => (x0[0] + a * d[0]) ** 2, alphaMax: 2 },
          { x0: [0, 0], d: [1, 1], f0: 1, gDotD: -2, costFn: (pt) => (1 - pt[0]) ** 2 + 100 * (pt[1] - pt[0] ** 2) ** 2, fAt: (a, x0, d) => { const x = x0[0] + a * d[0], y = x0[1] + a * d[1]; return (1 - x) ** 2 + 100 * (y - x * x) ** 2; }, alphaMax: 2 },
          { x0: [2, 2], d: [-1, -1], f0: 1, gDotD: 2, costFn: (pt) => (pt[0] - 1) ** 2 + (pt[1] - 1) ** 2, fAt: (a, x0, d) => { const x = x0[0] + a * d[0], y = x0[1] + a * d[1]; return (x - 1) ** 2 + (y - 1) ** 2; }, alphaMax: 2 },
        ];
        const linesearchResults = LINESEARCH_EXAMPLES.map((ex) => ({
          ...ex,
          alpha: lineSearchBacktracking(ex.x0, ex.d, ex.f0, ex.gDotD, ex.costFn),
        }));
        function drawLinesearchCanvas(ex) {
          const canvas = byId("canvas-linesearch");
          const w = canvas.width, h = canvas.height;
          const padding = { left: 40, right: 20, top: 20, bottom: 35 };
          const plotW = w - padding.left - padding.right, plotH = h - padding.top - padding.bottom;
          const alphaMin = 0, alphaMax = ex.alphaMax;
          const fAt = (a) => ex.fAt(a, ex.x0, ex.d);
          let fMin = fAt(alphaMin), fMax = fAt(alphaMin);
          for (let i = 0; i <= 100; i++) {
            const a = alphaMin + (i / 100) * (alphaMax - alphaMin);
            const v = fAt(a);
            fMin = Math.min(fMin, v);
            fMax = Math.max(fMax, v);
          }
          const fRange = fMax - fMin || 1;
          const ctx = canvas.getContext("2d");
          ctx.fillStyle = "#fff";
          ctx.fillRect(0, 0, w, h);
          ctx.strokeStyle = "#adb5bd";
          ctx.lineWidth = 1;
          ctx.beginPath();
          for (let i = 0; i <= 100; i++) {
            const a = alphaMin + (i / 100) * (alphaMax - alphaMin);
            const x = padding.left + (a - alphaMin) / (alphaMax - alphaMin) * plotW;
            const y = padding.top + plotH - (fAt(a) - fMin) / fRange * plotH;
            if (i === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);
          }
          ctx.stroke();
          const ax = padding.left + (ex.alpha - alphaMin) / (alphaMax - alphaMin) * plotW;
          const ay = padding.top + plotH - (fAt(ex.alpha) - fMin) / fRange * plotH;
          ctx.strokeStyle = "#0d6efd";
          ctx.lineWidth = 2;
          ctx.setLineDash([4, 4]);
          ctx.beginPath();
          ctx.moveTo(ax, padding.top);
          ctx.lineTo(ax, padding.top + plotH);
          ctx.stroke();
          ctx.setLineDash([]);
          ctx.fillStyle = "#0d6efd";
          ctx.beginPath();
          ctx.arc(ax, ay, 5, 0, 2 * Math.PI);
          ctx.fill();
          ctx.font = "11px \"DM Sans\", system-ui, sans-serif";
          ctx.fillStyle = "#495057";
          ctx.textAlign = "left";
          ctx.textBaseline = "top";
          ctx.fillText("α", w - padding.right - 15, padding.top + plotH + 5);
          ctx.textAlign = "right";
          ctx.fillText("f(x₀+αd)", padding.left - 5, padding.top - 2);
        }
        bindExampleSelector("linesearch-examples", ["Example 1", "Example 2", "Example 3", "Example 4"], (i) => {
          const r = linesearchResults[i];
          byId("out-linesearch").textContent =
            "x₀ = [" + r.x0.join(", ") + "], d = [" + r.d.join(", ") + "]\nf(x₀) = " + r.f0 + ", g·d = " + r.gDotD + "\nα = " + r.alpha.toFixed(6);
          drawLinesearchCanvas(r);
        });
        byId("out-linesearch").textContent =
          "x₀ = [" + linesearchResults[0].x0.join(", ") + "], d = [" + linesearchResults[0].d.join(", ") + "]\nf(x₀) = " + linesearchResults[0].f0 + ", g·d = " + linesearchResults[0].gDotD + "\nα = " + linesearchResults[0].alpha.toFixed(6);
        drawLinesearchCanvas(linesearchResults[0]);
      }
    } catch (err) {
      byId("out-linesearch").textContent = "Error: " + (err.message || err);
    }

    // —— 15. PSO ——
    try {
      if (typeof psoMinimize !== "function") {
        byId("out-pso").textContent = needRebuild;
      } else {
        const PSO_EXAMPLES = [
          { name: "Sphere", costFn: (pos) => pos[0] * pos[0] + pos[1] * pos[1], costAt: (x, y) => x * x + y * y },
          { name: "Shifted", costFn: (pos) => (pos[0] - 1) ** 2 + (pos[1] - 1) ** 2, costAt: (x, y) => (x - 1) ** 2 + (y - 1) ** 2 },
          { name: "Rastrigin", costFn: (pos) => 20 + pos[0] ** 2 + pos[1] ** 2 - 10 * (Math.cos(2 * Math.PI * pos[0]) + Math.cos(2 * Math.PI * pos[1])), costAt: (x, y) => 20 + x * x + y * y - 10 * (Math.cos(2 * Math.PI * x) + Math.cos(2 * Math.PI * y)) },
          { name: "Rosenbrock", costFn: (pos) => (1 - pos[0]) ** 2 + 100 * (pos[1] - pos[0] ** 2) ** 2, costAt: (x, y) => (1 - x) ** 2 + 100 * (y - x * x) ** 2 },
        ];
        const psoResults = PSO_EXAMPLES.map((ex) => {
          const res = psoMinimize([-5, -5], [5, 5], 20, 100, ex.costFn);
          return { ...ex, bestPos: res.getBestPosition(), bestCost: res.getBestCost() };
        });
        function drawPsoCanvas(ex) {
          const canvas = byId("canvas-pso");
          const w = canvas.width, h = canvas.height, gridSize = 50;
          const xMin = -5, xMax = 5, yMin = -5, yMax = 5;
          const costGrid = [];
          let cMin = Infinity, cMax = -Infinity;
          for (let i = 0; i < gridSize; i++) {
            for (let j = 0; j < gridSize; j++) {
              const x = xMin + (j / (gridSize - 1)) * (xMax - xMin);
              const y = yMax - (i / (gridSize - 1)) * (yMax - yMin);
              const c = ex.costAt(x, y);
              costGrid.push(c);
              cMin = Math.min(cMin, c);
              cMax = Math.max(cMax, c);
            }
          }
          const cRange = cMax - cMin || 1;
          const imgData = canvas.getContext("2d").createImageData(w, h);
          const cellW = w / gridSize, cellH = h / gridSize;
          for (let i = 0; i < gridSize; i++) {
            for (let j = 0; j < gridSize; j++) {
              const v = (costGrid[i * gridSize + j] - cMin) / cRange;
              const gray = Math.round(255 * (1 - v * 0.9));
              const px = Math.floor(j * cellW), py = Math.floor(i * cellH);
              const pxe = Math.min(w, Math.ceil((j + 1) * cellW)), pye = Math.min(h, Math.ceil((i + 1) * cellH));
              for (let yy = py; yy < pye; yy++)
                for (let xx = px; xx < pxe; xx++) {
                  const idx = (yy * w + xx) * 4;
                  imgData.data[idx] = imgData.data[idx + 1] = imgData.data[idx + 2] = gray;
                  imgData.data[idx + 3] = 255;
                }
            }
          }
          const ctx = canvas.getContext("2d");
          ctx.putImageData(imgData, 0, 0);
          const bx = ((ex.bestPos[0] - xMin) / (xMax - xMin)) * w;
          const by = ((yMax - ex.bestPos[1]) / (yMax - yMin)) * h;
          ctx.strokeStyle = "#0d6efd";
          ctx.lineWidth = 2;
          const crossR = 8;
          ctx.beginPath();
          ctx.moveTo(bx - crossR, by);
          ctx.lineTo(bx + crossR, by);
          ctx.moveTo(bx, by - crossR);
          ctx.lineTo(bx, by + crossR);
          ctx.stroke();
        }
        bindExampleSelector("pso-examples", ["Sphere", "Shifted", "Rastrigin", "Rosenbrock"], (i) => {
          const r = psoResults[i];
          byId("out-pso").textContent =
            r.name + " on [-5,5]², 20 particles, 100 iters\nbest position: [" + r.bestPos.map((x) => x.toFixed(4)).join(", ") + "]\nbest cost: " + r.bestCost.toFixed(6);
          drawPsoCanvas(r);
        });
        byId("out-pso").textContent =
          "Minimize x²+y² on [-5,5]², 20 particles, 100 iters\nbest position: [" + psoResults[0].bestPos.map((x) => x.toFixed(4)).join(", ") + "]\nbest cost: " + psoResults[0].bestCost.toFixed(6);
        drawPsoCanvas(psoResults[0]);
      }
    } catch (err) {
      byId("out-pso").textContent = "Error: " + (err.message || err);
    }

    // —— 16. Noise ——
    try {
      if (typeof wave2d !== "function" || typeof fbm2dPerlin !== "function") {
        byId("out-noise").textContent = needRebuild;
      } else {
        const NOISE_EXAMPLES = [
          { name: "Wave", sample: () => wave2d(0.5, 0.5), fill: (i, j, gs) => wave2d((j / gs) * 2, (i / gs) * 2), label: "wave2d(0.5, 0.5)" },
          { name: "Perlin", sample: () => perlin2d(1, 2), fill: (i, j, gs) => perlin2d((j / gs) * 2, (i / gs) * 2), label: "perlin2d(1, 2)" },
          { name: "FBM Perlin", sample: () => fbm2dPerlin(1, 1, 4, 2, 0.5), fill: (i, j, gs) => fbm2dPerlin((j / gs) * 2, (i / gs) * 2, 4, 2, 0.5), label: "fbm2dPerlin(1,1,4,2,0.5)" },
        ];
        const gridSize = 32;
        function drawNoiseCanvas(ex) {
          const noiseGrid = [];
          for (let i = 0; i < gridSize; i++)
            for (let j = 0; j < gridSize; j++) noiseGrid.push(ex.fill(i, j, gridSize));
          const nMin = Math.min(...noiseGrid), nMax = Math.max(...noiseGrid), nRange = nMax - nMin || 1;
          const ctxN = byId("canvas-noise").getContext("2d");
          const imgData = ctxN.createImageData(320, 280);
          for (let i = 0; i < gridSize; i++) {
            for (let j = 0; j < gridSize; j++) {
              const v = (noiseGrid[i * gridSize + j] - nMin) / nRange;
              const gray = Math.round(255 * v);
              const px = Math.floor((j / gridSize) * 320), py = Math.floor((i / gridSize) * 280);
              for (let dy = 0; dy < 9; dy++)
                for (let dx = 0; dx < 10; dx++) {
                  const x = px + dx, y = py + dy;
                  if (x < 320 && y < 280) {
                    const idx = (y * 320 + x) * 4;
                    imgData.data[idx] = imgData.data[idx + 1] = imgData.data[idx + 2] = gray;
                    imgData.data[idx + 3] = 255;
                  }
                }
            }
          }
          ctxN.putImageData(imgData, 0, 0);
        }
        bindExampleSelector("noise-examples", ["Wave", "Perlin", "FBM Perlin"], (i) => {
          const ex = NOISE_EXAMPLES[i];
          byId("out-noise").textContent = ex.label + " = " + ex.sample().toFixed(4);
          drawNoiseCanvas(ex);
        });
        byId("out-noise").textContent = NOISE_EXAMPLES[0].label + " = " + NOISE_EXAMPLES[0].sample().toFixed(4);
        drawNoiseCanvas(NOISE_EXAMPLES[0]);
      }
    } catch (err) {
      byId("out-noise").textContent = "Error: " + (err.message || err);
    }
  } catch (e) {
    const msg = (e.message || "").toLowerCase();
    const out = byId("out-vector");
    out.className = "error";
    out.textContent = "Error: " + (e.message || String(e));
    showError(
      (msg.includes("fetch") || msg.includes("import") ? needBuild + "\n\n" : "") +
        (e.message || String(e))
    );
  }
}
