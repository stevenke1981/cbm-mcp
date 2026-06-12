//! Cross-file LSP-style call resolution (reference `pass_lsp_cross.c` parity slice).
//!
//! In-process type-aware resolver — not an external language-server subprocess.
//! Phase 1: Python imported-class method dispatch (`Greeter().greet()`).

use crate::pipeline::import_map::ImportMap;
use crate::pipeline::registry::{confidence_band, CallResolution};
use crate::store::{Edge, SourceFile, Symbol};
use std::collections::{HashMap, HashSet};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Parser, Query, QueryCursor};

const LSP_CROSS_CONFIDENCE: f64 = 0.85;

pub fn resolve_cross_file_calls(symbols: &[Symbol], files: &[SourceFile]) -> Vec<Edge> {
    let mut edges = Vec::new();
    let class_index = build_class_index(symbols);
    let methods_by_file = build_methods_by_file(symbols);

    for file in files {
        if file.language != "python" {
            continue;
        }
        let file_syms: Vec<&Symbol> = symbols
            .iter()
            .filter(|s| s.file_path == file.path && s.label == "Function")
            .collect();
        if file_syms.is_empty() {
            continue;
        }
        let imports = ImportMap::parse(&file.path, &file.language, &file.content);
        let bindings = infer_python_type_bindings(&file.content, &imports);
        edges.extend(resolve_python_attribute_calls(
            &file.path,
            &file.content,
            &file_syms,
            &imports,
            &bindings,
            &class_index,
            &methods_by_file,
        ));
    }
    edges
}

fn build_class_index(symbols: &[Symbol]) -> HashMap<String, Vec<ClassEntry>> {
    let mut index: HashMap<String, Vec<ClassEntry>> = HashMap::new();
    for sym in symbols {
        if sym.label != "Class" {
            continue;
        }
        index
            .entry(sym.name.clone())
            .or_default()
            .push(ClassEntry {
                file: sym.file_path.clone(),
                line: sym.line_start,
            });
    }
    index
}

fn build_methods_by_file(symbols: &[Symbol]) -> HashMap<String, Vec<MethodEntry>> {
    let mut by_file: HashMap<String, Vec<MethodEntry>> = HashMap::new();
    for sym in symbols {
        if sym.label != "Function" {
            continue;
        }
        by_file
            .entry(sym.file_path.clone())
            .or_default()
            .push(MethodEntry {
                name: sym.name.clone(),
                qn: sym.qualified_name.clone(),
                line: sym.line_start,
            });
    }
    for methods in by_file.values_mut() {
        methods.sort_by_key(|m| m.line);
    }
    by_file
}

#[derive(Debug, Clone)]
struct ClassEntry {
    file: String,
    line: i64,
}

#[derive(Debug, Clone)]
struct MethodEntry {
    name: String,
    qn: String,
    line: i64,
}

fn infer_python_type_bindings(content: &str, imports: &ImportMap) -> HashMap<String, String> {
    let mut bindings = HashMap::new();
    for local in imports.bindings.keys() {
        bindings.insert(local.clone(), local.clone());
    }
    let assign_re =
        regex::Regex::new(r"(?m)^\s*(\w+)\s*=\s*(\w+)\s*\(").expect("assign regex");
    for cap in assign_re.captures_iter(content) {
        let var = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let class_name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        if !var.is_empty() && !class_name.is_empty() {
            bindings.insert(var.to_string(), class_name.to_string());
        }
    }
    bindings
}

fn resolve_python_attribute_calls(
    file_path: &str,
    content: &str,
    functions: &[&Symbol],
    imports: &ImportMap,
    bindings: &HashMap<String, String>,
    class_index: &HashMap<String, Vec<ClassEntry>>,
    methods_by_file: &HashMap<String, Vec<MethodEntry>>,
) -> Vec<Edge> {
    let lang: Language = tree_sitter_python::LANGUAGE.into();
    let mut parser = Parser::new();
    if parser.set_language(&lang).is_err() {
        return Vec::new();
    }
    let tree = match parser.parse(content, None) {
        Some(t) => t,
        None => return Vec::new(),
    };
    let query_src = r#"
(call
  function: (attribute
    object: (_) @recv
    attribute: (identifier) @method))
"#;
    let Ok(query) = Query::new(&lang, query_src) else {
        return Vec::new();
    };

    let mut cursor = QueryCursor::new();
    let mut edges = Vec::new();
    let mut seen = HashSet::new();

    for caller in functions {
        let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());
        while let Some(m) = matches.next() {
            let mut recv_node = None;
            let mut method_name = String::new();
            let mut call_line = 0i64;
            for cap in m.captures {
                let name = query.capture_names()[cap.index as usize];
                if name == "recv" {
                    recv_node = Some(cap.node);
                } else if name == "method" {
                    method_name = cap
                        .node
                        .utf8_text(content.as_bytes())
                        .unwrap_or("")
                        .to_string();
                    call_line = (cap.node.start_position().row + 1) as i64;
                }
            }
            if method_name.is_empty() || call_line < caller.line_start || call_line > caller.line_end {
                continue;
            }
            let Some(recv) = recv_node else {
                continue;
            };
            let Some(class_name) = infer_receiver_class(recv, content, bindings, imports) else {
                continue;
            };
            let Some(res) = resolve_class_method(
                &class_name,
                &method_name,
                file_path,
                imports,
                class_index,
                methods_by_file,
            ) else {
                continue;
            };
            push_lsp_edge(&mut edges, &mut seen, caller, &res);
        }
    }
    edges
}

