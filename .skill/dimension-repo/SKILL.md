---
name: dimension-repo
description: >
  Project context for the Dimension repository (mathlib, render, kinematics, parse).
  Use when editing code in this repo, adding features to mathlib/render/parse/kinematics,
  writing tests or docs, or when the user refers to Dimension, mathlib, or these crates.
---

# Dimension repository

This skill orients the agent when working in the **Dimension** repo. For full layout, domains, and where to add code, read [AGENTS.md](../../AGENTS.md) and [CLAUDE.md](../../CLAUDE.md) at the repo root.

## When to apply

- Editing any crate in this repo: mathlib, render, kinematics, parse
- Adding or changing tests, benchmarks, examples, or documentation
- User mentions Dimension, mathlib, render, kinematics, or parse

## Repo at a glance

| Crate       | Role |
|------------|------|
| **mathlib** | Linear algebra: matrices, vectors, SVD, solvers, 3D math, clustering, graph pathfinding, PCA, simplex, optional WASM/SIMD/parallel |
| **render**  | WASM-first 2D/3D GPU engine (wgpu); uses mathlib for CPU math |
| **kinematics** | Forward/inverse kinematics; joints, armature, Jacobian/FABRIK IK |
| **parse**   | Data and 3D formats: json, gltf, obj, image, archive; uses mathlib types |

## Layout (paths from repo root)

- `mathlib/src/` — crate source; public API in `lib.rs`
- `mathlib/tests/`, `mathlib/benches/`, `mathlib/examples/` — by domain (linear, ml, graph, cg, etc.)
- `render/src/`, `render/tests/`, `render/examples/`
- `kinematics/src/`, `kinematics/tests/`, `kinematics/examples/`
- `parse/src/`, `parse/tests/`

Domain-to-path mapping (where to add code) is in [AGENTS.md — Where to add code](../../AGENTS.md).

## Commands

From repo root (or use `just`; see [justfile](../../justfile)):

- mathlib: `cd mathlib && cargo build`, `cargo test`, `cargo doc --open`
- render: `just build-render`, `just run-render`, `just build-render-wasm`, `just test-render`
- kinematics: `just build-kinematics`, `just test-kinematics`
- parse: `just build-parse`, `just test-parse`
- WASM (mathlib): `just build-wasm`, `just test-wasm` — do **not** enable `parallel` on `wasm32`

## Conventions

- **Style**: `cargo fmt`, `cargo clippy` in the crate you change (e.g. `mathlib/`). See [CONTRIBUTING.md](../../CONTRIBUTING.md).
- **Tests**: Add or update tests when changing behavior; integration tests per domain (e.g. `cargo test --test linear`).
- **Docs**: Document public API; update [docs/DOCS.md](../../docs/DOCS.md) or crate docs when changing main types. Follow Rust API guidelines (project rust-skills: doc-*, api-*, err-*).
- **Result vs panic**: Solvers and decompositions return `Result`; indexing may panic in debug if out of bounds.

## Where to read

| Need | Read |
|------|------|
| Architecture, types, usage | [docs/DOCS.md](../../docs/DOCS.md) |
| Module map, features, errors, where to add code | [AGENTS.md](../../AGENTS.md) |
| Short project summary | [CLAUDE.md](../../CLAUDE.md) |
| Render crate | [docs/render.md](../../docs/render.md) |
| Parse crate | [docs/parse.md](../../docs/parse.md) |
| Kinematics | [kinematics/AGENTS.md](../../kinematics/AGENTS.md) |
| Full API | `cd mathlib && cargo doc --open` (or render/kinematics/parse) |

## Tools

- Use the **Rust skill** (`.cursor/skills/rust-skills` or `.skill/rust-skills` or `/rust-skills`) for Rust style, docs, and API design.
- Use **Context7 MCP** for up-to-date dependency docs when implementing or debugging.

## Complex tasks

- Use sequential thinking for multi-step work; break edits by **domain** (see [docs/domains.md](../../docs/domains.md)).
- Prefer minimal, focused changes; plan before executing.
