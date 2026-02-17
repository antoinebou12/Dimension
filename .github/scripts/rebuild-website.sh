#!/usr/bin/env bash
# Full sequential rebuild of the website: clean demo output, then build all WASM demos.
# Run from repo root. Ensures no stale demo pages; website/ is repopulated from scratch.
set -e
echo "Cleaning website demo dirs for full rebuild..."
rm -rf website/mathlib website/render website/kinematics website/physics website/geometry website/neural
echo "Building unified website (all WASM demos)..."
just website-build
echo "Rebuild complete."
