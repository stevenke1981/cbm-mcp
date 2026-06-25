#!/usr/bin/env bash
# Install cbm (codebase-memory-mcp) — one-liner for Linux & macOS.
#
# Quick start:
#   curl -fsSL https://raw.githubusercontent.com/stevenke1981/cbm-mcp/main/install.sh | bash
#
# Options (via -s --):
#   curl ... | bash -s -- --uninstall
#   curl ... | bash -s -- --version v0.2.4
#   curl ... | bash -s -- --prefix ~/.local/bin
#
# From a local clone:
#   ./install.sh
#   ./install.sh --from-source --all-agents

set -euo pipefail

# ── Color support (only when terminal is interactive) ────────────────────────
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
FROM_SOURCE=false
ALL_AGENTS=false

# ── Parse CLI arguments ────────────────────────────────────────────────────────
while [ $# -gt 0 ]; do
  case "$1" in
    --help|-h)
      cat <<EOF
Usage: install.sh [OPTIONS]

Install cbm (codebase-memory-mcp) from GitHub Release.

Quick start:
  curl -fsSL https://raw.githubusercontent.com/stevenke1981/cbm-mcp/main/install.sh | bash

Options:
  --help, -h              Show this help
  --uninstall             Remove cbm instead of installing
  --version VERSION       Release tag to install (default: latest)
  --prefix DIR            Binary install directory (default: \$HOME/.local/bin)
  --from-source           Build from local source (requires Rust toolchain)
  --all-agents            Configure MCP server for all detected agents
  --skip-build            With --from-source, skip cargo build

Environment:
  CBM_REPO                GitHub repo (default: stevenke1981/cbm-mcp)
  CBM_VERSION             Release tag (overrides --version)
  CBM_INSTALL_DIR         Binary directory (overrides --prefix)
  CBM_CONFIG_DIR          Config directory
  GITHUB_TOKEN / GH_TOKEN GitHub API token
EOF
      exit 0 ;;
    --uninstall)   UNINSTALL=true ;;
    --from-source) FROM_SOURCE=true ;;
    --all-agents)  ALL_AGENTS=true ;;
    --skip-build)  SKIP_BUILD=true ;;
    --version)     shift; VERSION="$1" ;;
    --prefix)      shift; INSTALL_DIR="$1" ;;
    *) die "Unknown option: $1. Use --help for usage." ;;
  esac
  shift
done

# ── Detect if running from a local clone ─────────────────────────────────────
SCRIPT_PATH="$(readlink -f "$0" 2>/dev/null || realpath "$0" 2>/dev/null || echo "$0")"
SCRIPT_DIR="$(cd "$(dirname "$SCRIPT_PATH")" 2>/dev/null && pwd || echo "")"

IS_LOCAL=false
if [ -n "$SCRIPT_DIR" ] && [ -f "$SCRIPT_DIR/packaging/linux/install.sh" ]; then
  IS_LOCAL=true
fi

# ── Determine OS ──────────────────────────────────────────────────────────────
OS="$(uname -s)"
case "$OS" in
  Linux)  PACKAGING_SCRIPT="packaging/linux/install.sh" ;;
  Darwin) PACKAGING_SCRIPT="packaging/macos/install.sh" ;;
  *)
    die "Unsupported OS: ${OS}. Use scripts/install.sh for source installs."
    ;;
esac

# ── --from-source: build locally ─────────────────────────────────────────────
if [ "$FROM_SOURCE" = true ]; then
  if [ "$IS_LOCAL" = false ]; then
    die "--from-source requires a local clone (run from repo root)."
  fi
  ARGS=()
  if [ "$ALL_AGENTS" = true ]; then ARGS+=(--all-agents); fi
  if [ "${SKIP_BUILD:-false}" = true ]; then ARGS+=(--skip-build); fi
  exec "$SCRIPT_DIR/scripts/install.sh" "${ARGS[@]}"
fi

# ── Build args for packaging script ──────────────────────────────────────────
SCRIPT_ARGS=()
if [ "$UNINSTALL" = true ]; then SCRIPT_ARGS+=(--uninstall); fi
if [ -n "$VERSION" ]; then SCRIPT_ARGS+=(--version "$VERSION"); fi
if [ -n "$INSTALL_DIR" ]; then SCRIPT_ARGS+=(--prefix "$INSTALL_DIR"); fi

# ── Run packaging script ─────────────────────────────────────────────────────
run_packaging_script() {
  local script="$1"; shift
  if [ -x "$script" ]; then
    exec "$script" "$@"
  elif [ -f "$script" ]; then
    exec bash "$script" "$@"
  else
    return 1
  fi
}

if [ "$IS_LOCAL" = true ]; then
  # Local clone — use the packaging script directly
  info "Installing from local clone..."
  run_packaging_script "$SCRIPT_DIR/$PACKAGING_SCRIPT" "${SCRIPT_ARGS[@]}"
  die "Packaging script not found: $SCRIPT_DIR/$PACKAGING_SCRIPT"
else
  # Running via curl | bash — download packaging script from GitHub
  detail "Downloading installer for ${OS}..."
  RAW_URL="https://raw.githubusercontent.com/${REPO}/main/${PACKAGING_SCRIPT}"
  TMP="$(mktemp -d)"
  trap 'rm -rf "$TMP"' EXIT

  # Export env vars for the child script
  export CBM_REPO="$REPO"
  export CBM_VERSION="$VERSION"
  export CBM_INSTALL_DIR="$INSTALL_DIR"
  export CBM_CONFIG_DIR="$CONFIG_DIR"

  curl -fsSL "$RAW_URL" -o "$TMP/install.sh" || die "Failed to download installer from ${RAW_URL}"
  exec bash "$TMP/install.sh" "${SCRIPT_ARGS[@]}"
fi
