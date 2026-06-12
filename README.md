# codebase-memory-mcp (Rust)

Independent Rust implementation of **[codebase-memory-mcp](https://github.com/DeusData/codebase-memory-mcp)** — knowledge-graph indexer and MCP server for AI coding agents.

**RLM map-reduce tools are not included.** Use the separate **[codebase-memory-rlm-mcp](../rlm-mcp)** server alongside this binary.

## Relationship to other repos

| Path | Role |
|------|------|
| `D:\cbm-mcp` | **This repo** — graph indexing, 14 MCP tools, SQLite store |
| `D:\rlm-mcp` | RLM orchestration MCP (calls this server via MCP client) |
| `D:\cbm\cbrlm` | Legacy combined binary (deprecated; split into the two repos above) |
| `D:\_cbm-ref` | Reference C implementation (full parity target) |

## Quick start

```powershell
cd D:\cbm-mcp
cargo build --release
.\target\release\codebase-memory-mcp.exe --version
```

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

### With RLM

Register **both** servers in your agent config:

1. `codebase-memory-mcp` — index, search, trace, snippets
2. `codebase-memory-rlm-mcp` (from `D:\rlm-mcp`) — `rlm_workflow`, `rlm_filter`, `rlm_scan`, …

## MCP tools (14)

`index_repository`, `index_status`, `search_graph`, `trace_path`, `get_code_snippet`, `get_graph_schema`, `get_architecture`, `query_graph`, `search_code`, `list_projects`, `delete_project`, `detect_changes`, `manage_adr`, `ingest_traces`

## Full clone status

This is a **Rust MVP** toward full reference parity with `D:\_cbm-ref`. See [`CLONE_ROADMAP.md`](CLONE_ROADMAP.md) and [`PARITY_MATRIX.md`](PARITY_MATRIX.md).

## Environment

| Variable | Purpose |
|----------|---------|
| `CBM_CACHE_DIR` | Cache dir (default `%LOCALAPPDATA%\codebase-memory-mcp`) |
| `CBRLM_CACHE_DIR` | Legacy alias for cache dir |
| `CBM_WATCHER` | `0` disables background reindex watcher |
| `CBM_SEMANTIC_ENABLED=1` | Enable semantic pass |

Projects use `cbm+` prefix (legacy `cbrlm+` accepted).