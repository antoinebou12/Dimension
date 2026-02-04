/**
 * mathlib WASM demo — Camera matrices.
 */
import {
  initLib, byId, showError, needBuild,
  bindExampleSelector, renderMatrix4x4Float,
} from "../shared.js";

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
  bindExampleSelector("camera-examples", ["Example 1", "Example 2"], (i) => {
    const r = cameraResults[i];
    byId("out-camera").innerHTML =
      "<strong>Perspective</strong> (aspect=" + r.aspect.toFixed(2) + ", fov=" + (r.fovY === Math.PI / 4 ? "π/4" : "π/6") + ", near=" + r.near + ", far=" + r.far + "):" +
      renderMatrix4x4Float(r.persp.toArray()) +
      "<strong>Look-at RH</strong> (eye 0,0,5 → target 0,0,0, up 0,1,0):" +
      renderMatrix4x4Float(r.lookAt.toArray());
  });
  byId("out-camera").innerHTML =
    "<strong>Perspective</strong> (aspect=1.78, fov=π/4, near=0.1, far=100):" +
    renderMatrix4x4Float(cameraResults[0].persp.toArray()) +
    "<strong>Look-at RH</strong> (eye 0,0,5 → target 0,0,0, up 0,1,0):" +
    renderMatrix4x4Float(cameraResults[0].lookAt.toArray());
} catch (e) {
  const out = byId("out-camera");
  if (out) { out.className = "error"; out.textContent = "Error: " + (e.message || String(e)); }
  showError((e.message || "").toLowerCase().includes("fetch") || (e.message || "").toLowerCase().includes("import") ? needBuild + "\n\n" : "" + (e.message || String(e)));
}
