# Dev tools and quality of life

Quick reference for development setup, CI, and optional tooling.

## CI and coverage

- **CI**: `.github/workflows/ci.yml` runs on push/PR to `main`/`master`: typos (spelling), format check, Clippy, tests, `cargo audit`, WASM check (`cargo check --target wasm32-unknown-unknown --features wasm`), and coverage (tarpaulin).
- **Coverage (CI)**: The coverage job uses the same command as `just coverage-xml` (tarpaulin with `--features parallel`, Cobertura XML and HTML in `mathlib/coverage/`). The parallel backend and related code paths are included. Artifact: `mathlib/coverage/`. Optional: add `CODECOV_TOKEN` in repo secrets to upload to Codecov for badges and PR comments.
- **Local coverage**: From repo root run `just coverage` (HTML + stdout) or `just coverage-xml` (same as CI, with Xml). Open `mathlib/coverage/tarpaulin-report.html` for the report. A pre-commit hook runs coverage at the **push** stage (`pre-commit run --hook-stage push`) so commits stay fast.

## WASM demo

- **Build**: From repo root, `just wasm-build` (builds pkg and copies into `mathlib/wasm-demo/pkg/`).
- **Serve**: From repo root, `just wasm-serve` (or `just demo` to build then serve). Open **/wasm-demo/** (use the URL shown by the server).
- **GitHub Pages**: The wasm-demo is built in CI and deployed to GitHub Pages on push to `main`/`master` (workflow: [.github/workflows/pages.yml](../.github/workflows/pages.yml)). Enable **Settings → Pages → GitHub Actions** to publish the live demo at your repo's Pages URL.
- **Details**: See [mathlib/wasm-demo/README.md](../mathlib/wasm-demo/README.md), [wasm.md](wasm.md), and the [WASM and browser demo](DOCS.md#wasm-and-browser-demo) section in DOCS.md.

## Running by domain

Tests and benchmarks are grouped by domain. From `mathlib/`:

```bash
# Run tests for one domain
cargo test --test linear
cargo test --test ml
cargo test --test cg
cargo test --test optimisation
cargo test --test graph
cargo test --test tree
cargo test --test transforms
# etc.

# Run benchmarks (filter by group name)
cargo bench -- benchmarks -- linear
cargo bench -- benchmarks -- transforms
```

See [domains.md](domains.md) for the full list of test/bench domains.

## Agents and MCP (Cursor)

- **Sequential Thinking**: Use for complex, multi-step reasoning (e.g. refactors, design decisions).
- **Context7**: Use for up-to-date documentation and examples for dependencies (e.g. Rust crates).

See [AGENTS.md](../AGENTS.md) for the project summary and module map.

## Devcontainer

- **Location**: `.devcontainer/devcontainer.json` — Rust image (Debian Bookworm), rust-analyzer, and initial `cargo build` in `mathlib/`.
- **Use**: Open the repo in VS Code or GitHub Codespaces and choose “Reopen in Container” (or use the Codespaces “Open in …” flow).

## Pre-commit

- **Config**: `.pre-commit-config.yaml` — hooks: trailing whitespace, end-of-file, YAML check, typos (spelling), `cargo fmt`, `cargo clippy` (run in `mathlib/`).
- **Setup**: `pip install pre-commit && pre-commit install`
- **Run all**: `pre-commit run --all-files`
  On Windows, the Rust hooks use `bash` (Git Bash or WSL).

## Editor and style

- **Rust**: `cargo fmt` and `cargo clippy` in `mathlib/` (see [CONTRIBUTING.md](../CONTRIBUTING.md)).
- **Spelling**: `.typos.toml` configures [typos](https://github.com/crate-ci/typos); run `typos` from repo root if installed.
- **EditorConfig**: `.editorconfig` enforces basic indentation and line endings across the repo.

## GitHub

- **Dependabot**: `.github/dependabot.yml` — weekly Cargo and GitHub Actions updates.
- **Templates**: Issue and PR templates under `.github/`.
