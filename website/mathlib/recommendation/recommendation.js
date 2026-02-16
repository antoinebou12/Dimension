/**
 * mathlib WASM — Recommendation demo: text embeddings in 3D (PCA) and nearest-doc list.
 */
import { initLib, byId, showError, needBuild, scaleToCanvas } from "../shared.js";

function project3dTo2d(points3, angleX = 0.3, angleY = 0.2) {
  return points3.map((p) => {
    const x = p[0];
    const y = p[1];
    const z = p[2] ?? 0;
    return [x + angleX * z, y + angleY * z];
  });
}

function draw3DScatter(ctx, points3, selectedIndex, width, height) {
  const points2 = project3dTo2d(points3);
  const scaled = scaleToCanvas(points2, width, height, 24);
  const bg = getComputedStyle(document.documentElement).getPropertyValue("--canvas-bg")?.trim() || "#1a1a1e";
  ctx.fillStyle = bg;
  ctx.fillRect(0, 0, width, height);
  ctx.font = "12px system-ui, sans-serif";
  for (let i = 0; i < scaled.length; i++) {
    ctx.fillStyle = i === selectedIndex ? "#0a84ff" : "#0d6efd";
    ctx.strokeStyle = i === selectedIndex ? "#fff" : "#333";
    ctx.lineWidth = i === selectedIndex ? 2 : 1;
    ctx.beginPath();
    ctx.arc(scaled[i][0], scaled[i][1], i === selectedIndex ? 8 : 5, 0, 6.28);
    ctx.fill();
    ctx.stroke();
  }
}

function cosineSimilarity(a, b) {
  let dot = 0, na = 0, nb = 0;
  for (let i = 0; i < a.length; i++) {
    dot += a[i] * b[i];
    na += a[i] * a[i];
    nb += b[i] * b[i];
  }
  const denom = Math.sqrt(na) * Math.sqrt(nb);
  return denom === 0 ? 0 : dot / denom;
}

function getTopK(embeddings, queryIndex, k = 5) {
  const q = embeddings[queryIndex];
  const withScore = embeddings.map((emb, i) => ({ i, score: cosineSimilarity(q, emb) }));
  withScore.sort((a, b) => b.score - a.score);
  return withScore.slice(0, k);
}

try {
  const lib = await initLib();
  const { WasmMatrix, WasmPca } = lib;

  const resp = await fetch("data.json");
  if (!resp.ok) throw new Error("Failed to load data.json");
  const data = await resp.json();
  const texts = data.texts || [];
  const embeddings = data.embeddings || [];
  if (texts.length === 0 || embeddings.length !== texts.length) {
    throw new Error("data.json must have texts and embeddings of same length");
  }

  const n = texts.length;
  const dim = embeddings[0].length;
  // mathlib WasmMatrix.fromArray expects column-major: index = col * rows + row
  const flat = [];
  for (let j = 0; j < dim; j++) for (let i = 0; i < n; i++) flat.push(embeddings[i][j]);
  const mat = WasmMatrix.fromArray(n, dim, flat);
  const pca = new WasmPca(mat, 3);
  const projected = pca.transform(mat);
  const points3 = [];
  for (let i = 0; i < n; i++) {
    points3.push([projected.get(i, 0), projected.get(i, 1), projected.get(i, 2)]);
  }

  let selectedIndex = 0;
  const canvas = byId("canvas-recommendation");
  const ctx = canvas.getContext("2d");
  const queryInput = byId("query-input");
  const listEl = byId("recommendation-list");

  function render() {
    draw3DScatter(ctx, points3, selectedIndex, canvas.width, canvas.height);
    const top = getTopK(embeddings, selectedIndex, 6);
    listEl.innerHTML = "";
    top.forEach(({ i, score }) => {
      const li = document.createElement("li");
      li.textContent = `[${i}] ${texts[i].slice(0, 50)}${texts[i].length > 50 ? "…" : ""} (${score.toFixed(3)})`;
      li.classList.toggle("selected", i === selectedIndex);
      li.addEventListener("click", () => {
        selectedIndex = i;
        queryInput.value = String(i);
        render();
      });
      listEl.appendChild(li);
    });
  }

  queryInput.addEventListener("change", () => {
    const v = parseInt(queryInput.value, 10);
    if (!Number.isNaN(v) && v >= 0 && v < n) {
      selectedIndex = v;
      render();
    }
  });

  canvas.addEventListener("click", (ev) => {
    const rect = canvas.getBoundingClientRect();
    const scaleX = canvas.width / rect.width;
    const scaleY = canvas.height / rect.height;
    const x = (ev.clientX - rect.left) * scaleX;
    const y = (ev.clientY - rect.top) * scaleY;
    const points2 = project3dTo2d(points3);
    const scaled = scaleToCanvas(points2, canvas.width, canvas.height, 24);
    let best = -1;
    let bestDist = 999;
    for (let i = 0; i < scaled.length; i++) {
      const d = (scaled[i][0] - x) ** 2 + (scaled[i][1] - y) ** 2;
      if (d < bestDist) {
        bestDist = d;
        best = i;
      }
    }
    if (best >= 0) {
      selectedIndex = best;
      queryInput.value = String(best);
      render();
    }
  });

  queryInput.setAttribute("max", String(n - 1));
  render();
} catch (e) {
  const msg = (e.message || String(e)).toLowerCase();
  const needMsg = msg.includes("fetch") || msg.includes("import") ? needBuild + "\n\n" : "";
  showError(needMsg + (e.message || String(e)));
}
