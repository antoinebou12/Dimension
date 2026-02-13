/**
 * mathlib WASM demo — Camera matrices.
 */
import {
  initLib, byId, showError, needBuild,
  bindExampleSelector, renderMatrix4x4Float, getCanvasThemeColors,
} from "../shared.js";

function drawFrustumCanvas(ctx, width, height, aspect, fovY, near, far) {
  const theme = getCanvasThemeColors();
  ctx.fillStyle = theme.bg;
  ctx.fillRect(0, 0, width, height);
  const pad = 32;
  const plotW = width - 2 * pad;
  const plotH = height - 2 * pad;
  const cy = pad + plotH / 2;
  const hNear = near * Math.tan(fovY / 2);
  const hFar = far * Math.tan(fovY / 2);
  const scaleX = plotW / far;
  const maxH = Math.max(hNear, hFar);
  const scaleY = maxH > 0 ? (plotH / 2) / maxH : 1;
  const xEye = pad;
  const xNear = pad + near * scaleX;
  const xFar = pad + far * scaleX;
  const yNearTop = cy - hNear * scaleY;
  const yNearBot = cy + hNear * scaleY;
  const yFarTop = cy - hFar * scaleY;
  const yFarBot = cy + hFar * scaleY;
  // Frustum interior (light fill)
  ctx.fillStyle = theme.barPos;
  ctx.globalAlpha = 0.08;
  ctx.beginPath();
  ctx.moveTo(xEye, cy);
  ctx.lineTo(xNear, yNearTop);
  ctx.lineTo(xFar, yFarTop);
  ctx.lineTo(xFar, yFarBot);
  ctx.lineTo(xNear, yNearBot);
  ctx.closePath();
  ctx.fill();
  ctx.globalAlpha = 1;
  // Frustum edges and eye
  ctx.strokeStyle = theme.stroke;
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(xEye, cy);
  ctx.lineTo(xNear, yNearTop);
  ctx.moveTo(xEye, cy);
  ctx.lineTo(xNear, yNearBot);
  ctx.moveTo(xNear, yNearTop);
  ctx.lineTo(xFar, yFarTop);
  ctx.moveTo(xNear, yNearBot);
  ctx.lineTo(xFar, yFarBot);
  ctx.moveTo(xEye, cy);
  ctx.lineTo(xFar, yFarTop);
  ctx.moveTo(xEye, cy);
  ctx.lineTo(xFar, yFarBot);
  ctx.stroke();
  ctx.fillStyle = theme.barPos;
  ctx.beginPath();
  ctx.arc(xEye, cy, 6, 0, 2 * Math.PI);
  ctx.fill();
  ctx.strokeStyle = theme.barPos;
  ctx.lineWidth = 1.5;
  ctx.stroke();
  ctx.font = "12px \"DM Sans\", system-ui, sans-serif";
  ctx.fillStyle = theme.stroke;
  ctx.fillText("eye", xEye - 14, cy + 22);
  ctx.fillText("near", xNear - 12, cy + 22);
  ctx.fillText("far", xFar - 10, cy + 22);
}

try {
  const lib = await initLib();
  const { WasmCg } = lib;

  const CAMERA_EXAMPLES = [
    { aspect: 16 / 9, fovY: Math.PI / 4, near: 0.1, far: 100 },
    { aspect: 1, fovY: Math.PI / 6, near: 0.5, far: 50 },
  ];
  const cameraResults = CAMERA_EXAMPLES.map((ex) => {
    const persp = WasmCg.newPerspective(ex.aspect, ex.fovY, ex.near, ex.far);
    const lookAt = WasmCg.lookAtRh(0, 0, 5, 0, 0, 0, 0, 1, 0);
    return { ...ex, persp, lookAt };
  });
  function updateCamera(i) {
    const r = cameraResults[i];
    const fovLabel = r.fovY === Math.PI / 4 ? "π/4" : "π/6";
    const perspParams = "aspect " + r.aspect.toFixed(2) + ", fov " + fovLabel + ", near " + r.near + ", far " + r.far;
    const perspHtml =
      "<div class=\"camera-matrix-card\">" +
      "<h3>Perspective</h3>" +
      "<p class=\"camera-params\">" + perspParams + "</p>" +
      renderMatrix4x4Float(r.persp.toArray()) +
      "</div>";
    const lookAtHtml =
      "<div class=\"camera-matrix-card\">" +
      "<h3>Look-at (RH)</h3>" +
      "<p class=\"camera-params\">eye (0, 0, 5) → target (0, 0, 0), up (0, 1, 0)</p>" +
      renderMatrix4x4Float(r.lookAt.toArray()) +
      "</div>";
    byId("out-camera").innerHTML = perspHtml + lookAtHtml;
    const canvas = byId("canvas-frustum");
    if (canvas && canvas.getContext) {
      drawFrustumCanvas(canvas.getContext("2d"), canvas.width, canvas.height, r.aspect, r.fovY, r.near, r.far);
    }
  }
  bindExampleSelector("camera-examples", ["Example 1", "Example 2"], updateCamera);
  updateCamera(0);
} catch (e) {
  const out = byId("out-camera");
  if (out) { out.className = "error"; out.textContent = "Error: " + (e.message || String(e)); }
  showError((e.message || "").toLowerCase().includes("fetch") || (e.message || "").toLowerCase().includes("import") ? needBuild + "\n\n" : "" + (e.message || String(e)));
}
