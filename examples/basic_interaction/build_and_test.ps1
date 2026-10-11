# Build the demo and run the interface tests, then leave the demo binary ready to
# play. Mirrors engine/build_and_test.ps1: the header is a build output, so
# compiling against it always exercises the current ABI.
$ErrorActionPreference = 'Stop'

# Take the header and the DLL straight from the engine's build, rather than from
# the copies checked in beside this script. A stale pair still compiles and still
# links, and then fails at runtime for reasons that look like the demo's.
$artifacts = Join-Path $PSScriptRoot '..\..\engine\artifacts'
if (-not (Test-Path -LiteralPath (Join-Path $artifacts 'louder_rust.h'))) {
  throw "no engine build found at $artifacts; run engine/build_and_test.ps1 first"
}
Copy-Item -LiteralPath (Join-Path $artifacts 'louder_rust.h') -Destination .
Copy-Item -LiteralPath (Join-Path $artifacts 'louder_rust.dll') -Destination .

Write-Host 'compiling the demo'
& g++ -std=c++17 -O2 -Wall -Wextra -I . main.cpp dialogue.cpp terminal.cpp -o basic_interaction.exe louder_rust.dll
if ($LASTEXITCODE -ne 0) { throw 'failed to compile the demo' }

Write-Host 'compiling the interface tests'
& g++ -std=c++17 -O2 -Wall -Wextra -I . interface_test.cpp dialogue.cpp terminal.cpp -o interface_test.exe louder_rust.dll
if ($LASTEXITCODE -ne 0) { throw 'failed to compile the interface tests' }

# The DLL sits beside each binary, which is where Windows looks first.
Write-Host ''
Write-Host 'interface tests'
& .\interface_test.exe
if ($LASTEXITCODE -ne 0) { throw "interface tests failed with exit code $LASTEXITCODE" }

Write-Host ''
Write-Host 'startup check'
& .\basic_interaction.exe --check
if ($LASTEXITCODE -ne 0) { throw "the demo failed to start with exit code $LASTEXITCODE" }

Write-Host ''
Write-Host 'demo ready: .\basic_interaction.exe'
