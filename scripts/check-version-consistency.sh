#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

PACKAGE_VERSION="$(node -p "require('./package.json').version")"
TAURI_VERSION="$(node -p "require('./src-tauri/tauri.conf.json').version")"
CARGO_VERSION="$(awk '
  /^\[package\]$/ { in_section = 1; next }
  /^\[/ { if (in_section == 1) { exit } }
  in_section && /^version = / {
    gsub(/^version = "/, "", $0)
    gsub(/"$/, "", $0)
    print
    exit
  }
' src-tauri/Cargo.toml)"

if [ -z "$PACKAGE_VERSION" ] || [ -z "$TAURI_VERSION" ] || [ -z "$CARGO_VERSION" ]; then
  echo "Unable to determine one or more versions."
  exit 1
fi

if [ "$PACKAGE_VERSION" != "$TAURI_VERSION" ] || [ "$PACKAGE_VERSION" != "$CARGO_VERSION" ]; then
  echo "Version mismatch detected."
  echo "package.json: $PACKAGE_VERSION"
  echo "src-tauri/tauri.conf.json: $TAURI_VERSION"
  echo "src-tauri/Cargo.toml: $CARGO_VERSION"
  exit 1
fi

echo "$PACKAGE_VERSION"
