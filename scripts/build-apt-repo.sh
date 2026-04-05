#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INPUT_DEB="${1:-}"
OUTPUT_DIR="${2:-$ROOT_DIR/target/apt-repo-site/apt}"

if [[ -z "$INPUT_DEB" ]]; then
  cat <<'EOF' >&2
Usage:
  scripts/build-apt-repo.sh /path/to/ziply_<version>_amd64.deb [/path/to/output/apt]
EOF
  exit 1
fi

if [[ ! -f "$INPUT_DEB" ]]; then
  echo "Input deb not found: $INPUT_DEB" >&2
  exit 1
fi

if ! command -v dpkg-scanpackages >/dev/null 2>&1; then
  echo "dpkg-scanpackages is required. Install dpkg-dev." >&2
  exit 1
fi

if ! command -v apt-ftparchive >/dev/null 2>&1; then
  echo "apt-ftparchive is required. Install apt-utils." >&2
  exit 1
fi

PACKAGE_NAME="$(dpkg-deb -f "$INPUT_DEB" Package)"
VERSION="$(dpkg-deb -f "$INPUT_DEB" Version)"
ARCHITECTURE="$(dpkg-deb -f "$INPUT_DEB" Architecture)"

APT_ORIGIN="${APT_ORIGIN:-Ziply}"
APT_LABEL="${APT_LABEL:-Ziply}"
APT_SUITE="${APT_SUITE:-stable}"
APT_CODENAME="${APT_CODENAME:-stable}"
APT_COMPONENT="${APT_COMPONENT:-main}"
APT_DESCRIPTION="${APT_DESCRIPTION:-Ziply APT repository}"
APT_REPO_URL="${APT_REPO_URL:-https://faker6996.github.io/ziply/apt}"

REPO_ROOT="$OUTPUT_DIR"
POOL_DIR="$REPO_ROOT/pool/$APT_COMPONENT/${PACKAGE_NAME:0:1}/$PACKAGE_NAME"
DIST_DIR="$REPO_ROOT/dists/$APT_SUITE/$APT_COMPONENT/binary-$ARCHITECTURE"

rm -rf "$REPO_ROOT"
mkdir -p "$POOL_DIR" "$DIST_DIR"

REPO_DEB_NAME="${PACKAGE_NAME}_${VERSION}_${ARCHITECTURE}.deb"
cp "$INPUT_DEB" "$POOL_DIR/$REPO_DEB_NAME"

pushd "$REPO_ROOT" >/dev/null
dpkg-scanpackages --multiversion "pool" > "dists/$APT_SUITE/$APT_COMPONENT/binary-$ARCHITECTURE/Packages"
gzip -kf "dists/$APT_SUITE/$APT_COMPONENT/binary-$ARCHITECTURE/Packages"

apt-ftparchive \
  -o "APT::FTPArchive::Release::Origin=$APT_ORIGIN" \
  -o "APT::FTPArchive::Release::Label=$APT_LABEL" \
  -o "APT::FTPArchive::Release::Suite=$APT_SUITE" \
  -o "APT::FTPArchive::Release::Codename=$APT_CODENAME" \
  -o "APT::FTPArchive::Release::Architectures=$ARCHITECTURE" \
  -o "APT::FTPArchive::Release::Components=$APT_COMPONENT" \
  -o "APT::FTPArchive::Release::Description=$APT_DESCRIPTION" \
  release "dists/$APT_SUITE" > "dists/$APT_SUITE/Release"
popd >/dev/null

SIGNED_REPO="false"

if [[ -n "${APT_GPG_PRIVATE_KEY:-}" ]]; then
  GNUPGHOME_DIR="$(mktemp -d)"
  export GNUPGHOME="$GNUPGHOME_DIR"
  trap 'rm -rf "$GNUPGHOME_DIR"' EXIT

  printf '%s\n' "$APT_GPG_PRIVATE_KEY" | gpg --batch --import

  KEY_FINGERPRINT="$(
    gpg --batch --list-secret-keys --with-colons |
      awk -F: '$1 == "fpr" { print $10; exit }'
  )"

  if [[ -z "$KEY_FINGERPRINT" ]]; then
    echo "Unable to determine imported GPG key fingerprint." >&2
    exit 1
  fi

  PASSPHRASE_ARGS=()
  if [[ -n "${APT_GPG_PASSPHRASE:-}" ]]; then
    PASSPHRASE_ARGS+=(--pinentry-mode loopback --passphrase "$APT_GPG_PASSPHRASE")
  fi

  gpg --batch --yes "${PASSPHRASE_ARGS[@]}" \
    --default-key "$KEY_FINGERPRINT" \
    --output "$REPO_ROOT/dists/$APT_SUITE/InRelease" \
    --clearsign "$REPO_ROOT/dists/$APT_SUITE/Release"

  gpg --batch --yes "${PASSPHRASE_ARGS[@]}" \
    --default-key "$KEY_FINGERPRINT" \
    --output "$REPO_ROOT/dists/$APT_SUITE/Release.gpg" \
    --armor --detach-sign "$REPO_ROOT/dists/$APT_SUITE/Release"

  gpg --batch --yes --armor --export "$KEY_FINGERPRINT" > "$REPO_ROOT/ziply-archive-keyring.asc"
  gpg --batch --yes --export "$KEY_FINGERPRINT" > "$REPO_ROOT/ziply-archive-keyring.gpg"
  SIGNED_REPO="true"
fi

INSTALL_SNIPPET='echo "Signing is not configured for this repository yet."'
if [[ "$SIGNED_REPO" == "true" ]]; then
  INSTALL_SNIPPET="curl -fsSL $APT_REPO_URL/ziply-archive-keyring.asc | sudo gpg --dearmor -o /usr/share/keyrings/ziply-archive-keyring.gpg
echo \"deb [signed-by=/usr/share/keyrings/ziply-archive-keyring.gpg] $APT_REPO_URL $APT_SUITE $APT_COMPONENT\" | sudo tee /etc/apt/sources.list.d/ziply.list
sudo apt update
sudo apt install ziply"
fi

cat > "$REPO_ROOT/index.html" <<EOF
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Ziply APT Repository</title>
  </head>
  <body>
    <h1>Ziply APT Repository</h1>
    <p>Repository URL: <code>$APT_REPO_URL</code></p>
    <p>Package: <code>$PACKAGE_NAME</code> version <code>$VERSION</code></p>
    <p>Signed repository: <code>$SIGNED_REPO</code></p>
    <p>Install commands:</p>
    <pre><code>$INSTALL_SNIPPET</code></pre>
  </body>
</html>
EOF

cat > "$REPO_ROOT/install.sh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
$INSTALL_SNIPPET
EOF

chmod +x "$REPO_ROOT/install.sh"

echo "Built APT repository at $REPO_ROOT"
