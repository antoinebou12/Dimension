# Dimension — Project context for Claude and other AI assistants

**Dimension** is the repository for **mathlib**, a Rust linear algebra library. All code lives in the **mathlib** crate. It provides dense and sparse matrices, vectors, SVD and other decompositions, 3D math, clustering, distance metrics, PCA, and camera/projection helpers.

## Layout

| Path | Role |
|------|------|
| `mathlib/src/` | Crate source; public API in `lib.rs`. |
| `mathlib/tests/` | Integration tests. |
| `mathlib/benches/` | Criterion benchmarks. |
| `mathlib/examples/` | Example binaries. |

## Commands (from repo root)

```bash
cd mathlib && cargo build
cd mathlib && cargo test
cd mathlib && cargo doc --open
```

Optional: use [just](https://github.com/casey/just) from the repo root — e.g. `just build`, `just test`, `just bench` (see [justfile](../justfile)).

## Conventions

- Follow [CONTRIBUTING.md](../CONTRIBUTING.md): run `cargo fmt`, `cargo clippy`, and `cargo test` inside `mathlib/` before submitting changes.
- Add or update tests when changing behavior; update docs (e.g. [docs/DOCS.md](DOCS.md) or doc comments) when changing public APIs.

## Where to read

- **[docs/DOCS.md](DOCS.md)** — Architecture, main types, operators, usage examples.
- **Rustdoc** — Full API: `cd mathlib && cargo doc --open`.
- **[AGENTS.md](../AGENTS.md)** — Structured module map and conventions for LLMs.

## Key files

| File | Role |
|------|------|
| `mathlib/src/lib.rs` | Crate root; re-exports and `prelude`. |
| `mathlib/src/structure/` | Storage (dense/sparse), `MatrixBase`, `SubMatrix`, sparse formats. |
| `mathlib/src/matrix.rs` | `Matrix<T>`; indexing, transpose, block views. |
| `mathlib/src/vector.rs` | `Vector<T>`; dot, norm, resize. |
| `mathlib/src/operators.rs` | `Add`, `Sub`, `Mul` for matrices and vectors. |
| `mathlib/src/solve.rs` | General linear solve; `Cholesky`, `Lu` in `chol.rs`, `lu.rs`. |
| `mathlib/src/svd.rs` | SVD and `svd_econ`. |
