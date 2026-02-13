# kinematics — Agent guide

Crate for forward and inverse kinematics. Uses mathlib Tree, Matrix4f, Vector3f, SVD, rotation helpers.

## Layout

| Path | Role |
|------|------|
| `src/joints/` | Joint types: fixed, revolute, prismatic, spherical; 2D: fixed2d, revolute2d, prismatic2d |
| `src/armature.rs` | Armature (Tree of JointData), update_kinematics, pack/unpack |
| `src/ik/` | IK solvers: Jacobian (SVD), FABRIK, Halley/QuIK |

## IK solvers

- **HalleyIk**: 6D pose solver inspired by QuIK (Halley's method). Best for serial chains; requires target as full `Matrix4f`.
- **JacobianIk**: Position-only solver using SVD pseudoinverse + line search.
- **FabrikIk**: FABRIK position-only solver for revolute chains (2D/3D).

## Joints

- **FixedJoint**: 0 DOF
- **RevoluteJoint**: 1 DOF, rotation about axis
- **PrismaticJoint**: 1 DOF, translation along axis
- **SphericalJoint**: 3 DOF, quaternion rotation
- **Fixed2dJoint**: 0 DOF, XY translation only (z = 0)
- **Revolute2dJoint**: 1 DOF, rotation about Z; use in XY plane
- **Prismatic2dJoint**: 1 DOF, slide along axis in XY plane

## Features

- `parallel` — batch update (not on wasm32)
- `simd` — SIMD (works on wasm)
- `wasm` — wasm-bindgen bindings; `WasmArmature` (see `src/wasm.rs`)

## Conventions

- DFS preorder for traversal
- Column-major matrices
- Radians for angles