fn infer_receiver_class(
    recv: tree_sitter::Node,
    content: &str,
    bindings: &HashMap<String, String>,
    imports: &ImportMap,
) -> Option<String> {
    match recv.kind() {
        "identifier" => {
            let name = recv.utf8_text(content.as_bytes()).ok()?;
            if let Some(class) = bindings.get(name) {
                return Some(class.clone());
            }
            if imports.bindings.contains_key(name) {
                return Some(name.to_string());
            }
            None
        }
        "call" => {
            let func = recv.child_by_field_name("function")?;
            if func.kind() == "identifier" {
                return func.utf8_text(content.as_bytes()).ok().map(str::to_string);
            }
            None
        }
        _ => None,
    }
}

fn resolve_class_method(
    class_name: &str,
    method_name: &str,
    caller_file: &str,
    imports: &ImportMap,
    class_index: &HashMap<String, Vec<ClassEntry>>,
    methods_by_file: &HashMap<String, Vec<MethodEntry>>,
) -> Option<CallResolution> {
    let candidates = class_index.get(class_name)?;
    let scoped: Vec<&ClassEntry> = if let Some(target) = imports.bindings.get(class_name) {
        candidates
            .iter()
            .filter(|c| import_map::path_matches(&c.file, target))
            .collect()
    } else {
        candidates
            .iter()
            .filter(|c| imports.is_reachable(&c.file) || c.file == caller_file)
            .collect()
    };
    if scoped.len() != 1 {
        return None;
    }
    let class_entry = scoped[0];
    let methods = methods_by_file.get(&class_entry.file)?;
    let class_methods: Vec<&MethodEntry> = methods
        .iter()
        .filter(|m| m.line > class_entry.line && m.name == method_name)
        .collect();
    if class_methods.len() != 1 {
        return None;
    }
    Some(CallResolution {
        qn: class_methods[0].qn.clone(),
        strategy: "lsp_cross".into(),
        confidence: LSP_CROSS_CONFIDENCE,
        band: confidence_band(LSP_CROSS_CONFIDENCE).to_string(),
    })
}

mod import_map {
    pub fn path_matches(file: &str, module: &str) -> bool {
        let norm_file = file.replace('\\', "/");
        let norm_mod = module.replace('\\', "/");
        norm_file == norm_mod
            || norm_file.ends_with(&norm_mod)
            || norm_mod.ends_with(&norm_file)
            || norm_file.strip_suffix(".py").is_some_and(|s| norm_mod.starts_with(s))
    }
}

fn push_lsp_edge(
    edges: &mut Vec<Edge>,
    seen: &mut HashSet<(String, String)>,
    caller: &Symbol,
    res: &CallResolution,
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
                r#"{{"confidence":"{}","method":"lsp_cross","strategy":"{}","score":{:.2}}}"#,
                res.band, res.strategy, res.confidence
            )),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol_id::qualified_name;

    fn sym(file: &str, label: &str, name: &str, line: i64) -> Symbol {
        Symbol {
            qualified_name: qualified_name(file, label, name, line),
            name: name.into(),
            label: label.into(),
            file_path: file.into(),
            line_start: line,
            line_end: line + 5,
            signature: None,
            properties_json: None,
        }
    }

    #[test]
    fn resolves_imported_class_method() {
        let symbols = vec![
            sym("main.py", "Function", "main", 3),
            sym("greeter.py", "Class", "Greeter", 1),
            sym("greeter.py", "Function", "greet", 2),
        ];
        let files = vec![SourceFile {
            path: "main.py".into(),
            language: "python".into(),
            content: "from greeter import Greeter\n\ndef main():\n    Greeter().greet()\n"
                .into(),
            line_count: 4,
        }];
        let edges = resolve_cross_file_calls(&symbols, &files);
        assert_eq!(edges.len(), 1);
        assert!(edges[0].dst_qn.starts_with("greeter.py::"));
        assert!(edges[0].dst_qn.contains("greet"));
        assert!(
            edges[0]
                .properties_json
                .as_ref()
                .is_some_and(|p| p.contains("lsp_cross"))
        );
    }
}