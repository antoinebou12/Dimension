/**
 * mathlib WASM demo — Monte Carlo (π estimation, scatter plot).
 * Uses estimatePi(seed, n_samples); scatter drawn with JS XorShift64 to match mathlib RNG.
 */
import {
  initLib,
  byId,
  showError,
  needBuild,
  needRebuild,
  getCanvasThemeColors,
} from "../shared.js";

/** XorShift64 matching mathlib monte_carlo for reproducible scatter (same seed = same points). */
function createXorShift64(seed) {
  let state = BigInt(seed === 0 ? 1 : seed);
  const mask64 = (1n << 64n) - 1n;
  return {
    next_u64() {
      const x = state;
      state = state ^ (state << 13n);
      state = state ^ (state >> 7n);
      state = state ^ (state << 17n);
      state = state & mask64;
      return x;
    },
    uniform_in_range(low, high) {
      const u = Number(this.next_u64() >> 11n) * (1 / 9007199254740992);
      return low + u * (high - low);
    },
  };
}

/** Draw up to maxPoints from [-1,1]²: inside circle (blue) vs outside (red). */
function drawScatter(seed, maxPoints = 4000) {
  const canvas = byId("canvas-mc");
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  const w = canvas.width;
  const h = canvas.height;
  const theme = getCanvasThemeColors();
  ctx.fillStyle = theme.bg;
  ctx.fillRect(0, 0, w, h);
  const rng = createXorShift64(seed);
  const pad = 20;
  const size = Math.min(w, h) - 2 * pad;
  const cx = pad + size / 2;
  const cy = pad + size / 2;
  const toX = (x) => cx + (x * size) / 2;
  const toY = (y) => cy - (y * size) / 2;
  for (let i = 0; i < maxPoints; i++) {
    const x = rng.uniform_in_range(-1, 1);
    const y = rng.uniform_in_range(-1, 1);
    const inside = x * x + y * y <= 1;
    ctx.fillStyle = inside ? "#1e5ac8" : "#c83c3c";
    ctx.beginPath();
    ctx.arc(toX(x), toY(y), 1.2, 0, Math.PI * 2);
    ctx.fill();
  }
  ctx.strokeStyle = theme.stroke;
  ctx.lineWidth = 1;
  ctx.setLineDash([2, 2]);
  ctx.beginPath();
  ctx.arc(cx, cy, size / 2, 0, Math.PI * 2);
  ctx.stroke();
  ctx.setLineDash([]);
}

function runEstimate(lib, seed, n) {
  const t0 = performance.now();
  const piEst = lib.estimatePi(BigInt(seed), BigInt(n));
  const elapsed = (performance.now() - t0).toFixed(1);
  const err = Math.abs(piEst - Math.PI);
  return { piEst, err, elapsed };
}

function updateOutput(result) {
  const pre = byId("out-mc");
  if (!pre) return;
  pre.textContent =
    `π estimate = ${result.piEst.toFixed(6)}  (error = ${result.err.toFixed(6)}, ${result.elapsed} ms)`;
}

try {
  const lib = await initLib();
  const estimatePi = lib.estimatePi;
  const integrateXSquared = lib.integrateXSquared;

  if (typeof estimatePi !== "function") {
    byId("out-mc").textContent = needRebuild;
  } else {
    const seedInput = byId("input-seed");
    const samplesSelect = byId("samples-select");
    const runBtn = byId("btn-run");
    const outIntegral = byId("out-integral");

    const SAMPLES_OPTS = [
      { value: 10000, label: "10k" },
      { value: 100000, label: "100k" },
      { value: 1000000, label: "1M" },
    ];

    function getSeed() {
      const s = parseInt(seedInput?.value ?? "42", 10);
      return Number.isFinite(s) ? s : 42;
    }
    function getSamples() {
      const v = samplesSelect?.value;
      const n = v ? parseInt(v, 10) : 100000;
      return Number.isFinite(n) && n > 0 ? n : 100000;
    }

    function run() {
      const seed = getSeed();
      const n = getSamples();
      const result = runEstimate(lib, seed, n);
      updateOutput(result);
      drawScatter(seed);
      if (typeof integrateXSquared === "function" && outIntegral) {
        const integral = integrateXSquared(0, 1, BigInt(100000), BigInt(seed + 1));
        outIntegral.textContent =
          `∫₀¹ x² dx ≈ ${integral.toFixed(6)}  (expected 1/3 ≈ 0.333333)`;
      }
    }

    if (samplesSelect) {
      SAMPLES_OPTS.forEach(({ value, label }) => {
        const opt = document.createElement("option");
        opt.value = String(value);
        opt.textContent = label;
        if (value === 100000) opt.selected = true;
        samplesSelect.appendChild(opt);
      });
    }
    if (runBtn) runBtn.addEventListener("click", run);

    run();
  }
} catch (e) {
  const out = byId("out-mc");
  if (out) {
    out.className = "error";
    out.textContent = "Error: " + (e.message || String(e));
  }
  showError(
    (e.message || "").toLowerCase().includes("fetch") ||
      (e.message || "").toLowerCase().includes("import")
      ? needBuild + "\n\n" + (e.message || String(e))
      : String(e.message || e)
  );
}
