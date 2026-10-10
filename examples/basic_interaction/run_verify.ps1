param([string]$Exe = '.\verify.exe')
& $Exe *>&1 | Out-String | Write-Output
Write-Output "exit code: $LASTEXITCODE"