# kinematics — Agent guide

Crate for forward and inverse kinematics. Uses mathlib Tree, Matrix4f, Vector3f, SVD, rotation helpers.

## Layout

| Path | Role |
|------|------|
| `src/joints/` | Joint types: fixed, revolute, prismatic, spherical; 2D: fixed2d, revolute2d, prismatic2d |
| `src/armature.rs` | Armature (Tree of JointData), update_kinematics, pack/unpack |
| `src/ik/` | IK solvers: Jacobian (SVD), FABRIK, Hessian (exact Hessian Newton), Halley/QuIK |

## IK solvers

- **FabrikIk**: Position-only; revolute and spherical chains (2D/3D); works on WASM; few iterations, low cost. Use for position goals on any chain type. Paper: Aristidou & Lasenby, *Graphical Models* 73(5), 2011.
- **JacobianIk**: Position-only; SVD pseudoinverse + line search. For spherical joints, packed state uses scaled axis; the solver converts world-frame angular increments to parent-local before applying. Use when FABRIK or Halley is not suitable.
- **HessianIk**: Position-only; exact Hessian Newton (Erleben & Andrews, MIG 2017). Adaptive regularization for indefinite Hessians; gradient-step fallback; works on WASM. Use when fewer iterations and higher accuracy are desired.
- **HalleyIk**: 6D pose solver (position + orientation) inspired by QuIK (Halley's method). Best for serial chains; target as full `Matrix4f`; works on wasm32. Use for pose goals with good initial guess. Paper: Lloyd et al., IEEE T-RO 2022.

## Joints

- **FixedJoint**: 0 DOF
- **RevoluteJoint**: 1 DOF, rotation about axis
- **PrismaticJoint**: 1 DOF, translation along axis
- **SphericalJoint**: 3 DOF, quaternion rotation; pack/unpack use scaled axis (tangent space) for Jacobian IK
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
