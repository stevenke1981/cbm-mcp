use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Per-file import bindings: local symbol name → resolved module path (repo-relative).
#[derive(Debug, Default, Clone)]
pub struct ImportMap {
    /// Imported bare name (e.g. `helper`) → target module path (`utils.py`).
    pub bindings: HashMap<String, String>,
    /// Module paths imported wholesale (e.g. `utils` from `import utils`).
    pub modules: Vec<String>,
}

impl ImportMap {
    pub fn parse(file_path: &str, language: &str, content: &str) -> Self {
        let mut map = ImportMap::default();
        let caller_dir = Path::new(file_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        match language {
            "python" => parse_python_imports(file_path, &caller_dir, content, &mut map),
            "rust" => parse_rust_imports(file_path, &caller_dir, content, &mut map),
            "javascript" | "typescript" | "tsx" | "jsx" => {
                parse_js_imports(file_path, &caller_dir, content, &mut map)
            }
            "go" => parse_go_imports(content, &mut map),
            _ => {}
        }
        map
    }

    pub fn is_reachable(&self, candidate_file: &str) -> bool {
        if self.bindings.is_empty() && self.modules.is_empty() {
            return false;
        }
        let norm = normalize_path(candidate_file);
        for module in &self.modules {
            if path_matches_module(&norm, module) {
                return true;
            }
        }
        for target in self.bindings.values() {
            if path_matches_module(&norm, target) {
                return true;
            }
        }
        false
    }

    pub fn target_files_for(&self, name: &str) -> Vec<String> {
        if let Some(target) = self.bindings.get(name) {
            return vec![target.clone()];
        }
        self.modules.clone()
    }
}

fn parse_python_imports(file_path: &str, caller_dir: &str, content: &str, map: &mut ImportMap) {
    let from_import =
        Regex::new(r"(?m)^\s*from\s+([\w.]+)\s+import\s+([\w.,\s]+)").unwrap();
    let plain_import = Regex::new(r"(?m)^\s*import\s+([\w.]+)").unwrap();

    for cap in from_import.captures_iter(content) {
        let module = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let names = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let target = resolve_python_module(file_path, caller_dir, module);
        for name in names.split(',') {
            let name = name.trim().split_whitespace().next().unwrap_or("").trim();
            if !name.is_empty() && name != "*" {
                map.bindings.insert(name.to_string(), target.clone());
            }
        }
        map.modules.push(target);
    }

    for cap in plain_import.captures_iter(content) {
        let module = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let target = resolve_python_module(file_path, caller_dir, module);
        let base = module.split('.').next_back().unwrap_or(module);
        map.bindings
            .entry(base.to_string())
            .or_insert_with(|| target.clone());
        map.modules.push(target);
    }
}

fn resolve_python_module(file_path: &str, caller_dir: &str, module: &str) -> String {
    let dotted = module.replace('.', "/");
    let candidates = [
        format!("{dotted}.py"),
        format!("{dotted}/__init__.py"),
    ];
    if let Some(hit) = candidates.iter().find(|c| !c.starts_with('/') && !c.contains("..")) {
        return hit.clone();
    }
    let _ = (file_path, caller_dir);
    candidates[0].clone()
}

fn parse_rust_imports(_file_path: &str, _caller_dir: &str, content: &str, map: &mut ImportMap) {
    let use_re = Regex::new(r"(?m)^\s*use\s+([\w:]+)(?:::\{([^}]+)\})?").unwrap();
    for cap in use_re.captures_iter(content) {
        let path = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        if let Some(items) = cap.get(2).map(|m| m.as_str()) {
            for item in items.split(',') {
                let name = item.trim();
                if !name.is_empty() {
                    map.bindings
                        .insert(name.to_string(), path_to_rs_module(path));
                }
            }
        } else {
            let simple = path.rsplit("::").next().unwrap_or(path);
            map.bindings
                .insert(simple.to_string(), path_to_rs_module(path));
        }
        map.modules.push(path_to_rs_module(path));
    }
}

fn path_to_rs_module(path: &str) -> String {
    let rel = path.replace("::", "/");
    if rel.ends_with(".rs") {
        rel
    } else {
        format!("{rel}.rs")
    }
}

fn parse_js_imports(file_path: &str, caller_dir: &str, content: &str, map: &mut ImportMap) {
    let named = Regex::new(
        r#"(?m)^\s*import\s+\{([^}]+)\}\s+from\s+['"]([^'"]+)['"]"#,
    )
    .unwrap();
    let default_import =
        Regex::new(r#"(?m)^\s*import\s+(\w+)\s+from\s+['"]([^'"]+)['"]"#).unwrap();
    let require_re = Regex::new(r#"(?m)(?:const|let|var)\s+(\w+)\s*=\s*require\(['"]([^'"]+)['"]\)"#)
        .unwrap();

    for cap in named.captures_iter(content) {
        let names = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let from = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let target = resolve_js_module(file_path, caller_dir, from);
        for name in names.split(',') {
            let name = name.trim();
            if !name.is_empty() {
                map.bindings.insert(name.to_string(), target.clone());
            }
        }
        map.modules.push(target);
    }

    for cap in default_import.captures_iter(content) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let from = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let target = resolve_js_module(file_path, caller_dir, from);
        if !name.is_empty() {
            map.bindings.insert(name.to_string(), target.clone());
        }
        map.modules.push(target);
    }

    for cap in require_re.captures_iter(content) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let from = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let target = resolve_js_module(file_path, caller_dir, from);
        if !name.is_empty() {
            map.bindings.insert(name.to_string(), target);
        }
    }
}

fn resolve_js_module(file_path: &str, caller_dir: &str, from: &str) -> String {
    if from.starts_with('.') {
        let base = if caller_dir.is_empty() {
            PathBuf::from(Path::new(file_path).parent().unwrap_or(Path::new(".")))
        } else {
            PathBuf::from(caller_dir)
        };
        let joined = base.join(from);
        let mut normalized = normalize_path(&joined.to_string_lossy());
        while normalized.contains("/./") {
            normalized = normalized.replace("/./", "/");
        }
        if !normalized.ends_with(".js")
            && !normalized.ends_with(".ts")
            && !normalized.ends_with(".jsx")
            && !normalized.ends_with(".tsx")
        {
            normalized.push_str(".js");
        }
        return normalized;
    }
    format!("{from}.js")
}

fn parse_go_imports(content: &str, map: &mut ImportMap) {
    let import_block = Regex::new(r#"import\s+(?:\(([^)]*)\)|"([^"]+)")"#).unwrap();
    for cap in import_block.captures_iter(content) {
        let block = cap
            .get(1)
            .or_else(|| cap.get(2))
            .map(|m| m.as_str())
            .unwrap_or("");
        for line in block.lines() {
            let line = line.trim().trim_matches('"');
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            let path = parts.last().copied().unwrap_or(line).trim_matches('"');
            let alias = if parts.len() > 1 {
                parts[0].to_string()
            } else {
                path.rsplit('/').next().unwrap_or(path).to_string()
            };
            map.bindings.insert(alias, format!("{path}.go"));
            map.modules.push(format!("{path}.go"));
        }
    }
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

fn path_matches_module(candidate_file: &str, module_path: &str) -> bool {
    let c = normalize_path(candidate_file);
    let m = normalize_path(module_path);
    c == m
        || c.ends_with(&m)
        || m.ends_with(&c)
        || c.strip_suffix(".py")
            .is_some_and(|stem| m.starts_with(stem))
        || c.strip_suffix(".rs")
            .is_some_and(|stem| m.starts_with(stem))
        || c.strip_suffix(".js")
            .is_some_and(|stem| m.starts_with(stem))
        || c.strip_suffix(".ts")
            .is_some_and(|stem| m.starts_with(stem))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_from_import_binds_helper_to_utils() {
        let src = "from utils import helper\n\ndef main():\n    pass\n";
        let map = ImportMap::parse("main.py", "python", src);
        assert_eq!(
            map.bindings.get("helper").map(String::as_str),
            Some("utils.py")
        );
    }

    #[test]
    fn js_relative_import_resolves_sibling() {
        let src = "import { helper } from './utils'\n";
        let map = ImportMap::parse("src/main.js", "javascript", src);
        assert_eq!(
            map.bindings.get("helper").map(String::as_str),
            Some("src/utils.js")
        );
    }
}