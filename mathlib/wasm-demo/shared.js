/**
 * mathlib WASM demo — shared utilities, DOM helpers, canvas helpers.
 * Import from domain pages: import { initLib, byId, ... } from '../shared.js';
 */

export const needHttp =
  "This page must be served over HTTP. Do not open the HTML file directly (file://).\n\nFrom the mathlib folder run:  npx serve .\nThen open:  /wasm-demo/";
export const needBuild =
  "Cannot load pkg/mathlib.js — build first (from repo root):  just wasm-build\nThen refresh.";
export const needRebuild =
  "This demo requires a rebuild. From repo root run:  just wasm-build\nThen refresh the page.";

export function byId(id) {
  return document.getElementById(id);
}

/** Create example selector buttons; onSelect(i) is called when user picks example i. */
export function bindExampleSelector(containerId, labels, onSelect) {
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

export function showError(message) {
  const callout = byId("error-callout");
  const pre = byId("error-message");
  if (callout && pre) {
    pre.textContent = message;
    callout.style.display = "block";
  }
}

/** Scale 2D points to canvas coordinates (y flipped). */
export function scaleToCanvas(points, width, height, padding) {
  if (points.length === 0) return [];
  let minX = points[0][0], maxX = minX, minY = points[0][1], maxY = minY;
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
export function renderMatrixHTML(rows, cols, data) {
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

/**
 * Render column-major matrix as HTML table with colored cells.
 * options: { colorBy: 'value' | 'structure', structure: 'lower' | 'upper' | 'full' }
 * - colorBy 'value': heatmap from min (light) to max (dark); zeros get a distinct light tint.
 * - colorBy 'structure': structure 'lower' = lower triangle + diagonal one tint, upper another; 'upper' opposite; 'full' no structure tint.
 * @param {number} rows
 * @param {number} cols
 * @param {number[]} data - column-major
 * @param {{ colorBy?: string, structure?: string }} options
 */
export function renderMatrixHTMLWithColors(rows, cols, data, options = {}) {
  const colorBy = options.colorBy || "value";
  const structure = options.structure || "full";
  const decimals = options.decimals != null ? options.decimals : 2;
  let minVal = data[0];
  let maxVal = data[0];
  for (let k = 0; k < data.length; k++) {
    const v = data[k];
    if (v < minVal) minVal = v;
    if (v > maxVal) maxVal = v;
  }
  const range = maxVal - minVal || 1;
  function valueToHex(v) {
    const t = (v - minVal) / range;
    const r = Math.round(230 - t * 120);
    const g = Math.round(240 - t * 150);
    const b = Math.round(255 - t * 80);
    return "#" + [r, g, b].map((x) => x.toString(16).padStart(2, "0")).join("");
  }
  let html = '<table class="matrix-table"><tbody>';
  for (let i = 0; i < rows; i++) {
    html += "<tr>";
    for (let j = 0; j < cols; j++) {
      const idx = j * rows + i;
      const val = data[idx];
      let bg = "#fff";
      if (colorBy === "structure" && structure !== "full") {
        const isLower = i >= j;
        const isUpper = i <= j;
        if (structure === "lower") bg = isLower ? "#e3f2fd" : "#f5f5f5";
        else if (structure === "upper") bg = isUpper ? "#e8f5e9" : "#f5f5f5";
      } else {
        if (val === 0) bg = "#fafafa";
        else bg = valueToHex(val);
      }
      const textColor = colorBy === "value" && val !== 0 && (val - minVal) / range > 0.6 ? "#fff" : "#212529";
      html += '<td style="background:' + bg + ";color:" + textColor + '">' + (Number(val).toFixed(decimals)) + "</td>";
    }
    html += "</tr>";
  }
  html += "</tbody></table>";
  return html;
}

/** Render 4×4 column-major float array as HTML table (for WasmMatrix32). */
export function renderMatrix4x4Float(data) {
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
export function graphNodePositions(n, width, height, padding) {
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

export const COLOR_PALETTE = [
  "#0d6efd", "#dc3545", "#198754", "#fd7e14", "#6f42c1",
  "#20c997", "#e83e8c", "#ffc107", "#0dcaf0",
];

/** Draw weighted directed graph on canvas. pathNodes = ordered list of node ids on shortest path. */
export function drawGraphOnCanvas(ctx, width, height, n, edges, pathNodes, distances, source) {
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
    const u = edges[i], v = edges[i + 1], w = edges[i + 2];
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
    const r = i === source ? pathRadius : nodeRadius;
    ctx.fillStyle = nodeFill;
    ctx.strokeStyle = nodeStroke;
    ctx.lineWidth = i === source ? 2.5 : 1.5;
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
    const u = edges[i], v = edges[i + 1], w = edges[i + 2];
    const [x1, y1] = positions[u], [x2, y2] = positions[v];
    const mx = (x1 + x2) / 2, my = (y1 + y2) / 2;
    ctx.fillText(Number(w) === w && w % 1 === 0 ? String(w) : w.toFixed(1), mx, my);
  }
}

/** Draw graph with vertex colors. colors = array of color indices. */
export function drawGraphColoringOnCanvas(ctx, width, height, n, edges, colors) {
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

/** Draw graph with BFS/DFS order labels. order = visit order, depth = optional depth array. */
export function drawGraphTreeOnCanvas(ctx, width, height, n, edges, order, depth, source) {
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

/** Draw storage layout grid: 3×3 matrix with cells colored by flat-array index. storage = "column" | "row". */
export function drawStorageGrid(ctx, width, height, rows, cols, storage, values) {
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

/**
 * Draw a sparse matrix and its CSR (Compressed Sparse Row) arrays.
 * triplets: array of [row, col, value]. Draws matrix grid (zeros light gray, non-zeros colored)
 * and below it row_ptr, col_ind, values with labels.
 */
export function drawSparseCSRDiagram(canvasId, width, height, matrixRows, matrixCols, triplets) {
  const canvas = byId(canvasId);
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  ctx.fillStyle = "#fff";
  ctx.fillRect(0, 0, width, height);

  const palette = ["#0d6efd", "#198754", "#fd7e14", "#6f42c1", "#20c997"];
  const cellW = Math.min(36, (width - 20) / matrixCols);
  const cellH = Math.min(28, (height - 80) / matrixRows);
  const matrixW = cellW * matrixCols;
  const matrixH = cellH * matrixRows;
  const startX = 10;
  const startY = 10;

  const matrix = [];
  for (let i = 0; i < matrixRows; i++) {
    matrix[i] = [];
    for (let j = 0; j < matrixCols; j++) matrix[i][j] = 0;
  }
  const tripletColors = [];
  triplets.forEach(([i, j, val], idx) => {
    if (i < matrixRows && j < matrixCols) {
      matrix[i][j] = val;
      tripletColors.push(palette[idx % palette.length]);
    }
  });

  const valToColor = new Map();
  triplets.forEach(([i, j, val], idx) => {
    valToColor.set(i + "," + j, palette[idx % palette.length]);
  });

  for (let i = 0; i < matrixRows; i++) {
    for (let j = 0; j < matrixCols; j++) {
      const x = startX + j * cellW;
      const y = startY + i * cellH;
      const key = i + "," + j;
      ctx.fillStyle = valToColor.has(key) ? valToColor.get(key) : "#eee";
      ctx.strokeStyle = "#adb5bd";
      ctx.lineWidth = 1;
      ctx.fillRect(x, y, cellW, cellH);
      ctx.strokeRect(x, y, cellW, cellH);
      ctx.fillStyle = "#212529";
      ctx.font = "11px \"DM Sans\", system-ui, sans-serif";
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText(String(matrix[i][j]), x + cellW / 2, y + cellH / 2);
    }
  }

  const sortedTriplets = triplets.slice().sort((a, b) => (a[0] !== b[0] ? a[0] - b[0] : a[1] - b[1]));
  const rowPtr = [0];
  for (let i = 0; i < matrixRows; i++) {
    const count = sortedTriplets.filter(([r]) => r === i).length;
    rowPtr.push(rowPtr[rowPtr.length - 1] + count);
  }
  const colInd = sortedTriplets.map(([, j]) => j);
  const values = sortedTriplets.map(([, , v]) => v);

  const arrH = 22;
  const gap = 6;
  let y = startY + matrixH + 14;
  ctx.font = "10px \"DM Sans\", system-ui, sans-serif";
  ctx.fillStyle = "#495057";

  function drawArray(label, arr, colors) {
    ctx.fillStyle = "#212529";
    ctx.fillText(label, startX, y - 2);
    y += 4;
    const boxW = Math.min(28, (width - startX - 20) / arr.length - 2);
    for (let k = 0; k < arr.length; k++) {
      const x = startX + k * (boxW + 2);
      ctx.fillStyle = colors && colors[k] != null ? colors[k] : "#e3f2fd";
      ctx.strokeStyle = "#495057";
      ctx.fillRect(x, y, boxW, arrH);
      ctx.strokeRect(x, y, boxW, arrH);
      ctx.fillStyle = "#212529";
      ctx.textAlign = "center";
      ctx.fillText(String(arr[k]), x + boxW / 2, y + arrH / 2);
    }
    y += arrH + gap;
  }

  drawArray("row_ptr", rowPtr);
  const colIndColors = colInd.map((_, k) => palette[k % palette.length]);
  drawArray("col_ind", colInd, colIndColors);
  drawArray("values", values, colIndColors);
}

/** Draw vector bar chart (reusable). */
export function drawVectorBarsGeneric(canvasId, c) {
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

/** Load WASM lib; call from domain page. Path relative to shared.js (same dir as pkg). */
export async function initLib() {
  const lib = await import("./pkg/mathlib.js");
  await lib.default();
  return lib;
}
