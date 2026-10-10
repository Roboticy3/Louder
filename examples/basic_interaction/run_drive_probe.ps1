param([string]$Exe = '.\drive_probe.exe', [int]$Width = 80, [int]$Height = 24)
& $Exe $Width $Height *>&1 | Out-String | Write-Output
Write-Output "exit code: $LASTEXITCODE"