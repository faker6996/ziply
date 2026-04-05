#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SKELETON_DIR="$ROOT_DIR/packaging/homebrew/tap-skeleton"
RENDERED_CASK="$ROOT_DIR/packaging/homebrew/Casks/ziply.rb"
OUTPUT_DIR="${1:-}"

if [[ -z "$OUTPUT_DIR" ]]; then
  cat <<'EOF' >&2
Usage:
  scripts/bootstrap-homebrew-tap.sh /path/to/homebrew-tap
EOF
  exit 1
fi

mkdir -p "$OUTPUT_DIR/Casks"
mkdir -p "$OUTPUT_DIR/.github/workflows"

cp "$SKELETON_DIR/README.md" "$OUTPUT_DIR/README.md"

if [[ -f "$RENDERED_CASK" ]]; then
  cp "$RENDERED_CASK" "$OUTPUT_DIR/Casks/ziply.rb"
  cp \
    "$SKELETON_DIR/.github/workflows/validate-cask.yml.template" \
    "$OUTPUT_DIR/.github/workflows/validate-cask.yml"
else
  cp \
    "$SKELETON_DIR/Casks/ziply.rb.template" \
    "$OUTPUT_DIR/Casks/ziply.rb.template"
  cp \
    "$SKELETON_DIR/.github/workflows/validate-cask.yml.template" \
    "$OUTPUT_DIR/.github/workflows/validate-cask.yml.template"
fi

cat <<EOF
Bootstrapped Homebrew tap skeleton at $OUTPUT_DIR

Next steps:
  1. Render a real cask with scripts/render-homebrew-cask.sh
  2. Copy or replace Casks/ziply.rb in the tap repo
  3. Enable the validation workflow
EOF
