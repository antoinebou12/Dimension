/**
 * mathlib WASM demo — Transforms (FFT, DCT, wavelets, convolution).
 */
import {
  initLib, byId, showError, needBuild,
  bindExampleSelector,
} from "../shared.js";

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
      if (spec && spec.length >= 2) return "fft(8-point impulse) → " + spec.slice(0, 4).map((x) => x.toFixed(2)).join(", ") + "...";
      return "FFT: " + (spec ? "ok" : "error");
    }},
    { name: "DCT-2", run: () => {
      const sig = [1, 2, 3, 4, 4, 3, 2, 1];
      const coeffs = dct2Forward(sig);
      if (coeffs && coeffs.length >= 2) return "dct2([1,2,3,4,4,3,2,1]) → " + coeffs.slice(0, 4).map((x) => x.toFixed(2)).join(", ") + "...";
      return "DCT: " + (coeffs ? "ok" : "error");
    }},
    { name: "Haar DWT", run: () => {
      const sig = [1, 2, 3, 4];
      const c = dwtHaarForward(sig);
      return "dwtHaar([1,2,3,4]) → [" + c.map((x) => x.toFixed(2)).join(", ") + "]";
    }},
    { name: "Convolution", run: () => {
      const sig = [1, 2, 3, 4, 5];
      const ker = [1, 1, 1];
      const out = conv1dSame(sig, ker);
      return "conv1dSame([1,2,3,4,5], [1,1,1]) → [" + out.map((x) => x.toFixed(0)).join(", ") + "]";
    }},
  ];
  bindExampleSelector("transforms-examples", TRANSFORMS_EXAMPLES.map((e) => e.name), (i) => {
    try {
      byId("out-transforms").textContent = TRANSFORMS_EXAMPLES[i].run();
    } catch (err) {
      byId("out-transforms").textContent = "Error: " + (err.message || err);
    }
  });
  byId("out-transforms").textContent = TRANSFORMS_EXAMPLES[0].run();
} catch (e) {
  const out = byId("out-transforms");
  if (out) { out.className = "error"; out.textContent = "Error: " + (e.message || String(e)); }
  showError((e.message || "").toLowerCase().includes("fetch") || (e.message || "").toLowerCase().includes("import") ? needBuild + "\n\n" : "" + (e.message || String(e)));
}
