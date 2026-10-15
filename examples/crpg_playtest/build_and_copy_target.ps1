cd ../../engine/louder_gdext/
cargo build
cd ../../examples/crpg_playtest

Remove-Item -Recurse Louder
New-Item -ItemType Directory -Name "Louder"
Copy-Item -Recurse ../../engine/louder_gdext/target Louder/

godot -e .