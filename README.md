# codebase-memory-mcp (Rust)

Independent Rust implementation of **[codebase-memory-mcp](https://github.com/DeusData/codebase-memory-mcp)** — knowledge-graph indexer and MCP server for AI coding agents.

**RLM tools are not included.** For long-text map-reduce, use the separate **[rlm-mcp](https://github.com/stevenke1981/rlm-mcp)** MCP server (independent project).

## Relationship to other repos

| Path | MCP server | Role |
|------|------------|------|
| `D:\cbm-mcp` | `codebase-memory-mcp` | **This repo** — graph indexing, 14 MCP tools |
| `D:\rlm-mcp` | `rlm-mcp` | Standalone RLM sessions (scan/peek/chunk) |
| `D:\cbm\cbrlm` | `cbrlm-mcp` (legacy) | Deprecated combined binary |

The two servers are **not integrated** — enable both in the agent only if you want graph search **and** RLM sessions.

## Quick start

```powershell
cd D:\cbm-mcp
.\install.ps1
codebase-memory-mcp --version
```

`install.ps1` / `install.sh` download the latest GitHub Release binary by default. Agents can install directly from a checkout without compiling Rust.

## Install

### From checkout without compiling

```powershell
cd D:\cbm-mcp
.\install.ps1
```

This downloads the latest release archive, verifies `SHA256SUMS.txt`, installs the binary to a stable config path, installs agent MCP config, and writes the session hooks.

Unix:

```bash
./install.sh
```

Pin a version:

```powershell
.\install.ps1 -Version v0.1.0
```

```bash
CBM_VERSION=v0.1.0 ./install.sh
```

### Build from source checkout

Only use this for development or local unreleased changes:

```powershell
.\install.ps1 -FromSource -AllAgents
```

```bash
./install.sh --from-source --all-agents
```

### From GitHub Release

Windows:

```powershell
irm https://raw.githubusercontent.com/stevenke1981/cbm-mcp/main/packaging/windows/install.ps1 | iex
```

Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/stevenke1981/cbm-mcp/main/packaging/linux/install.sh | bash
```

macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/stevenke1981/cbm-mcp/main/packaging/macos/install.sh | bash
```

Release archives include the binary, `README.md`, `LICENSE`, and `mcp-templates/` for agent handoff.

### Index + search

```powershell
codebase-memory-mcp cli index_repository --json --quiet '{"repo_path":".","project":"my-app","mode":"fast"}'
codebase-memory-mcp cli search_graph --json '{"project":"my-app","query":"handler","limit":10}'
```

### MCP server (stdio)

```powershell
codebase-memory-mcp
# MCP server name: codebase-memory-mcp
```

### Optional: with rlm-mcp

Register **two independent** MCP servers when you need both graph and RLM:

```json
{
  "mcp": {
    "codebase-memory-mcp": {
      "type": "local",
      "command": ["codebase-memory-mcp"],
      "enabled": true
    },
    "rlm-mcp": {
      "type": "local",
      "command": ["rlm-mcp"],
      "enabled": true
    }
  }
}
```

See [`packaging/mcp/dual-servers.example.json`](packaging/mcp/dual-servers.example.json) and [rlm-mcp](https://github.com/stevenke1981/rlm-mcp).

## MCP tools (14)

`index_repository`, `index_status`, `search_graph`, `trace_path`, `get_code_snippet`, `get_graph_schema`, `get_architecture`, `query_graph`, `search_code`, `list_projects`, `delete_project`, `detect_changes`, `manage_adr`, `ingest_traces`

## Full clone status

Rust MVP toward full reference parity with `D:\_cbm-ref`. See [`TODO.md`](TODO.md), [`CLONE_ROADMAP.md`](CLONE_ROADMAP.md), and [`PARITY_MATRIX.md`](PARITY_MATRIX.md).

## Environment

| Variable | Purpose |
|----------|---------|
| `CBM_CACHE_DIR` | Cache dir (default `%LOCALAPPDATA%\codebase-memory-mcp`) |
| `CBRLM_CACHE_DIR` | Legacy alias for cache dir |
| `CBM_WATCHER` | `0` disables background reindex watcher |
| `CBM_SEMANTIC_ENABLED=1` | Enable semantic pass |

Projects use `cbm+` prefix (legacy `cbrlm+` accepted).

## Agent handoff

Use [`packaging/mcp/`](packaging/mcp/) for ready templates:

- `opencode.json`
- `codex-config.toml`
- `claude-settings.json`
- `generic-mcp.json`
- `manifest.json`

Replace `{{CBM_BINARY}}` with the absolute path printed by `install.ps1` / `install.sh`, or use the stable installed binary path.
