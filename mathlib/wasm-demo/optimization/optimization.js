/**
 * mathlib WASM demo — Optimization (Simplex, Line search, PSO).
 */
import {
  initLib, byId, showError, needBuild, needRebuild,
  bindExampleSelector, renderMatrixHTML,
} from "../shared.js";

try {
  const lib = await initLib();
  const { WasmMatrix, WasmVector, WasmSimplexResult } = lib;
  const lineSearchBacktracking = lib.lineSearchBacktracking;
  const psoMinimize = lib.psoMinimize;

  // —— Simplex ——
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

  // —— Line search ——
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
      if (!canvas) return;
      const w = canvas.width, h = canvas.height;
      const padding = { left: 40, right: 20, top: 20, bottom: 35 };
      const plotW = w - padding.left - padding.right, plotH = h - padding.top - padding.bottom;
      const alphaMin = 0, alphaMax = ex.alphaMax;
      const fAt = (a) => ex.fAt(a, ex.x0, ex.d);
      let fMin = fAt(alphaMin), fMax = fAt(alphaMin);
      for (let i = 0; i <= 100; i++) {
        const a = alphaMin + (i / 100) * (alphaMax - alphaMin);
        fMin = Math.min(fMin, fAt(a));
        fMax = Math.max(fMax, fAt(a));
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

  // —— PSO ——
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
      if (!canvas) return;
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
} catch (e) {
  const out = byId("out-simplex");
  if (out) { out.className = "error"; out.textContent = "Error: " + (e.message || String(e)); }
  showError((e.message || "").toLowerCase().includes("fetch") || (e.message || "").toLowerCase().includes("import") ? needBuild + "\n\n" : "" + (e.message || String(e)));
}
