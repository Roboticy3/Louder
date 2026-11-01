use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir_path = PathBuf::from(&manifest_dir);
    let src_root = manifest_dir_path.join("..").join("louder_rust").join("src");

    let mut structs: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for path in walkdir(&src_root) {
        let content = fs::read_to_string(&path).unwrap();
        // simple regex extraction of pub struct fields
        // We'll just collect names; fields are handled per-type below.
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("pub struct ") {
                let rest = &line["pub struct ".len()..];
                let name = rest.split_whitespace().next().unwrap_or("");
                let name = name.trim_end_matches('{').trim_end_matches('<').trim();
                if !name.is_empty() {
                    structs.push((name.to_string(), Vec::new()));
                }
            }
        }
    }
    structs.sort();
    structs.dedup_by(|a, b| a.0 == b.0);

    let targets = ["Motion", "AdditionalDataChange", "LoadFailure", "ChoiceAppearance", "ConditionOutcome", "Interrupt", "Speaker"];

    let mut generated = String::new();
    generated.push_str("use louder_rust::godot_api;\n");
    generated.push_str("use godot::prelude::*;\n\n");

    // We need field info; parse manually for these types.
    for name in &targets {
        // find struct in louder_rust
        let mut fields: Vec<(String, String)> = Vec::new();
        for path in walkdir(&src_root) {
            let content = fs::read_to_string(&path).unwrap();
            // find "pub struct Name {" ... "}"
            let marker = format!("pub struct {} ", name);
            if let Some(idx) = content.find(&marker) {
                // find opening brace
                let brace = content[idx..].find('{').unwrap() + idx;
                // find matching closing brace
                let mut depth = 0;
                let mut end = brace;
                for (i, c) in content[brace..].char_indices() {
                    if c == '{' { depth += 1; }
                    else if c == '}' { depth -= 1; if depth == 0 { end = brace + i; break; } }
                }
                let body = &content[brace+1..end];
                for line in body.lines() {
                    let line = line.trim();
                    if line.starts_with("pub ") {
                        // pub field: type,
                        let line = line.trim_end_matches(',').trim();
                        let parts: Vec<&str> = line.splitn(2, ':').collect();
                        if parts.len() == 2 {
                            let fname = parts[0].trim().trim_start_matches("pub ").trim();
                            let ftype = parts[1].trim();
                            if fname.chars().all(|c| c.is_alphanumeric() || c == '_') {
                                fields.push((fname.to_string(), ftype.to_string()));
                            }
                        }
                    }
                }
                break;
            }
        }

        if fields.is_empty() {
            continue;
        }

        generated.push_str(&format!(
            "#[derive(Clone, Default)]\n#[derive(GodotClass)]\n#[class(init, tool)]\npub struct {name} {{\n    pub(crate) inner: engine::{name},\n}}\n\n",
            name = name
        ));
        generated.push_str("godot_api! {\n");
        generated.push_str(&format!("    impl {name} {{\n", name = name));
        for (field, ty) in &fields {
            // only generate for simple types
            let simple = ty == "i32" || ty == "i64" || ty == "u32" || ty == "u64" || ty == "f32" || ty == "f64" || ty == "bool" || ty == "usize" || ty == "isize" || ty == "LouderPosition" || ty == "LouderName" || ty == "LouderActor" || ty == "LouderRule";
            if !simple { continue; }
            let (out_ty, body) = if ty == "LouderRule" {
              ("i32".to_string(), format!("self.inner.{}.clone() as i32", field))
            } else {
              let mapped = if ty.starts_with("Louder") { format!("engine::{}", ty) } else { ty.clone() };
              (mapped, format!("self.inner.{}.clone()", field))
            };
            let method = format!("get_{}", field);
            generated.push_str(&format!(
                "        #[func]\n        pub fn {method}(&self) -> {out_ty} {{\n            {body}\n        }}\n\n",
                method = method,
                out_ty = out_ty,
                body = body
            ));
        }
        generated.push_str("    }\n}\n\n");
    }

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = PathBuf::from(out_dir).join("generated.rs");
    fs::write(dest, &generated).unwrap();
    // Also write a tracked copy for inspection.
    let inspect = manifest_dir_path.join("generated_bindings.rs");
    fs::write(inspect, &generated).unwrap();
    println!("cargo:rerun-if-changed=../louder_rust/src");
}

fn walkdir(root: &PathBuf) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.extend(walkdir(&path));
            } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
                result.push(path);
            }
        }
    }
    result
}
