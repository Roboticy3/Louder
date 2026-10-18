# Builds the engine, copies it where the Godot extension looks for it, and runs
# test scripts through test_runner.gd.
#
#   .\run_test.ps1 test_louder.gd
#   .\run_test.ps1 test_louder.gd test_dialogue.gd --frames 60
#
# Arguments naming a .gd file are resolved as res:// paths, so relative paths
# work; everything else, including the runner's own options, is passed straight
# through. The exit code is the runner's, so a failing test fails this too.

# $ErrorActionPreference is deliberately not Stop. cargo writes its progress and
# its warnings to stderr, which Windows PowerShell hands to the error stream, so
# a blanket Stop would abort the run on "Compiling louder_rust" rather than on a
# build that actually failed. The build is judged on $LASTEXITCODE below.
$ErrorActionPreference = 'Continue'

$project = $PSScriptRoot
$engine = (Resolve-Path (Join-Path $project "..\..\engine")).Path

Push-Location $engine
cargo build -p louder_godot
$buildExit = $LASTEXITCODE
Pop-Location
if ($buildExit -ne 0) { throw "the extension failed to build with exit code $buildExit" }

# Louder.gdextension loads the library out of res://Louder/target/debug, so the
# fresh build has to be in place before Godot starts. Only the library is
# copied: the rest of target/ is build scratch Godot never looks at, and
# deleting it first means a locked file anywhere in there fails the whole run.
# The extension is louder_godot; the louder_rust library beside it is the engine
# on its own and Godot has no use for it.
$debug = Join-Path $project "Louder\target\debug"
$built = Join-Path $engine "target\debug\louder_godot.dll"
if (-not (Test-Path $built)) { throw "the build produced no library to copy" }
New-Item -ItemType Directory -Force $debug | Out-Null
Copy-Item -LiteralPath $built -Destination $debug -Force
foreach ($stale in @("louder_rust.dll", "liblouder_rust.so", "liblouder_rust.dylib")) {
  Remove-Item -LiteralPath (Join-Path $debug $stale) -ErrorAction SilentlyContinue
}

$testArguments = @()
foreach ($argument in $args) {
  # PowerShell hands bare numbers over as Int32, and a frame count is exactly
  # that, so every argument is treated as text before it is inspected.
  $argument = [string]$argument
  if ($argument.EndsWith(".gd") -and -not $argument.Contains("://")) {
    $testArguments += "res://$argument"
  } else {
    $testArguments += $argument
  }
}

& godot --headless --path $project --script res://test_runner.gd -- @testArguments
exit $LASTEXITCODE