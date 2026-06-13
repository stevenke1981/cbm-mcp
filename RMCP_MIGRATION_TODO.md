# Official rmcp MCP Server Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the custom CBM JSON-RPC/stdio implementation with the official Rust MCP SDK while preserving all graph behavior, tool contracts, installation paths, and release usability.

**Architecture:** Keep the existing graph engine and `ToolHandler` as domain logic. Add a thin typed `rmcp` adapter that owns protocol negotiation, stdio framing, tool routing, input schemas, cancellation, and MCP errors. Remove the custom transport only after process-level parity tests pass.

**Tech Stack:** Rust 2021, `rmcp 1.7.0`, Tokio, Serde, Schemars, SQLite, tree-sitter, existing CBM graph modules.

---

## Status and hard decisions

Implementation snapshot (2026-06-13): official `rmcp` stdio, `ServerHandler`,
typed Schemars inputs, 14-tool router, SDK-generated schemas, protocol/process
tests, and release artifact smoke are complete. Request cancellation propagation,
optional resources, and progress notifications remain open below.

- [x] Use the current official stable SDK: `rmcp = 1.7.0`.
- [x] Pin the exact SDK version in `Cargo.lock`; update it only in a dedicated dependency change.
- [x] Use `rmcp` server and stdio transport APIs. Do not maintain a second JSON-RPC parser or framing implementation.
- [x] Keep the binary name `cbm` and MCP server name `codebase-memory-mcp`.
- [x] Keep CBM independent from `D:\rlm-mcp`; do not add RLM tools or dependencies.
- [x] Keep `.codebase-memory/graph.db.zst` and the existing SQLite/cache model as the persistent index contract.
- [x] Preserve the current 14 graph tool names and response payloads unless a documented MCP compliance issue requires a versioned change.
- [x] Write all logs and diagnostics to stderr. Stdout is reserved exclusively for MCP stdio frames.
- [x] Do not advertise capabilities that are not implemented and tested.

## Current implementation to replace

- `src/mcp/server.rs` manually parses requests, hard-codes protocol version `2024-11-05`, dispatches methods, and formats responses.
- `src/mcp/transport.rs` manually reads/writes newline and `Content-Length` framed messages.
- `src/mcp/tool_specs.rs` manually assembles JSON Schema as `serde_json::Value`.
- `src/mcp/tools.rs` contains reusable graph-domain dispatch and should remain the behavioral compatibility layer during migration.
- Existing protocol coverage lives in `tests/mcp_jsonrpc_test.rs`, `tests/mcp_process_test.rs`, and `tests/mcp_tool_schema_test.rs`.

## Target file map

- Modify: `Cargo.toml` - add official SDK/schema dependencies and Tokio features.
- Modify: `src/main.rs` - run the MCP server from an async Tokio main without writing non-protocol output to stdout.
- Replace: `src/mcp/server.rs` - implement `ServerHandler`, server metadata, capabilities, lifecycle, and stdio serving.
- Create: `src/mcp/router.rs` - define the `#[tool_router]` adapter and typed tool entrypoints.
- Create: `src/mcp/params.rs` - define `Deserialize + JsonSchema` input structs for all 14 tools.
- Modify: `src/mcp/tools.rs` - expose typed/domain calls or retain a single compatibility dispatcher without protocol concerns.
- Modify: `src/mcp/mod.rs` - export the new server/router/parameter modules.
- Delete after parity: `src/mcp/transport.rs` and `src/mcp/tool_specs.rs`.
- Create: `tests/rmcp_protocol_test.rs` - SDK-level initialize, negotiation, list, call, cancellation, and error tests.
- Modify: `tests/mcp_process_test.rs` - test the compiled binary over stdio.
- Modify: `tests/mcp_tool_schema_test.rs` - compare SDK-generated schemas with checked-in public contracts.
- Modify: `scripts/smoke-release-artifact.ps1` - test the extracted binary as an MCP server.
- Modify: `scripts/smoke-release-artifact.sh` - mirror the release MCP smoke on Unix.
- Modify: `README.md` and `packaging/mcp/README.md` - document official SDK transport and troubleshooting.

## P0 Task 1 - Add the SDK dependency baseline

- [ ] Add these dependencies to `Cargo.toml`:

```toml
rmcp = { version = "1.7.0", features = ["server", "transport-io", "macros"] }
schemars = "1"
tokio = { version = "1", features = ["full"] }
```

- [ ] Remove obsolete Tokio feature declarations rather than keeping duplicate `tokio` entries.
- [ ] Run `cargo check` and record any SDK API adjustment in this file before broad code changes.
- [ ] Run `cargo tree -i rmcp` and verify exactly one `rmcp` version is selected.
- [ ] Commit only the dependency baseline and lockfile update.

Acceptance:

- `cargo check` succeeds with `rmcp 1.7.0`.
- No custom server behavior changes in this task.

## P0 Task 2 - Lock the public tool contract before migration

