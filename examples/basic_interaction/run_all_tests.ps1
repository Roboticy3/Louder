Write-Output '--- render tests ---'
& .\render_test.exe *>&1 | Out-String | Write-Output
Write-Output "render exit code: $LASTEXITCODE"

Write-Output ''
Write-Output '--- interface tests ---'
& .\interface_test.exe *>&1 | Out-String | Write-Output
Write-Output "interface exit code: $LASTEXITCODE"