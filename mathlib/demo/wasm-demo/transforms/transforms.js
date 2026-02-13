/**
 * mathlib WASM demo — Transforms (FFT, DCT, wavelets, convolution).
 */
import {
  initLib, byId, showError, needBuild,
  bindExampleSelector,
} from "../shared.js";

function drawBarChart(ctx, width, height, data, color) {
  if (!data || data.length === 0) return;
  ctx.fillStyle = "#fff";
  ctx.fillRect(0, 0, width, height);
  const n = data.length;
  const pad = 4;
  const barW = Math.max(1, (width - 2 * pad) / n - 2);
  const maxVal = Math.max(...data.map((v) => Math.abs(v)), 1e-10);
  const halfH = (height - 2 * pad) / 2;
  ctx.fillStyle = color || "#0d6efd";
  for (let i = 0; i < n; i++) {
    const x = pad + i * ((width - 2 * pad) / n);
    const val = data[i];
    const h = (val / maxVal) * halfH;
    const y = h >= 0 ? halfH + pad - h : halfH + pad;
    ctx.fillRect(x, y, barW, Math.abs(h) || 1);
  }
  ctx.strokeStyle = "#dee2e6";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(0, halfH + pad);
  ctx.lineTo(width, halfH + pad);
  ctx.stroke();
}

try {
  const lib = await initLib();
  const fftForwardReal = lib.fftForwardReal;
  const fftInverse = lib.fftInverse;
  const dct2Forward = lib.dct2Forward;
  const dct2Inverse = lib.dct2Inverse;
  const dwtHaarForward = lib.dwtHaarForward;
  const dwtHaarInverse = lib.dwtHaarInverse;
  const conv1dSame = lib.conv1dSame;

  const TRANSFORMS_EXAMPLES = [
    { name: "FFT", run: () => {
      const sig = [1, 0, 0, 0, 0, 0, 0, 0];
      const spec = fftForwardReal(sig);
      if (!spec || spec.length < 2) return { text: "FFT: " + (spec ? "ok" : "error"), signal: sig, output: [] };
      const magnitude = [];
      for (let i = 0; i < spec.length; i += 2) magnitude.push(Math.sqrt(spec[i] * spec[i] + spec[i + 1] * spec[i + 1]));
      return { text: "fft(8-point impulse) → " + spec.slice(0, 4).map((x) => x.toFixed(2)).join(", ") + "...", signal: sig, output: magnitude };
    }},
    { name: "DCT-2", run: () => {
      const sig = [1, 2, 3, 4, 4, 3, 2, 1];
      const coeffs = dct2Forward(sig);
      if (!coeffs || coeffs.length < 2) return { text: "DCT: " + (coeffs ? "ok" : "error"), signal: sig, output: [] };
      return { text: "dct2([1,2,3,4,4,3,2,1]) → " + coeffs.slice(0, 4).map((x) => x.toFixed(2)).join(", ") + "...", signal: sig, output: coeffs };
    }},
    { name: "Haar DWT", run: () => {
      const sig = [1, 2, 3, 4];
      const c = dwtHaarForward(sig);
      return { text: "dwtHaar([1,2,3,4]) → [" + c.map((x) => x.toFixed(2)).join(", ") + "]", signal: sig, output: c };
    }},
    { name: "Convolution", run: () => {
      const sig = [1, 2, 3, 4, 5];
      const ker = [1, 1, 1];
      const out = conv1dSame(sig, ker);
      return { text: "conv1dSame([1,2,3,4,5], [1,1,1]) → [" + out.map((x) => x.toFixed(0)).join(", ") + "]", signal: sig, output: out };
    }},
  ];

  function updateTransforms(i) {
    try {
      const result = TRANSFORMS_EXAMPLES[i].run();
      const text = typeof result === "string" ? result : result.text;
      const signal = typeof result === "object" && result.signal ? result.signal : [];
      const output = typeof result === "object" && result.output ? result.output : [];
      byId("out-transforms").textContent = text;
      const canvasSig = byId("canvas-transforms-signal");
      const canvasOut = byId("canvas-transforms-output");
      if (canvasSig && canvasSig.getContext) drawBarChart(canvasSig.getContext("2d"), canvasSig.width, canvasSig.height, signal, "#0d6efd");
      if (canvasOut && canvasOut.getContext) drawBarChart(canvasOut.getContext("2d"), canvasOut.width, canvasOut.height, output, "#198754");
    } catch (err) {
      byId("out-transforms").textContent = "Error: " + (err.message || err);
    }
  }

  bindExampleSelector("transforms-examples", TRANSFORMS_EXAMPLES.map((e) => e.name), updateTransforms);
  updateTransforms(0);
} catch (e) {
  const out = byId("out-transforms");
  if (out) { out.className = "error"; out.textContent = "Error: " + (e.message || String(e)); }
  showError((e.message || "").toLowerCase().includes("fetch") || (e.message || "").toLowerCase().includes("import") ? needBuild + "\n\n" : "" + (e.message || String(e)));
}