- [ ] Update the schema snapshot test to assert exactly these tools:
  - `index_repository`
  - `index_status`
  - `search_graph`
  - `trace_path`
  - `get_code_snippet`
  - `get_graph_schema`
  - `get_architecture`
  - `query_graph`
  - `search_code`
  - `list_projects`
  - `delete_project`
  - `detect_changes`
  - `manage_adr`
  - `ingest_traces`
- [ ] Snapshot each tool name, description, required fields, defaults, enums, and top-level `type: object`.
- [ ] Add a response compatibility fixture for one read tool, one write/index tool, one pagination tool, and one validation failure.
- [ ] Run the tests against the current custom server first; they must pass before adapter work begins.

Acceptance:

- A tool rename, missing required field, changed default, or response envelope drift fails CI.
- The snapshot is a public contract, not generated documentation that silently overwrites itself.

## P0 Task 3 - Define typed tool inputs with Schemars

- [ ] Create one Rust input struct per tool in `src/mcp/params.rs`.
- [ ] Derive `Debug`, `Clone`, `Deserialize`, and `JsonSchema` for every input type.
- [ ] Use `#[serde(default)]` and Schemars metadata so runtime defaults and advertised defaults are identical.
- [ ] Use enums for closed sets such as index mode, ADR mode, trace direction, and search mode.
- [ ] Preserve optional versus required fields from the locked schema tests.
- [ ] Reject unknown or ill-typed fields with an MCP invalid-params error instead of silently coercing them.
- [ ] Add unit tests that deserialize minimum valid input, full input, invalid enum input, and missing required input for every parameter family.

Acceptance:

- No hand-written tool input JSON Schema remains.
- SDK-generated schema passes the locked public schema tests.

## P0 Task 4 - Add the rmcp tool router adapter

- [ ] Create `src/mcp/router.rs` with `#[tool_router]` and one typed method per public tool.
- [ ] Keep graph algorithms, storage, indexing, and watcher logic outside the router.
- [ ] Convert typed parameters into existing domain calls in `ToolHandler`.
- [ ] Return structured MCP content with the existing JSON payload serialized as text for backward compatibility.
- [ ] Centralize response conversion so every tool consistently sets content and `isError` semantics.
- [ ] Prevent a panic in one tool from terminating the stdio server; return an internal tool failure and log details to stderr.

Acceptance:

- `tools/list` is generated from the router and Schemars types.
- All 14 tools reach the existing domain implementation through typed entrypoints.
- No protocol request parsing exists in `ToolHandler`.

## P0 Task 5 - Implement ServerHandler and capability negotiation

- [ ] Replace the manual initialize response with an official `ServerHandler` implementation.
- [ ] Return server info with:
  - name: `codebase-memory-mcp`
  - version: `env!("CARGO_PKG_VERSION")`
  - concise instructions describing index, search, trace, and query workflow
- [ ] Declare tools capability through the SDK builder.
- [ ] Set `listChanged` only if the implementation can actually emit tool-list change notifications.
- [ ] Let `rmcp` negotiate the protocol version; remove the hard-coded protocol version from application code.
- [ ] Do not advertise resources, prompts, logging, completion, or subscriptions until each has an implementation and conformance test.
- [ ] Add an initialize test using at least the newest protocol supported by `rmcp` and one older client version accepted by the SDK.

Acceptance:

- Initialization succeeds through an official `rmcp` client/service test.
- Unsupported capabilities are absent from the response.
- Protocol version selection is owned by the SDK.

## P0 Task 6 - Move stdio serving and shutdown to Tokio/rmcp

- [ ] Make the executable entrypoint async with `#[tokio::main]`.
- [ ] Serve the CBM handler with `rmcp` stdio transport and wait for service completion.
- [ ] Treat stdin EOF as graceful shutdown.
- [ ] Stop the watcher and release graph resources when the MCP service exits.
- [ ] Propagate Ctrl+C/shutdown into the existing `Shutdown` abstraction.
- [ ] Ensure indexing work that blocks a Tokio worker is moved to `spawn_blocking` or an owned worker thread.
- [ ] Add a process test proving the server exits after stdin closes and leaves no watcher thread running.

Acceptance:

- OpenCode, Codex, and a process test can initialize and list tools over stdio.
- The process shuts down cleanly on EOF and Ctrl+C.
- Stdout contains no banners, logs, progress messages, or panic text.

## P0 Task 7 - Implement spec-correct error boundaries

- [ ] Map malformed tool arguments to JSON-RPC/MCP invalid params.
- [ ] Let the SDK return method-not-found for unsupported protocol methods.
- [ ] Return unknown tool names using the SDK's tool-call error contract.
- [ ] Return domain failures such as missing project/session, invalid path, unsafe SQL, or index failure as tool execution errors with `isError: true` when the request itself is valid.
- [ ] Keep internal error details in stderr logs; return stable, actionable messages without Rust backtraces or secrets.
- [ ] Distinguish cancellation from internal failure.
- [ ] Add tests for parse failure, invalid params, unknown method, unknown tool, domain error, cancellation, and unexpected internal error.

Acceptance:

- Protocol errors and tool execution errors are not conflated.
- Every failure produces a valid MCP response and the server remains usable afterward.

## P0 Task 8 - Cancellation and long-running indexing

