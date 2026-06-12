#!/usr/bin/env bash
# Install cbm-mcp from GitHub Release (macOS x64 / Apple Silicon).
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/stevenke1981/cbm-mcp/main/packaging/macos/install.sh | bash
#   CBM_VERSION=v0.1.0 ./packaging/macos/install.sh

set -euo pipefail

REPO="${CBM_REPO:-${CBRLM_REPO:-stevenke1981/cbm-mcp}}"
VERSION="${CBM_VERSION:-${CBRLM_VERSION:-latest}}"
INSTALL_DIR="${CBM_INSTALL_DIR:-${CBRLM_INSTALL_DIR:-$HOME/.local/bin}}"
CONFIG_DIR="${CBM_CONFIG_DIR:-${CBRLM_CONFIG_DIR:-$HOME/.config/cbm-mcp/bin}}"

arch="$(uname -m)"
case "$arch" in
  x86_64) ARTIFACT="cbm-mcp-macos-x64" ;;
  arm64) ARTIFACT="cbm-mcp-macos-arm64" ;;
  *)
    echo "Unsupported macOS architecture: $arch" >&2
    exit 1
    ;;
esac

if [ "$VERSION" = "latest" ]; then
  API="https://api.github.com/repos/${REPO}/releases/latest"
  VERSION="$(curl -fsSL "$API" | grep -m1 '"tag_name"' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')"
fi

BASE="https://github.com/${REPO}/releases/download/${VERSION}"
ARCHIVE="${ARTIFACT}.tar.gz"
URL="${BASE}/${ARCHIVE}"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "Downloading ${URL} ..."
curl -fsSL "$URL" -o "$TMP/${ARCHIVE}"

echo "Verifying checksum ..."
curl -fsSL "${BASE}/SHA256SUMS.txt" -o "$TMP/SHA256SUMS.txt"
expected="$(grep " ${ARCHIVE}$" "$TMP/SHA256SUMS.txt" | awk '{print $1}')"
if [ -z "$expected" ]; then
  echo "checksum for ${ARCHIVE} not found in SHA256SUMS.txt" >&2
  exit 1
fi
actual="$(shasum -a 256 "$TMP/${ARCHIVE}" | awk '{print $1}')"
if [ "$actual" != "$expected" ]; then
  echo "checksum mismatch for ${ARCHIVE}" >&2
  exit 1
fi

tar -xzf "$TMP/${ARCHIVE}" -C "$TMP"

mkdir -p "$INSTALL_DIR" "$CONFIG_DIR"
install -m 755 "$TMP/codebase-memory-mcp" "$CONFIG_DIR/codebase-memory-mcp"
ln -sf "$CONFIG_DIR/codebase-memory-mcp" "$INSTALL_DIR/codebase-memory-mcp"

if ! echo ":$PATH:" | grep -q ":${INSTALL_DIR}:"; then
  echo ""
  echo "Add to PATH: export PATH=\"${INSTALL_DIR}:\$PATH\""
fi

echo "Configuring MCP agents..."
"$CONFIG_DIR/codebase-memory-mcp" install --yes --all || true

echo ""
echo "Installed codebase-memory-mcp ${VERSION} → ${CONFIG_DIR}/codebase-memory-mcp"
