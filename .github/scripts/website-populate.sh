#!/usr/bin/env bash
# Populate website/ from built WASM demos. Reads .github/website-demos.txt for the demo list.
# Same logic as former justfile _website-populate; used by just website-build (local and cloud).
# Run from repo root.
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

DEMO_DIRS=$(grep -v '^#' .github/website-demos.txt 2>/dev/null | grep -v '^[[:space:]]*$' | tr '\n' ' ')
if [[ -z "$DEMO_DIRS" ]]; then
  echo "No demo dirs in .github/website-demos.txt" >&2
  exit 1
fi

for d in $DEMO_DIRS; do
  mkdir -p "website/$d"
done
mkdir -p website/neural/embedding

cp -r mathlib/demo/wasm-demo/* website/mathlib/
find website/mathlib -name '*.html' -exec sed -i.bak 's|<base href="/wasm-demo/">|<base href="/mathlib/">|g' {} \;
find website/mathlib -name '*.html.bak' -delete

cp -r render/demo/wasm-demo/* website/render/
cp -r kinematics/demo/wasm-demo/* website/kinematics/
cp -r physics/demo/wasm-demo/* website/physics/

cp website/geometry-index.html website/geometry/index.html
cp -r geometry/demo/wasm-demo/pkg website/geometry/

# neural: no WASM pkg; hub and embedding pages are committed in website/neural/
