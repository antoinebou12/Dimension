# kinematics

Forward and inverse kinematics for articulated structures, built on [mathlib](../mathlib).

## Features

- **Joint types** (in `src/joints/`): fixed, revolute, prismatic, spherical
- **Armature**: Tree of joints with forward kinematics, pack/unpack state
- **IK solvers**: Jacobian (SVD + line search), FABRIK
- **WASM bindings**: `wasm-pack build --target web --features wasm`; `WasmArmature` for chains, IK
- **Optional SIMD/parallel**: `--features simd`, `--features parallel` (parallel not on wasm32)

## Quick start

```rust
use kinematics::{Armature, JointData, JointVariant};
use kinematics::joints::RevoluteJoint;
use mathlib::cg::vector3;

let root = JointData::new(JointVariant::Revolute(RevoluteJoint::new(
    vector3(0.0, 0.0, 0.0),
    (0.0, 1.0, 0.0),
    0.0,
)));
let mut arm = Armature::new(root);
arm.add_child(0, 1, JointData::new(JointVariant::Revolute(RevoluteJoint::new(
    vector3(1.0, 0.0, 0.0),
    (0.0, 1.0, 0.0),
    0.0,
))));
arm.update_kinematics();
let ee = arm.end_effector_position(1);
```

## Build, test, bench

```bash
cd kinematics
cargo build
cargo test
cargo bench
cargo run --example forward_kinematics
cargo run --example jacobian_ik
```

## WASM

Build for WebAssembly:

```bash
cd kinematics
wasm-pack build --target web --features wasm
```

Use from JavaScript:

```js
import init, { WasmArmature } from './pkg/kinematics.js';

await init();
const arm = new WasmArmature(4, 1.0);  // 4 links, length 1
arm.updateKinematics();
const pos = arm.getEndEffectorPosition(3);  // Float32Array [x, y, z]
const err = arm.solveJacobianIk(3, 2.5, 0.5, 0, 30);  // target, max iters
```

## References

- [IK Guide](https://theorangeduck.com/page/simple-two-joint)
- Rodolphe Vaillant, CCD-based IK
