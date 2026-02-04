# Mathlib build recipes. Default: sequential. Use build-parallel or build-simd for acceleration.
# Run from repo root: just build, just test, just build-simd, etc.

[private]
_set_cargo := "cd mathlib &&"

# Default: build sequential (no extra features)
build:
    {{_set_cargo}} cargo build

build-release:
    {{_set_cargo}} cargo build --release

# Update Cargo.lock to latest compatible dependencies
update:
    {{_set_cargo}} cargo update

# Remove build artifacts (target/)
clean:
    {{_set_cargo}} cargo clean

# Build with parallel (rayon) backend
build-parallel:
    {{_set_cargo}} cargo build --features parallel

build-parallel-release:
    {{_set_cargo}} cargo build --release --features parallel

# Build with SIMD (wide) backend
build-simd:
    {{_set_cargo}} cargo build --features simd

build-simd-release:
    {{_set_cargo}} cargo build --release --features simd

# Build with both parallel and SIMD (full = parallel + simd)
build-full:
    {{_set_cargo}} cargo build --features full

build-full-release:
    {{_set_cargo}} cargo build --release --features full

# Test targets
test:
    {{_set_cargo}} cargo test

test-parallel:
    {{_set_cargo}} cargo test --features parallel

test-simd:
    {{_set_cargo}} cargo test --features simd

test-full:
    {{_set_cargo}} cargo test --features full

# Benchmark targets (cargo bench skips unit tests by design; use bench-check to run tests first)
bench:
    {{_set_cargo}} cargo bench

# Run unit tests then benchmarks so tests are not skipped
bench-check:
    {{_set_cargo}} cargo test && {{_set_cargo}} cargo bench

bench-parallel:
    {{_set_cargo}} cargo bench --features parallel

bench-simd:
    {{_set_cargo}} cargo bench --features simd

bench-full:
    {{_set_cargo}} cargo bench --features full

bench-gpu:
    {{_set_cargo}} cargo bench --features gpu --bench gpu

# Coverage (tarpaulin; CI uses same as coverage-xml)
coverage:
    {{_set_cargo}} cargo tarpaulin --features parallel --out Html --out Stdout --output-dir coverage --timeout 300

coverage-xml:
    {{_set_cargo}} cargo tarpaulin --features parallel --out Xml --out Html --output-dir coverage --timeout 300

# Run coverage then open HTML report
coverage-open: coverage
    -[windows] start mathlib/coverage/tarpaulin-report.html
    -[macos] open mathlib/coverage/tarpaulin-report.html
    -[linux] xdg-open mathlib/coverage/tarpaulin-report.html

# Check (no build artifacts)
check:
    {{_set_cargo}} cargo check

check-parallel:
    {{_set_cargo}} cargo check --features parallel

check-simd:
    {{_set_cargo}} cargo check --features simd

# WASM: demo is documented in docs/DOCS.md (WASM and browser demo) and mathlib/wasm-demo/README.md
# WASM targets (target wasm32-unknown-unknown; use --features wasm or wasm,simd; parallel not supported)
build-wasm:
    {{_set_cargo}} cargo build --target wasm32-unknown-unknown --features wasm

build-wasm-release:
    {{_set_cargo}} cargo build --release --target wasm32-unknown-unknown --features wasm

build-wasm-simd:
    {{_set_cargo}} cargo build --target wasm32-unknown-unknown --features "wasm simd"

check-wasm:
    {{_set_cargo}} cargo check --target wasm32-unknown-unknown --features wasm

test-wasm:
    {{_set_cargo}} cargo test --target wasm32-unknown-unknown --features wasm

doc-wasm:
    {{_set_cargo}} cargo doc --target wasm32-unknown-unknown --features wasm --no-deps

clippy-wasm:
    {{_set_cargo}} cargo clippy --target wasm32-unknown-unknown --features wasm

# WASM: build pkg/ for web, copy into wasm-demo/pkg, then serve (demo at /wasm-demo/)
# wasm-pack uses release profile + wasm-opt -Oz (see Cargo.toml). If wasm-pack fails, use wasm-build-manual
wasm-build:
    {{_set_cargo}} wasm-pack build --target web --features wasm
    {{_set_cargo}} cp -r pkg wasm-demo/

# WASM pkg/ with GPU support (initGpuAsync, gpuAvailable in demo). GPU feature uses wgpu 26 for rustc compatibility; wgpu 28 requires rustc 1.92+.
wasm-build-gpu:
    {{_set_cargo}} wasm-pack build --target web --features "wasm gpu"
    {{_set_cargo}} cp -r pkg wasm-demo/

# WASM pkg/ without wasm-pack (cargo + wasm-bindgen), then copy into wasm-demo
wasm-build-manual:
    {{_set_cargo}} cargo build --release --target wasm32-unknown-unknown --features wasm
    {{_set_cargo}} wasm-bindgen target/wasm32-unknown-unknown/release/mathlib.wasm --out-dir pkg --target web --out-name mathlib
    {{_set_cargo}} cp -r pkg wasm-demo/
    {{_set_cargo}} cp -r  wasm-demo/

wasm-serve:
    # Run 'just wasm-build' first. Then open /wasm-demo/ (use the URL shown by the server)
    {{_set_cargo}} npx serve .

# Build WASM pkg, copy to wasm-demo, then serve (open /wasm-demo/)
demo: wasm-build wasm-serve

# Build WASM pkg with GPU, copy to wasm-demo, then serve (open /wasm-demo/)
demo-gpu: wasm-build-gpu wasm-serve

# Run pre-commit hooks on all files (from repo root)
pre-commit:
    pre-commit run --all-files

# Lint / format
clippy:
    {{_set_cargo}} cargo clippy

fmt:
    {{_set_cargo}} cargo fmt -- --check

fmt-fix:
    {{_set_cargo}} cargo fmt

lint: fmt clippy test
