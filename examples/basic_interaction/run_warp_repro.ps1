Write-Output '--- 0: follower on the same hub, same channel, same priority ---'
& .\warp_repro.exe 0 *>&1 | Out-String | Write-Output
Write-Output "exit code: $LASTEXITCODE"

Write-Output ''
Write-Output '--- 1: follower elsewhere on the main channel ---'
& .\warp_repro.exe 1 *>&1 | Out-String | Write-Output
Write-Output "exit code: $LASTEXITCODE"

Write-Output ''
Write-Output '--- 2: follower on a different channel (cannot be dragged) ---'
& .\warp_repro.exe 2 *>&1 | Out-String | Write-Output
Write-Output "exit code: $LASTEXITCODE"

Write-Output ''
Write-Output '--- 3: follower at higher priority (background, not warped) ---'
& .\warp_repro.exe 3 *>&1 | Out-String | Write-Output
Write-Output "exit code: $LASTEXITCODE"