#!/usr/bin/env bash
# Rewrite absolute paths in website/ for GitHub Project Pages (site served at /<repo>/ not /).
# Usage: ./fix-pages-base-path.sh [REPO_NAME]
#   REPO_NAME defaults to the part after / in GITHUB_REPOSITORY (e.g. Dimension).
set -e
REPO_NAME="${1:-${GITHUB_REPOSITORY#*/}}"
if [[ -z "$REPO_NAME" ]]; then
  echo "Usage: $0 REPO_NAME or set GITHUB_REPOSITORY" >&2
  exit 1
fi
find website -type f \( -name '*.html' -o -name '*.js' \) -print0 | xargs -0 sed -i \
  -e "s|base href=\"/|base href=\"/$REPO_NAME/|g" \
  -e "s|href=\"/\"|href=\"/$REPO_NAME/\"|g" \
  -e "s|href=\"/mathlib/|href=\"/$REPO_NAME/mathlib/|g" \
  -e "s|href=\"/neural/|href=\"/$REPO_NAME/neural/|g" \
  -e "s|href=\"/kinematics/|href=\"/$REPO_NAME/kinematics/|g" \
  -e "s|href=\"/physics/|href=\"/$REPO_NAME/physics/|g" \
  -e "s|href=\"/geometry/|href=\"/$REPO_NAME/geometry/|g" \
  -e "s|href=\"/render/|href=\"/$REPO_NAME/render/|g" \
  -e "s|src=\"/|src=\"/$REPO_NAME/|g" \
  -e "s|import(\"/|import(\"/$REPO_NAME/|g" \
  -e "s|fetch(\"/|fetch(\"/$REPO_NAME/|g"