- [ ] Read the request cancellation signal/context supplied by `rmcp` for long-running tool calls.
- [ ] Thread cancellation into `index_repository`, incremental indexing, trace ingestion, and other long operations.
- [ ] Stop scheduling new pipeline passes after cancellation.
- [ ] Roll back or preserve the previous valid graph when cancellation interrupts a bulk transaction.
- [ ] Emit no late success response after a request has been cancelled.
- [ ] Add a fixture that starts a deliberately slow index, sends cancellation, and verifies bounded shutdown time and database integrity.

Acceptance:

- Cancellation does not corrupt `.codebase-memory` state.
- A cancelled request releases locks and permits a later index/query call.

## P0 Task 9 - Remove the custom protocol stack

- [ ] Delete `src/mcp/transport.rs` after SDK process tests pass on Windows and Unix CI.
- [ ] Delete `src/mcp/tool_specs.rs` after generated schemas pass snapshots.
- [ ] Remove manual JSON-RPC constants, response formatting, initialize dispatch, ping dispatch, and `tools/list` dispatch.
- [ ] Keep only domain-level JSON conversion that is part of the public tool result.
- [ ] Run `rg` for `Content-Length`, `protocolVersion`, `tools/list`, `format_response`, and `format_error`; remaining matches must be tests/docs or SDK usage.

Acceptance:

- There is exactly one MCP protocol implementation: `rmcp`.
- The CBM codebase does not parse raw JSON-RPC requests itself.

## P0 Task 10 - Protocol and release artifact verification

- [ ] Add an in-process SDK conformance test for initialize, ping, tools/list, and tools/call.
- [ ] Keep a compiled-process stdio test to catch stdout pollution and executable wiring errors.
- [ ] Update release smoke scripts to run initialize and tools/list against the extracted `cbm`/`cbm.exe`, not `target/release`.
- [ ] Verify install dry-run produces the stable installed binary path and correct OpenCode/Codex command.
- [ ] Add Windows JSONC and Codex TOML configuration smoke tests.
- [ ] Test a real index followed by `search_graph`, `trace_path`, and `query_graph` through MCP.

Acceptance:

- A release archive is proven usable without Cargo or a source checkout.
- OpenCode and Codex connect using the installed stable path.

## P1 Task 11 - Optional MCP resources, only after tools are stable

- [ ] Write a short design decision before adding resources.
- [ ] If resources are useful, start with read-only URIs for graph schema, architecture summary, and project status.
- [ ] Define URI templates, MIME types, size limits, and stale-index behavior.
- [ ] Add list/read/resource-template tests before enabling the resources capability.
- [ ] Do not expose the SQLite database file or unrestricted local files as resources.

Acceptance:

- The resources capability is absent until all resource operations are implemented.
- Resource reads enforce project boundaries and output limits.

## P1 Task 12 - Auto-sync and notifications

- [ ] Keep auto-sync opt-in and preserve current watcher configuration.
- [ ] Integrate watcher startup/stop with the `rmcp` service lifetime.
- [ ] Debounce file events and avoid concurrent indexes of the same project.
- [ ] Consider resource/tool change notifications only when a client-visible contract actually changes.
- [ ] Add tests for stdin close during watcher activity and cancellation during incremental indexing.

Acceptance:

- Auto-sync cannot outlive the MCP process.
- Repeated file events produce bounded, serialized indexing work.

## P1 Task 13 - Documentation and migration notes

- [ ] Update `README.md` architecture and installation sections to state that CBM uses the official Rust MCP SDK.
- [ ] Document that MCP runs on stdio and stdout must remain protocol-only.
- [ ] Document cache/index location and watcher behavior.
- [ ] Add troubleshooting for client timeout, stale command path, stdout pollution, invalid config, and permission errors.
- [ ] Update `PARITY_MATRIX.md` to mark official SDK migration separately from graph feature parity.
- [ ] Add a changelog/release note warning maintainers not to restore the custom transport.

Acceptance:

- Humans and agents can install, diagnose, and verify the MCP server without reading Rust source.

## Required verification before completion

- [ ] `cargo fmt --check`
- [ ] `cargo test --all-targets`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo build --release`
- [ ] `cargo tree -i rmcp` shows `rmcp 1.7.0`
- [ ] `cargo test --test rmcp_protocol_test`
- [ ] `cargo test --test mcp_process_test`
- [ ] `cargo test --test mcp_tool_schema_test`
- [ ] `powershell -ExecutionPolicy Bypass -File .\scripts\smoke-quality-gates.ps1 -SkipBuild`
- [ ] `powershell -ExecutionPolicy Bypass -File .\scripts\smoke-release-artifact.ps1 -SkipBuild`
- [ ] Fresh OpenCode connection smoke using the installed binary
- [ ] Fresh Codex connection smoke using the installed binary
- [ ] `git status --short` contains no generated cache, local config, or test database files

## Completion definition

The migration is complete only when the custom transport and manual protocol dispatcher are gone, all 14 tools retain their locked contracts, SDK negotiation and errors pass conformance tests, long-running indexes cancel safely, and release-installed binaries connect from OpenCode and Codex without recompilation.
