#!/bin/bash
set -e

# Install just
mkdir -p "$HOME/bin"
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to "$HOME/bin"
echo 'export PATH="$HOME/bin:$PATH"' >> "$HOME/.bashrc"
export PATH="$HOME/bin:$PATH"

# System packages (binaryen for wasm-opt)
sudo apt-get update && sudo apt-get upgrade -y
sudo apt-get install -y wget curl binaryen

# Rust toolchain and WASM target
rustup toolchain install stable
rustup target add wasm32-unknown-unknown --toolchain stable

# Build mathlib (validates setup)
cd mathlib && cargo build && cd ..

# Cargo tools for WASM
cargo install wasm-bindgen-cli
cargo install wasm-pack

# E2E: install Playwright browsers
cd e2e && npm install && npx playwright install chromium && cd ..
