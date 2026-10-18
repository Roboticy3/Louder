cd ../../engine
cargo build -p louder_godot
cd ../examples/crpg_playtest

Remove-Item -Recurse Louder
New-Item -ItemType Directory -Name "Louder"
Copy-Item -Recurse ../../engine/target Louder/

godot -e .