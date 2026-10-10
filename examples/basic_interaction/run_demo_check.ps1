Write-Output '--- demo startup, one frame, then exit ---'
& .\basic_interaction.exe --check *>&1 | Out-String | Write-Output
Write-Output "exit code: $LASTEXITCODE"