/**
 * mathlib WASM demo — ML (K-means, PCA, SVM, DBSCAN).
 */
import {
  initLib, byId, showError, needBuild, needRebuild,
  bindExampleSelector, scaleToCanvas,
} from "../shared.js";

function project3dTo2d(points) {
  return points.map((p) => [p[0] + 0.3 * (p[2] || 0), p[1] + 0.2 * (p[2] || 0)]);
}

function drawPcaCanvas(ctx, pointsP) {
  const scaledP = scaleToCanvas(pointsP, 320, 280);
  ctx.fillStyle = "#fff";
  ctx.fillRect(0, 0, 320, 280);
  let cx = 0, cy = 0;
  for (const p of scaledP) { cx += p[0]; cy += p[1]; }
  cx /= scaledP.length; cy /= scaledP.length;
  const pad = 20;
  const halfW = 0.4 * (320 - 2 * pad);
  const halfH = 0.4 * (280 - 2 * pad);
  ctx.strokeStyle = "#dc3545";
  ctx.lineWidth = 2;
  ctx.setLineDash([4, 4]);
  ctx.beginPath();
  ctx.moveTo(cx - halfW, cy);
  ctx.lineTo(cx + halfW, cy);
  ctx.stroke();
  ctx.strokeStyle = "#0d6efd";
  ctx.beginPath();
  ctx.moveTo(cx, cy - halfH);
  ctx.lineTo(cx, cy + halfH);
  ctx.stroke();
  ctx.setLineDash([]);
  ctx.font = "10px \"DM Sans\", system-ui, sans-serif";
  ctx.fillStyle = "#dc3545";
  ctx.fillText("PC1", cx + halfW + 4, cy + 4);
  ctx.fillStyle = "#0d6efd";
  ctx.fillText("PC2", cx + 4, cy - halfH - 4);
  ctx.fillStyle = "#0d6efd";
  for (const p of scaledP) { ctx.beginPath(); ctx.arc(p[0], p[1], 5, 0, 6.28); ctx.fill(); }
}

function drawSvmCanvas(r, kernel, predictFn) {
  const canvas = byId("canvas-svm");
  if (!canvas) return;
  const w = canvas.width;
  const h = canvas.height;
  const ctxS = canvas.getContext("2d");
  ctxS.fillStyle = "#fff";
  ctxS.fillRect(0, 0, w, h);
  const n = r.pointsS.length;
  let minX = r.pointsS[0][0], maxX = minX, minY = r.pointsS[0][1], maxY = minY;
  for (const p of r.pointsS) {
    minX = Math.min(minX, p[0]); maxX = Math.max(maxX, p[0]);
    minY = Math.min(minY, p[1]); maxY = Math.max(maxY, p[1]);
  }
  const pad = 0.3 * Math.max(maxX - minX || 1, maxY - minY || 1) || 1;
  minX -= pad; maxX += pad; minY -= pad; maxY += pad;
  const gridRes = 48;
  const getPred = (x, y) => {
    if (kernel === "rbf" && predictFn) return predictFn([x, y]);
    if (kernel === "linear" && r.w && r.bias != null) return r.w[0] * x + r.w[1] * y + r.bias;
    return 0;
  };
  for (let gi = 0; gi < gridRes; gi++) {
    for (let gj = 0; gj < gridRes; gj++) {
      const x = minX + (gj / (gridRes - 1)) * (maxX - minX);
      const y = maxY - (gi / (gridRes - 1)) * (maxY - minY);
      const pred = getPred(x, y);
      ctxS.fillStyle = pred >= 0 ? "rgba(13,110,253,0.22)" : "rgba(253,126,20,0.22)";
      ctxS.fillRect((gj / gridRes) * w, (gi / gridRes) * h, Math.ceil(w / gridRes) + 1, Math.ceil(h / gridRes) + 1);
    }
  }
  const allS = r.pointsS.slice();
  const drawLine = kernel === "linear" && r.w && Math.abs(r.w[1]) > 1e-10;
  if (drawLine) {
    allS.push([minX, -(r.w[0] * minX + r.bias) / r.w[1]], [maxX, -(r.w[0] * maxX + r.bias) / r.w[1]]);
  }
  const scaledS = scaleToCanvas(allS, w, h, 15);
  if (drawLine) {
    ctxS.strokeStyle = "#000";
    ctxS.lineWidth = 2;
    ctxS.beginPath();
    ctxS.moveTo(scaledS[n][0], scaledS[n][1]);
    ctxS.lineTo(scaledS[n + 1][0], scaledS[n + 1][1]);
    ctxS.stroke();
  }
  for (let i = 0; i < n; i++) {
    ctxS.fillStyle = r.labels[i] === 1 ? "#0d6efd" : "#fd7e14";
    ctxS.strokeStyle = "#212529";
    ctxS.lineWidth = 1;
    ctxS.beginPath();
    ctxS.arc(scaledS[i][0], scaledS[i][1], 6, 0, 6.28);
    ctxS.fill();
    ctxS.stroke();
  }
  ctxS.font = "11px \"DM Sans\", system-ui, sans-serif";
  ctxS.fillStyle = "#0d6efd";
  ctxS.fillText("+1", w - 28, 18);
  ctxS.fillStyle = "#fd7e14";
  ctxS.fillText("-1", w - 28, 32);
}

