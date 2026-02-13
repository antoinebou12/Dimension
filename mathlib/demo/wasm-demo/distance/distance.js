/**
 * mathlib WASM demo — Distance metrics.
 */
import {
  initLib, byId, showError, needBuild,
  bindExampleSelector, scaleToCanvas,
} from "../shared.js";

try {
  const lib = await initLib();
  const { WasmVector, WasmDistance } = lib;

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
} catch (e) {
  const out = byId("out-distance");
  if (out) { out.className = "error"; out.textContent = "Error: " + (e.message || String(e)); }
  showError((e.message || "").toLowerCase().includes("fetch") || (e.message || "").toLowerCase().includes("import") ? needBuild + "\n\n" : "" + (e.message || String(e)));
}
