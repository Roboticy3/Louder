# Builds the engine, copies it where the Godot extension looks for it, and runs
# test scripts through test_runner.gd.
#
#   .\run_test.ps1 test_louder.gd
#   .\run_test.ps1 test_louder.gd --frames 60
#
# Arguments naming a .gd file are resolved as res:// paths, so relative paths
# work; everything else, including the runner's own options, is passed straight
# through. The exit code is the runner's, so a failing test fails this too.

$ErrorActionPreference = "Stop"
$project = $PSScriptRoot
$crate = (Resolve-Path (Join-Path $project "..\..\engine\louder_rust")).Path

Push-Location $crate
cargo build --features godot-bindings
$buildExit = $LASTEXITCODE
Pop-Location
if ($buildExit -ne 0) { throw "the engine failed to build" }

# Louder.gdextension loads the library out of res://Louder/target/debug, so the
# fresh build has to be in place before Godot starts. Only the library is
# copied: the rest of target/ is build scratch Godot never looks at, and
# deleting it first means a locked file anywhere in there fails the whole run.
$debug = Join-Path $project "Louder\target\debug"
$built = Get-ChildItem (Join-Path $crate "target\debug") -File |
  Where-Object { $_.Name -match '^(lib)?louder_rust\.(dll|so|dylib)$' }
if (!$built) { throw "the build produced no library to copy" }
New-Item -ItemType Directory -Force $debug | Out-Null
Copy-Item -LiteralPath $built.FullName -Destination $debug -Force

$testArguments = @()
foreach ($argument in $args) {
  if ($argument.EndsWith(".gd") -and -not $argument.Contains("://")) {
    $testArguments += "res://$argument"
  } else {
    $testArguments += $argument
  }
}

& godot --headless --path $project --script res://test_runner.gd -- @testArguments
exit $LASTEXITCODE