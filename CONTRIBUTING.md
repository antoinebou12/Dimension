# Contributing to Dimension

Thanks for your interest in contributing. This project hosts **mathlib**, a Rust linear algebra library.

## Getting started

All code lives in the **mathlib/** crate. From the repo root:

```bash
cd mathlib && cargo build
cd mathlib && cargo test
```

Optional: use [just](https://github.com/casey/just) from the repo root (see `justfile`) for common tasks. For a containerized dev environment, see [.devcontainer](.devcontainer/). For pre-commit hooks (fmt, clippy, typos), see [.pre-commit-config.yaml](.pre-commit-config.yaml) and [docs/dev-tools.md](docs/dev-tools.md).

## Development workflow

1. **Fork and clone** the repository.
2. **Create a branch** for your change (e.g. `fix/svd-edge-case`, `feat/sparse-format`).
3. **Make your changes** in `mathlib/`. Keep formatting and style consistent with the existing codebase.
4. **Run tests and lints:**
   ```bash
   cd mathlib && cargo test
   cd mathlib && cargo clippy
   cd mathlib && cargo fmt -- --check
   ```
5. **Submit a pull request** with a clear description of the change and why it’s needed.

## Code style

* Follow Rust standard formatting: `cargo fmt` in `mathlib/`.
* Respect existing patterns in the crate (e.g. in `mathlib/src/`, `mathlib/tests/`).
* Add or update tests when changing behavior; run benchmarks if touching performance-critical paths.

**Note:** `cargo bench` runs the lib test binary with `--bench`, so unit tests are reported as "ignored" (skipped). To run unit tests and then benchmarks, use `cargo test && cargo bench` or `just bench-check` from the repo root.

## Profiling

Release builds use `debug = true` and `strip = false` so sampling and allocation profilers can attribute time and allocations to code instead of `[unknown]`.

**CPU (sampling)** — Use [samply](https://github.com/mstange/samply) for a cross-platform UI:

* `cargo install samply`
* From `mathlib/`: `cargo build --release`, then e.g. `samply record cargo run --release -F genetic --example cmaes`, or run your benchmark/example under `samply record ./target/release/<binary>`.

**Linux** — If stacks look wrong or missing, build with frame pointers: `RUSTFLAGS="-C force-frame-pointers=yes" cargo build --release`, then run under samply or perf as usual.

**Allocations** — Use [dhat](https://github.com/nnethercote/dhat-rs) (or dhat-viewer) when hunting allocation hot spots: add the crate, instrument the code path, and inspect the generated report.

## Pull requests

* Keep PRs focused; prefer several small PRs over one large one when possible.
* Update docs (e.g. `docs/DOCS.md` or doc comments) if you change public APIs or behavior.
* CI must pass (see `.github/workflows/ci.yml`). For coverage and optional tools (e.g. Codecov, typos), see [docs/dev-tools.md](docs/dev-tools.md).

## Security

If you believe you have found a security vulnerability, do not open a public issue. See [SECURITY.md](SECURITY.md) for how to report it.

## Questions

Open an issue for bugs, feature ideas, or questions. For behavior that violates our community standards, see [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
