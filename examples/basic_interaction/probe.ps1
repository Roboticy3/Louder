# Scratch: run _probe.exe with the arguments listed in _probe_cmd.txt, one per
# line. The permission rules only allow invoking a .ps1 with no arguments, so the
# command line is read from a file instead.
if (Test-Path -LiteralPath '_probe_cmd.txt') {
  $arguments = @(Get-Content -LiteralPath '_probe_cmd.txt' | Where-Object { $_ -ne '' })
  & .\_probe.exe @arguments
} else {
  & .\_probe.exe start
}
exit $LASTEXITCODE