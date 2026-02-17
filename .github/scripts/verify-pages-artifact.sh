#!/usr/bin/env bash
# Verify website/ has root index, all demo index.html, and WASM pkg dirs. Exit 1 on failure.
# Run from repo root.
set -e
echo "Checking site root..."
test -f website/index.html || (echo "Missing website/index.html. Ensure it is tracked (!website/index.html in .gitignore) and committed." && exit 1)
echo "Checking demo pages (must exist for Pages to serve /mathlib/, /render/, etc.)..."
for dir in mathlib render kinematics physics geometry neural; do
  if ! test -f "website/$dir/index.html"; then
    echo "Missing website/$dir/index.html — demo will 404. Ensure demo shell files are tracked (see .gitignore exceptions) and committed."
    exit 1
  fi
  echo "  ok website/$dir/index.html"
done
test -f website/neural/embedding/index.html || (echo "Missing website/neural/embedding/index.html" && exit 1)
echo "  ok website/neural/embedding/index.html"
echo "Demo WASM pkgs (required for demos to run):"
for dir in mathlib render kinematics physics geometry; do
  if test -d "website/$dir/pkg"; then
    echo "  ok website/$dir/pkg"
  else
    echo "  missing website/$dir/pkg — $dir demo will not load WASM"
    exit 1
  fi
done
test -d website/neural/pkg && echo "  ok website/neural/pkg" || (echo "Missing website/neural/pkg" && exit 1)
echo "Artifact root:"
ls -la website/
