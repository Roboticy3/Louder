param([string]$Exe = '.\auto_probe.exe')
& $Exe *>&1 | Out-String | Write-Output
Write-Output "exit code: $LASTEXITCODE"