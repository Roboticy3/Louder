param([string]$Exe = '.\offset_probe.exe')
& $Exe *>&1 | Out-String | Write-Output