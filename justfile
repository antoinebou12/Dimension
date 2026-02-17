# Mathlib build recipes. Default: sequential. Use build-parallel or build-simd for acceleration.
# Run from repo root: just build, just test, just build-simd, etc.
# Rust toolchain: rust-toolchain.toml (stable). Explicit runner so default rustc isn't used (e.g. edition 2024 needs 1.85+).
#
# Caching: In the dev container, CARGO_HOME and CARGO_TARGET_DIR are persisted via Docker volumes
# (dimension-cargo, dimension-rust-target). Outside the container, set CARGO_TARGET_DIR to a
# directory to reuse build artifacts (e.g. CARGO_TARGET_DIR=./target-cache just build).
cache-info:
    # Show Rust cache locations (set in dev container for volume persistence)
    @echo "CARGO_HOME: {{env('CARGO_HOME', '~/.cargo (default)')}}"
    @echo "CARGO_TARGET_DIR: {{env('CARGO_TARGET_DIR', '<per-crate target/> (default)')}}"

[private]
_rust := "rustup run stable -- "
_set_cargo := "cd mathlib && " + _rust
_set_render := "cd render && " + _rust
_set_parse := "cd parse && " + _rust
_set_collision := "cd collision && " + _rust
_set_network := "cd network && " + _rust
_set_kinematics_demo := "cd kinematics/demo && " + _rust
_set_physics_demo := "cd physics/demo && " + _rust
_set_render_demo := "cd render/demo && " + _rust
_set_geometry := "cd geometry && " + _rust
_set_geometry_demo := "cd geometry/demo && " + _rust

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

# Build with parallel backend
build-parallel:
    {{_set_cargo}} cargo build --features parallel

build-parallel-release:
    {{_set_cargo}} cargo build --release --features parallel

# Build with SIMD (wide) backend
build-simd:
    {{_set_cargo}} cargo build --features simd

build-simd-release:
    {{_set_cargo}} cargo build --release --features simd

# Build with SIMD + GPU (always both)
build-simd-gpu:
    {{_set_cargo}} cargo build --features "simd gpu"

build-simd-gpu-release:
    {{_set_cargo}} cargo build --release --features "simd gpu"

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

test-simd-gpu:
    {{_set_cargo}} cargo test --features "simd gpu"

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
    {{_set_cargo}} cargo bench --features "simd gpu" --bench gpu

# Bench GPU (same code paths used by wasm when built with gpu)
bench-wasm-gpu:
    {{_set_cargo}} cargo bench --features "wasm simd gpu" --bench gpu

# Coverage (tarpaulin; CI uses same as coverage-xml)
coverage:
    {{_set_cargo}} cargo tarpaulin --features "parallel genetic" --out Html --out Stdout --output-dir coverage --timeout 300

coverage-xml:
    {{_set_cargo}} cargo tarpaulin --features "parallel genetic" --out Xml --out Html --output-dir coverage --timeout 300

# Run coverage then open HTML report
coverage-open: coverage
    -[windows] start mathlib/coverage/tarpaulin-report.html
    -[macos] open mathlib/coverage/tarpaulin-report.html
    -[linux] xdg-open mathlib/coverage/tarpaulin-report.html

# Private: build demo WASM (cargo + wasm-bindgen). Usage: just _build-demo-wasm <demo_dir> <example_name>
[private]
_build-demo-wasm demo_dir example_name:
    cd {{ demo_dir }} && {{ _rust }} cargo build --release --target wasm32-unknown-unknown --example {{ example_name }}
    cd {{ demo_dir }} && wasm-bindgen ../../target/wasm32-unknown-unknown/release/examples/{{ example_name }}.wasm --out-dir wasm-demo/pkg --target web --out-name {{ example_name }}

# Private: serve demo dir. Usage: just _serve-demo <demo_dir>
[private]
_serve-demo demo_dir:
    bash -c 'cd {{ demo_dir }} && npx serve --no-clipboard .'

