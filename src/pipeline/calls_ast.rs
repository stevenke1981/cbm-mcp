use crate::pipeline::registry::{CallResolution, CallTargetKind, FileCallResolver};
use crate::store::{Edge, Symbol};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Parser, Query, QueryCursor};

/// AST-aware CALLS using tree-sitter; returns edges with high confidence when parsing succeeds.
pub fn resolve_calls_ast(
    language: &str,
    symbols: &[Symbol],
    content: &str,
    resolver: &mut FileCallResolver<'_>,
) -> Vec<Edge> {
    let Some(lang) = language_to_tree_sitter(language) else {
        return Vec::new();
    };
    let Some(query_src) = call_query_for(language) else {
        return Vec::new();
    };

    let mut parser = Parser::new();
    if parser.set_language(&lang).is_err() {
        return Vec::new();
    }
    let tree = match parser.parse(content, None) {
        Some(t) => t,
        None => return Vec::new(),
    };
    let Ok(query) = Query::new(&lang, query_src) else {
        return Vec::new();
    };

    let mut cursor = QueryCursor::new();
    let mut edges = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let functions: Vec<&Symbol> = symbols.iter().filter(|s| s.label == "Function").collect();

    for sym in functions {
        let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());
        while let Some(m) = matches.next() {
            let mut callee = String::new();
            let mut line = 0usize;
            let mut kind = CallTargetKind::FreeFunction;
            for cap in m.captures {
                let name = query.capture_names()[cap.index as usize];
                match name {
                    "callee" | "scoped" => {
                        callee = cap
                            .node
                            .utf8_text(content.as_bytes())
                            .unwrap_or("")
                            .to_string();
                        line = cap.node.start_position().row;
                        kind = CallTargetKind::FreeFunction;
                    }
                    "method" => {
                        callee = cap
                            .node
                            .utf8_text(content.as_bytes())
                            .unwrap_or("")
                            .to_string();
                        line = cap.node.start_position().row;
                        kind = CallTargetKind::Method;
                    }
                    _ => {}
                }
            }
            if callee.is_empty() || is_noise_callee(language, &callee) || callee == sym.name {
                continue;
            }
            let call_line = (line + 1) as i64;
            if call_line < sym.line_start || call_line > sym.line_end {
                continue;
            }
            if let Some(res) = resolver.resolve_kind(&callee, kind) {
                push_edge(&mut edges, &mut seen, sym, &res, "ast");
            }
        }
    }
    edges
}

fn language_to_tree_sitter(language: &str) -> Option<Language> {
    Some(match language {
        "rust" => tree_sitter_rust::LANGUAGE.into(),
        "python" => tree_sitter_python::LANGUAGE.into(),
        "javascript" | "jsx" => tree_sitter_javascript::LANGUAGE.into(),
        "typescript" | "tsx" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        "go" => tree_sitter_go::LANGUAGE.into(),
        "java" => tree_sitter_java::LANGUAGE.into(),
        _ => return None,
    })
}

fn call_query_for(language: &str) -> Option<&'static str> {
    Some(match language {
        "rust" => r#"
(call_expression
  function: (identifier) @callee)
(call_expression
  function: (field_expression
    field: (field_identifier) @method))
(call_expression
  function: (scoped_identifier
    name: (identifier) @scoped))
"#,
        "python" => r#"
(call
  function: (identifier) @callee)
(call
  function: (attribute
    attribute: (identifier) @method))
"#,
        "javascript" | "jsx" | "typescript" | "tsx" => r#"
(call_expression
  function: (identifier) @callee)
(call_expression
  function: (member_expression
    property: (property_identifier) @method))
"#,
        "go" => r#"
(call_expression
  function: (identifier) @callee)
(call_expression
  function: (selector_expression
    field: (field_identifier) @method))
"#,
        "java" => r#"
(method_invocation
  name: (identifier) @callee)
(method_invocation
  object: (_)
  name: (identifier) @method)
"#,
        _ => return None,
    })
}

fn is_noise_callee(language: &str, name: &str) -> bool {
    match language {
        "rust" => matches!(
            name,
            "if" | "for" | "while" | "match" | "return" | "let" | "loop" | "move" | "async"
                | "await"
        ),
        "python" => matches!(name, "if" | "for" | "while" | "return" | "print" | "len"),
        "javascript" | "jsx" | "typescript" | "tsx" => {
            matches!(name, "if" | "for" | "while" | "return" | "console" | "require")
        }
        _ => false,
    }
}

fn push_edge(
    edges: &mut Vec<Edge>,
    seen: &mut std::collections::HashSet<(String, String)>,
    caller: &Symbol,
    res: &CallResolution,
    method: &str,
) {
    if res.qn == caller.qualified_name {
        return;
    }
    let key = (caller.qualified_name.clone(), res.qn.clone());
    if seen.insert(key) {
        edges.push(Edge {
            src_qn: caller.qualified_name.clone(),
            dst_qn: res.qn.clone(),
            edge_type: "CALLS".into(),
            properties_json: Some(format!(
                r#"{{"confidence":"{}","method":"{method}","strategy":"{}","score":{:.2}}}"#,
                res.band, res.strategy, res.confidence
            )),
        });
    }
}