//! Cross-file CALLS via import map (registry P0 slice).

mod support;

use codebase_memory_mcp::discover::IndexMode;
use codebase_memory_mcp::pipeline::Pipeline;
use codebase_memory_mcp::store::Store;
use support::isolated_cache;
use tempfile::TempDir;

fn calls_edges(store: &Store) -> Vec<codebase_memory_mcp::store::Edge> {
    store
        .list_edges_limited(500)
        .unwrap()
        .into_iter()
        .filter(|e| e.edge_type == "CALLS")
        .collect()
}

#[test]
fn python_pipeline_resolves_imported_cross_file_helper() {
    let (_guard, _cache, _) = isolated_cache();
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("main.py"),
        "from utils import helper\n\ndef main():\n    helper()\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("utils.py"), "def helper():\n    pass\n").unwrap();
    std::fs::write(dir.path().join("decoy.py"), "def helper():\n    pass\n").unwrap();

    let pipeline = Pipeline::new(IndexMode::Full);
    let index = pipeline.run(dir.path(), Some("py-import-calls")).unwrap();
    let store = Store::open(&index.project).unwrap();

    let main_calls: Vec<_> = calls_edges(&store)
        .into_iter()
        .filter(|e| e.src_qn.contains("main.py::Function::main@"))
        .collect();
    assert_eq!(main_calls.len(), 1, "expected one CALLS from main: {main_calls:?}");
    assert!(
        main_calls[0].dst_qn.starts_with("utils.py::"),
        "should resolve to utils.helper, got {}",
        main_calls[0].dst_qn
    );
    assert!(
        main_calls[0]
            .properties_json
            .as_ref()
            .is_some_and(|p| p.contains("import_binding") || p.contains("ast")),
        "expected import-aware metadata"
    );

    let _ = codebase_memory_mcp::store::delete_project_db(&index.project);
}

#[test]
fn python_pipeline_resolves_imported_class_method_via_lsp_cross() {
    let (_guard, _cache, _) = isolated_cache();
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("main.py"),
        "from greeter import Greeter\n\ndef main():\n    Greeter().greet()\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("greeter.py"),
        "class Greeter:\n    def greet(self):\n        return 'hi'\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("decoy.py"), "def greet():\n    pass\n").unwrap();

    let pipeline = Pipeline::new(IndexMode::Full);
    let index = pipeline.run(dir.path(), Some("py-lsp-cross")).unwrap();
    let store = Store::open(&index.project).unwrap();

    let main_calls: Vec<_> = calls_edges(&store)
        .into_iter()
        .filter(|e| e.src_qn.contains("main.py::Function::main@"))
        .collect();
    assert!(
        main_calls.iter().any(|e| e.dst_qn.starts_with("greeter.py::")),
        "expected CALLS to greeter.py method, got {main_calls:?}"
    );
    assert!(
        main_calls
            .iter()
            .any(|e| e.dst_qn.contains("greet") && e.dst_qn.starts_with("greeter.py::")),
        "expected CALLS to greeter.greet, got {main_calls:?}"
    );

    let _ = codebase_memory_mcp::store::delete_project_db(&index.project);
}

#[test]
fn javascript_pipeline_resolves_imported_class_method_via_lsp_cross() {
    let (_guard, _cache, _) = isolated_cache();
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("main.js"),
        "import { Greeter } from './greeter';\n\nfunction main() {\n  new Greeter().greet();\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("greeter.js"),
        "export class Greeter {\n  greet() { return 'hi'; }\n}\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("decoy.js"), "export function greet() {}\n").unwrap();

    let pipeline = Pipeline::new(IndexMode::Full);
    let index = pipeline.run(dir.path(), Some("js-lsp-cross")).unwrap();
    let store = Store::open(&index.project).unwrap();

    let main_calls: Vec<_> = calls_edges(&store)
        .into_iter()
        .filter(|e| e.src_qn.contains("main.js::Function::main@"))
        .collect();
    assert!(
        main_calls
            .iter()
            .any(|e| e.dst_qn.contains("greet") && e.dst_qn.starts_with("greeter.js::")),
        "expected CALLS to greeter.greet, got {main_calls:?}"
    );

    let _ = codebase_memory_mcp::store::delete_project_db(&index.project);
}