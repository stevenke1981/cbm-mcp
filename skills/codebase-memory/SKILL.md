---
name: codebase-memory
description: Use cbm/codebase-memory-mcp graph tools before broad text search when exploring, tracing, reviewing, or editing a codebase.
compatibility: opencode, claude-code, codex
---

## Quick Decision Matrix

| Question | Start with |
| --- | --- |
| Is this repo indexed? | `list_projects`, then `index_status` |
| Where is a function, type, route, or class? | `search_graph` |
| What calls this or what does it call? | `trace_path` |
| What is the exact source for a symbol? | `get_code_snippet` |
| What is the high-level repo structure? | `get_architecture` |
| Which files changed since the last index? | `detect_changes` |
| I need a precise graph query | `query_graph` |
| I need source-text matches with graph context | `search_code` |
| I need huge log/blob analysis | Use separate RLM MCP tools, not cbm |
| I need config/docs/generated files | Read files or use text search directly |

## Exploration Workflow

1. Run `list_projects` and look for the repo project. Prefer the `cbm+` prefix; legacy `cbrlm+` project names may still exist.
2. If the repo is missing or stale, run `index_repository` with the repository path. Use incremental indexing when the existing index only needs refresh.
3. Run `get_graph_schema` when you need exact node, edge, or file-table fields before writing `query_graph` SQL.
4. Use `search_graph` with `query`, `name_pattern`, `qn_pattern`, `label`, or `file_pattern` to find likely symbols.
5. Use `get_code_snippet` for exact source before editing. Graph hits orient you; source reads verify the edit target.
6. Use `get_architecture` when you need module boundaries, hotspots, or a quick map before choosing files.

## Tracing Workflow

1. Find the symbol with `search_graph`.
2. Run `trace_path` in the relevant direction for call chains and data flow.
3. Use `include_connected`, `relationship`, `min_degree`, or `max_degree` in `search_graph` when you need fan-in/fan-out context.
4. Use `detect_changes` after edits to decide whether an incremental reindex is needed.
5. Re-read the exact source files before applying patches.

## Quality Analysis

- Dead or isolated code: search for low-degree symbols with `search_graph` degree filters, then verify with source reads.
- High-risk shared code: search for high-degree functions/classes, then trace callers before changing behavior.
- Architectural review: use `get_architecture`, then `query_graph` for exact counts or file/symbol joins.
- Regression risk: combine `trace_path` with focused tests for the changed module.

## Tool Reference

| Tool | Use |
| --- | --- |
| `index_repository` | Build or refresh the graph index for a repo path. |
| `index_status` | Check whether a project exists and when it was indexed. |
| `search_graph` | Search symbols and optionally connected graph context. |
| `trace_path` | Trace callers/callees around a function or symbol. |
| `get_code_snippet` | Read exact source for one qualified symbol. |
| `get_graph_schema` | Inspect available graph tables/fields before SQL. |
| `get_architecture` | Summarize repository structure and graph metrics. |
| `query_graph` | Run read-only SQL over symbols, edges, and files. |
| `search_code` | Search source with graph-aware context. |
| `list_projects` | List indexed projects. |
| `delete_project` | Remove an index when it is wrong or obsolete. |
| `detect_changes` | Detect git changes against the indexed repository. |
| `manage_adr` | Create or update architecture decision records. |
| `ingest_traces` | Add runtime trace facts to enrich the graph. |

## Query Examples

Use `query_graph` for exact repository questions after checking `get_graph_schema`.

```sql
SELECT name, label, file_path, line_start
FROM symbols
WHERE name LIKE '%Auth%'
LIMIT 20;
```

```sql
SELECT source_qn, target_qn, relationship
FROM edges
WHERE relationship = 'calls'
LIMIT 20;
```

## Edge Types

Common edge names include `calls`, `imports`, `contains`, `defines`, and language-specific relationships discovered by the parser. Check `get_graph_schema` or sample `query_graph` rows before relying on an edge name in a new repo.

## Gotchas

- Graph context is an index, not a substitute for reading the file you will edit.
- Use `Grep`/`Glob`/file reads for configs, docs, generated files, lockfiles, and assets.
- Do not run graph SQL from memory. Inspect the schema first.
- If graph results look stale, run `detect_changes` or reindex.
- For huge logs and non-code blobs, switch to RLM MCP tools such as `rlm_scan`, `rlm_peek`, and `rlm_chunk`.
- Keep edits scoped and verify with the repo's existing tests or the smallest meaningful smoke check.
