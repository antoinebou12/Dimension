# kinematics-demo

3D demo of kinematics: 3-joint arm with forward kinematics and FABRIK/Jacobian IK. You can switch between **Spherical** (all-direction joints) and **Revolute** (root fixed, revolute about Y) arm presets. The IK target is moved via view-plane projection (right-drag) and scroll for depth. **Random target** places the target within the current end-effector's reach for stable solver behavior. The demo renders joint spheres, link segments, and the IK target cube. Joint orientations are synced to the render scene using quaternions.

## Controls

- **Left drag**: Orbit camera
- **Ctrl+left drag**: Pan camera
- **Right drag**: Set IK target (view-plane projection; moves target in the screen plane)
- **Scroll**: Zoom; while right-dragging: move IK target in/out along view direction (depth)

## UI panels

- **Armature**: Tree of nodes with joint type and current angle (rad) for revolute joints. Spherical joints show type only. Optional limits shown as `[min, max]` when set.
- **Arm**: Preset selector (Spherical vs Revolute). Spherical uses all spherical joints; Revolute uses a fixed root and revolute joints (rotation about Y). Switching rebuilds the scene.
- **End effector**: Solver selector (FABRIK vs Jacobian IK). Buttons to choose which node is the IK target tip (EE: 1, 2, 3). **Random target** picks a new target within the selected end-effector's reach. Readouts show current end-effector position and target position.

## Run native (winit)

```bash
just run-kinematics-demo
# or
cargo run --example kinematics_native
```

## Run WASM (browser)

```bash
just build-kinematics-demo-wasm
just wasm-kinematics-demo
```

Then open http://localhost:3000/wasm-demo/ in a browser that supports WebGPU. After changing the kinematics crate, rebuild the demo WASM (`just build-kinematics-demo-wasm`) and refresh the page to see solver updates.

## E2E tests (Playwright)

From repo root, run the kinematics-demo e2e tests:

```bash
just e2e-kinematics
```

Or run all WASM e2e tests (render, mathlib, kinematics): `just e2e`. See [e2e/README.md](../../e2e/README.md).
