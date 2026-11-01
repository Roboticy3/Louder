
$workdir = $PSScriptRoot
Write-Output "Current working directory: $workdir"
Start-Process -FilePath "powershell" -ArgumentList "-File", (Join-Path $workdir "build_and_copy_target.ps1") -WorkingDirectory $workdir
$editorPath = Resolve-Path (Join-Path $workdir "..\..\editor")
$marrowPath = Resolve-Path (Join-Path $workdir "conversations\marrow.json")
Start-Process -FilePath "godot" -ArgumentList @("--path", $editorPath.Path, "++", "--open-project", $marrowPath.Path) -WorkingDirectory $workdir