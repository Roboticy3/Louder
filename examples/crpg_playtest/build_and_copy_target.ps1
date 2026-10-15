cd ../../engine/louder_rust/
cargo build --features godot-bindings
cd ../../examples/crpg_playtest

Remove-Item -Recurse Louder
New-Item -ItemType Directory -Name "Louder"
Copy-Item -Recurse ../../engine/louder_rust/target Louder/

godot -e .