cd ../../engine
cargo clean -p louder_godot
cargo build -p louder_godot
cd ../examples/crpg_playtest

if (Test-Path Louder) {
    Remove-Item -Recurse -Force Louder
}
New-Item -ItemType Directory -Name "Louder" | Out-Null
Copy-Item -Recurse ../../engine/target Louder/

godot -e .