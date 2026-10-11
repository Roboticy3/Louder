# Build the demo and run the interface tests, then leave the demo binary ready to
# play. Mirrors engine/build_and_test.ps1: the header is a build output, so
# compiling against it always exercises the current ABI.
#
# The engine binaries come from the build published on gh-pages by Engine CI, so
# this directory works when it is separated from the rest of the repo. If that
# host is unreachable, fall back to a local engine build, which is what makes
# this usable offline. Override the host with LOUDER_BASE_URL, and set
# LOUDER_OFFLINE to skip the network entirely.
$ErrorActionPreference = 'Stop'

$base = if ($env:LOUDER_BASE_URL) { $env:LOUDER_BASE_URL.TrimEnd('/') }
        else { 'https://roboticy3.github.io/Louder' }

# Platform directory published by Engine CI. This demo is built with MinGW on
# windows, so it takes the Windows library.
$platform = 'windows-x86_64'

function Get-EngineBinary($Name) {
  $destination = Join-Path $PSScriptRoot $Name

  if (-not $env:LOUDER_OFFLINE) {
    $url = "$base/$platform/$Name"
    Write-Host "fetching $url"
    try {
      # -UseBasicParsing keeps this working on 5.1 without an IE first-run.
      Invoke-WebRequest -Uri $url -OutFile $destination -UseBasicParsing -TimeoutSec 30
      return $destination
    } catch {
      Write-Warning "could not fetch ${url}: $($_.Exception.Message)"
    }
  }

  # The engine build beside the checkout. A stale pair still compiles and still
  # links, and then fails at runtime for reasons that look like the demo's.
  $artifacts = Join-Path $PSScriptRoot '..\..\engine\artifacts'
  $local = Join-Path $artifacts $Name
  if (-not (Test-Path -LiteralPath $local)) {
    throw "no engine binary for $Name; tried $base/$platform/$Name and $local"
  }
  Write-Host "using the local build at $local"
  Copy-Item -LiteralPath $local -Destination $destination
  return $destination
}

Get-EngineBinary 'louder_rust.h' | Out-Null
Get-EngineBinary 'louder_rust.dll' | Out-Null

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
