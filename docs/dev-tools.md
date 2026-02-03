# Dev tools and quality of life

Quick reference for development setup, CI, and optional tooling.

## CI and coverage

- **CI**: `.github/workflows/ci.yml` runs on push/PR to `main`/`master`: typos (spelling), format check, Clippy, tests, `cargo audit`, and coverage (tarpaulin).
- **Coverage**: The coverage job produces Cobertura XML and HTML in `mathlib/coverage/` (artifact). Optional: add `CODECOV_TOKEN` in repo secrets to upload to Codecov for badges and PR comments.
- **Local coverage**: From `mathlib/`, run `cargo tarpaulin --out Html --out Stdout --output-dir coverage`. Output is in `coverage/` (gitignored).

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
