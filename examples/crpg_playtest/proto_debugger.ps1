
$workdir = $PSScriptRoot
Write-Output "Current working directory: $workdir"
Start-Process -FilePath "powershell" -ArgumentList "-File", (Join-Path $workdir "build_and_copy_target.ps1") -WorkingDirectory $workdir
$editorPath = Resolve-Path (Join-Path $workdir "..\..\editor")
Start-Process -FilePath "godot" -ArgumentList @("--path", $editorPath.Path) -WorkingDirectory $workdir