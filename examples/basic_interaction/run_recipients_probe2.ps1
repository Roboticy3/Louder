param([string]$Exe = '.\recipients_probe2.exe')
& $Exe *>&1 | Out-String | Write-Output
Write-Output "exit code: $LASTEXITCODE"