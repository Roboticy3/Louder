// Generated from the C header of the Louder engine.
//
// The engine is built into its own workspace, so its library is not beside
// this crate. The search below walks up from the crate to the workspace
// that holds both and points the linker at wherever cargo left it.

use std::path::Path;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let staging = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR")).join("link");
    let _ = std::fs::create_dir_all(&staging);

    for workspace in manifest.ancestors() {
        let Ok(text) = std::fs::read_to_string(workspace.join("Cargo.toml")) else {
            continue;
        };
        if !text.contains("louder_rust") {
            continue;
        };

        for profile in ["release", "debug"] {
            let built = workspace.join("target").join(profile);
            if !built.is_dir() {
                continue;
            }
            stage(&built, &staging, "louder_rust");
            println!("cargo:rustc-link-search=native={}", built.display());
            println!("cargo:rustc-link-search=native={}", staging.display());
            return;
        }

        println!(
            "cargo:warning=the engine is not built; run cargo build -p louder_rust",
        );
        return;
    }

    println!("cargo:warning=the engine workspace was not found above this crate");
}

/// Put the import library in the staging directory under the name the
/// linker asks for, if it is not already called that.
///
/// Windows writes `louder_rust.dll.lib` where MSVC `link` opens
/// `louder_rust.lib`. Everywhere else the file already has the right
/// name and this copies nothing.
fn stage(built: &Path, staging: &Path, stem: &str) {
    let wanted = format!("{stem}.lib");
    for name in [format!("{stem}.dll.lib"), format!("lib{stem}.a"), wanted.clone()] {
        let from = built.join(&name);
        if from.is_file() {
            let _ = std::fs::copy(&from, staging.join(&wanted));
            return;
        }
    }
}
