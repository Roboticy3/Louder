use std::fs;
use std::path::PathBuf;

use syn::{File, Item, Type};

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_root = PathBuf::from(manifest_dir).join("..").join("louder_rust").join("src");

    // Collect all pub structs from louder_rust.
    let mut structs: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for path in walkdir(&src_root) {
        let content = fs::read_to_string(&path).unwrap();
        if let Ok(file) = syn::parse_str::<File>(&content) {
            for item in &file.items {
                if let Item::Struct(s) = item {
                    if matches!(s.vis, syn::Visibility::Public(_)) {
                        let mut fields = Vec::new();
                        if let syn::Fields::Named(named) = &s.fields {
                            for f in &named.named {
                                if matches!(f.vis, syn::Visibility::Public(_)) {
                                    if let Some(ident) = &f.ident {
                                        let ty = quote_type(&f.ty);
                                        fields.push((ident.to_string(), ty));
                                    }
                                }
                            }
                        }
                        structs.push((s.ident.to_string(), fields));
                    }
                }
            }
        }
    }

    // Generate wrappers for types with public fields.
    let targets = ["Motion", "AdditionalDataChange", "LoadFailure", "ChoiceAppearance", "ConditionOutcome", "Interrupt"];
    let mut generated = String::new();
    generated.push_str("use louder_rust::godot_api;\n");
    generated.push_str("use godot::prelude::*;\n");
    generated.push_str("use engine::*;\n\n");

    for (name, fields) in &structs {
        if !targets.contains(&name.as_str()) || fields.is_empty() {
            continue;
        }
        generated.push_str(&format!(
            "#[derive(Clone, Default)]\n#[derive(GodotClass)]\n#[class(init, tool)]\npub struct {name} {{\n    pub(crate) inner: engine::{name},\n}}\n\n",
            name = name
        ));
        generated.push_str("godot_api! {\n");
        generated.push_str(&format!("    impl {name} {{\n", name = name));
        let allowed = ["i32","i64","u32","u64","f32","f64","bool","usize","isize","engine::LouderName","engine::LouderPosition","engine::LouderActor","engine::LouderName"];
        for (field, ty) in fields {
            if !allowed.contains(&ty.as_str()) { continue; }
            let method = format!("get_{}", field);
            generated.push_str(&format!(
                "        #[func]\n        pub fn {method}(&self) -> {ty} {{\n            self.inner.{field}\n        }}\n\n",
                method = method,
                ty = ty,
                field = field
            ));
        }
        generated.push_str("    }\n}\n\n");
    }

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = PathBuf::from(out_dir).join("generated.rs");
    fs::write(dest, generated).unwrap();
    println!("cargo:rerun-if-changed=../louder_rust/src");
}

fn quote_type(ty: &Type) -> String {
    quote::quote!(#ty).to_string().replace(" ", "")
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
