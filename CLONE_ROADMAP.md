# Full clone roadmap — codebase-memory-mcp

Target: feature-equivalent Rust rewrite of `D:\_cbm-ref` (DeusData/codebase-memory-mcp C core).

## Architecture split (done)

- **cbm-mcp** (`D:\cbm-mcp`) — this repo; 14 graph MCP tools only
- **rlm-mcp** (`D:\rlm-mcp`) — RLM orchestration; separate MCP server

## P0 — graph correctness

- [ ] Hybrid LSP CALLS (Python, TS/JS, PHP, C#, Go, C/C++)
- [ ] Store bulk transaction API + rollback tests
- [ ] Honest `get_graph_schema` vs emitted edges

## P1 — reference pipeline

- [ ] Usages / TypeRef pass
- [ ] `HTTP_CALLS`, `ASYNC_CALLS`, cross-service edges
- [ ] Leiden/Louvain communities (replace connected-components)
- [ ] BM25 + camelCase tokenization in `search_graph`
- [ ] Cypher in `query_graph` (or document SQL-only deviation)
- [ ] `trace_path` data_flow / cross_service modes
- [ ] 159-language tree-sitter coverage (vendored grammars)
- [ ] Graph buffer staging layer

## P2 — platform parity

- [ ] React/Three.js graph-ui (or ship reference UI variant)
- [ ] Go/PyPI/npm/Chocolatey/AUR wrappers
- [ ] Reference-grade semantic signal tuning
- [ ] Git history / cross-repo index modes

## Omitted by design

- FoundationDB backend (SQLite canonical)
- Foundation C runtime layer (Rust std + targeted crates)

## Verification

```powershell
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
.\scripts\smoke-quality-gates.ps1
```

Reference specs: `docs/reference/` (from `knowledge-graph/`).