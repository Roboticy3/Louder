# Build the demo and run the interface tests, then leave the demo binary ready to
# play. Mirrors engine/build_and_test.ps1: the header is a build output, so
# compiling against it always exercises the current ABI.
$ErrorActionPreference = 'Stop'

$gpp = 'g++'
$sources = @('main.cpp', 'engine_bridge.cpp', 'dialogue_view.cpp', 'screen.cpp')

Write-Host 'compiling the demo'
& $gpp -std=c++17 -O2 -Wall -I . @sources -o basic_interaction.exe louder_rust.dll
if ($LASTEXITCODE -ne 0) { throw 'failed to compile the demo' }

Write-Host 'compiling the interface tests'
& $gpp -std=c++17 -O2 -Wall -I . interface_test.cpp engine_bridge.cpp dialogue_view.cpp -o interface_test.exe louder_rust.dll
if ($LASTEXITCODE -ne 0) { throw 'failed to compile the interface tests' }

Write-Host 'compiling the dialogue tests'
& $gpp -std=c++17 -O2 -Wall -I . dialogue_test.cpp engine_bridge.cpp dialogue_view.cpp -o dialogue_test.exe louder_rust.dll
if ($LASTEXITCODE -ne 0) { throw 'failed to compile the dialogue tests' }

# The DLL sits beside each test binary, which is where Windows looks first.
Write-Host ''
Write-Host 'interface tests'
& .\interface_test.exe
if ($LASTEXITCODE -ne 0) { throw "interface tests failed with exit code $LASTEXITCODE" }

Write-Host ''
Write-Host 'dialogue tests'
& .\dialogue_test.exe
if ($LASTEXITCODE -ne 0) { throw "dialogue tests failed with exit code $LASTEXITCODE" }

Write-Host ''
Write-Host 'demo ready: .\basic_interaction.exe'