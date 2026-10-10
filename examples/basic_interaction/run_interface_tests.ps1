$ErrorActionPreference = 'Continue'
& .\interface_test.exe *>&1 | Out-String | Write-Output
Write-Output "exit code: $LASTEXITCODE"