# Private: copy wasm-demo/pkg to demo pkg/. Usage: just _copy-demo-pkg <demo_dir>
[private]
_copy-demo-pkg demo_dir:
    mkdir -p {{ demo_dir }}/pkg
    cp -r {{ demo_dir }}/wasm-demo/pkg/* {{ demo_dir }}/pkg/

# Crate demos: native (winit) = build-*-demo, run-*-demo (alias demo-kinematics); WASM = build-*-demo-wasm, wasm-*-demo (alias demo-kinematics-wasm); copy pkg = demo-*-build (kinematics only).

# Kinematics demo
build-kinematics-demo: build-kinematics build-render
    {{_set_kinematics_demo}} cargo build

run-kinematics-demo: build-kinematics-demo
    {{_set_kinematics_demo}} cargo run --example kinematics_native

build-kinematics-demo-wasm: build-wasm _wasm-render-deps
    just _build-demo-wasm kinematics/demo kinematics_wasm

# Kinematics WASM with SIMD (mathlib/kinematics simd; can improve Hessian/cross and frame rate)
build-kinematics-demo-wasm-simd: build-wasm _wasm-render-deps
    cd kinematics/demo && {{ _rust }} cargo build --release --target wasm32-unknown-unknown --example kinematics_wasm --features simd
    cd kinematics/demo && wasm-bindgen ../../target/wasm32-unknown-unknown/release/examples/kinematics_wasm.wasm --out-dir wasm-demo/pkg --target web --out-name kinematics_wasm

    cd kinematics/demo && wasm-bindgen ../../target/wasm32-unknown-unknown/release/examples/kinematics_wasm.wasm --out-dir wasm-demo/pkg --target web --out-name kinematics_wasm

wasm-kinematics-demo: build-kinematics-demo-wasm
    just _serve-demo kinematics/demo

demo-kinematics-build: build-kinematics-demo-wasm
    just _copy-demo-pkg kinematics/demo

demo-kinematics: run-kinematics-demo

demo-kinematics-wasm: wasm-kinematics-demo

# Physics crate
build-physics: build
    cd physics && {{ _rust }} cargo build

test-physics: build-physics
    cd physics && {{ _rust }} cargo test

bench-physics: build-physics
    cd physics && {{ _rust }} cargo bench

build-physics-simd: build-physics
    cd physics && {{ _rust }} cargo build --features simd

build-physics-parallel: build-physics
    cd physics && {{ _rust }} cargo build --features parallel

build-physics-full: build-physics
    cd physics && {{ _rust }} cargo build --features "simd parallel"

test-physics-serde: build-physics
    cd physics && {{ _rust }} cargo test --features serde --test serialization

# Physics demo
build-physics-demo: build-physics build-render
    {{_set_physics_demo}} cargo build

run-physics-demo: build-physics-demo
    {{_set_physics_demo}} cargo run --example physics_native

build-physics-demo-wasm: build-wasm _wasm-render-deps
    just _build-demo-wasm physics/demo physics_wasm

wasm-physics-demo: build-physics-demo-wasm
    just _serve-demo physics/demo

# Geometry crate
build-geometry: build
    {{_set_geometry}} cargo build

test-geometry: build-geometry
    {{_set_geometry}} cargo test

bench-geometry: build-geometry
    {{_set_geometry}} cargo bench

build-geometry-wasm: build-wasm
    {{_set_geometry}} cargo build --target wasm32-unknown-unknown --features wasm

# Geometry demo
build-geometry-demo: build-geometry build-render
    {{_set_geometry_demo}} cargo build

run-geometry-demo: build-geometry-demo
    {{_set_geometry_demo}} cargo run --example geometry_native

build-geometry-demo-wasm: build-geometry-wasm _wasm-render-deps
    just _build-demo-wasm geometry/demo geometry_wasm

wasm-geometry-demo: build-geometry-demo-wasm
    just _serve-demo geometry/demo

# Kinematics crate
build-kinematics: build
    cd kinematics && {{ _rust }} cargo build

test-kinematics: build-kinematics
    cd kinematics && {{ _rust }} cargo test

# Kinematics WASM (requires wasm-pack: cargo install wasm-pack)
build-kinematics-wasm: _require_wasm_pack
    cd kinematics && {{ _rust }} wasm-pack build --target web --features wasm

# Check (no build artifacts)
check:
    {{_set_cargo}} cargo check

check-parallel:
    {{_set_cargo}} cargo check --features parallel

check-simd:
    {{_set_cargo}} cargo check --features simd

check-simd-gpu:
    {{_set_cargo}} cargo check --features "simd gpu"

# WASM: demo is documented in docs/DOCS.md (WASM and browser demo) and mathlib/demo/wasm-demo/README.md
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

# Check WASM+SIMD+GPU (syntax only)
check-wasm-gpu:
    {{_set_cargo}} cargo check --target wasm32-unknown-unknown --features "wasm simd gpu"

# Test WASM+SIMD+GPU (runs on native host; tests the Rust API used by bindings)
test-wasm-gpu:
    {{_set_cargo}} cargo test --features "wasm simd gpu" wasm

# WASM: build pkg/ for web, copy into wasm-demo/pkg, then serve (demo at /wasm-demo/)
# wasm-pack uses release profile + wasm-opt -Oz (see Cargo.toml). If wasm-pack fails, use wasm-build-manual
[private]
_require_wasm_pack:
    command -v wasm-pack >/dev/null 2>&1 || (echo "error: wasm-pack not found. Install with: cargo install wasm-pack" && exit 1)

wasm-build: _require_wasm_pack
    cd mathlib && rm -rf pkg
    {{_set_cargo}} wasm-pack build --target web --features wasm
    {{_set_cargo}} cp -r pkg demo/wasm-demo/

# WASM pkg/ with SIMD+GPU (initGpuAsync, gpuAvailable in demo). Always build simd and gpu together.
wasm-build-gpu: _require_wasm_pack
    cd mathlib && rm -rf pkg
    {{_set_cargo}} wasm-pack build --target web --features "wasm simd gpu"
    {{_set_cargo}} cp -r pkg demo/wasm-demo/

# WASM pkg/ without wasm-pack (cargo + wasm-bindgen), then copy into demo/wasm-demo
wasm-build-manual:
    {{_set_cargo}} cargo build --release --target wasm32-unknown-unknown --features wasm
    {{_set_cargo}} wasm-bindgen target/wasm32-unknown-unknown/release/mathlib.wasm --out-dir pkg --target web --out-name mathlib
    {{_set_cargo}} cp -r pkg demo/wasm-demo/

# On Windows, use PowerShell so npx is found (Node's Windows PATH); on Unix/WSL use bash so npx is on PATH (e.g. nvm).
[windows]
wasm-serve:
    # Run 'just wasm-build' first. Then open /wasm-demo/
    powershell -NoProfile -Command "Set-Location mathlib/demo; npx serve ."

[unix]
wasm-serve:
    # Run 'just wasm-build' first. Then open /wasm-demo/
    bash -c 'cd mathlib/demo && npx serve --no-clipboard .'

# Mathlib WASM: build + serve (open /wasm-demo/). Use "just wasm gpu" for GPU build
wasm gpu='':
    if [ -n "{{ gpu }}" ]; then just wasm-build-gpu; else just wasm-build; fi
    just wasm-serve

demo: wasm
demo-gpu:
    just wasm gpu

# Download ONNX models for neural crate (text/image/graph etc.) into neural/models/
download-neural-models:
    cargo run -p neural --bin download_models -- --all

# Render crate (builds mathlib first)
build-render: build
    {{_set_render}} cargo build

run-render: build-render
    {{_set_render_demo}} cargo run --example render_native

# SDL3 render demo (requires SDL3 installed, e.g. vcpkg install sdl3:x64-windows)
run-render-sdl3:
    {{_set_render_demo}} cargo run --example sdl3_quad --features sdl3

# 2D AABB/Circle collision demo (orthographic, Lissajous + fixed circle, AABB outlines)
run-render-aabb2d: build-render
    {{_set_render_demo}} cargo run --example aabb2d_native

build-render-release: build-release
    {{_set_render}} cargo build --release

build-render-wasm: build-wasm
    {{_set_render_demo}} cargo build --target wasm32-unknown-unknown --example render_wasm

# Check wasm-bindgen on PATH (required for _wasm-render-build). Install: cargo install wasm-bindgen-cli
[unix]
_wasm-render-deps:
    command -v wasm-bindgen || (echo "wasm-bindgen not found. Install with: cargo install wasm-bindgen-cli" && exit 1)

[windows]
_wasm-render-deps:
    powershell -NoProfile -Command "if (-not (Get-Command wasm-bindgen -ErrorAction SilentlyContinue)) { Write-Error 'wasm-bindgen not found. Install with: cargo install wasm-bindgen-cli'; exit 1 }"

# Render WASM web: build render-demo example + wasm-bindgen. pkg/ in render/demo/wasm-demo/pkg
# Requires wasm-bindgen on PATH: cargo install wasm-bindgen-cli
_wasm-render-build: build-wasm _wasm-render-deps
    {{_set_render_demo}} cargo build --release --target wasm32-unknown-unknown --example render_wasm
    {{_set_render_demo}} wasm-bindgen ../../target/wasm32-unknown-unknown/release/examples/render_wasm.wasm --out-dir wasm-demo/pkg --target web --out-name render_wasm

# Render WASM with SIMD (can improve frame rate). Use for website-build-simd or local serve.
_wasm-render-build-simd: build-wasm-simd _wasm-render-deps
    {{_set_render_demo}} cargo build --release --target wasm32-unknown-unknown --example render_wasm --features simd
    {{_set_render_demo}} wasm-bindgen ../../target/wasm32-unknown-unknown/release/examples/render_wasm.wasm --out-dir wasm-demo/pkg --target web --out-name render_wasm

# Serve render wasm-demo. Use "just wasm-render-serve" to serve only, or "just render-wasm" to build + serve.
[windows]
_wasm-render-serve:
    powershell -NoProfile -Command "Set-Location render/demo; npx serve ."

[unix]
_wasm-render-serve:
    bash -c 'cd render/demo && npx serve --no-clipboard .'

# Public alias: build only (no serve). Use wasm-render-serve to serve, or render-wasm to build + serve.
wasm-render-build: _wasm-render-build

# Build render WASM with SIMD (then use wasm-render-serve to serve). Can improve frame rate.
wasm-render-build-simd: _wasm-render-build-simd

# Public alias: serve render wasm-demo (run after building with render-wasm or wasm-render-build).
wasm-render-serve: _wasm-render-serve

# Render WASM: build + serve (open http://localhost:3000/wasm-demo/)
render-wasm: _wasm-render-build _wasm-render-serve

# Alias for discoverability (same as render-wasm).
wasm-render: render-wasm

demo-render: render-wasm

# Serve render wasm-demo in background. Run wasm-render-build or demo-render first. Use demo-render-stop to stop.
[unix]
demo-render-bg:
    cd render/demo && nohup npx serve --no-clipboard . > /dev/null 2>&1 & echo "$$!" > .serve.pid && echo "Serving at http://localhost:3000/wasm-demo/. PID: $$(cat .serve.pid). Run 'just demo-render-stop' to stop."

[windows]
demo-render-bg:
    powershell -NoProfile -Command "Set-Location render/demo; $$p = Start-Process -NoNewWindow -PassThru npx -ArgumentList 'serve','.'; $$p.Id | Out-File -FilePath .serve.pid -Encoding ascii; Write-Host ('Serving at http://localhost:3000/wasm-demo/. PID: ' + $$p.Id + '. Run just demo-render-stop to stop.')"

# Stop the render demo server (when run with demo-render-bg).
[unix]
demo-render-stop:
    -kill $$(cat render/demo/.serve.pid 2>/dev/null) 2>/dev/null; rm -f render/demo/.serve.pid
    @echo "Stopped (or server was not running)."

[windows]
demo-render-stop:
    powershell -NoProfile -Command "if (Test-Path render/demo/.serve.pid) { $$pid = Get-Content render/demo/.serve.pid; Stop-Process -Id $$pid -ErrorAction SilentlyContinue; Remove-Item render/demo/.serve.pid }; Write-Host 'Stopped (or server was not running).'"

# Unified website: all WASM demos in website/ (hub at /, demos at /mathlib/, /render/, etc.)
website-build: wasm-build _wasm-render-build build-kinematics-demo-wasm build-physics-demo-wasm build-geometry-demo-wasm _website-populate

# Website build with GPU-enabled mathlib (initGpuAsync, matmulF32GpuAsync, etc.)
website-build-gpu: wasm-build-gpu _wasm-render-build build-kinematics-demo-wasm build-physics-demo-wasm build-geometry-demo-wasm _website-populate

# Website build with SIMD-enabled render and kinematics demos (can improve frame rate and Hessian path)
website-build-simd: wasm-build _wasm-render-build-simd build-kinematics-demo-wasm-simd build-physics-demo-wasm build-geometry-demo-wasm _website-populate

# Full Pages pipeline locally: clean → build → fix base path → verify (default repo Dimension for local testing).
website-pages repo="Dimension":
    bash ./.github/scripts/prepare-pages-artifact.sh "{{repo}}"

[private]
_website-populate:
    bash ./.github/scripts/website-populate.sh

# Serve website/ locally (hub at /, demos at /mathlib/, /render/, etc.). Port 3000.
[unix]
website-serve:
    bash -c 'cd website && npx serve --no-clipboard .'

[windows]
website-serve:
    powershell -NoProfile -Command "Set-Location website; npx serve ."

# Require Docker on PATH (used by website-docker-*)
[private]
[unix]
_require_docker:
    command -v docker >/dev/null 2>&1 || (echo "Docker not found. Install Docker or run locally: just website-build && just website-serve" && exit 127)

[private]
[windows]
_require_docker:
    powershell -NoProfile -Command "if (-not (Get-Command docker -ErrorAction SilentlyContinue)) { Write-Host 'Docker not found. Install Docker or run locally: just website-build && just website-serve'; exit 127 }"

# Build Docker image (builds WASM inside container). Run from repo root; context is .
website-docker-build: _require_docker
    docker build -f website/Dockerfile -t dimension-wasm-website .

# Run the website container (http://localhost:8080). Requires Docker; without it use: just website-build && just website-serve
website-docker-run: _require_docker
    docker run --rm -p 8080:80 dimension-wasm-website

build-render-wasm-simd: build-wasm-simd
    {{_set_render_demo}} cargo build --target wasm32-unknown-unknown --example render_wasm --features simd

test-render: test
    {{_set_render}} cargo test

bench-render: bench
    {{_set_render}} cargo bench

# E2E (Playwright): build unified website (all demos), serve from website/, run e2e tests
[private]
_e2e-install:
    cd e2e && npm install && npx playwright install chromium

e2e: website-build _e2e-install
    cd e2e && npx playwright test

# E2E with UI (debug)
e2e-ui: website-build _e2e-install
    cd e2e && npx playwright test --ui

# E2E kinematics/demo only (still needs full website so server has /kinematics/)
e2e-kinematics: website-build _e2e-install
    cd e2e && npx playwright test --project=kinematics

# Run GitHub Actions locally with act (requires Docker). If act not found, run: just act-install
# Unix/WSL: uses ./bin/act when present (e.g. after install script in repo root), else act from PATH.
[unix]
act:
    (test -f ./bin/act && ./bin/act || act)

[windows]
act:
    act

# Print install instructions for act (Linux/WSL: curl script; Windows: winget/scoop)
act-install:
    @echo "Install act (https://github.com/nektos/act) and ensure Docker is running."
    @echo ""
    @echo "Linux / WSL:"
    @echo "  curl -s https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash"
    @echo "  # Installs to ./bin/act (just act will use it). Or add bin/ to PATH."
    @echo ""
    @echo "Windows (PowerShell): winget install nektos.act   or   scoop install act"

# List workflows and jobs (act -l)
[unix]
act-list:
    (test -f ./bin/act && ./bin/act || act) -l

[windows]
act-list:
    act -l

# Run only the main CI job (format, clippy, test, examples)
[unix]
act-ci:
    (test -f ./bin/act && ./bin/act || act) -j ci

[windows]
act-ci:
    act -j ci

# Run CI workflow in dry-run (show what would run)
[unix]
act-dry:
    (test -f ./bin/act && ./bin/act || act) -n

[windows]
act-dry:
    act -n

# Run spelling check (typos). Install: cargo install typos-cli. Config: .typos.toml
typos:
    typos --config .typos.toml

# Run pre-commit hooks on all files (from repo root)
pre-commit:
    pre-commit run --all-files

# Parse crate
build-parse: build
    {{_set_parse}} cargo build

test-parse:
    {{_set_parse}} cargo test --features "json,bjson,toon,xml,obj,mtl,bvh,gltf,ply,image,archive"

bench-parse:
    {{_set_parse}} cargo bench --features "json,bjson,toon,xml,obj,mtl,bvh,gltf,ply,image,archive"

check-parse:
    {{_set_parse}} cargo check

build-parse-wasm: build-wasm
    {{_set_parse}} cargo build --target wasm32-unknown-unknown --features "json,bjson,toon,xml,obj,wasm"

# Collision crate
build-collision:
    {{_set_collision}} cargo build

test-collision:
    {{_set_collision}} cargo test

bench-collision:
    {{_set_collision}} cargo bench

build-collision-simd:
    {{_set_collision}} cargo build --features simd

build-collision-parallel:
    {{_set_collision}} cargo build --features parallel

# Network crate
build-network:
    {{_set_network}} cargo build

build-network-full:
    {{_set_network}} cargo build --features full

test-network:
    {{_set_network}} cargo test

bench-network:
    {{_set_network}} cargo bench

run-network-server:
    {{_set_network}} cargo run --example server --features server

run-network-client:
    {{_set_network}} cargo run --example client --features client

# Lint / format
clippy:
    {{_set_cargo}} cargo clippy

fmt:
    {{_set_cargo}} cargo fmt -- --check

fmt-fix:
    {{_set_cargo}} cargo fmt

lint: fmt clippy test
