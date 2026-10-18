# Regenerates the engine API reference from the Rust source and builds the site.
#
# 1. cargo doc: produces the rustdoc HTML for louder_rust and louder_godot
# 2. copies the output into the mkdocs source tree (docs/docs/rustdoc)
# 3. mkdocs build: emits the finished site under docs/site
#
# Every path is anchored to this script, so it runs the same from anywhere:
#
#   .\docs\build_docs.ps1
#
# $ErrorActionPreference is deliberately left alone. cargo reports its warnings
# on stderr, which Windows PowerShell hands to the error stream, so a blanket
# Stop would abort the run on the first rustdoc warning instead of on a failure.

# The workspace, not the crate: the target directory is one level up, since the
# engine and its adapters share it.
$rustCrate = (Resolve-Path (Join-Path $PSScriptRoot "..\engine")).Path
$generated = Join-Path $rustCrate "target\doc"
$rustdocDir = Join-Path $PSScriptRoot "docs\rustdoc"

# rustdoc leaves a page behind for anything it documented last time, so a stale
# target\doc is copied into the site whole unless it goes first. Only the two
# crates in this workspace are documented, so rebuilding after the clean costs
# seconds.
if (Test-Path $generated) {
  Remove-Item -Recurse -Force -ErrorAction Stop $generated
}

# Both crates: the engine, which is the reference, and the Godot adapter, which
# is where the classes GDScript actually calls are declared.
Push-Location $rustCrate
cargo doc -p louder_rust -p louder_godot --no-deps --document-private-items
$docExit = $LASTEXITCODE
Pop-Location

if ($docExit -ne 0) {
  exit $docExit
}

foreach ($documented in @("louder_rust", "louder_godot")) {
  if (!(Test-Path (Join-Path $generated "$documented\index.html"))) {
    throw "cargo doc succeeded but wrote no $documented\index.html"
  }
}

# The mkdocs source tree keeps the site, so it has to be replaced rather than
# merged into, for the same reason target\doc was.
if (Test-Path $rustdocDir) {
  Remove-Item -Recurse -Force -ErrorAction Stop $rustdocDir
}
Copy-Item -Recurse -ErrorAction Stop $generated $rustdocDir

# mkdocs reads mkdocs.yml from the directory it runs in, which is this one.
Push-Location $PSScriptRoot
mkdocs serve