try {
  const lib = await initLib();
  const { WasmMatrix, WasmVector, WasmKmeans, WasmPca, WasmSvm, WasmDbscan } = lib;
  const WasmSvmRbf = lib.WasmSvmRbf;
  const NOISE_LABEL = lib.NOISE_LABEL;
  const WasmMatrix32 = lib.WasmMatrix32;
  const gpuAvailable = lib.gpuAvailable;

  // —— K-means ——
  const KMEANS_EXAMPLES = [
    { data: [0, 0, 1, 0, 0.5, 0.5, 5, 5, 6, 5, 5.5, 5.5, 10, 0, 11, 0, 10.5, 0.5], rows: 9, cols: 2, k: 3 },
    { data: [0, 1, 2, 10, 11, 12, 0, 0, 0, 10, 10, 10], rows: 6, cols: 2, k: 2 },
    { data: [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1], rows: 8, cols: 3, k: 2 },
  ];
  const kmeansResults = KMEANS_EXAMPLES.map((ex) => {
    const dataK = WasmMatrix.fromArray(ex.rows, ex.cols, ex.data);
    const km = new WasmKmeans(dataK, ex.k, 100);
    const labelsK = km.getLabels();
    const centroidsK = km.getCentroids();
    const cents = [];
    for (let i = 0; i < ex.k; i++) {
      if (ex.cols === 3) cents.push([centroidsK.get(i, 0), centroidsK.get(i, 1), centroidsK.get(i, 2)]);
      else cents.push([centroidsK.get(i, 0), centroidsK.get(i, 1)]);
    }
    const pointsK = [];
    for (let i = 0; i < ex.rows; i++) {
      if (ex.cols === 3) pointsK.push([dataK.get(i, 0), dataK.get(i, 1), dataK.get(i, 2)]);
      else pointsK.push([dataK.get(i, 0), dataK.get(i, 1)]);
    }
    return { labelsK, cents, pointsK, n: ex.rows, k: ex.k, cols: ex.cols };
  });
  function showKmeans(i) {
    const r = kmeansResults[i];
    let centStr = "";
    for (let j = 0; j < r.k; j++) centStr += (j ? ", " : "") + "(" + r.cents[j].map((x) => Number(x).toFixed(2)).join(", ") + ")";
    byId("out-kmeans").textContent = "Labels: [" + r.labelsK.join(", ") + "]\nCentroids: " + centStr;
    const pts2d = r.cols === 3 ? project3dTo2d(r.pointsK) : r.pointsK;
    const cents2d = r.cols === 3 ? project3dTo2d(r.cents) : r.cents;
    const allK = [...pts2d, ...cents2d];
    const scaledK = scaleToCanvas(allK, 320, 280);
    const ctxK = byId("canvas-kmeans").getContext("2d");
    ctxK.fillStyle = "#fff";
    ctxK.fillRect(0, 0, 320, 280);
    const pal = ["#0d6efd", "#fd7e14", "#198754"];
    for (let i = 0; i < r.n; i++) {
      ctxK.fillStyle = pal[r.labelsK[i] % pal.length] || "#0d6efd";
      ctxK.beginPath();
      ctxK.arc(scaledK[i][0], scaledK[i][1], 5, 0, 6.28);
      ctxK.fill();
    }
    ctxK.strokeStyle = "#000";
    ctxK.lineWidth = 2;
    for (let i = r.n; i < r.n + r.k; i++) {
      ctxK.beginPath();
      ctxK.arc(scaledK[i][0], scaledK[i][1], 8, 0, 6.28);
      ctxK.stroke();
    }
  }
  bindExampleSelector("kmeans-examples", ["3 blobs 2D", "2 clusters", "Cube 3D"], showKmeans);
  showKmeans(0);

  // —— PCA ——
  const PCA_EXAMPLES = [
    (() => { const d = []; for (let i = 0; i < 10; i++) for (let j = 0; j < 4; j++) d.push(i * 0.5 + j); return { data: d, rows: 10, cols: 4 }; })(),
    (() => { const d = []; for (let i = 0; i < 8; i++) for (let j = 0; j < 3; j++) d.push(i + j * 2); return { data: d, rows: 8, cols: 3 }; })(),
  ];
  const useGpuTransform = typeof gpuAvailable === "function" && gpuAvailable() && typeof WasmMatrix32 !== "undefined";
  const pcaResults = await Promise.all(PCA_EXAMPLES.map(async (ex) => {
    const matP = WasmMatrix.fromArray(ex.rows, ex.cols, ex.data);
    const pca = new WasmPca(matP, 2);
    const meanP = pca.getMean().toArray();
    const evP = pca.getExplainedVariance().toArray();
    let proj = null;
    if (useGpuTransform && typeof pca.transformF32GpuAsync === "function") {
      const mat32 = WasmMatrix32.fromArray(ex.rows, ex.cols, ex.data.map((x) => x));
      proj = await pca.transformF32GpuAsync(mat32);
    }
    if (proj == null) proj = pca.transform(matP);
    const pointsP = [];
    for (let i = 0; i < ex.rows; i++) pointsP.push([proj.get(i, 0), proj.get(i, 1)]);
    return { meanP, evP, pointsP, rows: ex.rows };
  }));
  bindExampleSelector("pca-examples", ["Example 1", "Example 2"], (i) => {
    const r = pcaResults[i];
    byId("out-pca").textContent =
      "Mean: [" + r.meanP.map((x) => x.toFixed(3)).join(", ") + "]\nExplained variance (2): [" + r.evP.map((x) => x.toFixed(4)).join(", ") + "]";
    drawPcaCanvas(byId("canvas-pca").getContext("2d"), r.pointsP);
  });
  byId("out-pca").textContent =
    "Mean (4): [" + pcaResults[0].meanP.map((x) => x.toFixed(3)).join(", ") + "]\nExplained variance (2): [" + pcaResults[0].evP.map((x) => x.toFixed(4)).join(", ") + "]";
  drawPcaCanvas(byId("canvas-pca").getContext("2d"), pcaResults[0].pointsP);

  // —— SVM ——
  const SVM_LINEAR_EXAMPLES = [
    { data: [1, 2, 1, 2, 3, 4, 3, 4, 6, 7, 6, 7, 8, 9, 8, 9], labels: [1, 1, 1, 1, -1, -1, -1, -1], n: 8 },
    { data: [0, 0, 1, 0, 0, 1, 1, 1, 3, 3, 4, 3, 3, 4, 4, 4], labels: [1, 1, 1, 1, -1, -1, -1, -1], n: 8 },
  ];
  const SVM_RBF_EXAMPLES = [
    { data: [0, 0, 0.5, 0, 0.35, 0.35, -0.5, 0, -0.35, -0.35, 2, 0, 2, 2, 0, 2, -2, 0, -1, -1.5], labels: [1, 1, 1, 1, 1, -1, -1, -1, -1, -1], n: 10, gamma: 0.5 },
    { data: [0.5, 0, 0.35, 0.35, 0, 0.5, -0.35, 0.35, -0.5, 0, 1.5, 0, 1, 1, 0, 1.5, -1, 0, -1, -1], labels: [1, 1, 1, 1, 1, -1, -1, -1, -1, -1], n: 10, gamma: 0.8 },
  ];
  let svmKernelIndex = 0;
  let svmExampleIndex = 0;
  function getSvmExample() {
    const exs = svmKernelIndex === 0 ? SVM_LINEAR_EXAMPLES : SVM_RBF_EXAMPLES;
    return exs[svmExampleIndex];
  }
  function updateSvmDemo() {
    const ex = getSvmExample();
    try {
      const dataS = WasmMatrix.fromArray(ex.n, 2, ex.data);
      if (svmKernelIndex === 0) {
        const res = WasmSvm.train(dataS, ex.labels);
        const w = res.getWeights().toArray();
        const bias = res.getBias();
        const preds = res.predictAll(dataS);
        const pointsS = [];
        for (let i = 0; i < ex.n; i++) pointsS.push([dataS.get(i, 0), dataS.get(i, 1)]);
        const r = { w, bias, preds, pointsS, labels: ex.labels };
        byId("out-svm").textContent = "Linear: weights [" + r.w.map((x) => Number(x).toFixed(4)).join(", ") + "], bias " + r.bias.toFixed(4) + "\nPredictions: [" + r.preds.map((x) => Number(x)).join(", ") + "]";
        drawSvmCanvas(r, "linear");
      } else {
        const gamma = ex.gamma || 0.5;
        const res = WasmSvmRbf.train(dataS, ex.labels, gamma);
        const preds = res.predictAll(dataS);
        const pointsS = [];
        for (let i = 0; i < ex.n; i++) pointsS.push([dataS.get(i, 0), dataS.get(i, 1)]);
        const r = { preds, pointsS, labels: ex.labels };
        const predFn = (p) => res.predict(p);
        byId("out-svm").textContent = "RBF γ=" + gamma + ", n_sv=" + res.getSupportVectors().rows + "\nPredictions: [" + r.preds.map((x) => Number(x)).join(", ") + "]";
        drawSvmCanvas(r, "rbf", predFn);
      }
    } catch (err) {
      byId("out-svm").textContent = "Error: " + (err.message || err);
    }
  }
  bindExampleSelector("svm-kernel", ["Linear", "RBF"], (i) => {
    svmKernelIndex = i;
    svmExampleIndex = 0;
    byId("svm-examples").querySelectorAll("button").forEach((b, j) => b.classList.toggle("active", j === 0));
    updateSvmDemo();
  });
  bindExampleSelector("svm-examples", ["Example 1", "Example 2"], (i) => {
    svmExampleIndex = i;
    updateSvmDemo();
  });
  updateSvmDemo();

  // —— DBSCAN ——
  if (typeof WasmDbscan !== "function") {
    byId("out-dbscan").textContent = needRebuild;
  } else {
    const noiseVal = typeof NOISE_LABEL !== "undefined" && typeof NOISE_LABEL === "function" ? NOISE_LABEL() : 4294967295;
    const DBSCAN_EXAMPLES = [
      { data: [0, 1, 0, 10, 0, 0, 1, 10], n: 4, eps: 2.0, minPts: 2 },
      { data: [0, 0, 1, 0, 2, 0, 0, 1, 1, 1, 10, 10], n: 6, eps: 1.5, minPts: 2 },
    ];
    const dbscanResults = DBSCAN_EXAMPLES.map((ex) => {
      const dataDb = WasmMatrix.fromArray(ex.n, 2, ex.data);
      const db = new WasmDbscan(dataDb, ex.eps, ex.minPts);
      const labelsDb = db.getLabels();
      const pointsDb = [];
      for (let i = 0; i < ex.n; i++) pointsDb.push([dataDb.get(i, 0), dataDb.get(i, 1)]);
      return { labelsDb, pointsDb, nClusters: db.nClusters(), n: ex.n, eps: ex.eps, minPts: ex.minPts, noiseVal };
    });
    function showDbscan(i) {
      const r = dbscanResults[i];
      byId("out-dbscan").textContent =
        r.n + " points; eps=" + r.eps + ", min_pts=" + r.minPts + "\nlabels: [" + r.labelsDb.join(", ") + "]\nn_clusters: " + r.nClusters;
      const scaledDb = scaleToCanvas(r.pointsDb, 320, 280);
      const ctxDb = byId("canvas-dbscan").getContext("2d");
      ctxDb.fillStyle = "#fff";
      ctxDb.fillRect(0, 0, 320, 280);
      for (let j = 0; j < r.n; j++) {
        const isNoise = r.labelsDb[j] === r.noiseVal || r.labelsDb[j] > 1000;
        ctxDb.fillStyle = isNoise ? "#fff" : r.labelsDb[j] === 0 ? "#0d6efd" : "#fd7e14";
        ctxDb.strokeStyle = "#333";
        ctxDb.lineWidth = 1;
        ctxDb.beginPath();
        ctxDb.arc(scaledDb[j][0], scaledDb[j][1], 8, 0, 6.28);
        if (isNoise) ctxDb.stroke();
        else ctxDb.fill();
      }
    }
    bindExampleSelector("dbscan-examples", ["Example 1", "Example 2"], showDbscan);
    showDbscan(0);
  }
} catch (e) {
  const out = byId("out-kmeans");
  if (out) { out.className = "error"; out.textContent = "Error: " + (e.message || String(e)); }
  showError((e.message || "").toLowerCase().includes("fetch") || (e.message || "").toLowerCase().includes("import") ? needBuild + "\n\n" : "" + (e.message || String(e)));
}
