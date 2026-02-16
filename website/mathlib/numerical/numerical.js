/**
 * mathlib WASM demo — Numerical: roots, quadrature, differentiation, ODE.
 */
import {
  initLib,
  byId,
  showError,
  needRebuild,
  getCanvasThemeColors,
  bindExampleSelector,
} from "../shared.js";

try {
  const lib = await initLib();
  const bisectionRoot = lib.bisectionRoot;
  const newtonRoot = lib.newtonRoot;
  const secantRoot = lib.secantRoot;
  const brentRoot = lib.brentRoot;
  const trapezoidalQuad = lib.trapezoidalQuad;
  const simpsonQuad = lib.simpsonQuad;
  const gaussLegendreQuad = lib.gaussLegendreQuad;
  const diffCentral = lib.diffCentral;
  const eulerOde = lib.eulerOde;
  const rk4Ode = lib.rk4Ode;

  const hasRoots =
    typeof bisectionRoot === "function" && typeof brentRoot === "function";
  const hasQuad =
    typeof trapezoidalQuad === "function" && typeof simpsonQuad === "function";
  const hasDiff = typeof diffCentral === "function";
  const hasOde = typeof eulerOde === "function" && typeof rk4Ode === "function";

  // ——— Roots ———
  const ROOTS_EXAMPLES = [
    {
      name: "x² − 2 (√2)",
      f: (x) => x * x - 2,
      df: (x) => 2 * x,
      a: 1,
      b: 2,
      exact: Math.SQRT2,
    },
    {
      name: "cos(x) − x",
      f: (x) => Math.cos(x) - x,
      df: (x) => -Math.sin(x) - 1,
      a: 0,
      b: 1,
      exact: 0.7390851332151607,
    },
    {
      name: "x³ − 2x − 5",
      f: (x) => x * x * x - 2 * x - 5,
      df: (x) => 3 * x * x - 2,
      a: 2,
      b: 3,
      exact: 2.0945514815423265,
    },
  ];

  function runRoots(ex, method) {
    if (!hasRoots) return null;
    const f = (x) => ex.f(x);
    const df = (x) => ex.df(x);
    const tol = 1e-10;
    const maxIter = 100;
    if (method === "bisection") {
      return bisectionRoot(f, ex.a, ex.b, tol);
    }
    if (method === "newton") {
      return newtonRoot(f, df, (ex.a + ex.b) / 2, tol, maxIter);
    }
    if (method === "secant") {
      return secantRoot(f, ex.a, ex.b, tol, maxIter);
    }
    return brentRoot(f, ex.a, ex.b, tol, maxIter);
  }

  function drawRootsCanvas(ex, result) {
    const canvas = byId("canvas-roots");
    if (!canvas) return;
    const w = canvas.width;
    const h = canvas.height;
    const theme = getCanvasThemeColors();
    const ctx = canvas.getContext("2d");
    ctx.fillStyle = theme.bg;
    ctx.fillRect(0, 0, w, h);
    const pad = { left: 45, right: 20, top: 20, bottom: 35 };
    const plotW = w - pad.left - pad.right;
    const plotH = h - pad.top - pad.bottom;
    const a = ex.a;
    const b = ex.b;
    const n = 200;
    let minF = Infinity;
    let maxF = -Infinity;
    const pts = [];
    for (let i = 0; i <= n; i++) {
      const x = a + (i / n) * (b - a);
      const y = ex.f(x);
      pts.push([x, y]);
      minF = Math.min(minF, y);
      maxF = Math.max(maxF, y);
    }
    const rangeY = maxF - minF || 1;
    const marginY = rangeY * 0.1;
    minF -= marginY;
    maxF += marginY;
    const rangeY2 = maxF - minF;
    const toX = (x) => pad.left + ((x - a) / (b - a)) * plotW;
    const toY = (y) => pad.top + plotH - ((y - minF) / rangeY2) * plotH;
    ctx.strokeStyle = theme.stroke;
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.moveTo(toX(pts[0][0]), toY(pts[0][1]));
    for (let i = 1; i < pts.length; i++) {
      ctx.lineTo(toX(pts[i][0]), toY(pts[i][1]));
    }
    ctx.stroke();
    ctx.strokeStyle = "rgba(0,0,0,0.3)";
    ctx.setLineDash([2, 2]);
    const zeroY = toY(0);
    ctx.beginPath();
    ctx.moveTo(pad.left, zeroY);
    ctx.lineTo(w - pad.right, zeroY);
    ctx.stroke();
    ctx.setLineDash([]);
    if (result && result.getConverged()) {
      const rx = result.getX();
      ctx.fillStyle = "#0d6efd";
      ctx.beginPath();
      ctx.arc(toX(rx), zeroY, 6, 0, Math.PI * 2);
      ctx.fill();
      ctx.strokeStyle = theme.stroke;
      ctx.stroke();
    }
  }

  function updateRoots(i, method) {
    const ex = ROOTS_EXAMPLES[i];
    const result = runRoots(ex, method);
    drawRootsCanvas(ex, result);
    let text = "";
    if (!hasRoots) text = needRebuild;
    else if (result) {
      text = `x = ${result.getX().toFixed(10)}  f(x) = ${result.getFx().toExponential(4)}  converged = ${result.getConverged()}  iters = ${result.getIterations()}`;
      if (ex.exact != null) {
        text += `  |x - exact| = ${Math.abs(result.getX() - ex.exact).toExponential(4)}`;
      }
    }
    byId("out-roots").textContent = text;
  }

  if (hasRoots) {
    bindExampleSelector(
      "roots-examples",
      ROOTS_EXAMPLES.map((e) => e.name),
      (i) => updateRoots(i, byId("roots-method").value)
    );
    byId("roots-method").addEventListener("change", () => {
      const idx = byId("roots-examples").querySelector("button.active");
      const i = idx ? Array.from(byId("roots-examples").children).indexOf(idx) : 0;
      updateRoots(i, byId("roots-method").value);
    });
    updateRoots(0, "bisection");
  } else {
    byId("out-roots").textContent = needRebuild;
  }

  // ——— Quadrature ———
  const QUAD_EXAMPLES = [
    { name: "x² on [0,1]", f: (x) => x * x, a: 0, b: 1, exact: 1 / 3 },
    { name: "exp(x) on [0,1]", f: (x) => Math.exp(x), a: 0, b: 1, exact: Math.E - 1 },
    { name: "sin(x) on [0,π]", f: (x) => Math.sin(x), a: 0, b: Math.PI, exact: 2 },
  ];

  function runQuad(ex, rule, n) {
    if (!hasQuad) return null;
    const f = (x) => ex.f(x);
    if (rule === "trapezoidal") return trapezoidalQuad(f, ex.a, ex.b, n);
    if (rule === "simpson") return simpsonQuad(f, ex.a, ex.b, n % 2 === 0 ? n : n + 1);
    return gaussLegendreQuad(f, ex.a, ex.b, Math.min(n, 10));
  }

  function drawQuadCanvas(ex, rule, n, integral) {
    const canvas = byId("canvas-quad");
    if (!canvas) return;
    const w = canvas.width;
    const h = canvas.height;
    const theme = getCanvasThemeColors();
    const ctx = canvas.getContext("2d");
    ctx.fillStyle = theme.bg;
    ctx.fillRect(0, 0, w, h);
    const pad = { left: 45, right: 20, top: 20, bottom: 35 };
    const plotW = w - pad.left - pad.right;
    const plotH = h - pad.top - pad.bottom;
    const a = ex.a;
    const b = ex.b;
    const nSample = 150;
    let maxF = -Infinity;
    const pts = [];
    for (let i = 0; i <= nSample; i++) {
      const x = a + (i / nSample) * (b - a);
      const y = ex.f(x);
      pts.push([x, y]);
      maxF = Math.max(maxF, y);
    }
    const toX = (x) => pad.left + ((x - a) / (b - a)) * plotW;
    const toY = (y) => pad.top + plotH - (y / (maxF || 1)) * plotH;
    ctx.fillStyle = "rgba(13, 110, 253, 0.2)";
    ctx.beginPath();
    ctx.moveTo(toX(a), pad.top + plotH);
    for (const [x, y] of pts) ctx.lineTo(toX(x), toY(y));
    ctx.lineTo(toX(b), pad.top + plotH);
    ctx.closePath();
    ctx.fill();
    ctx.strokeStyle = theme.stroke;
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.moveTo(toX(pts[0][0]), toY(pts[0][1]));
    for (let i = 1; i < pts.length; i++) ctx.lineTo(toX(pts[i][0]), toY(pts[i][1]));
    ctx.stroke();
  }

  function updateQuad(i, rule, n) {
    const ex = QUAD_EXAMPLES[i];
    const integral = runQuad(ex, rule, n);
    drawQuadCanvas(ex, rule, n, integral);
    let text = "";
    if (!hasQuad) text = needRebuild;
    else {
      text = `∫f(x)dx ≈ ${integral.toFixed(10)}`;
      if (ex.exact != null) {
        text += `  exact = ${ex.exact.toFixed(10)}  error = ${Math.abs(integral - ex.exact).toExponential(4)}`;
      }
    }
    byId("out-quad").textContent = text;
  }

  if (hasQuad) {
    bindExampleSelector("quad-examples", QUAD_EXAMPLES.map((e) => e.name), (i) =>
      updateQuad(i, byId("quad-rule").value, parseInt(byId("quad-n").value, 10) || 20)
    );
    byId("quad-rule").addEventListener("change", () => {
      const idx = byId("quad-examples").querySelector("button.active");
      updateQuad(idx ? Array.from(byId("quad-examples").children).indexOf(idx) : 0, byId("quad-rule").value, parseInt(byId("quad-n").value, 10) || 20);
    });
    byId("quad-n").addEventListener("change", () => {
      const idx = byId("quad-examples").querySelector("button.active");
      updateQuad(idx ? Array.from(byId("quad-examples").children).indexOf(idx) : 0, byId("quad-rule").value, parseInt(byId("quad-n").value, 10) || 20);
    });
    updateQuad(0, "trapezoidal", 20);
  } else {
    byId("out-quad").textContent = needRebuild;
  }

  // ——— Differentiation ———
  const DIFF_EXAMPLES = [
    { name: "x²", f: (x) => x * x, df: (x) => 2 * x },
    { name: "exp(x)", f: (x) => Math.exp(x), df: (x) => Math.exp(x) },
    { name: "sin(x)", f: (x) => Math.sin(x), df: (x) => Math.cos(x) },
  ];

  function updateDiff(i) {
    const ex = DIFF_EXAMPLES[i];
    const a = parseFloat(byId("diff-a").value) || 0;
    const b = parseFloat(byId("diff-b").value) || 2;
    const h = parseFloat(byId("diff-h").value) || 1e-5;
    const canvas = byId("canvas-diff");
    if (!canvas) return;
    const w = canvas.width;
    const h_c = canvas.height;
    const theme = getCanvasThemeColors();
    const ctx = canvas.getContext("2d");
    ctx.fillStyle = theme.bg;
    ctx.fillRect(0, 0, w, h_c);
    const pad = { left: 45, right: 20, top: 20, bottom: 35 };
    const plotW = w - pad.left - pad.right;
    const plotH = h_c - pad.top - pad.bottom;
    const n = 100;
    let minF = Infinity;
    let maxF = -Infinity;
    const ptsF = [];
    const ptsDf = [];
    for (let i = 0; i <= n; i++) {
      const x = a + (i / n) * (b - a);
      ptsF.push([x, ex.f(x)]);
      const dfVal = hasDiff ? diffCentral((x) => ex.f(x), x, h) : ex.df(x);
      ptsDf.push([x, dfVal]);
      minF = Math.min(minF, ex.f(x), dfVal);
      maxF = Math.max(maxF, ex.f(x), dfVal);
    }
    const rangeY = maxF - minF || 1;
    const toX = (x) => pad.left + ((x - a) / (b - a)) * plotW;
    const toY = (y) => pad.top + plotH - ((y - minF) / rangeY) * plotH;
    ctx.strokeStyle = "#0d6efd";
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    ctx.moveTo(toX(ptsF[0][0]), toY(ptsF[0][1]));
    for (let i = 1; i < ptsF.length; i++) ctx.lineTo(toX(ptsF[i][0]), toY(ptsF[i][1]));
    ctx.stroke();
    ctx.strokeStyle = "#fd7e14";
    ctx.setLineDash([4, 2]);
    ctx.beginPath();
    ctx.moveTo(toX(ptsDf[0][0]), toY(ptsDf[0][1]));
    for (let i = 1; i < ptsDf.length; i++) ctx.lineTo(toX(ptsDf[i][0]), toY(ptsDf[i][1]));
    ctx.stroke();
    ctx.setLineDash([]);
    ctx.strokeStyle = "rgba(0,0,0,0.2)";
    ctx.setLineDash([2, 2]);
    ctx.beginPath();
    ctx.moveTo(toX(a), toY(0));
    ctx.lineTo(toX(b), toY(0));
    ctx.stroke();
    ctx.setLineDash([]);
    let text = "";
    if (!hasDiff) text = needRebuild;
    else {
      const xMid = (a + b) / 2;
      const dfNum = diffCentral(new Function("x", `return (${ex.f.toString()})(x);`), xMid, h);
      const dfExact = ex.df(xMid);
      const dfNum2 = diffCentral((x) => ex.f(x), xMid, h);
      text = `f'(${xMid.toFixed(4)}) ≈ ${dfNum2.toFixed(8)}  exact = ${dfExact.toFixed(8)}  error = ${Math.abs(dfNum2 - dfExact).toExponential(4)}`;
    }
    byId("out-diff").textContent = text;
  }

  if (hasDiff) {
    bindExampleSelector("diff-examples", DIFF_EXAMPLES.map((e) => e.name), updateDiff);
    byId("diff-h").addEventListener("change", () => {
      const idx = byId("diff-examples").querySelector("button.active");
      updateDiff(idx ? Array.from(byId("diff-examples").children).indexOf(idx) : 0);
    });
    updateDiff(0);
  } else {
    byId("out-diff").textContent = needRebuild;
  }

  // ——— ODE ———
  const ODE_EXAMPLES = [
    {
      name: "dy/dt = y (exp)",
      dydt: (t, y) => [y[0]],
      y0: [1],
      exact: (t) => Math.exp(t),
    },
    {
      name: "dy/dt = -y (decay)",
      dydt: (t, y) => [-y[0]],
      y0: [1],
      exact: (t) => Math.exp(-t),
    },
  ];

  function updateOde(i) {
    const ex = ODE_EXAMPLES[i];
    const dt = parseFloat(byId("ode-dt").value) || 0.1;
    const n = parseInt(byId("ode-n").value, 10) || 50;
    let eulerRes = null;
    let rk4Res = null;
    if (hasOde) {
      const dydt = (t, y) => {
        const yArr = y instanceof Float64Array ? Array.from(y) : y;
        const dy = ex.dydt(t, yArr);
        return new Float64Array(dy);
      };
      const y0Arr = new Float64Array(ex.y0);
      eulerRes = eulerOde(dydt, y0Arr, 0, dt, n);
      rk4Res = rk4Ode(dydt, y0Arr, 0, dt, n);
    }
    const canvas = byId("canvas-ode");
    if (!canvas) return;
    const w = canvas.width;
    const h = canvas.height;
    const theme = getCanvasThemeColors();
    const ctx = canvas.getContext("2d");
    ctx.fillStyle = theme.bg;
    ctx.fillRect(0, 0, w, h);
    const pad = { left: 45, right: 20, top: 20, bottom: 35 };
    const plotW = w - pad.left - pad.right;
    const plotH = h - pad.top - pad.bottom;
    const tMax = n * dt;
    let minY = 0;
    let maxY = 1;
    if (eulerRes && rk4Res) {
      const te = eulerRes.getT();
      const ye = eulerRes.getY();
      const yr = rk4Res.getY();
      for (let i = 0; i < te.length; i++) {
        minY = Math.min(minY, ye[i], yr[i]);
        maxY = Math.max(maxY, ye[i], yr[i]);
      }
      if (ex.exact) {
        for (let i = 0; i <= 50; i++) {
          const t = (i / 50) * tMax;
          const y = ex.exact(t);
          minY = Math.min(minY, y);
          maxY = Math.max(maxY, y);
        }
      }
    }
    const rangeY = maxY - minY || 1;
    const toX = (t) => pad.left + (t / tMax) * plotW;
    const toY = (y) => pad.top + plotH - ((y - minY) / rangeY) * plotH;
    if (eulerRes && rk4Res) {
      const te = eulerRes.getT();
      const ye = eulerRes.getY();
      const tr = rk4Res.getT();
      const yr = rk4Res.getY();
      ctx.strokeStyle = "#0d6efd";
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.moveTo(toX(te[0]), toY(ye[0]));
      for (let i = 1; i < te.length; i++) ctx.lineTo(toX(te[i]), toY(ye[i]));
      ctx.stroke();
      ctx.strokeStyle = "#fd7e14";
      ctx.beginPath();
      ctx.moveTo(toX(tr[0]), toY(yr[0]));
      for (let i = 1; i < tr.length; i++) ctx.lineTo(toX(tr[i]), toY(yr[i]));
      ctx.stroke();
      if (ex.exact) {
        ctx.strokeStyle = "rgba(0,0,0,0.4)";
        ctx.setLineDash([3, 3]);
        ctx.beginPath();
        for (let i = 0; i <= 50; i++) {
          const t = (i / 50) * tMax;
          const y = ex.exact(t);
          if (i === 0) ctx.moveTo(toX(t), toY(y));
          else ctx.lineTo(toX(t), toY(y));
        }
        ctx.stroke();
        ctx.setLineDash([]);
      }
    }
    let text = "";
    if (!hasOde) text = needRebuild;
    else if (eulerRes && rk4Res) {
      const tLast = n * dt;
      const yEuler = eulerRes.getYAt(n);
      const yRk4 = rk4Res.getYAt(n);
      text = `t=${tLast.toFixed(2)}  Euler: y=${yEuler ? yEuler[0].toFixed(6) : "—"}  RK4: y=${yRk4 ? yRk4[0].toFixed(6) : "—"}`;
      if (ex.exact) {
        text += `  exact: ${ex.exact(tLast).toFixed(6)}`;
      }
    }
    byId("out-ode").textContent = text;
  }

  if (hasOde) {
    bindExampleSelector("ode-examples", ODE_EXAMPLES.map((e) => e.name), updateOde);
    byId("ode-dt").addEventListener("change", () => {
      const idx = byId("ode-examples").querySelector("button.active");
      updateOde(idx ? Array.from(byId("ode-examples").children).indexOf(idx) : 0);
    });
    byId("ode-n").addEventListener("change", () => {
      const idx = byId("ode-examples").querySelector("button.active");
      updateOde(idx ? Array.from(byId("ode-examples").children).indexOf(idx) : 0);
    });
    updateOde(0);
  } else {
    byId("out-ode").textContent = needRebuild;
  }
} catch (e) {
  const out = byId("out-roots");
  if (out) {
    out.className = "error";
    out.textContent = "Error: " + (e.message || String(e));
  }
  showError((e.message || "").toLowerCase().includes("fetch") || (e.message || "").toLowerCase().includes("import") ? needBuild + "\n\n" : "" + (e.message || String(e)));
}
