# Mathlib build recipes. Default: sequential. Use build-parallel or build-simd for acceleration.
# Run from repo root: just build, just test, just build-simd, etc.

[private]
_set_cargo := "cd mathlib &&"

# Default: build sequential (no extra features)
build:
    {{_set_cargo}} cargo build

build-release:
    {{_set_cargo}} cargo build --release

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

# Check (no build artifacts)
check:
    {{_set_cargo}} cargo check

check-parallel:
    {{_set_cargo}} cargo check --features parallel

check-simd:
    {{_set_cargo}} cargo check --features simd

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
# If wasm-pack fails (cargo --artifact-dir unstable), use wasm-build-manual
wasm-build:
    {{_set_cargo}} wasm-pack build --target web --features wasm
    {{_set_cargo}} cp -r pkg wasm-demo/

# WASM pkg/ without wasm-pack (cargo + wasm-bindgen), then copy into wasm-demo
wasm-build-manual:
    {{_set_cargo}} cargo build --release --target wasm32-unknown-unknown --features wasm
    {{_set_cargo}} wasm-bindgen target/wasm32-unknown-unknown/release/mathlib.wasm --out-dir pkg --target web --out-name mathlib
    {{_set_cargo}} cp -r pkg wasm-demo/

wasm-serve:
    # Run 'just wasm-build' first. Then open http://localhost:3000/wasm-demo/
    {{_set_cargo}} npx serve .

# Lint / format
clippy:
    {{_set_cargo}} cargo clippy

fmt:
    {{_set_cargo}} cargo fmt -- --check

fmt-fix:
    {{_set_cargo}} cargo fmt

lint: fmt clippy test
