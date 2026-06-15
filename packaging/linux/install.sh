#!/usr/bin/env bash
# Install cbm (codebase-memory-mcp) on Linux from GitHub Release.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/stevenke1981/cbm-mcp/main/install.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/stevenke1981/cbm-mcp/main/install.sh | bash -s -- --uninstall
#   CBM_VERSION=v0.2.3 bash <(curl -fsSL ...)
#   ./packaging/linux/install.sh --prefix ~/.local/bin
#
# Environment variables:
#   CBM_REPO       GitHub repo (default: stevenke1981/cbm-mcp)
#   CBM_VERSION    Release tag (default: latest)
#   CBM_INSTALL_DIR Binary install directory (default: $HOME/.local/bin)
#   CBM_CONFIG_DIR  Config directory (default: $HOME/.config/cbm-mcp/bin)
#   GITHUB_TOKEN / GH_TOKEN  GitHub API token (for rate limiting)

set -euo pipefail

# ── Color support ──────────────────────────────────────────────────────────────
if [ -t 1 ] && [ -n "$TERM" ] && [ "$TERM" != "dumb" ]; then
  GREEN='\033[0;32m';  BOLD='\033[1m'
  YELLOW='\033[0;33m'; RED='\033[0;31m'
  GRAY='\033[0;90m';   NC='\033[0m'
else
  GREEN=''; BOLD=''; YELLOW=''; RED=''; GRAY=''; NC=''
fi
info()  { printf "${GREEN}%s${NC}\n" "$*"; }
warn()  { printf "${YELLOW}WARN:${NC} %s\n" "$*"; }
error() { printf "${RED}ERROR:${NC} %s\n" "$*" >&2; }
die()   { error "$1"; exit "${2:-1}"; }
detail(){ printf "${GRAY}%s${NC}\n" "$*"; }

# ── Defaults ───────────────────────────────────────────────────────────────────
REPO="${CBM_REPO:-stevenke1981/cbm-mcp}"
VERSION="${CBM_VERSION:-latest}"
INSTALL_DIR="${CBM_INSTALL_DIR:-$HOME/.local/bin}"
CONFIG_DIR="${CBM_CONFIG_DIR:-$HOME/.config/cbm-mcp/bin}"
UNINSTALL=false

# ── Parse CLI arguments ────────────────────────────────────────────────────────
while [ $# -gt 0 ]; do
  case "$1" in
    --help|-h)
      cat <<EOF
Usage: packaging/linux/install.sh [OPTIONS]

Install cbm (codebase-memory-mcp) on Linux from GitHub Release.

Options:
  --help, -h              Show this help
  --uninstall             Remove cbm instead of installing
  --version VERSION       Release tag to install (default: latest)
  --prefix DIR            Binary install directory (default: \$HOME/.local/bin)

Environment:
  CBM_REPO                GitHub repo (default: stevenke1981/cbm-mcp)
  CBM_VERSION             Release tag (overrides --version)
  CBM_INSTALL_DIR         Binary directory (overrides --prefix)
  CBM_CONFIG_DIR          Config directory
  GITHUB_TOKEN / GH_TOKEN GitHub API token

One-liner:
  curl -fsSL https://raw.githubusercontent.com/stevenke1981/cbm-mcp/main/install.sh | bash
EOF
      exit 0 ;;
    --uninstall) UNINSTALL=true ;;
    --version) shift; VERSION="$1" ;;
    --prefix)  shift; INSTALL_DIR="$1" ;;
    *) die "Unknown option: $1. Use --help for usage." ;;
  esac
  shift
done

# ── Uninstall mode ────────────────────────────────────────────────────────────
if [ "$UNINSTALL" = true ]; then
  BIN="$INSTALL_DIR/cbm"
  CONFIG_BIN="$CONFIG_DIR/cbm"
  REMOVED=false

  if [ -L "$BIN" ]; then
    rm -f "$BIN" && info "Removed symlink: $BIN" && REMOVED=true
  elif [ -f "$BIN" ]; then
    rm -f "$BIN" && info "Removed binary: $BIN" && REMOVED=true
  fi

  if [ -f "$CONFIG_BIN" ]; then
    rm -f "$CONFIG_BIN" && info "Removed binary: $CONFIG_BIN" && REMOVED=true
  fi

  if [ -d "$CONFIG_DIR" ]; then
    rmdir "$CONFIG_DIR" 2>/dev/null && detail "Removed empty directory: $CONFIG_DIR" || true
  fi

  if [ "$REMOVED" = true ]; then
    info "cbm has been uninstalled."
  else
    info "cbm is not installed at $INSTALL_DIR or $CONFIG_DIR."
  fi
  exit 0
fi

# ── Dependency check ──────────────────────────────────────────────────────────
check_deps() {
  local missing=false
  for cmd in curl tar; do
    if ! command -v "$cmd" &>/dev/null; then
      error "Required command not found: $cmd"
      missing=true
    fi
  done
  # prefer sha256sum; fallback to shasum (macOS compat, kept for cross-platform)
  if command -v sha256sum &>/dev/null; then
    SHASUM=sha256sum
  elif command -v shasum &>/dev/null; then
    SHASUM="shasum -a 256"
  else
    error "Required command not found: sha256sum or shasum"
    missing=true
  fi
  [ "$missing" = true ] && die "Install missing dependencies and re-run."
}
check_deps

# ── Distro detection (informational) ─────────────────────────────────────────
detect_distro() {
  local id=""
  if [ -f /etc/os-release ]; then
    id=$(grep -oP '(?<=^ID=).*' /etc/os-release | tr -d '"')
  elif command -v lsb_release &>/dev/null; then
    id=$(lsb_release -si 2>/dev/null | tr '[:upper:]' '[:lower:]')
  fi
  echo "$id"
}
DISTRO=$(detect_distro)

