# codebase-memory-mcp MCP package

Handoff templates for agents wiring **codebase-memory-mcp** (graph index only).

Server name: `codebase-memory-mcp`  
Transport: stdio  
Binary: `codebase-memory-mcp` or absolute path to the release binary

## Fast path

```powershell
cargo build --release
.\target\release\codebase-memory-mcp.exe install --yes --all
```

Restart the target agent after install.

## Manual config

| Template | Target |
|----------|--------|
| `generic-mcp.json` | Claude-style `mcpServers`, Gemini CLI, Zed |
| `codex-config.toml` | Codex `config.toml` snippet |
| `opencode.json` | OpenCode `opencode.json` snippet |
| `claude-settings.json` | Claude Code / Desktop settings |
| `manifest.json` | Machine-readable package summary |
| `dual-servers.example.json` | Optional second server: `codebase-memory-rlm-mcp` |

Replace `{{CBM_BINARY}}` with an absolute binary path.

## Environment

```json
{
  "CBM_PROJECT_PREFIX": "cbm+",
  "CBM_AGENT": "generic"
}
```

Legacy aliases `CBRLM_*` are still accepted by the binary.

## Tool contract (14 graph tools)

`index_repository`, `index_status`, `search_graph`, `trace_path`, `get_code_snippet`, `get_graph_schema`, `get_architecture`, `query_graph`, `search_code`, `list_projects`, `delete_project`, `detect_changes`, `manage_adr`, `ingest_traces`

Use graph tools before broad file search when a project is indexed.

## RLM (separate project)

RLM session tools (`rlm_scan`, `rlm_peek`, `rlm_chunk`, `rlm_workflow`, …) live in **[rlm-mcp](https://github.com/stevenke1981/rlm-mcp)** as MCP server **`codebase-memory-rlm-mcp`**.

- Not bundled with this binary
- No code dependency between repos
- Optional: enable both servers in the same agent (see `dual-servers.example.json`)