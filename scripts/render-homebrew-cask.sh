#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEMPLATE_PATH="$ROOT_DIR/packaging/homebrew/Casks/ziply.rb.template"
OUTPUT_PATH="${1:-$ROOT_DIR/packaging/homebrew/Casks/ziply.rb}"

VERSION="${VERSION:-$(node -p "require('$ROOT_DIR/package.json').version")}"
SHA256="${SHA256:-}"
URL="${URL:-}"
MACOS_REQUIREMENT="${MACOS_REQUIREMENT:->= :ventura}"

if [[ -z "$SHA256" || -z "$URL" ]]; then
  cat <<'EOF' >&2
Missing required inputs.

Usage:
  VERSION=0.1.0 \
  SHA256=<sha256> \
  URL=https://github.com/<owner>/<repo>/releases/download/v0.1.0/Ziply_0.1.0_universal.dmg \
  scripts/render-homebrew-cask.sh

Optional:
  MACOS_REQUIREMENT=">= :sonoma"
  scripts/render-homebrew-cask.sh /path/to/output/ziply.rb
EOF
  exit 1
fi

mkdir -p "$(dirname "$OUTPUT_PATH")"

sed \
  -e "s|__VERSION__|$VERSION|g" \
  -e "s|__SHA256__|$SHA256|g" \
  -e "s|__URL__|$URL|g" \
  -e "s|__MACOS_REQUIREMENT__|$MACOS_REQUIREMENT|g" \
  "$TEMPLATE_PATH" > "$OUTPUT_PATH"

echo "Rendered Homebrew cask to $OUTPUT_PATH"
