#!/bin/sh
# Sequential pipeline: clean website demo dirs → build (just website-build) → fix base path → verify.
# Same script for local (just website-pages) and GitHub Actions (pages.yml); no divergence.
# For root-serving (e.g. Docker or custom host), skip the fix step or run only build+verify;
# the sed fix is for GitHub Project Pages only (site under /<repo>/).
# Usage: ./prepare-pages-artifact.sh [REPO_NAME]
#   REPO_NAME defaults to the part after / in GITHUB_REPOSITORY (e.g. Dimension). Run from repo root.
set -e

# Run from repo root so just and paths are correct (e.g. when invoked from .github/workflows).
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

# Single source of truth for demo dirs (reads .github/website-demos.txt).
DEMO_DIRS=$(grep -v '^#' .github/website-demos.txt 2>/dev/null | grep -v '^[[:space:]]*$' | tr '\n' ' ')
if [ -z "$DEMO_DIRS" ]; then
  echo "No demo dirs in .github/website-demos.txt" >&2
  exit 1
fi

echo "Cleaning website demo dirs for full rebuild..."
for d in $DEMO_DIRS; do
  case "$d" in
    neural) rm -rf "website/neural/pkg" ;;  # preserve tracked index.html and embedding/
    *)     rm -rf "website/$d" ;;
  esac
done
echo "Building unified website (all WASM demos) — run just website-build each time we rebuild WASM..."
just website-build

REPO_NAME="${1:-${GITHUB_REPOSITORY#*/}}"
if [ -z "$REPO_NAME" ]; then
  echo "Usage: $0 REPO_NAME or set GITHUB_REPOSITORY" >&2
  exit 1
fi
echo "Fixing base path for Project Pages (REPO_NAME=$REPO_NAME)..."
SED_SCRIPT="s|base href=\"/|base href=\"/$REPO_NAME/|g"
SED_SCRIPT="$SED_SCRIPT; s|href=\"/\"|href=\"/$REPO_NAME/\"|g"
SED_SCRIPT="$SED_SCRIPT; s|src=\"/|src=\"/$REPO_NAME/|g"
SED_SCRIPT="$SED_SCRIPT; s|import(\"/|import(\"/$REPO_NAME/|g"
SED_SCRIPT="$SED_SCRIPT; s|fetch(\"/|fetch(\"/$REPO_NAME/|g"
for d in $DEMO_DIRS; do
  SED_SCRIPT="$SED_SCRIPT; s|href=\"/$d/|href=\"/$REPO_NAME/$d/|g"
done
find website -type f \( -name '*.html' -o -name '*.js' \) -exec sed -i "$SED_SCRIPT" {} +

echo "Checking site root..."
test -f website/index.html || (echo "Missing website/index.html. Ensure it is tracked (!website/index.html in .gitignore) and committed." && exit 1)
echo "Checking demo pages (must exist for Pages to serve /mathlib/, /render/, etc.)..."
for dir in $DEMO_DIRS; do
  if ! test -f "website/$dir/index.html"; then
    echo "Missing website/$dir/index.html — demo will 404. Ensure demo shell files are tracked (see .gitignore exceptions) and committed."
    exit 1
  fi
  echo "  ok website/$dir/index.html"
done
test -f website/neural/embedding/index.html || (echo "Missing website/neural/embedding/index.html" && exit 1)
echo "  ok website/neural/embedding/index.html"
echo "Demo WASM pkgs (required for demos to run):"
for dir in $DEMO_DIRS; do
  if [ "$dir" = "neural" ]; then
    echo "  ok website/neural (hub, no WASM)"
  elif test -d "website/$dir/pkg"; then
    echo "  ok website/$dir/pkg"
  else
    echo "  missing website/$dir/pkg — $dir demo will not load WASM"
    exit 1
  fi
done
echo "Artifact root:"
ls -la website/
