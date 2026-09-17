# Regenerates the engine API reference from the Rust source and builds the site.
#
# 1. cargo doc: produces the rustdoc HTML for louder_rust
# 2. copies the output into the mkdocs source tree (docs/rustdoc)
# 3. mkdocs build: emits the finished site under docs/site

$rustCrate = Resolve-Path ..\engine\louder_rust
$trimCss = Join-Path $PSScriptRoot rustdoc_trim.css
$rustdocDir = Join-Path $PSScriptRoot docs\rustdoc

$env:RUSTDOCFLAGS = "--html-in-header=$trimCss"
Push-Location $rustCrate
cargo doc -p louder_rust --no-deps
$docExit = $LASTEXITCODE
Pop-Location
Remove-Item Env:\RUSTDOCFLAGS

if ($docExit -ne 0) {
  exit $docExit
}

if (Test-Path $rustdocDir) {
  Remove-Item -Recurse -Force $rustdocDir
}
Copy-Item -Recurse (Join-Path $rustCrate target\doc) $rustdocDir

mkdocs build
exit $LASTEXITCODE