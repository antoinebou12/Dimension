/**
 * mathlib WASM demo — Viz (heightmap from noise).
 */
import {
  initLib, byId, showError, needBuild, needRebuild,
  bindExampleSelector,
} from "../shared.js";

try {
  const lib = await initLib();
  const wave2d = lib.wave2d;
  const wave2dParams = lib.wave2dParams;
  const fbm2dPerlin = lib.fbm2dPerlin;

  if (typeof wave2d !== "function" || typeof fbm2dPerlin !== "function") {
    byId("out-viz").textContent = needRebuild;
  } else {
    const VIZ_EXAMPLES = [
      { name: "Wave", fill: (i, j, gs) => wave2d((j / gs) * 4, (i / gs) * 4), label: "wave2d" },
      { name: "Wave (params)", fill: (i, j, gs) => wave2dParams((j / gs) * 4, (i / gs) * 4, 1, 1), label: "wave2dParams" },
      { name: "FBM Perlin", fill: (i, j, gs) => fbm2dPerlin((j / gs) * 3, (i / gs) * 3, 6, 2, 0.5), label: "fbm2dPerlin" },
    ];
    const gridSize = 64;
    function drawVizCanvas(ex) {
      const grid = [];
      for (let i = 0; i < gridSize; i++)
        for (let j = 0; j < gridSize; j++) grid.push(ex.fill(i, j, gridSize));
      const nMin = Math.min(...grid), nMax = Math.max(...grid), nRange = nMax - nMin || 1;
      const ctx = byId("canvas-viz").getContext("2d");
      const imgData = ctx.createImageData(320, 280);
      for (let i = 0; i < gridSize; i++) {
        for (let j = 0; j < gridSize; j++) {
          const v = (grid[i * gridSize + j] - nMin) / nRange;
          const gray = Math.round(255 * v);
          const px = Math.floor((j / gridSize) * 320), py = Math.floor((i / gridSize) * 280);
          for (let dy = 0; dy < 5; dy++)
            for (let dx = 0; dx < 6; dx++) {
              const x = px + dx, y = py + dy;
              if (x < 320 && y < 280) {
                const idx = (y * 320 + x) * 4;
                imgData.data[idx] = imgData.data[idx + 1] = imgData.data[idx + 2] = gray;
                imgData.data[idx + 3] = 255;
              }
            }
        }
      }
      ctx.putImageData(imgData, 0, 0);
    }
    bindExampleSelector("viz-examples", VIZ_EXAMPLES.map((e) => e.name), (i) => {
      byId("out-viz").textContent = VIZ_EXAMPLES[i].label + " 64×64 heightmap";
      drawVizCanvas(VIZ_EXAMPLES[i]);
    });
    byId("out-viz").textContent = VIZ_EXAMPLES[0].label + " 64×64 heightmap";
    drawVizCanvas(VIZ_EXAMPLES[0]);
  }
} catch (e) {
  const out = byId("out-viz");
  if (out) { out.className = "error"; out.textContent = "Error: " + (e.message || String(e)); }
  showError((e.message || "").toLowerCase().includes("fetch") || (e.message || "").toLowerCase().includes("import") ? needBuild + "\n\n" : "" + (e.message || String(e)));
}
