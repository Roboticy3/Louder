use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = PathBuf::from(out_dir).join("generated.rs");

    let generated = r#"
use louder_rust::godot_api;
use godot::prelude::*;

#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct Interrupt {
    pub(crate) inner: engine::Interrupt,
}

godot_api! {
    impl Interrupt {
        #[func]
        pub fn get_actor(&self) -> engine::LouderActor {
            self.inner.actor
        }

        #[func]
        pub fn get_expression(&self) -> engine::LouderName {
            self.inner.expression
        }
    }
}

#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct AdditionalDataChange {
    pub(crate) inner: engine::AdditionalDataChange,
}

godot_api! {
    impl AdditionalDataChange {
        #[func]
        pub fn get_key(&self) -> engine::LouderName {
            self.inner.key
        }
    }
}
"#;

    fs::write(dest, generated).unwrap();
    println!("cargo:rerun-if-changed=../louder_rust/src");
}
