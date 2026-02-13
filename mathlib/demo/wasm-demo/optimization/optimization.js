/**
 * mathlib WASM demo — Optimization (Simplex, Line search, PSO).
 */
import {
  initLib, byId, showError, needBuild, needRebuild,
  bindExampleSelector, renderMatrixHTML, renderMatrixHTMLWithColors, heightToRgb,
} from "../shared.js";

try {
  const lib = await initLib();
  const { WasmMatrix, WasmVector, WasmSimplexResult } = lib;
  const lineSearchBacktracking = lib.lineSearchBacktracking;
  const psoMinimize = lib.psoMinimize;

  // —— Simplex ——
  const SIMPLEX_EXAMPLES = [
    { c: [1, 1], A: [1, 1, 2, 0], rows: 2, cols: 2, b: [4, 2] },
    { c: [2, 1], A: [1, 1, 1, 0], rows: 2, cols: 2, b: [6, 4] },
    { c: [2, 1, 1], A: [1, 2, 1, 1, 1, 0], rows: 2, cols: 3, b: [6, 5] },
  ];
  const simplexResults = SIMPLEX_EXAMPLES.map((ex) => {
    const c = WasmVector.fromArray(ex.c);
    const A = WasmMatrix.fromArray(ex.rows, ex.cols, ex.A);
    const b = WasmVector.fromArray(ex.b);
    const s = new WasmSimplexResult(c, A, b);
    return { ...ex, status: s.getStatus(), obj: s.getObjective(), x: s.getX().toArray() };
  });
  function renderSimplexOut(i) {
    const ex = SIMPLEX_EXAMPLES[i];
    const r = simplexResults[i];
    const cBlock = "<div class=\"simplex-block\"><span class=\"simplex-label\">c′</span>" + renderMatrixHTMLWithColors(1, ex.c.length, ex.c, { decimals: 2 }) + "</div>";
    const aBlock = "<div class=\"simplex-block\"><span class=\"simplex-label\">A</span>" + renderMatrixHTMLWithColors(ex.rows, ex.cols, ex.A, { colorBy: "value", decimals: 2 }) + "</div>";
    const bBlock = "<div class=\"simplex-block\"><span class=\"simplex-label\">b</span>" + renderMatrixHTMLWithColors(ex.b.length, 1, ex.b, { decimals: 2 }) + "</div>";
    const statusLine = "Status: " + r.status + " · Objective: " + r.obj.toFixed(4) + " · x: [" + r.x.map((x) => x.toFixed(4)).join(", ") + "]";
    byId("out-simplex").innerHTML = "<div class=\"simplex-tables\">" + cBlock + aBlock + bBlock + "</div><p class=\"simplex-result\">" + statusLine + "</p>";
  }
  bindExampleSelector("simplex-examples", ["Example 1", "Example 2", "Example 3"], (i) => renderSimplexOut(i));
  renderSimplexOut(0);

  // —— Line search ——
  if (typeof lineSearchBacktracking !== "function") {
    byId("out-linesearch").textContent = needRebuild;
  } else {
    const c1 = 1e-4;
    const beta = 0.5;
    const alphaInit = 1.0;
    const maxBacktrack = 40;
    function backtrackSteps(x0, d, f0, gDotD, costFn) {
      const n = x0.length;
      const steps = [];
      let alpha = alphaInit;
      let backtracks = 0;
      for (;;) {
        const armijoRhs = f0 + c1 * alpha * gDotD;
        const pt = x0.map((xi, i) => xi + alpha * d[i]);
        const fNew = costFn(pt);
        steps.push({ alpha, fNew });
        if (fNew <= armijoRhs || backtracks >= maxBacktrack) return { alpha, steps };
        alpha *= beta;
        backtracks += 1;
      }
    }
    const styb = (pt) => {
      const x = pt[0], y = pt[1];
      return (x ** 4 - 16 * x * x + 5 * x + y ** 4 - 16 * y * y + 5 * y) / 2;
    };
    const stybAt = (a, x0, d) => styb([x0[0] + a * d[0], x0[1] + a * d[1]]);
    const LINESEARCH_EXAMPLES = [
      { x0: [-1], d: [1], f0: 4, gDotD: -4, costFn: (pt) => (pt[0] - 1) ** 2, fAt: (a, x0, d) => (x0[0] + a * d[0] - 1) ** 2, alphaMax: 2 },
      { x0: [0], d: [1], f0: 1, gDotD: -2, costFn: (pt) => pt[0] ** 2, fAt: (a, x0, d) => (x0[0] + a * d[0]) ** 2, alphaMax: 2 },
      { x0: [0, 0], d: [1, 1], f0: 1, gDotD: -2, costFn: (pt) => (1 - pt[0]) ** 2 + 100 * (pt[1] - pt[0] ** 2) ** 2, fAt: (a, x0, d) => { const x = x0[0] + a * d[0], y = x0[1] + a * d[1]; return (1 - x) ** 2 + 100 * (y - x * x) ** 2; }, alphaMax: 2 },
      { x0: [2, 2], d: [-1, -1], f0: 1, gDotD: 2, costFn: (pt) => (pt[0] - 1) ** 2 + (pt[1] - 1) ** 2, fAt: (a, x0, d) => { const x = x0[0] + a * d[0], y = x0[1] + a * d[1]; return (x - 1) ** 2 + (y - 1) ** 2; }, alphaMax: 2 },
      { x0: [0, 0], d: [-1, -1], f0: 0, gDotD: -5, costFn: styb, fAt: stybAt, alphaMax: 5, name: "Styblinski–Tang" },
    ];
    const linesearchResults = LINESEARCH_EXAMPLES.map((ex) => {
      const alpha = lineSearchBacktracking(ex.x0, ex.d, ex.f0, ex.gDotD, ex.costFn);
      const { steps } = backtrackSteps(ex.x0, ex.d, ex.f0, ex.gDotD, ex.costFn);
      return { ...ex, alpha, steps };
    });
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
      const aRange = alphaMax - alphaMin || 1;
      const ctx = canvas.getContext("2d");
      ctx.fillStyle = "#fff";
      ctx.fillRect(0, 0, w, h);
      for (let i = 0; i < 100; i++) {
        const a0 = alphaMin + (i / 100) * aRange;
        const a1 = alphaMin + ((i + 1) / 100) * aRange;
        const t = (i + 0.5) / 100;
        const [r, g, b] = heightToRgb(t);
        ctx.strokeStyle = "rgb(" + r + "," + g + "," + b + ")";
        ctx.lineWidth = 2;
        ctx.beginPath();
        const x0 = padding.left + (a0 - alphaMin) / aRange * plotW;
        const y0 = padding.top + plotH - (fAt(a0) - fMin) / fRange * plotH;
        const x1 = padding.left + (a1 - alphaMin) / aRange * plotW;
        const y1 = padding.top + plotH - (fAt(a1) - fMin) / fRange * plotH;
        ctx.moveTo(x0, y0);
        ctx.lineTo(x1, y1);
        ctx.stroke();
      }
      if (ex.steps && ex.steps.length > 0) {
        ctx.strokeStyle = "rgba(0,0,0,0.35)";
        ctx.lineWidth = 1;
        ctx.setLineDash([2, 2]);
        for (const s of ex.steps) {
          const ax = padding.left + (s.alpha - alphaMin) / aRange * plotW;
          ctx.beginPath();
          ctx.moveTo(ax, padding.top);
          ctx.lineTo(ax, padding.top + plotH);
          ctx.stroke();
        }
        ctx.setLineDash([]);
        ctx.fillStyle = "#333";
        for (const s of ex.steps) {
          const ax = padding.left + (s.alpha - alphaMin) / aRange * plotW;
          const ay = padding.top + plotH - (s.fNew - fMin) / fRange * plotH;
          ctx.beginPath();
          ctx.arc(ax, ay, 3, 0, 2 * Math.PI);
          ctx.fill();
        }
      }
      const ax = padding.left + (ex.alpha - alphaMin) / aRange * plotW;
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
      ctx.strokeStyle = "#fff";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(ax, ay, 6, 0, 2 * Math.PI);
      ctx.fill();
      ctx.stroke();
      ctx.font = "11px \"DM Sans\", system-ui, sans-serif";
      ctx.fillStyle = "#495057";
      ctx.textAlign = "left";
      ctx.textBaseline = "top";
      ctx.fillText("α", w - padding.right - 15, padding.top + plotH + 5);
      ctx.textAlign = "right";
      ctx.fillText("f(x₀+αd)", padding.left - 5, padding.top - 2);
    }
    function drawLinesearchHeatmap(ex) {
      const wrap = byId("linesearch-heatmap-wrap");
      const canvas = byId("canvas-linesearch-heatmap");
      if (!wrap || !canvas || ex.x0.length !== 2) {
        if (wrap) wrap.style.display = "none";
        return;
      }
      wrap.style.display = "block";
      const w = canvas.width, h = canvas.height, gridSize = 48;
      const padding = 25;
      const plotW = w - 2 * padding, plotH = h - 2 * padding;
      const xEnd = ex.x0[0] + ex.alphaMax * ex.d[0];
      const yEnd = ex.x0[1] + ex.alphaMax * ex.d[1];
      const margin = 0.8;
      const xMin = Math.min(ex.x0[0], xEnd) - margin;
      const xMax = Math.max(ex.x0[0], xEnd) + margin;
      const yMin = Math.min(ex.x0[1], yEnd) - margin;
      const yMax = Math.max(ex.x0[1], yEnd) + margin;
      const xRange = xMax - xMin || 1;
      const yRange = yMax - yMin || 1;
      let cMin = Infinity, cMax = -Infinity;
      for (let i = 0; i < gridSize; i++) {
        for (let j = 0; j < gridSize; j++) {
          const x = xMin + (j / (gridSize - 1)) * xRange;
          const y = yMax - (i / (gridSize - 1)) * yRange;
          const c = ex.costFn([x, y]);
          cMin = Math.min(cMin, c);
          cMax = Math.max(cMax, c);
        }
      }
      const cRange = cMax - cMin || 1;
      const imgData = canvas.getContext("2d").createImageData(w, h);
      const cellW = w / gridSize, cellH = h / gridSize;
      for (let i = 0; i < gridSize; i++) {
        for (let j = 0; j < gridSize; j++) {
          const x = xMin + (j / (gridSize - 1)) * xRange;
          const y = yMax - (i / (gridSize - 1)) * yRange;
          const v = (ex.costFn([x, y]) - cMin) / cRange;
          const [r, g, b] = heightToRgb(v);
          const px = Math.floor(j * cellW), py = Math.floor(i * cellH);
          const pxe = Math.min(w, Math.ceil((j + 1) * cellW)), pye = Math.min(h, Math.ceil((i + 1) * cellH));
          for (let yy = py; yy < pye; yy++)
            for (let xx = px; xx < pxe; xx++) {
              const idx = (yy * w + xx) * 4;
              imgData.data[idx] = r;
              imgData.data[idx + 1] = g;
              imgData.data[idx + 2] = b;
              imgData.data[idx + 3] = 255;
            }
        }
      }
      const ctx = canvas.getContext("2d");
      ctx.putImageData(imgData, 0, 0);
      const toPx = (x, y) => [
        padding + ((x - xMin) / xRange) * plotW,
        padding + ((yMax - y) / yRange) * plotH,
      ];
      ctx.strokeStyle = "rgba(255,255,255,0.9)";
      ctx.lineWidth = 2;
      ctx.beginPath();
      for (let k = 0; k <= 30; k++) {
        const a = (k / 30) * ex.alphaMax;
        const x = ex.x0[0] + a * ex.d[0];
        const y = ex.x0[1] + a * ex.d[1];
        const [px, py] = toPx(x, y);
        if (k === 0) ctx.moveTo(px, py);
        else ctx.lineTo(px, py);
      }
      ctx.stroke();
      const [acceptedPx, acceptedPy] = toPx(ex.x0[0] + ex.alpha * ex.d[0], ex.x0[1] + ex.alpha * ex.d[1]);
      ctx.fillStyle = "#0d6efd";
      ctx.strokeStyle = "#fff";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(acceptedPx, acceptedPy, 6, 0, 2 * Math.PI);
      ctx.fill();
      ctx.stroke();
    }
    const labels = ["Example 1", "Example 2", "Example 3", "Example 4", "Styblinski–Tang"];
    bindExampleSelector("linesearch-examples", labels, (i) => {
      const r = linesearchResults[i];
      byId("out-linesearch").textContent =
        "x₀ = [" + r.x0.join(", ") + "], d = [" + r.d.join(", ") + "]\nf(x₀) = " + r.f0 + ", g·d = " + r.gDotD + "\nα = " + r.alpha.toFixed(6) + (r.steps.length > 1 ? " (" + r.steps.length + " trials)" : "");
      drawLinesearchCanvas(r);
      drawLinesearchHeatmap(r);
    });
    byId("out-linesearch").textContent =
      "x₀ = [" + linesearchResults[0].x0.join(", ") + "], d = [" + linesearchResults[0].d.join(", ") + "]\nf(x₀) = " + linesearchResults[0].f0 + ", g·d = " + linesearchResults[0].gDotD + "\nα = " + linesearchResults[0].alpha.toFixed(6) + (linesearchResults[0].steps.length > 1 ? " (" + linesearchResults[0].steps.length + " trials)" : "");
    drawLinesearchCanvas(linesearchResults[0]);
    drawLinesearchHeatmap(linesearchResults[0]);
  }

  // —— PSO ——
  const psoMinimizeWithHistory = lib.psoMinimizeWithHistory;
  if (typeof psoMinimize !== "function") {
    byId("out-pso").textContent = needRebuild;
  } else {
    const ackley = (pos) => {
      const x = pos[0], y = pos[1];
      const t = 0.5 * (x * x + y * y);
      return -20 * Math.exp(-0.2 * Math.sqrt(t)) - Math.exp(0.5 * (Math.cos(2 * Math.PI * x) + Math.cos(2 * Math.PI * y))) + Math.E + 20;
    };
    const ackleyAt = (x, y) => ackley([x, y]);
    const PSO_EXAMPLES = [
      { name: "Sphere", costFn: (pos) => pos[0] * pos[0] + pos[1] * pos[1], costAt: (x, y) => x * x + y * y },
      { name: "Shifted", costFn: (pos) => (pos[0] - 1) ** 2 + (pos[1] - 1) ** 2, costAt: (x, y) => (x - 1) ** 2 + (y - 1) ** 2 },
      { name: "Rastrigin", costFn: (pos) => 20 + pos[0] ** 2 + pos[1] ** 2 - 10 * (Math.cos(2 * Math.PI * pos[0]) + Math.cos(2 * Math.PI * pos[1])), costAt: (x, y) => 20 + x * x + y * y - 10 * (Math.cos(2 * Math.PI * x) + Math.cos(2 * Math.PI * y)) },
      { name: "Rosenbrock", costFn: (pos) => (1 - pos[0]) ** 2 + 100 * (pos[1] - pos[0] ** 2) ** 2, costAt: (x, y) => (1 - x) ** 2 + 100 * (y - x * x) ** 2 },
      { name: "Ackley", costFn: ackley, costAt: ackleyAt },
    ];
    const numParticles = 20;
    const maxIters = 100;
    const lower = [-5, -5];
    const upper = [5, 5];
    function runPsoForExample(ex, seed) {
      if (typeof psoMinimizeWithHistory === "function") {
        const res = psoMinimizeWithHistory(lower, upper, numParticles, maxIters, ex.costFn, seed);
        const dim = 2;
        const histPos = res.getHistoryPositions();
        const trajectory = [];
        for (let i = 0; i < res.getIterations(); i++) {
          trajectory.push([histPos[i * dim], histPos[i * dim + 1]]);
        }
        return { ...ex, bestPos: res.getBestPosition(), bestCost: res.getBestCost(), iterations: res.getIterations(), trajectory };
      }
      const res = psoMinimize(lower, upper, numParticles, maxIters, ex.costFn, seed);
      return { ...ex, bestPos: res.getBestPosition(), bestCost: res.getBestCost(), iterations: res.getIterations(), trajectory: null };
    }
    const psoResults = PSO_EXAMPLES.map((ex) => runPsoForExample(ex, undefined));
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
          const [r, g, b] = heightToRgb(v);
          const px = Math.floor(j * cellW), py = Math.floor(i * cellH);
          const pxe = Math.min(w, Math.ceil((j + 1) * cellW)), pye = Math.min(h, Math.ceil((i + 1) * cellH));
          for (let yy = py; yy < pye; yy++)
            for (let xx = px; xx < pxe; xx++) {
              const idx = (yy * w + xx) * 4;
              imgData.data[idx] = r;
              imgData.data[idx + 1] = g;
              imgData.data[idx + 2] = b;
              imgData.data[idx + 3] = 255;
            }
        }
      }
      const ctx = canvas.getContext("2d");
      ctx.putImageData(imgData, 0, 0);
      const toPx = (xx, yy) => [
        ((xx - xMin) / (xMax - xMin)) * w,
        ((yMax - yy) / (yMax - yMin)) * h,
      ];
      if (ex.trajectory && ex.trajectory.length > 1) {
        ctx.strokeStyle = "rgba(255,255,255,0.85)";
        ctx.lineWidth = 2;
        ctx.beginPath();
        const [fx, fy] = toPx(ex.trajectory[0][0], ex.trajectory[0][1]);
        ctx.moveTo(fx, fy);
        for (let i = 1; i < ex.trajectory.length; i++) {
          const [px, py] = toPx(ex.trajectory[i][0], ex.trajectory[i][1]);
          ctx.lineTo(px, py);
        }
        ctx.stroke();
      }
      const [bx, by] = toPx(ex.bestPos[0], ex.bestPos[1]);
      const crossR = 8;
      ctx.strokeStyle = "#fff";
      ctx.lineWidth = 3;
      ctx.beginPath();
      ctx.moveTo(bx - crossR, by);
      ctx.lineTo(bx + crossR, by);
      ctx.moveTo(bx, by - crossR);
      ctx.lineTo(bx, by + crossR);
      ctx.stroke();
      ctx.strokeStyle = "#0d6efd";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(bx - crossR, by);
      ctx.lineTo(bx + crossR, by);
      ctx.moveTo(bx, by - crossR);
      ctx.lineTo(bx, by + crossR);
      ctx.stroke();
    }
    let currentPsoIndex = 0;
    function updatePsoOutput(r) {
      byId("out-pso").textContent =
        r.name + " on [-5,5]², " + numParticles + " particles, " + r.iterations + " iters\nbest: [" + r.bestPos.map((x) => x.toFixed(4)).join(", ") + "]  cost: " + r.bestCost.toFixed(6);
      drawPsoCanvas(r);
    }
    const psoLabels = PSO_EXAMPLES.map((e) => e.name);
    bindExampleSelector("pso-examples", psoLabels, (i) => {
      currentPsoIndex = i;
      updatePsoOutput(psoResults[i]);
    });
    const psoRunBtn = byId("pso-run");
    if (psoRunBtn) {
      psoRunBtn.addEventListener("click", () => {
        const ex = PSO_EXAMPLES[currentPsoIndex];
        const seed = Math.floor(Math.random() * 0xFFFFFFFF);
        psoResults[currentPsoIndex] = runPsoForExample(ex, seed);
        updatePsoOutput(psoResults[currentPsoIndex]);
      });
    }
    updatePsoOutput(psoResults[0]);
  }
} catch (e) {
  const out = byId("out-simplex");
  if (out) { out.className = "error"; out.textContent = "Error: " + (e.message || String(e)); }
  showError((e.message || "").toLowerCase().includes("fetch") || (e.message || "").toLowerCase().includes("import") ? needBuild + "\n\n" : "" + (e.message || String(e)));
}
