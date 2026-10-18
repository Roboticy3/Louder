# $ErrorActionPreference is set deliberately, and only here. A caller may have it
# on Stop, which turns anything cargo writes to stderr — including its ordinary
# "Compiling" and "warning:" lines — into a terminating error, so a script that
# worked from one directory fails from another for reasons that have nothing to
# do with the build. Every step below is judged on $LASTEXITCODE instead, which
# says whether the command actually failed.
$ErrorActionPreference = 'Continue'

$engine = $PSScriptRoot

# The engine alone: no adapter, and so no Godot in the build. `louder_godot` is a
# separate package in this workspace and is deliberately not built here, since
# what the C++ tests exercise is the C boundary and nothing above it.
Push-Location $engine
cargo build -p louder_rust
$buildExit = $LASTEXITCODE
Pop-Location
if ($buildExit -ne 0) { throw "the engine failed to build with exit code $buildExit" }

$built = Join-Path $engine "target\debug\louder_rust.dll"
$header = Join-Path $engine "target\debug\louder_rust.h"

# cbindgen runs on the crate directory, since its config and its output path are
# both written relative to one.
Push-Location (Join-Path $engine "louder_rust")
cbindgen --config cbindgen.toml --crate louder_rust --output (Join-Path $engine "target\debug\louder_rust.h")
$cbindgenExit = $LASTEXITCODE
Pop-Location
if ($cbindgenExit -ne 0) { throw "cbindgen failed with exit code $cbindgenExit" }

Copy-Item -LiteralPath $built -Destination (Join-Path $engine "artifacts\louder_rust.dll") -Force
Copy-Item -LiteralPath $header -Destination (Join-Path $engine "artifacts\louder_rust.h") -Force
Copy-Item -LiteralPath $built -Destination (Join-Path $engine "tests\louder_rust.dll") -Force
Copy-Item -LiteralPath $header -Destination (Join-Path $engine "tests\louder_rust.h") -Force

# Compile the C++ tests against the freshly generated header and DLL, then run
# them. The header is an output, so the tests always exercise the current ABI.
Push-Location $engine
g++ -std=c++17 -I artifacts tests/engine_test.cpp -o artifacts/engine_test.exe artifacts/louder_rust.dll
if ($LASTEXITCODE -ne 0) { throw "failed to compile the engine tests" }

# The DLL sits beside the test binary, which is where Windows looks first.
& artifacts/engine_test.exe
if ($LASTEXITCODE -ne 0) { throw "engine tests failed with exit code $LASTEXITCODE" }
Pop-Location