//! JSON-RPC 2.0 protocol parity with reference `test_mcp.c` expectations.

use codebase_memory_mcp::mcp::McpServer;
use serde_json::{json, Value};

fn server() -> McpServer {
    std::env::set_var("CBM_WATCHER", "0");
    std::env::set_var("CBRLM_WATCHER", "0");
    McpServer::new()
}

fn parse_response(body: &str) -> Value {
    serde_json::from_str(body).expect("valid JSON-RPC response")
}

#[test]
fn invalid_json_returns_parse_error() {
    let resp = server()
        .handle_message("this is not json at all")
        .unwrap()
        .expect("response");
    let value = parse_response(&resp);
    assert_eq!(value.get("id"), Some(&Value::Null));
    assert_eq!(
        value.pointer("/error/code").and_then(|v| v.as_i64()),
        Some(-32700)
    );
    assert_eq!(
        value.pointer("/error/message").and_then(|v| v.as_str()),
        Some("Parse error")
    );
}

#[test]
fn empty_object_without_method_returns_parse_error() {
    let resp = server().handle_message("{}").unwrap().expect("response");
    let value = parse_response(&resp);
    assert_eq!(
        value.pointer("/error/code").and_then(|v| v.as_i64()),
        Some(-32700)
    );
}

#[test]
fn unknown_method_returns_method_not_found() {
    let req = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "unknown/method"
    });
    let resp = server()
        .handle_message(&req.to_string())
        .unwrap()
        .expect("response");
    let value = parse_response(&resp);
    assert_eq!(value.get("id"), Some(&Value::from(3)));
    assert_eq!(
        value.pointer("/error/code").and_then(|v| v.as_i64()),
        Some(-32601)
    );
}

#[test]
fn notification_without_id_has_no_response() {
    let req = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    let resp = server().handle_message(&req.to_string()).unwrap();
    assert!(resp.is_none());
}

#[test]
fn tools_call_unknown_tool_uses_is_error_result() {
    let req = json!({
        "jsonrpc": "2.0",
        "id": 12,
        "method": "tools/call",
        "params": {
            "name": "nonexistent_tool",
            "arguments": {}
        }
    });
    let resp = server()
        .handle_message(&req.to_string())
        .unwrap()
        .expect("response");
    let value = parse_response(&resp);
    assert!(value.get("result").is_some());
    assert!(value.get("error").is_none());
    assert_eq!(
        value.pointer("/result/isError").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn ping_returns_empty_result() {
    let req = json!({
        "jsonrpc": "2.0",
        "id": 99,
        "method": "ping"
    });
    let resp = server()
        .handle_message(&req.to_string())
        .unwrap()
        .expect("response");
    let value = parse_response(&resp);
    assert_eq!(value.get("id"), Some(&Value::from(99)));
    assert_eq!(value.get("result"), Some(&json!({})));
}