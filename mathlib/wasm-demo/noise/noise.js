/**
 * mathlib WASM demo — Noise (wave, Perlin, FBM).
 */
import {
  initLib, byId, showError, needBuild, needRebuild,
  bindExampleSelector,
} from "../shared.js";

try {
  const lib = await initLib();
  const wave2d = lib.wave2d;
  const perlin2d = lib.perlin2d;
  const fbm2dPerlin = lib.fbm2dPerlin;

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
} catch (e) {
  const out = byId("out-noise");
  if (out) { out.className = "error"; out.textContent = "Error: " + (e.message || String(e)); }
  showError((e.message || "").toLowerCase().includes("fetch") || (e.message || "").toLowerCase().includes("import") ? needBuild + "\n\n" : "" + (e.message || String(e)));
}
