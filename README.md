# Dimension

Dimension is the repository for **mathlib**, a Rust linear algebra library providing dense and sparse matrices, vectors, SVD decomposition, and 3D math helpers.

All code lives in the **mathlib/** crate. Build, test, benchmark, and generate docs from that folder:

```bash
cd mathlib && cargo build
cd mathlib && cargo test
cd mathlib && cargo bench
cd mathlib && cargo doc --open
```

**Linting and coverage (optional):** From repo root run `just clippy` and `just coverage` (or see [docs/dev-tools.md](docs/dev-tools.md)). Open `mathlib/coverage/tarpaulin-report.html` for the coverage report.

## Documentation

- **[docs/DOCS.md](docs/DOCS.md)** — Architecture, main types, operators, usage examples, and performance notes.
- **[docs/claude.md](docs/claude.md)** — Project context for Claude and other AI assistants.
- **[AGENTS.md](AGENTS.md)** — Structured project and API summary for LLMs.
- **API docs** — From the repo root: `cd mathlib && cargo doc --open` (Rustdoc for the `mathlib` crate).

## Security

To report a vulnerability, see [SECURITY.md](SECURITY.md). Do not open a public issue for security issues.

## License

See [LICENSE](LICENSE).