# ── Architecture ──────────────────────────────────────────────────────────────
arch="$(uname -m)"
case "$arch" in
  x86_64|amd64) ARTIFACT="cbm-mcp-linux-x64" ;;
  aarch64|arm64) ARTIFACT="cbm-mcp-linux-arm64" ;;
  *) die "Unsupported Linux architecture: $arch" ;;
esac

# ── Resolve version ───────────────────────────────────────────────────────────
if [ "$VERSION" = "latest" ]; then
  detail "Resolving latest release from GitHub API..."
  API="https://api.github.com/repos/${REPO}/releases/latest"
  token="${GITHUB_TOKEN:-${GH_TOKEN:-}}"
  if [ -n "$token" ]; then
    VERSION=$(curl -fsSL -H "User-Agent: cbm-mcp-installer" \
      -H "Authorization: Bearer ${token}" "$API" \
      | grep -m1 '"tag_name"' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/' || true)
  else
    VERSION=$(curl -fsSL -H "User-Agent: cbm-mcp-installer" "$API" \
      | grep -m1 '"tag_name"' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/' || true)
  fi

  if [ -z "$VERSION" ]; then
    detail "API fallback: resolving via redirect..."
    latest_url=$(curl -fsSL -o /dev/null -w '%{url_effective}' \
      "https://github.com/${REPO}/releases/latest" || true)
    VERSION=$(printf '%s\n' "$latest_url" | sed -E 's#^.*/releases/tag/([^/?#]+).*$#\1#')
    if [ -z "$VERSION" ] || [ "$VERSION" = "$latest_url" ]; then
      die "Failed to resolve latest GitHub Release for ${REPO}"
    fi
  fi
  detail "Resolved version: ${VERSION}"
fi

# ── Check existing installation ──────────────────────────────────────────────
CONFIG_BIN="$CONFIG_DIR/cbm"
if [ -f "$CONFIG_BIN" ]; then
  EXISTING_VER=$("$CONFIG_BIN" --version 2>/dev/null || true)
  if [ -n "$EXISTING_VER" ]; then
    info "Existing installation found: ${EXISTING_VER}"
  fi
  if [ -t 0 ]; then
    printf "${YELLOW}Overwrite existing installation?${NC} [Y/n] "
    read -r resp; resp="${resp:-Y}"
    case "$resp" in
      n*|N*) die "Aborted by user." 0 ;;
    esac
  fi
fi

# ── Download ──────────────────────────────────────────────────────────────────
BASE="https://github.com/${REPO}/releases/download/${VERSION}"
ARCHIVE="${ARTIFACT}.tar.gz"
URL="${BASE}/${ARCHIVE}"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

detail "Downloading ${URL} ..."
curl -fsSL "$URL" -o "$TMP/${ARCHIVE}" || die "Download failed: ${URL}"

# ── Checksum verification ────────────────────────────────────────────────────
detail "Verifying checksum ..."
curl -fsSL "${BASE}/SHA256SUMS.txt" -o "$TMP/SHA256SUMS.txt" || \
  warn "Checksum file not found, skipping verification"

if [ -f "$TMP/SHA256SUMS.txt" ]; then
  expected=$(grep " ${ARCHIVE}$" "$TMP/SHA256SUMS.txt" | awk '{print $1}')
  if [ -z "$expected" ]; then
    warn "Checksum for ${ARCHIVE} not found in SHA256SUMS.txt, skipping verification"
  else
    actual=$($SHASUM "$TMP/${ARCHIVE}" | awk '{print $1}')
    if [ "$actual" != "$expected" ]; then
      die "Checksum mismatch for ${ARCHIVE} (expected: ${expected}, actual: ${actual})"
    fi
    detail "Checksum verified successfully."
  fi
fi

# ── Extract & install ────────────────────────────────────────────────────────
tar -xzf "$TMP/${ARCHIVE}" -C "$TMP"

if [ ! -f "$TMP/cbm" ]; then
  die "Archive does not contain expected binary 'cbm'"
fi

mkdir -p "$INSTALL_DIR" "$CONFIG_DIR"

install -m 755 "$TMP/cbm" "$CONFIG_DIR/cbm"
ln -sf "$CONFIG_DIR/cbm" "$INSTALL_DIR/cbm"
info "Installed: ${CONFIG_DIR}/cbm -> ${INSTALL_DIR}/cbm"

# ── PATH check ───────────────────────────────────────────────────────────────
if ! echo ":$PATH:" | grep -q ":${INSTALL_DIR}:"; then
  echo ""
  warn "${INSTALL_DIR} is not in PATH."
  echo "  Add to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
  echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
  echo ""
  # Auto-install for bash/zsh users
  if [ -f "$HOME/.bashrc" ]; then
    echo "source $HOME/.bashrc" >> /dev/null
  fi
fi

# ── Configure MCP agents ─────────────────────────────────────────────────────
info "Configuring MCP agents..."
"$CONFIG_DIR/cbm" install --yes --all

DISTRO_MSG=""
[ -n "$DISTRO" ] && DISTRO_MSG=" on ${DISTRO}"

echo ""
info "${BOLD}Installed cbm ${VERSION}${NC}${DISTRO_MSG}"
detail "Binary:     ${CONFIG_DIR}/cbm"
detail "Symlink:    ${INSTALL_DIR}/cbm"
detail "To uninstall: ${0} --uninstall"
