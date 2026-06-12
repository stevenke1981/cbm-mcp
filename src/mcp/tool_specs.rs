use serde_json::{json, Value};

macro_rules! tool_from_spec {
    ($path:literal) => {{
        let spec: Value = serde_json::from_str(include_str!($path)).expect("invalid tool spec JSON");
        json!({
            "name": spec["name"],
            "description": spec["description"],
            "inputSchema": spec["inputSchema"]
        })
    }};
}

/// MCP tool list — schemas sourced from `mcps/codebase-memory-mcp/tools/*.json`.
pub fn tool_definitions() -> Vec<Value> {
    vec![
        tool_from_spec!("../../mcps/codebase-memory-mcp/tools/index_repository.json"),
        tool_from_spec!("../../mcps/codebase-memory-mcp/tools/index_status.json"),
        tool_from_spec!("../../mcps/codebase-memory-mcp/tools/search_graph.json"),
        tool_from_spec!("../../mcps/codebase-memory-mcp/tools/trace_path.json"),
        tool_from_spec!("../../mcps/codebase-memory-mcp/tools/get_code_snippet.json"),
        tool_from_spec!("../../mcps/codebase-memory-mcp/tools/get_graph_schema.json"),
        tool_from_spec!("../../mcps/codebase-memory-mcp/tools/get_architecture.json"),
        tool_from_spec!("../../mcps/codebase-memory-mcp/tools/search_code.json"),
        tool_from_spec!("../../mcps/codebase-memory-mcp/tools/list_projects.json"),
        tool_from_spec!("../../mcps/codebase-memory-mcp/tools/delete_project.json"),
        tool_from_spec!("../../mcps/codebase-memory-mcp/tools/query_graph.json"),
        tool_from_spec!("../../mcps/codebase-memory-mcp/tools/detect_changes.json"),
        tool_from_spec!("../../mcps/codebase-memory-mcp/tools/manage_adr.json"),
        tool_from_spec!("../../mcps/codebase-memory-mcp/tools/ingest_traces.json"),
    ]
}
