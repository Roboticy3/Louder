cd ../../engine
cargo clean -p louder_godot
cargo build -p louder_godot
cd ../examples/crpg_playtest

Remove-Item -Recurse -Force Louder -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Name "Louder" | Out-Null
Copy-Item -Recurse ../../engine/target Louder/

godot -e .